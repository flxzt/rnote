pub mod chrono_comp;
pub mod keytree;
pub mod render_comp;
pub mod selection_comp;
pub mod trash_comp;

use std::collections::VecDeque;
use std::sync::Arc;

// Re-exports
pub use chrono_comp::ChronoComponent;
use keytree::KeyTree;
pub use render_comp::RenderComponent;
use rnote_compose::penpath::Segment;
pub use selection_comp::SelectionComponent;
pub use trash_comp::TrashComponent;

use crate::pens::penholder::PenStyle;
use crate::pens::tools::DragProximityTool;
use crate::strokes::strokebehaviour::GeneratedStrokeImages;
use crate::strokes::BitmapImage;
use crate::strokes::Stroke;
use crate::strokes::StrokeBehaviour;
use crate::strokes::VectorImage;
use crate::surfaceflags::SurfaceFlags;
use crate::{render, Camera};
use rnote_compose::helpers::{self};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::transform::TransformBehaviour;

use p2d::bounding_volume::{BoundingSphere, BoundingVolume, AABB};
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};

use self::render_comp::RenderCompState;

/*
StrokeStore implements a Entity - Component - System pattern.
The Entities are the StrokeKey's, which represent a stroke. There are different components for them:
    * 'strokes': Hold geometric data. These components are special in that they are the primary map. A new stroke must have this component. (could also be called geometric components)
    * 'trash_components': Hold state wether the strokes are trashed
    * 'selection_components': Hold state wether the strokes are selected
    * 'chrono_components': Hold state about the time, chronological ordering
    * 'render_components': Hold state about the current rendering of the strokes.

The systems are implemented as methods on StrokesStore, loosely categorized to the different components (but often modify others as well).
Most systems take a key or a slice of keys, and iterate with them over the different components.
There also is a different category of methods which return filtered keys, e.g. `.keys_sorted_chrono` returns the keys in chronological ordering,
    `.stroke_keys_in_order_rendering` filters and returns keys in the order which they should be rendered.
*/

#[derive(Debug, Clone)]
/// A store task, usually coming from a spawned thread and to be processed with `process_received_task()`.
pub enum StoreTask {
    /// Replace the images of the render_comp.
    /// Note that usually the state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid large queues of already outdated rendering tasks.
    UpdateStrokeWithImages {
        key: StrokeKey,
        images: GeneratedStrokeImages,
    },
    /// Appends the images to the rendering of the stroke
    /// Note that usually the state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid large queues of already outdated rendering tasks.
    AppendImagesToStroke {
        key: StrokeKey,
        images: GeneratedStrokeImages,
    },
    /// Inserts a new stroke to the store
    /// Note that usually the state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid large queues of already outdated rendering tasks.
    InsertStroke { stroke: Stroke },
    /// indicates that the application is quitting. Usually handled to quit the async loop which receives the tasks
    Quit,
}

fn default_threadpool() -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::default()
        .build()
        .unwrap_or_else(|e| {
            log::error!("default_render_threadpool() failed with Err {}", e);
            panic!()
        })
}

slotmap::new_key_type! {
    pub struct StrokeKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    strokes: Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>,
    trash_components: Arc<SecondaryMap<StrokeKey, Arc<TrashComponent>>>,
    selection_components: Arc<SecondaryMap<StrokeKey, Arc<SelectionComponent>>>,
    chrono_components: Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "stroke_store")]
pub struct StrokeStore {
    // Components
    #[serde(rename = "strokes")]
    strokes: Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>,
    #[serde(rename = "trash_components")]
    trash_components: Arc<SecondaryMap<StrokeKey, Arc<TrashComponent>>>,
    #[serde(rename = "selection_components")]
    selection_components: Arc<SecondaryMap<StrokeKey, Arc<SelectionComponent>>>,
    #[serde(rename = "chrono_components")]
    chrono_components: Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>,
    #[serde(skip)]
    render_components: SecondaryMap<StrokeKey, RenderComponent>,

    // The history
    #[serde(skip)]
    history: VecDeque<Arc<HistoryEntry>>,
    #[serde(skip)]
    history_pos: Option<usize>,

    // A rtree backed by the slotmap, for faster spatial queries. Needs to be updated with update_with_key() when strokes changed their geometry of position!
    #[serde(skip)]
    key_tree: KeyTree,

