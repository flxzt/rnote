pub mod chrono_comp;
pub mod render_comp;
pub mod selection_comp;
pub mod trash_comp;

use std::sync::{Arc, RwLock};

use chrono_comp::ChronoComponent;
use p2d::query::PointQuery;
use render_comp::RenderComponent;
use selection_comp::SelectionComponent;
use trash_comp::TrashComponent;

use crate::compose::geometry;
use crate::drawbehaviour::DrawBehaviour;
use crate::pens::tools::DragProximityTool;
use crate::render;
use crate::strokes::bitmapimage::BitmapImage;
use crate::strokes::strokebehaviour::StrokeBehaviour;
use crate::strokes::strokestyle::{Element, StrokeStyle};
use crate::strokes::vectorimage::VectorImage;
use crate::ui::appwindow::RnoteAppWindow;

use gtk4::{glib, glib::clone, prelude::*};
use p2d::bounding_volume::BoundingVolume;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};

#[derive(Debug, Clone)]
pub enum StateTask {
    UpdateStrokeWithImages {
        key: StrokeKey,
        images: Vec<render::Image>,
    },
    AppendImagesToStroke {
        key: StrokeKey,
        images: Vec<render::Image>,
    },
    InsertStroke {
        stroke: StrokeStyle,
    },
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
#[serde(default)]
pub struct StrokesState {
    // Components
    strokes: HopSlotMap<StrokeKey, StrokeStyle>,
    trash_components: SecondaryMap<StrokeKey, TrashComponent>,
    selection_components: SecondaryMap<StrokeKey, SelectionComponent>,
    chrono_components: SecondaryMap<StrokeKey, ChronoComponent>,
    render_components: SecondaryMap<StrokeKey, RenderComponent>,

    // Other state
    /// value is equal chrono_component of the newest inserted or modified stroke.
    chrono_counter: u32,
    pub selection_bounds: Option<p2d::bounding_volume::AABB>,
    #[serde(skip)]
    pub zoom: f64, // changes with the canvas zoom
    #[serde(skip)]
    pub renderer: Arc<RwLock<render::Renderer>>,
    #[serde(skip)]
    pub tasks_tx: Option<glib::Sender<StateTask>>,
    #[serde(skip)]
    pub tasks_rx: Option<glib::Receiver<StateTask>>,
    #[serde(skip)]
    pub channel_source: Option<glib::Source>,
    #[serde(skip, default = "default_threadpool")]
    pub threadpool: rayon::ThreadPool,
}

impl Default for StrokesState {
    fn default() -> Self {
        let threadpool = default_threadpool();

        let (render_tx, render_rx) =
            glib::MainContext::channel::<StateTask>(glib::PRIORITY_HIGH_IDLE);

        Self {
            strokes: HopSlotMap::with_key(),
            trash_components: SecondaryMap::new(),
            selection_components: SecondaryMap::new(),
            chrono_components: SecondaryMap::new(),
            render_components: SecondaryMap::new(),

            chrono_counter: 0,
            zoom: 1.0,
            renderer: Arc::new(RwLock::new(render::Renderer::default())),
            tasks_tx: Some(render_tx),
            tasks_rx: Some(render_rx),
            channel_source: None,
            selection_bounds: None,
            threadpool,
        }
    }
}

impl StrokesState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, appwindow: &RnoteAppWindow) {
        let main_cx = glib::MainContext::default();

