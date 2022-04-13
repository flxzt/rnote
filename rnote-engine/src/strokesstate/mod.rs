pub mod chrono_comp;
pub mod keytree;
pub mod render_comp;
pub mod selection_comp;
pub mod trash_comp;

// Re-exports
pub use chrono_comp::ChronoComponent;
use keytree::KeyTree;
pub use render_comp::RenderComponent;
use rnote_compose::penpath::{Element, Segment};
pub use selection_comp::SelectionComponent;
pub use trash_comp::TrashComponent;

use crate::pens::penholder::PenStyle;
use crate::pens::tools::DragProximityTool;
use crate::pens::Shaper;
use crate::strokes::BitmapImage;
use crate::strokes::Stroke;
use crate::strokes::StrokeBehaviour;
use crate::strokes::VectorImage;
use crate::surfaceflags::SurfaceFlags;
use crate::{render, Camera};
use rnote_compose::helpers::{self, AABBHelpers};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::transform::TransformBehaviour;

use p2d::bounding_volume::{BoundingSphere, BoundingVolume, AABB};
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};

/*
StrokesState implements a Entity - Component - System pattern.
The Entities are the StrokeKey's, which represent a stroke. There are different components for them:
    * 'strokes': Hold geometric data. These components are special in that they are the primary map. A new stroke must have this component. (could also be called geometric components)
    * 'trash_components': Hold state wether the strokes are trashed
    * 'selection_components': Hold state wether the strokes are selected
    * 'chrono_components': Hold state about the time, chronological ordering
    * 'render_components': Hold state about the current rendering of the strokes.

The systems are implemented as methods on StrokesState, loosely categorized to the different components (but often modify others as well).
Most systems take a key or a slice of keys, and iterate with them over the different components.
There also is a different category of methods which return filtered keys, e.g. `.keys_sorted_chrono` returns the keys in chronological ordering,
    `.stoke_keys_in_order_rendering` filters and returns keys in the order which they should be rendered.
*/

#[derive(Debug, Clone)]
pub enum StateTask {
    /// Replace the images of the render_comp
    UpdateStrokeWithImages {
        key: StrokeKey,
        images: Vec<render::Image>,
    },
    /// Inserts a new stroke to the store
    InsertStroke { stroke: Stroke },
    /// indicates that the application is in the process of quitting
    Quit,
}

pub fn default_threadpool() -> rayon::ThreadPool {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "strokes_state")]
pub struct StrokesState {
    // Components
    #[serde(rename = "strokes")]
    strokes: HopSlotMap<StrokeKey, Stroke>,
    #[serde(rename = "trash_components")]
    trash_components: SecondaryMap<StrokeKey, TrashComponent>,
    #[serde(rename = "selection_components")]
    selection_components: SecondaryMap<StrokeKey, SelectionComponent>,
    #[serde(rename = "chrono_components")]
    chrono_components: SecondaryMap<StrokeKey, ChronoComponent>,
    #[serde(rename = "render_components")]
    render_components: SecondaryMap<StrokeKey, RenderComponent>,

    // A rtree backed by the slotmap, for faster spatial queries. Needs to be updated with update_with_key() when strokes changed their geometry of position!
    #[serde(skip)]
    key_tree: KeyTree,

    // Other state
    /// value is equal chrono_component of the newest inserted or modified stroke.
    #[serde(rename = "chrono_counter")]
    chrono_counter: u32,

    #[serde(skip)]
    pub tasks_tx: futures::channel::mpsc::UnboundedSender<StateTask>,
    /// To be taken out into a loop which processes the receiver stream. The received tasks should be processed with process_received_task()
    #[serde(skip)]
    pub tasks_rx: Option<futures::channel::mpsc::UnboundedReceiver<StateTask>>,
    #[serde(skip, default = "default_threadpool")]
    threadpool: rayon::ThreadPool,
}

impl Default for StrokesState {
    fn default() -> Self {
        let threadpool = default_threadpool();

        let (tasks_tx, tasks_rx) = futures::channel::mpsc::unbounded::<StateTask>();

        Self {
            strokes: HopSlotMap::with_key(),
            trash_components: SecondaryMap::new(),
            selection_components: SecondaryMap::new(),
            chrono_components: SecondaryMap::new(),
            render_components: SecondaryMap::new(),

            key_tree: KeyTree::default(),

            chrono_counter: 0,

            tasks_tx,
            tasks_rx: Some(tasks_rx),
            threadpool,
        }
    }
}