    // Other state
    /// value is equal chrono_component of the newest inserted or modified stroke.
    #[serde(rename = "chrono_counter")]
    chrono_counter: u32,

    #[serde(skip)]
    pub tasks_tx: futures::channel::mpsc::UnboundedSender<StoreTask>,
    /// To be taken out into a loop which processes the receiver stream. The received tasks should be processed with process_received_task()
    #[serde(skip)]
    pub tasks_rx: Option<futures::channel::mpsc::UnboundedReceiver<StoreTask>>,
    #[serde(skip, default = "default_threadpool")]
    threadpool: rayon::ThreadPool,
}

impl Default for StrokeStore {
    fn default() -> Self {
        let threadpool = default_threadpool();

        let (tasks_tx, tasks_rx) = futures::channel::mpsc::unbounded::<StoreTask>();

        Self {
            strokes: Arc::new(HopSlotMap::with_key()),
            trash_components: Arc::new(SecondaryMap::new()),
            selection_components: Arc::new(SecondaryMap::new()),
            chrono_components: Arc::new(SecondaryMap::new()),
            render_components: SecondaryMap::new(),

            history: VecDeque::new(),
            history_pos: None,

            key_tree: KeyTree::default(),

            chrono_counter: 0,

            tasks_tx,
            tasks_rx: Some(tasks_rx),
            threadpool,
        }
    }
}

impl StrokeStore {
    /// The max length of the history
    pub(crate) const HISTORY_LEN: usize = 100;

    pub fn new() -> Self {
        Self::default()
    }

    /// A new strokes state should always be imported with this method, to not replace the threadpool, channel handlers..
    /// needs rendering regeneration after calling
    pub fn import_store(&mut self, store: Self) {
        self.clear();
        self.strokes = store.strokes;
        self.trash_components = store.trash_components;
        self.selection_components = store.selection_components;
        self.chrono_components = store.chrono_components;

        self.chrono_counter = store.chrono_counter;

        self.reload_tree();
        self.reload_render_components_slotmap();
    }

    pub fn reload_tree(&mut self) {
        let tree_objects = self
            .strokes
            .iter()
            .map(|(key, stroke)| (key, stroke.bounds()))
            .collect();
        self.key_tree.reload_with_vec(tree_objects);
    }

    pub(crate) fn process_received_task(
        &mut self,
        task: StoreTask,
        camera: &Camera,
    ) -> SurfaceFlags {
        let viewport_expanded = camera.viewport();
        let image_scale = camera.image_scale();
        let mut surface_flags = SurfaceFlags::default();

        match task {
            StoreTask::UpdateStrokeWithImages { key, images } => {
                if let Err(e) = self.replace_rendering_with_images(key, images) {
                    log::error!("replace_rendering_with_images() in process_received_task() failed with Err {}", e);
                }

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;
            }
            StoreTask::AppendImagesToStroke { key, images } => {
                if let Err(e) = self.append_rendering_images(key, images) {
                    log::error!(
                        "append_rendering_images() in process_received_task() failed with Err {}",
                        e
                    );
                }

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;
            }
            StoreTask::InsertStroke { stroke } => {
                self.record();

                match stroke {
                    Stroke::BrushStroke(brushstroke) => {
                        let _inserted = self.insert_stroke(Stroke::BrushStroke(brushstroke));

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                    }
                    Stroke::ShapeStroke(shapestroke) => {
                        let _inserted = self.insert_stroke(Stroke::ShapeStroke(shapestroke));

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                    }
                    Stroke::VectorImage(vectorimage) => {
                        let inserted = self.insert_stroke(Stroke::VectorImage(vectorimage));
                        self.set_selected(inserted, true);

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.resize_to_fit_strokes = true;
                        surface_flags.change_to_pen = Some(PenStyle::Selector);
                        surface_flags.sheet_changed = true;
                        surface_flags.update_selector = true;
                    }
                    Stroke::BitmapImage(bitmapimage) => {
                        let inserted = self.insert_stroke(Stroke::BitmapImage(bitmapimage));
                        self.set_selected(inserted, true);

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.resize_to_fit_strokes = true;
                        surface_flags.change_to_pen = Some(PenStyle::Selector);
                        surface_flags.sheet_changed = true;
                        surface_flags.update_selector = true;
                    }
                }

                self.regenerate_rendering_in_viewport_threaded(
                    false,
                    viewport_expanded,
                    image_scale,
                );
            }
            StoreTask::Quit => {
                surface_flags.quit = true;
            }
        }

        surface_flags
    }