        let source_id = self.tasks_rx.take().unwrap().attach(
            Some(&main_cx),
            clone!(@weak appwindow => @default-return glib::Continue(false), move |render_task| {
                match render_task {
                    StateTask::UpdateStrokeWithImages { key, images } => {
                        appwindow
                            .canvas()
                            .sheet()
                            .strokes_state()
                            .borrow_mut()
                            .regenerate_rendering_with_images(key, images);

                        appwindow.canvas().queue_draw();
                    }
                    StateTask::AppendImagesToStroke { key, images } => {
                        appwindow
                            .canvas()
                            .sheet()
                            .strokes_state()
                            .borrow_mut()
                            .append_images_to_rendering(key, images);

                        appwindow.canvas().queue_draw();
                    }
                    StateTask::InsertStroke { stroke } => {
                        match stroke {
                            StrokeStyle::MarkerStroke(markerstroke) => {
                                appwindow.canvas().sheet()
                                    .strokes_state()
                                    .borrow_mut()
                                    .insert_stroke_threaded(StrokeStyle::MarkerStroke(markerstroke));
                            }
                            StrokeStyle::BrushStroke(brushstroke) => {
                                appwindow.canvas().sheet()
                                    .strokes_state()
                                    .borrow_mut()
                                    .insert_stroke_threaded(StrokeStyle::BrushStroke(brushstroke));
                            }
                            StrokeStyle::ShapeStroke(shapestroke) => {
                                appwindow.canvas().sheet()
                                    .strokes_state()
                                    .borrow_mut()
                                    .insert_stroke_threaded(StrokeStyle::ShapeStroke(shapestroke));
                            }
                            StrokeStyle::VectorImage(vectorimage) => {
                                let inserted = appwindow.canvas().sheet()
                                    .strokes_state()
                                    .borrow_mut()
                                    .insert_stroke_threaded(StrokeStyle::VectorImage(vectorimage));
                                appwindow.canvas().sheet()
                                    .strokes_state()
                                    .borrow_mut()
                                    .set_selected(inserted, true);

                                appwindow.canvas().selection_modifier().set_visible(true);
                                appwindow.mainheader().selector_toggle().set_active(true);

                                appwindow.canvas().sheet().resize_to_format();
                                appwindow.canvas().update_background_rendernode(true);
                            }
                            StrokeStyle::BitmapImage(bitmapimage) => {
                                let inserted = appwindow
                                    .canvas()
                                    .sheet()
                                    .strokes_state()
                                    .borrow_mut()
                                    .insert_stroke_threaded(StrokeStyle::BitmapImage(bitmapimage));

                                appwindow.canvas().sheet()
                                    .strokes_state()
                                    .borrow_mut()
                                    .set_selected(inserted, true);

                                appwindow.canvas().selection_modifier().set_visible(true);
                                appwindow.mainheader().selector_toggle().set_active(true);

                                appwindow.canvas().sheet().resize_to_format();
                                appwindow.canvas().update_background_rendernode(false);
                            }
                        }

                    }
                    StateTask::Quit => {
                        return glib::Continue(false);
                    }
                }

                glib::Continue(true)
            }),
        );