impl StrokesState {
    pub fn new() -> Self {
        Self::default()
    }

    /// A new strokes state should always be imported with this method, to not replace the threadpool, channel handlers..
    /// needs rendering regeneration after calling
    pub fn import_strokes_state(&mut self, strokes_state: Self) {
        self.clear();
        self.strokes = strokes_state.strokes;
        self.trash_components = strokes_state.trash_components;
        self.selection_components = strokes_state.selection_components;
        self.chrono_components = strokes_state.chrono_components;
        self.render_components = strokes_state.render_components;

        self.chrono_counter = strokes_state.chrono_counter;

        self.reload_tree();
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
        task: StateTask,
        camera: &Camera,
    ) -> SurfaceFlags {
        let viewport_expanded = camera.viewport_extended();
        let image_scale = camera.image_scale();
        let mut surface_flags = SurfaceFlags::default();

        match task {
            StateTask::UpdateStrokeWithImages { key, images } => {
                if let Err(e) = self.replace_rendering_with_images(key, images) {
                    log::error!("replace_rendering_with_images() in process_received_task() failed with Err {}", e);
                }

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;
            }
            StateTask::InsertStroke { stroke } => {
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
                        surface_flags.selection_changed = true;
                    }
                    Stroke::BitmapImage(bitmapimage) => {
                        let inserted = self.insert_stroke(Stroke::BitmapImage(bitmapimage));
                        self.set_selected(inserted, true);

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.resize_to_fit_strokes = true;
                        surface_flags.change_to_pen = Some(PenStyle::Selector);
                        surface_flags.sheet_changed = true;
                        surface_flags.selection_changed = true;
                    }
                }

                self.regenerate_rendering_in_viewport_threaded(
                    false,
                    viewport_expanded,
                    image_scale,
                );
            }
            StateTask::Quit => {
                surface_flags.quit = true;
            }
        }

        surface_flags
    }

    /// Needs rendering regeneration after calling
    pub fn insert_stroke(&mut self, stroke: Stroke) -> StrokeKey {
        let bounds = stroke.bounds();

        let key = self.strokes.insert(stroke);
        self.key_tree.insert_with_key(key, bounds);
        self.chrono_counter += 1;

        let mut render_comp = RenderComponent::default();
        render_comp.regenerate_flag = true;

        self.trash_components.insert(key, TrashComponent::default());
        self.selection_components
            .insert(key, SelectionComponent::default());
        self.render_components.insert(key, render_comp);
        self.chrono_components
            .insert(key, ChronoComponent::new(self.chrono_counter));

        key
    }

    pub fn remove_stroke(&mut self, key: StrokeKey) -> Option<Stroke> {
        self.trash_components.remove(key);
        self.selection_components.remove(key);
        self.chrono_components.remove(key);
        self.render_components.remove(key);

        self.key_tree.remove_with_key(key);
        self.strokes.remove(key)
    }

    /// stroke geometry needs to be updated and rendering regeneration after calling
    pub fn add_segment_to_brushstroke(&mut self, key: StrokeKey, segment: Segment) {
        if let Some(Stroke::BrushStroke(ref mut brushstroke)) = self.strokes.get_mut(key) {
            brushstroke.push_segment(segment);

            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.regenerate_flag = true;
            }
        }
    }

    /// Needs rendering regeneration after calling
    pub fn update_shapestroke(
        &mut self,
        key: StrokeKey,
        shaper: &mut Shaper,
        new_element: Element,
    ) {
        if let Some(Stroke::ShapeStroke(ref mut shapestroke)) = self.strokes.get_mut(key) {
            shapestroke.update_shape(shaper, new_element);
            self.key_tree.update_with_key(key, shapestroke.bounds());

            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.regenerate_flag = true;
            }
        }
    }

    /// Clears every stroke and every component
    pub fn clear(&mut self) {
        self.chrono_counter = 0;

        self.strokes.clear();
        self.trash_components.clear();
        self.selection_components.clear();
        self.chrono_components.clear();
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
                if self.does_render(key).unwrap_or(false)
                    && !(self.trashed(key).unwrap_or(false))
                    && !(self.selected(key).unwrap_or(false))
                {
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
                if self.does_render(key).unwrap_or(false)
                    && !(self.trashed(key).unwrap_or(false))
                    && !(self.selected(key).unwrap_or(false))
                {
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
                self.does_render(key).unwrap_or(false)
                    && !(self.trashed(key).unwrap_or(false))
                    && !(self.selected(key).unwrap_or(false))
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub fn clone_strokes(&self, keys: &[StrokeKey]) -> Vec<Stroke> {
        keys.iter()
            .filter_map(|&key| Some(self.strokes.get(key)?.clone()))
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

                                tasks_tx.unbounded_send(StateTask::InsertStroke {
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

                        tasks_tx.unbounded_send(StateTask::InsertStroke {
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
                            tasks_tx.unbounded_send(StateTask::InsertStroke {
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

                            tasks_tx.unbounded_send(StateTask::InsertStroke {
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
        if let Some(stroke) = self.strokes.get_mut(key) {
            match stroke {
                Stroke::BrushStroke(ref mut brushstroke) => {
                    brushstroke.update_geometry();
                    self.key_tree.update_with_key(key, stroke.bounds());

                    if let Some(render_comp) = self.render_components.get_mut(key) {
                        render_comp.regenerate_flag = true;
                    }
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

    /// Generates the bounds which enclose the strokes
    pub fn gen_bounds(&self, keys: &[StrokeKey]) -> Option<AABB> {
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

    /// Translate the strokes with the offset
    pub fn translate_strokes(&mut self, strokes: &[StrokeKey], offset: na::Vector2<f64>) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = self.strokes.get_mut(key) {
                stroke.translate(offset);
                self.key_tree.update_with_key(key, stroke.bounds());

                if let Some(render_comp) = self.render_components.get_mut(key) {
                    for image in render_comp.images.iter_mut() {
                        image.bounds = image.bounds.translate(offset);
                    }

                    match render::Image::images_to_rendernodes(&render_comp.images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                        }
                        Err(e) => log::error!(
                            "images_to_rendernode() failed in translate_strokes() with Err {}",
                            e
                        ),
                    }
                }
            }
        });
    }

    /// Rotates the stroke with angle (rad) around the center
    /// Rendering needs to be regenerated
    pub fn rotate_strokes(&mut self, strokes: &[StrokeKey], angle: f64, center: na::Point2<f64>) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = self.strokes.get_mut(key) {
                stroke.rotate(angle, center);
                self.key_tree.update_with_key(key, stroke.bounds());

                if let Some(render_comp) = self.render_components.get_mut(key) {
                    render_comp.regenerate_flag = true;
                }
            }
        });
    }

    /// Resizes the strokes to new bounds
    /// Needs rendering regeneration after calling
    pub fn resize_strokes(&mut self, strokes: &[StrokeKey], old_bounds: AABB, new_bounds: AABB) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = self.strokes.get_mut(key) {
                let old_stroke_bounds = stroke.bounds();
                let new_stroke_bounds = helpers::scale_inner_bounds_to_new_outer_bounds(
                    stroke.bounds(),
                    old_bounds,
                    new_bounds,
                );

                let rel_offset = new_stroke_bounds.center() - old_stroke_bounds.center();
                let scale = new_stroke_bounds
                    .extents()
                    .component_div(&old_stroke_bounds.extents());

                stroke.translate(-old_stroke_bounds.center().coords);

                // Translate in relation to the outer bounds
                stroke.translate(rel_offset);
                stroke.scale(scale);

                stroke.translate(old_stroke_bounds.center().coords);

                self.key_tree.update_with_key(key, stroke.bounds());

                if let Some(render_comp) = self.render_components.get_mut(key) {
                    render_comp.regenerate_flag = true;
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

        self.strokes
            .iter_mut()
            .par_bridge()
            .filter_map(|(key, stroke)| -> Option<StrokeKey> {
                match stroke {
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