    fn ptr_eq_history(&self, history_entry: &Arc<HistoryEntry>) -> bool {
        Arc::ptr_eq(&self.strokes, &history_entry.strokes)
            && Arc::ptr_eq(&self.trash_components, &history_entry.trash_components)
            && Arc::ptr_eq(
                &self.selection_components,
                &history_entry.selection_components,
            )
            && Arc::ptr_eq(&self.chrono_components, &history_entry.chrono_components)
    }

    fn history_entry_from_current_state(&self) -> Arc<HistoryEntry> {
        Arc::new(HistoryEntry {
            strokes: Arc::clone(&self.strokes),
            trash_components: Arc::clone(&self.trash_components),
            selection_components: Arc::clone(&self.selection_components),
            chrono_components: Arc::clone(&self.chrono_components),
        })
    }

    fn import_history_entry(&mut self, history_entry: &Arc<HistoryEntry>) {
        self.strokes = Arc::clone(&history_entry.strokes);
        self.trash_components = Arc::clone(&history_entry.trash_components);
        self.selection_components = Arc::clone(&history_entry.selection_components);
        self.chrono_components = Arc::clone(&history_entry.chrono_components);
    }

    pub fn record(&mut self) {
        self.history_pos = None;

        if !self
            .history
            .back()
            .map(|last| self.ptr_eq_history(last))
            .unwrap_or(false)
        {
            self.history
                .push_back(self.history_entry_from_current_state());

            if self.history.len() > Self::HISTORY_LEN {
                self.history.pop_front();
            }
        } else {
            log::trace!("state has not changed, no need to record");
        }
    }

    // Needs rendering regeneration after calling
    pub(crate) fn undo(&mut self) {
        let index = self.history_pos.unwrap_or(self.history.len());

        if index > 0 {
            let current = self.history_entry_from_current_state();
            let prev = Arc::clone(&self.history[index - 1]);

            self.history.push_back(current);

            self.import_history_entry(&prev);
            self.history_pos = Some(index - 1);

            // Since we don't store the tree in the history, we rebuild them everytime we undo
            self.reload_tree();
            // render_components are also not stored in the history, but for the duration of the running app we don't ever remove from render_components, so we can actually skip rebuilding it on undo.
            // Avoids visual glitches where we have already rebuilt the components and can't display anything until the asynchronous rendering is finished
            //self.reload_render_components_slotmap();
        } else {
            log::debug!("no history, can't undo");
        }
    }

    pub(crate) fn break_undo_chain(&mut self) {
        // move the position to the end, so we can do "undo the undoes", emacs-style
        self.history_pos = None;
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_pos = None;
    }

    /// Needs rendering regeneration after calling
    pub fn insert_stroke(&mut self, stroke: Stroke) -> StrokeKey {
        let bounds = stroke.bounds();

        let key = Arc::make_mut(&mut self.strokes).insert(Arc::new(stroke));
        self.key_tree.insert_with_key(key, bounds);
        self.chrono_counter += 1;

        Arc::make_mut(&mut self.trash_components).insert(key, Arc::new(TrashComponent::default()));
        Arc::make_mut(&mut self.selection_components)
            .insert(key, Arc::new(SelectionComponent::default()));
        Arc::make_mut(&mut self.chrono_components)
            .insert(key, Arc::new(ChronoComponent::new(self.chrono_counter)));
        self.render_components
            .insert(key, RenderComponent::default());

        key
    }

    pub fn remove_stroke(&mut self, key: StrokeKey) -> Option<Stroke> {
        Arc::make_mut(&mut self.trash_components).remove(key);
        Arc::make_mut(&mut self.selection_components).remove(key);
        Arc::make_mut(&mut self.chrono_components).remove(key);
        self.render_components.remove(key);

        self.key_tree.remove_with_key(key);
        Arc::make_mut(&mut self.strokes)
            .remove(key)
            .map(|stroke| (*stroke).clone())
    }