        let source = main_cx.find_source_by_id(&source_id).unwrap_or_else(|| {
            log::error!("find_source_by_id() in StrokeState init() failed.");
            panic!();
        });
        self.channel_source.replace(source);
    }

    pub fn insert_stroke(&mut self, stroke: StrokeStyle) -> StrokeKey {
        let key = self.strokes.insert(stroke);
        self.chrono_counter += 1;

        self.trash_components.insert(key, TrashComponent::default());
        self.selection_components
            .insert(key, SelectionComponent::default());
        self.render_components
            .insert(key, RenderComponent::default());
        self.chrono_components
            .insert(key, ChronoComponent::new(self.chrono_counter));

        self.regenerate_rendering_for_stroke(key);
        key
    }

    pub fn insert_stroke_threaded(&mut self, stroke: StrokeStyle) -> StrokeKey {
        let key = self.strokes.insert(stroke);
        self.chrono_counter += 1;

        self.trash_components.insert(key, TrashComponent::default());
        self.selection_components
            .insert(key, SelectionComponent::default());
        self.render_components
            .insert(key, RenderComponent::default());
        self.chrono_components
            .insert(key, ChronoComponent::new(self.chrono_counter));

        self.regenerate_rendering_for_stroke_threaded(key);
        key
    }

    pub fn remove_stroke(&mut self, key: StrokeKey) -> Option<StrokeStyle> {
        self.trash_components.remove(key);
        self.selection_components.remove(key);
        self.chrono_components.remove(key);
        self.render_components.remove(key);

        self.strokes.remove(key)
    }

    /// returns key to last stroke
    pub fn add_to_stroke(&mut self, key: StrokeKey, element: Element) -> Option<StrokeKey> {
        match self.strokes.get_mut(key).unwrap() {
            StrokeStyle::MarkerStroke(ref mut markerstroke) => {
                markerstroke.push_elem(element);
            }
            StrokeStyle::BrushStroke(ref mut brushstroke) => {
                brushstroke.push_elem(element);
            }
            StrokeStyle::ShapeStroke(ref mut shapestroke) => {
                shapestroke.update_shape(element);
            }
            StrokeStyle::VectorImage(_vectorimage) => {}
            StrokeStyle::BitmapImage(_bitmapimage) => {}
        }

        self.append_rendering_new_elem_threaded_fifo(key);
        Some(key)
    }

    /// Clears every stroke and every component
    pub fn clear(&mut self) {
        self.chrono_counter = 0;
        self.selection_bounds = None;

        self.strokes.clear();
        self.trash_components.clear();
        self.selection_components.clear();
        self.chrono_components.clear();
        self.render_components.clear();
    }

    pub fn insert_vectorimage_bytes_threaded(&mut self, pos: na::Vector2<f64>, bytes: glib::Bytes) {
        let renderer = self.renderer.clone();

        if let Some(tasks_tx) = self.tasks_tx.clone() {
            self.threadpool.spawn(move || {
                let svg = String::from_utf8_lossy(&bytes);

                match VectorImage::import_from_svg_data(&svg, pos, None, &renderer.read().unwrap()) {
                    Ok(vectorimage) => {
                        let vectorimage = StrokeStyle::VectorImage(vectorimage);

                        tasks_tx.send(StateTask::InsertStroke {
                            stroke: vectorimage
                        }).unwrap_or_else(|e| {
                            log::error!("tasks_tx.send() failed in insert_vectorimage_bytes_threaded() with Err, {}", e);
                        });
                    }
                    Err(e) => {
                        log::error!("VectorImage::import_from_svg_data() failed in insert_vectorimage_bytes_threaded() with Err, {}", e);
                    }
                }
            });
        }
    }

    pub fn insert_bitmapimage_bytes_threaded(&mut self, pos: na::Vector2<f64>, bytes: glib::Bytes) {
        if let Some(tasks_tx) = self.tasks_tx.clone() {
            self.threadpool.spawn(move || {
                match BitmapImage::import_from_image_bytes(bytes, pos) {
                    Ok(bitmapimage) => {
                        let bitmapimage = StrokeStyle::BitmapImage(bitmapimage);

                        tasks_tx.send(StateTask::InsertStroke {
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
    }

    pub fn insert_pdf_bytes_as_vector_threaded(
        &mut self,
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
        bytes: glib::Bytes,
    ) {
        let renderer = self.renderer.clone();

        if let Some(tasks_tx) = self.tasks_tx.clone() {
            self.threadpool.spawn(move || {
                match VectorImage::import_from_pdf_bytes(&bytes, pos, page_width, &renderer.read().unwrap()) {
                    Ok(images) => {
                        for image in images {
                            let image = StrokeStyle::VectorImage(image);

                            tasks_tx.send(StateTask::InsertStroke {
                                stroke: image
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
    }

    pub fn insert_pdf_bytes_as_bitmap_threaded(
        &mut self,
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
        bytes: glib::Bytes,
    ) {
        if let Some(tasks_tx) = self.tasks_tx.clone() {
            self.threadpool.spawn(move || {
                match BitmapImage::import_from_pdf_bytes(&bytes, pos, page_width) {
                    Ok(images) => {
                        for image in images {
                            let image = StrokeStyle::BitmapImage(image);

                            tasks_tx.send(StateTask::InsertStroke {
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
    }

    pub fn import_state(&mut self, strokes_state: &Self) {
        self.clear();
        self.chrono_counter = strokes_state.chrono_counter;
        self.selection_bounds = strokes_state.selection_bounds;

        self.strokes = strokes_state.strokes.clone();
        self.trash_components = strokes_state.trash_components.clone();
        self.selection_components = strokes_state.selection_components.clone();
        self.chrono_components = strokes_state.chrono_components.clone();
        self.render_components = strokes_state.render_components.clone();

        self.regenerate_strokes_current_view_threaded(None, true);
    }

    pub fn update_geometry_for_stroke(&mut self, key: StrokeKey) {
        if let Some(stroke) = self.strokes.get_mut(key) {
            match stroke {
                StrokeStyle::MarkerStroke(ref mut markerstroke) => {
                    markerstroke.update_geometry();
                }
                StrokeStyle::BrushStroke(ref mut brushstroke) => {
                    brushstroke.update_geometry();
                }
                StrokeStyle::ShapeStroke(shapestroke) => {
                    shapestroke.update_geometry();
                }
                StrokeStyle::VectorImage(ref mut _vectorimage) => {}
                StrokeStyle::BitmapImage(ref mut _bitmapimage) => {}
            }
        } else {
            log::debug!(
                "get stroke in update_stroke_geometry() returned None in complete_stroke() for key {:?}",
                key
            );
        }
    }

    pub fn update_geometry_all_strokes(&mut self) {
        let keys: Vec<StrokeKey> = self.strokes.keys().collect();

        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);
        });
    }

    pub fn update_geometry_selection_strokes(&mut self) {
        let keys: Vec<StrokeKey> = self.selection_keys();

        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);
        });
    }

    pub fn regenerate_strokes_current_view(
        &mut self,
        viewport: Option<p2d::bounding_volume::AABB>,
        force_regenerate: bool,
    ) {
        let keys = self.render_components.keys().collect::<Vec<StrokeKey>>();

        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);

            if let (Some(stroke), Some(render_comp)) =
                (self.strokes.get(key), self.render_components.get_mut(key))
            {
                // skip if stroke is not in viewport or does not need regeneration
                if let Some(viewport) = viewport {
                    if !viewport.intersects(&stroke.bounds()) {
                        return;
                    }
                }
                if !force_regenerate && !render_comp.regenerate_flag {
                    return;
                }

                match stroke.gen_image(self.zoom, &self.renderer.read().unwrap()) {
                    Ok(image) => {
                        render_comp.regenerate_flag = false;
                        render_comp.rendernode = render::image_to_rendernode(&image, self.zoom);
                        render_comp.images = vec![image];
                    }
                    Err(e) => {
                        log::debug!(
                            "gen_image() failed in regenerate_rendering_current_view() for stroke with key: {:?}, with Err {}",
                            key,
                            e
                        )
                    }
                }
            } else {
                log::debug!(
                    "get stroke, render_comp returned None in regenerate_rendering_current_view() for stroke with key {:?}",
                    key
                );
            }
        })
    }

    pub fn regenerate_strokes_current_view_threaded(
        &mut self,
        viewport: Option<p2d::bounding_volume::AABB>,
        force_regenerate: bool,
    ) {
        let keys = self.render_components.keys().collect::<Vec<StrokeKey>>();

        keys.iter().for_each(|&key| {
            if let (Some(stroke), Some(render_comp)) =
                (self.strokes.get(key), self.render_components.get_mut(key))
            {
                // skip if stroke is not in viewport or does not need regeneration
                if let Some(viewport) = viewport {
                    if !viewport.intersects(&stroke.bounds()) {
                        return;
                    }
                }
                if !force_regenerate && !render_comp.regenerate_flag {
                    return;
                }

                self.update_geometry_for_stroke(key);
                self.regenerate_rendering_for_stroke_threaded(key);
            } else {
                log::debug!(
                    "get stroke, render_comp returned None in regenerate_rendering_current_view_threaded() for stroke with key {:?}",
                    key
                );
            }
        })
    }

    /// Calculates the height needed to fit all strokes
    pub fn calc_height(&self) -> i32 {
        let new_height = if let Some(stroke) = self
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
            .max_by_key(|&stroke| stroke.bounds().maxs[1].round() as i32)
        {
            // max_by_key() returns the element, so we need to extract the height again
            stroke.bounds().maxs[1].round() as i32
        } else {
            0
        };

        new_height
    }

    /// Generates the bounds needed to fit the strokes
    pub fn gen_bounds(&self, keys: &[StrokeKey]) -> Option<p2d::bounding_volume::AABB> {
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

    /// Generates a Svg for all strokes as drawn onto the canvas without xml headers or svg roots. Does not include the selection.
    pub fn gen_svgs_for_strokes(&self) -> Result<Vec<render::Svg>, anyhow::Error> {
        let chrono_sorted = self.keys_sorted_chrono();

        let svgs = chrono_sorted
            .iter()
            .filter(|&&key| {
                self.does_render(key).unwrap_or(false)
                    && !(self.trashed(key).unwrap_or(false))
                    && !(self.selected(key).unwrap_or(false))
                    && (self.does_render(key).unwrap_or(false))
            })
            .filter_map(|&key| {
                let stroke = self.strokes.get(key)?;

                match stroke.gen_svgs(na::vector![0.0, 0.0]) {
                    Ok(svgs) => Some(svgs),
                    Err(e) => {
                        log::error!(
                            "stroke.gen_svgs() failed in gen_svg_all_strokes() with Err {}",
                            e
                        );
                        None
                    }
                }
            })
            .flatten()
            .collect::<Vec<render::Svg>>();

        Ok(svgs)
    }

    pub fn translate_strokes(&mut self, strokes: &[StrokeKey], offset: na::Vector2<f64>) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = self.strokes.get_mut(key) {
                stroke.translate(offset);

                if let Some(render_comp) = self.render_components.get_mut(key) {
                    for image in render_comp.images.iter_mut() {
                        image.bounds = geometry::aabb_translate(image.bounds, offset);
                    }

                    if let Some(new_rendernode) =
                        render::images_to_rendernode(&render_comp.images, self.zoom)
                    {
                        render_comp.rendernode = new_rendernode;
                    }
                }
            }
        });
    }

    pub fn resize_strokes(
        &mut self,
        strokes: &[StrokeKey],
        old_bounds: p2d::bounding_volume::AABB,
        new_bounds: p2d::bounding_volume::AABB,
    ) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = self.strokes.get_mut(key) {
                let old_stroke_bounds = stroke.bounds();
                let new_stroke_bounds = geometry::scale_inner_bounds_to_new_outer_bounds(
                    stroke.bounds(),
                    old_bounds,
                    new_bounds,
                );
                stroke.resize(new_stroke_bounds);

                if let Some(render_comp) = self.render_components.get_mut(key) {
                    for image in render_comp.images.iter_mut() {
                        image.bounds = geometry::scale_inner_bounds_to_new_outer_bounds(
                            image.bounds,
                            old_stroke_bounds,
                            new_stroke_bounds,
                        )
                    }

                    if let Some(new_rendernode) =
                        render::images_to_rendernode(&render_comp.images, self.zoom)
                    {
                        render_comp.rendernode = new_rendernode;
                    }
                    render_comp.regenerate_flag = true;
                }
            }
        });
    }

    /// Returns all strokes below the y_pos
    pub fn strokes_below_y_pos(&self, y_pos: f64) -> Vec<StrokeKey> {
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

    pub fn drag_strokes_proximity(&mut self, drag_proximity_tool: &DragProximityTool) {
        let sphere = p2d::bounding_volume::BoundingSphere {
            center: na::Point2::from(drag_proximity_tool.pos),
            radius: drag_proximity_tool.radius,
        };
        let tool_bounds = geometry::aabb_new_positive(
            na::vector![
                drag_proximity_tool.pos[0] - drag_proximity_tool.radius,
                drag_proximity_tool.pos[1] - drag_proximity_tool.radius
            ],
            na::vector![
                drag_proximity_tool.pos[0] + drag_proximity_tool.radius,
                drag_proximity_tool.pos[1] + drag_proximity_tool.radius
            ],
        );

        self.strokes
            .iter_mut()
            .par_bridge()
            .filter_map(|(key, stroke)| match stroke {
                StrokeStyle::MarkerStroke(markerstroke) => {
                    if markerstroke.bounds().intersects(&tool_bounds) {
                        markerstroke.elements.iter_mut().for_each(|element| {
                            if sphere
                                .contains_local_point(&na::Point2::from(element.inputdata.pos()))
                            {
                                // Zero when right at drag_proximity_tool position, One when right at the radius
                                let distance_ratio = (1.0
                                    - (element.inputdata.pos() - drag_proximity_tool.pos)
                                        .magnitude()
                                        / drag_proximity_tool.radius)
                                    .clamp(0.0, 1.0);

                                element.inputdata.set_pos(
                                    element.inputdata.pos()
                                        + drag_proximity_tool.offset * distance_ratio,
                                );
                            }
                        });
                        Some(key)
                    } else {
                        None
                    }
                }
                StrokeStyle::BrushStroke(brushstroke) => {
                    if brushstroke.bounds().intersects(&tool_bounds) {
                        brushstroke.elements.iter_mut().for_each(|element| {
                            if sphere
                                .contains_local_point(&na::Point2::from(element.inputdata.pos()))
                            {
                                // Zero when right at drag_proximity_tool position, One when right at the radius
                                let distance_ratio = (1.0
                                    - (element.inputdata.pos() - drag_proximity_tool.pos)
                                        .magnitude()
                                        / drag_proximity_tool.radius)
                                    .clamp(0.0, 1.0);

                                element.inputdata.set_pos(
                                    element.inputdata.pos()
                                        + drag_proximity_tool.offset * distance_ratio,
                                );
                            }
                        });
                        Some(key)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<StrokeKey>>()
            .iter()
            .for_each(|&key| {
                self.update_geometry_for_stroke(key);
                self.regenerate_rendering_for_stroke_threaded(key);
            })
    }
}