    /// stroke geometry needs to be updated and rendering regeneration after calling
    pub fn add_segment_to_brushstroke(&mut self, key: StrokeKey, segment: Segment) {
        if let Some(Stroke::BrushStroke(brushstroke)) = Arc::make_mut(&mut self.strokes)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            brushstroke.push_segment(segment);

            self.set_rendering_dirty(key);
        }
    }

    /// Clears every stroke and every component
    pub fn clear(&mut self) {
        Arc::make_mut(&mut self.strokes).clear();
        Arc::make_mut(&mut self.trash_components).clear();
        Arc::make_mut(&mut self.selection_components).clear();
        Arc::make_mut(&mut self.chrono_components).clear();

        self.chrono_counter = 0;
        self.clear_history();

        self.render_components.clear();
        self.key_tree.clear();
    }

    /// All keys
    pub fn keys_unordered(&self) -> Vec<StrokeKey> {
        self.strokes.keys().collect()
    }

    /// All stroke keys, not including the selection
    pub fn stroke_keys_unordered(&self) -> Vec<StrokeKey> {
        self.strokes
            .keys()
            .filter_map(|key| {
                if !(self.trashed(key).unwrap_or(false)) && !(self.selected(key).unwrap_or(false)) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the stroke keys in the order that they should be rendered. Does not include the selection.
    pub fn stroke_keys_as_rendered(&self) -> Vec<StrokeKey> {
        self.keys_sorted_chrono()
            .into_iter()
            .filter_map(|key| {
                if !(self.trashed(key).unwrap_or(false)) && !(self.selected(key).unwrap_or(false)) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub fn stroke_keys_as_rendered_intersecting_bounds(&self, bounds: AABB) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(bounds)
            .into_iter()
            .filter(|&key| {
                !(self.trashed(key).unwrap_or(false)) && !(self.selected(key).unwrap_or(false))
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub fn clone_strokes(&self, keys: &[StrokeKey]) -> Vec<Stroke> {
        keys.iter()
            .filter_map(|&key| Some((**self.strokes.get(key)?).clone()))
            .collect::<Vec<Stroke>>()
    }

    pub fn insert_vectorimage_bytes_threaded(&mut self, pos: na::Vector2<f64>, bytes: Vec<u8>) {
        let tasks_tx = self.tasks_tx.clone();

        let all_strokes = self.keys_unordered();
        self.set_selected_keys(&all_strokes, false);

        self.threadpool.spawn(move || {
                match String::from_utf8(bytes) {
                    Ok(svg) => {
                        match VectorImage::import_from_svg_data(svg.as_str(), pos, None) {
                            Ok(vectorimage) => {
                                let vectorimage = Stroke::VectorImage(vectorimage);

                                tasks_tx.unbounded_send(StoreTask::InsertStroke {
                                    stroke: vectorimage
                                }).unwrap_or_else(|e| {
                                    log::error!("tasks_tx.send() failed in insert_vectorimage_bytes_threaded() with Err, {}", e);
                                });
                            }
                            Err(e) => {
                                log::error!("VectorImage::import_from_svg_data() failed in insert_vectorimage_bytes_threaded() with Err, {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("from_utf8() failed in thread from insert_vectorimages_bytes_threaded() with Err {}", e);
                    }
                }
            });
    }

    pub fn insert_bitmapimage_bytes_threaded(&mut self, pos: na::Vector2<f64>, bytes: Vec<u8>) {
        let tasks_tx = self.tasks_tx.clone();

        let all_strokes = self.keys_unordered();
        self.set_selected_keys(&all_strokes, false);

        self.threadpool.spawn(move || {
                match BitmapImage::import_from_image_bytes(&bytes, pos) {
                    Ok(bitmapimage) => {
                        let bitmapimage = Stroke::BitmapImage(bitmapimage);

                        tasks_tx.unbounded_send(StoreTask::InsertStroke {
                            stroke: bitmapimage
                        }).unwrap_or_else(|e| {
                            log::error!("tasks_tx.send() failed in insert_bitmapimage_bytes_threaded() with Err, {}", e);
                        });
                    }
                    Err(e) => {
                        log::error!("BitmapImage::import_from_svg_data() failed in insert_bitmapimage_bytes_threaded() with Err, {}", e);
                    }
                }
            });
    }

    pub fn insert_pdf_bytes_as_vector_threaded(
        &mut self,
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
        bytes: Vec<u8>,
    ) {
        let tasks_tx = self.tasks_tx.clone();

        let all_strokes = self.keys_unordered();
        self.set_selected_keys(&all_strokes, false);

        self.threadpool.spawn(move || {
                match VectorImage::import_from_pdf_bytes(&bytes, pos, page_width) {
                    Ok(images) => {
                        for image in images {
                            tasks_tx.unbounded_send(StoreTask::InsertStroke {
                                stroke: Stroke::VectorImage(image)
                            }).unwrap_or_else(|e| {
                                log::error!("tasks_tx.send() failed in insert_pdf_bytes_as_vector_threaded() with Err, {}", e);
                            });
                        }
                    }
                    Err(e) => {
                        log::error!("VectorImage::import_from_pdf_bytes() failed in insert_pdf_bytes_as_vector_threaded() with Err, {}", e);
                    }
                }
            });
    }

    pub fn insert_pdf_bytes_as_bitmap_threaded(
        &mut self,
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
        bytes: Vec<u8>,
    ) {
        let tasks_tx = self.tasks_tx.clone();

        let all_strokes = self.keys_unordered();
        self.set_selected_keys(&all_strokes, false);

        self.threadpool.spawn(move || {
                match BitmapImage::import_from_pdf_bytes(&bytes, pos, page_width) {
                    Ok(images) => {
                        for image in images {
                            let image = Stroke::BitmapImage(image);

                            tasks_tx.unbounded_send(StoreTask::InsertStroke {
                                stroke: image
                            }).unwrap_or_else(|e| {
                                log::error!("tasks_tx.send() failed in insert_pdf_bytes_as_bitmap_threaded() with Err, {}", e);
                            });
                        }
                    }
                    Err(e) => {
                        log::error!("BitmapImage::import_from_pdf_bytes() failed in insert_pdf_bytes_as_bitmap_threaded() with Err, {}", e);
                    }
                }
            });
    }

    /// Needs rendering regeneration after calling
    pub fn update_geometry_for_stroke(&mut self, key: StrokeKey) {
        if let Some(stroke) = Arc::make_mut(&mut self.strokes)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            match stroke {
                Stroke::BrushStroke(ref mut brushstroke) => {
                    brushstroke.update_geometry();
                    self.key_tree.update_with_key(key, stroke.bounds());

                    self.set_rendering_dirty(key);
                }
                Stroke::ShapeStroke(_) => {}
                Stroke::VectorImage(_) => {}
                Stroke::BitmapImage(_) => {}
            }
        }
    }

    pub fn update_geometry_for_strokes(&mut self, keys: &[StrokeKey]) {
        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);
        });
    }

    /// Calculates the width needed to fit all strokes
    pub fn calc_width(&self) -> f64 {
        let new_width = if let Some(stroke) = self
            .strokes
            .iter()
            .filter_map(|(key, stroke)| {
                if let Some(trash_comp) = self.trash_components.get(key) {
                    if !trash_comp.trashed {
                        return Some(stroke);
                    }
                }
                None
            })
            .max_by_key(|&stroke| stroke.bounds().maxs[0].round() as i32)
        {
            // max_by_key() returns the element, so we need to extract the width again
            stroke.bounds().maxs[0]
        } else {
            0.0
        };

        new_width
    }

    /// Calculates the height needed to fit all strokes
    pub fn calc_height(&self) -> f64 {
        let new_height = if let Some(stroke) = self
            .stroke_keys_unordered()
            .into_iter()
            .filter_map(|key| self.strokes.get(key))
            .max_by_key(|&stroke| stroke.bounds().maxs[1].round() as i32)
        {
            // max_by_key() returns the element, so we need to extract the height again
            stroke.bounds().maxs[1]
        } else {
            0.0
        };

        new_height
    }

    /// Generates the enclosing bounds for the given stroke keys
    pub fn gen_bounds_for_strokes(&self, keys: &[StrokeKey]) -> Option<AABB> {
        let mut keys_iter = keys.iter();
        if let Some(&key) = keys_iter.next() {
            if let Some(first) = self.strokes.get(key) {
                let mut bounds = first.bounds();

                keys_iter
                    .filter_map(|&key| self.strokes.get(key))
                    .for_each(|stroke| {
                        bounds.merge(&stroke.bounds());
                    });

                return Some(bounds);
            }
        }

        None
    }

    /// Collects all bounds for the given strokes
    pub fn bounds_for_strokes(&self, keys: &[StrokeKey]) -> Vec<AABB> {
        keys.iter()
            .filter_map(|&key| Some(self.strokes.get(key)?.bounds()))
            .collect::<Vec<AABB>>()
    }

    /// Generates a Svg for all strokes as drawn onto the canvas without xml headers or svg roots. Does not include the selection.
    pub fn gen_svgs_for_strokes(&self, keys: &[StrokeKey]) -> Vec<render::Svg> {
        keys.iter()
            .filter_map(|&key| {
                let stroke = self.strokes.get(key)?;

                match stroke.gen_svg() {
                    Ok(svgs) => Some(svgs),
                    Err(e) => {
                        log::error!(
                            "stroke.gen_svg() failed in gen_svg_for_strokes() with Err {}",
                            e
                        );
                        None
                    }
                }
            })
            .collect::<Vec<render::Svg>>()
    }

    /// Translate the strokes with the offset.
    /// Rendering needs to be regenerated
    pub fn translate_strokes(&mut self, strokes: &[StrokeKey], offset: na::Vector2<f64>) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.strokes)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    // translate the stroke geometry
                    stroke.translate(offset);
                    self.key_tree.update_with_key(key, stroke.bounds());
                }
            }
        });
    }

    pub fn translate_strokes_images(&mut self, strokes: &[StrokeKey], offset: na::Vector2<f64>) {
        strokes.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                for image in render_comp.images.iter_mut() {
                    image.translate(offset);
                }

                match render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in translate_strokes_images() with Err {}",
                        e
                    ),
                }
            }
        });
    }

    /// Rotates the stroke with angle (rad) around the center.
    /// Rendering needs to be regenerated
    pub fn rotate_strokes(&mut self, strokes: &[StrokeKey], angle: f64, center: na::Point2<f64>) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.strokes)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    // rotate the stroke geometry
                    stroke.rotate(angle, center);
                    self.key_tree.update_with_key(key, stroke.bounds());
                }
            }
        });
    }

    pub fn rotate_strokes_images(
        &mut self,
        strokes: &[StrokeKey],
        angle: f64,
        center: na::Point2<f64>,
    ) {
        strokes.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.state = RenderCompState::Dirty;

                for image in render_comp.images.iter_mut() {
                    image.rotate(angle, center);
                }

                match render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in rotate_strokes() with Err {}",
                        e
                    ),
                }
            }
        });
    }

    /// Scales the strokes with the factor.
    /// Rendering needs to be regenerated
    pub fn scale_strokes(&mut self, strokes: &[StrokeKey], scale: na::Vector2<f64>) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.strokes)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    // rotate the stroke geometry
                    stroke.scale(scale);
                    self.key_tree.update_with_key(key, stroke.bounds());
                }
            }
        });
    }

    pub fn scale_strokes_images(&mut self, strokes: &[StrokeKey], scale: na::Vector2<f64>) {
        strokes.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.state = RenderCompState::Dirty;

                for image in render_comp.images.iter_mut() {
                    image.scale(scale);
                }

                match render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in rotate_strokes() with Err {}",
                        e
                    ),
                }
            }
        });
    }

    pub fn scale_strokes_with_pivot(
        &mut self,
        strokes: &[StrokeKey],
        scale: na::Vector2<f64>,
        pivot: na::Vector2<f64>,
    ) {
        self.translate_strokes(strokes, -pivot);
        self.scale_strokes(strokes, scale);
        self.translate_strokes(strokes, pivot);
    }

    pub fn scale_strokes_images_with_pivot(
        &mut self,
        strokes: &[StrokeKey],
        scale: na::Vector2<f64>,
        pivot: na::Vector2<f64>,
    ) {
        self.translate_strokes_images(strokes, -pivot);
        self.scale_strokes_images(strokes, scale);
        self.translate_strokes_images(strokes, pivot);
    }

    /// Resizes the strokes to new bounds.
    /// Needs rendering regeneration after calling
    pub fn resize_strokes(&mut self, strokes: &[StrokeKey], new_bounds: AABB) {
        let old_bounds = match self.gen_bounds_for_strokes(strokes) {
            Some(old_bounds) => old_bounds,
            None => return,
        };

        strokes.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.strokes)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    // resize the stroke geometry
                    let old_stroke_bounds = stroke.bounds();
                    let new_stroke_bounds = helpers::scale_inner_bounds_in_context_new_outer_bounds(
                        old_stroke_bounds,
                        old_bounds,
                        new_bounds,
                    );
                    let scale = new_stroke_bounds
                        .extents()
                        .component_div(&old_stroke_bounds.extents());
                    let rel_offset = new_stroke_bounds.center() - old_stroke_bounds.center();

                    // Translate in relation to the outer bounds
                    stroke.translate(rel_offset - old_stroke_bounds.center().coords);
                    stroke.scale(scale);
                    stroke.translate(old_stroke_bounds.center().coords);

                    self.key_tree.update_with_key(key, stroke.bounds());
                }
            }
        });
    }

    pub fn resize_strokes_images(&mut self, strokes: &[StrokeKey], new_bounds: AABB) {
        let old_bounds = match self.gen_bounds_for_strokes(strokes) {
            Some(old_bounds) => old_bounds,
            None => return,
        };

        strokes.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.state = RenderCompState::Dirty;

                for image in render_comp.images.iter_mut() {
                    // resize the stroke geometry
                    let old_image_bounds = image.rect.bounds();
                    let new_image_bounds = helpers::scale_inner_bounds_in_context_new_outer_bounds(
                        old_image_bounds,
                        old_bounds,
                        new_bounds,
                    );
                    let scale = new_image_bounds
                        .extents()
                        .component_div(&old_image_bounds.extents());
                    let rel_offset = new_image_bounds.center() - old_image_bounds.center();

                    // Translate in relation to the outer bounds
                    image.translate(rel_offset - old_image_bounds.center().coords);
                    image.scale(scale);
                    image.translate(old_image_bounds.center().coords);
                }

                match render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in resize_strokes() with Err {}",
                        e
                    ),
                }
            }
        });
    }

    /// Returns all keys below the y_pos
    pub fn keys_below_y_pos(&self, y_pos: f64) -> Vec<StrokeKey> {
        self.strokes
            .iter()
            .filter_map(|(key, stroke)| {
                if stroke.bounds().mins[1] > y_pos {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    /// Needs rendering regeneration for current viewport after calling
    pub fn drag_strokes_proximity(&mut self, drag_proximity_tool: &DragProximityTool) {
        let sphere = BoundingSphere {
            center: na::Point2::from(drag_proximity_tool.pos),
            radius: drag_proximity_tool.radius,
        };

        #[allow(dead_code)]
        fn calc_distance_ratio(
            pos: na::Vector2<f64>,
            tool_pos: na::Vector2<f64>,
            radius: f64,
        ) -> f64 {
            // Zero when right at drag_proximity_tool position, One when right at the radius
            (1.0 - (pos - tool_pos).magnitude() / radius).clamp(0.0, 1.0)
        }

        Arc::make_mut(&mut self.strokes)
            .iter_mut()
            .par_bridge()
            .filter_map(|(key, stroke)| -> Option<StrokeKey> {
                match Arc::make_mut(stroke) {
                    Stroke::BrushStroke(brushstroke) => {
                        if sphere.intersects(&brushstroke.path.bounds().bounding_sphere()) {
                            for segment in brushstroke.path.iter_mut() {
                                let segment_sphere = segment.bounds().bounding_sphere();

                                if sphere.intersects(&segment_sphere) {
                                    /*                                     match segment {
                                        Segment::QuadBez { start, cp, end } => {
                                            start.pos += drag_proximity_tool.offset
                                                * calc_distance_ratio(
                                                    start.pos,
                                                    drag_proximity_tool.pos,
                                                    drag_proximity_tool.radius,
                                                );
                                            *cp += drag_proximity_tool.offset
                                                * calc_distance_ratio(
                                                    *cp,
                                                    drag_proximity_tool.pos,
                                                    drag_proximity_tool.radius,
                                                );
                                            end.pos += drag_proximity_tool.offset
                                                * calc_distance_ratio(
                                                    end.pos,
                                                    drag_proximity_tool.pos,
                                                    drag_proximity_tool.radius,
                                                );
                                            return Some(key);
                                        }
                                    } */
                                    return Some(key);
                                }
                            }
                        }
                    }
                    _ => {}
                }

                None
            })
            .collect::<Vec<StrokeKey>>()
            .iter()
            .for_each(|&key| {
                self.update_geometry_for_stroke(key);
            });
    }
}
