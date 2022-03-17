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

use crate::compose::geometry::{self, AABBHelpers};
use crate::compose::transformable::Transformable;
use crate::drawbehaviour::DrawBehaviour;
use crate::pens::shaper::Shaper;
use crate::pens::tools::DragProximityTool;
use crate::pens::PenStyle;
use crate::render::{self, Renderer};
use crate::strokes::bitmapimage::BitmapImage;
use crate::strokes::element::Element;
use crate::strokes::strokestyle::StrokeStyle;
use crate::strokes::vectorimage::VectorImage;
use crate::surfaceflags::SurfaceFlags;

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
    `.stoke_keys_in_order_rendering` returns keys in the order which they should be rendered.
*/

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
#[serde(default, rename = "strokes_state")]
pub struct StrokesState {
    // Components
    #[serde(rename = "strokes")]
    strokes: HopSlotMap<StrokeKey, StrokeStyle>,
    #[serde(rename = "trash_components")]
    trash_components: SecondaryMap<StrokeKey, TrashComponent>,
    #[serde(rename = "selection_components")]
    selection_components: SecondaryMap<StrokeKey, SelectionComponent>,
    #[serde(rename = "chrono_components")]
    chrono_components: SecondaryMap<StrokeKey, ChronoComponent>,
    #[serde(rename = "render_components")]
    render_components: SecondaryMap<StrokeKey, RenderComponent>,

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
    pub threadpool: rayon::ThreadPool,
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

    // A new strokes state should always be imported with this method, to not replace the threadpool, channel handlers..
    pub fn import_strokes_state(&mut self, strokes_state: Self) {
        self.strokes = strokes_state.strokes;
        self.trash_components = strokes_state.trash_components;
        self.selection_components = strokes_state.selection_components;
        self.chrono_components = strokes_state.chrono_components;
        self.render_components = strokes_state.render_components;
        self.chrono_counter = strokes_state.chrono_counter;
    }

    /// processes the received task from tasks_rx.
    /// Returns surface flags for what to update in the frontend UI.
    pub fn process_received_task(
        &mut self,
        task: StateTask,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();

        match task {
            StateTask::UpdateStrokeWithImages { key, images } => {
                self.regenerate_rendering_with_images(key, images, zoom);

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;
            }
            StateTask::AppendImagesToStroke { key, images } => {
                self.append_images_to_rendering(key, images, zoom);

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;
            }
            StateTask::InsertStroke { stroke } => match stroke {
                StrokeStyle::BrushStroke(brushstroke) => {
                    let inserted = self.insert_stroke(StrokeStyle::BrushStroke(brushstroke));

                    self.regenerate_rendering_for_stroke_threaded(inserted, renderer, zoom);

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.sheet_changed = true;
                }
                StrokeStyle::ShapeStroke(shapestroke) => {
                    let inserted = self.insert_stroke(StrokeStyle::ShapeStroke(shapestroke));

                    self.regenerate_rendering_for_stroke_threaded(inserted, renderer, zoom);

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.sheet_changed = true;
                }
                StrokeStyle::VectorImage(vectorimage) => {
                    let inserted = self.insert_stroke(StrokeStyle::VectorImage(vectorimage));
                    self.set_selected(inserted, true);

                    self.regenerate_rendering_for_stroke_threaded(inserted, renderer, zoom);

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.resize_to_fit_strokes = true;
                    surface_flags.pen_change = Some(PenStyle::SelectorStyle);
                    surface_flags.sheet_changed = true;
                    surface_flags.selection_changed = true;
                }
                StrokeStyle::BitmapImage(bitmapimage) => {
                    let inserted = self.insert_stroke(StrokeStyle::BitmapImage(bitmapimage));

                    self.set_selected(inserted, true);

                    self.regenerate_rendering_for_stroke_threaded(inserted, renderer, zoom);

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.resize_to_fit_strokes = true;
                    surface_flags.pen_change = Some(PenStyle::SelectorStyle);
                    surface_flags.sheet_changed = true;
                    surface_flags.selection_changed = true;
                }
            },
            StateTask::Quit => {
                surface_flags.quit = true;
            }
        }

        surface_flags
    }

    pub fn insert_stroke(&mut self, stroke: StrokeStyle) -> StrokeKey {
        let key = self.strokes.insert(stroke);
        self.chrono_counter += 1;

        let mut render_comp = RenderComponent::default();
        // set flag for rendering regeneration
        render_comp.regenerate_flag = true;

        self.trash_components.insert(key, TrashComponent::default());
        self.selection_components
            .insert(key, SelectionComponent::default());
        self.render_components.insert(key, render_comp);
        self.chrono_components
            .insert(key, ChronoComponent::new(self.chrono_counter));

        key
    }

    pub fn remove_stroke(&mut self, key: StrokeKey) -> Option<StrokeStyle> {
        self.trash_components.remove(key);
        self.selection_components.remove(key);
        self.chrono_components.remove(key);
        self.render_components.remove(key);

        self.strokes.remove(key)
    }

    pub fn add_to_brushstroke(
        &mut self,
        key: StrokeKey,
        element: Element,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        if let Some(StrokeStyle::BrushStroke(ref mut brushstroke)) = self.strokes.get_mut(key) {
            brushstroke.push_elem(element);
        }

        self.append_rendering_new_elem_threaded(key, renderer, zoom);
    }

    pub fn add_to_shapestroke(
        &mut self,
        key: StrokeKey,
        shaper: &mut Shaper,
        element: Element,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        if let Some(StrokeStyle::ShapeStroke(ref mut shapestroke)) = self.strokes.get_mut(key) {
            shapestroke.update_shape(shaper, element);
        }

        self.append_rendering_new_elem_threaded(key, renderer, zoom);
    }

    /// Clears every stroke and every component
    pub fn clear(&mut self) {
        self.chrono_counter = 0;

        self.strokes.clear();
        self.trash_components.clear();
        self.selection_components.clear();
        self.chrono_components.clear();
        self.render_components.clear();
    }

    /// Returns the stroke keys in the order that they should be rendered. Does not return the selection keys!
    pub fn keys_as_rendered(&self) -> Vec<StrokeKey> {
        let keys_sorted_chrono = self.keys_sorted_chrono();

        keys_sorted_chrono
            .iter()
            .filter_map(|&key| {
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

    pub fn keys_intersecting_bounds(&self, bounds: AABB) -> Vec<StrokeKey> {
        self.keys_as_rendered()
            .iter()
            .filter_map(|&key| {
                let stroke = self.strokes.get(key)?;
                if stroke.bounds().intersects(&bounds) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub fn clone_strokes_for_keys(&self, keys: &[StrokeKey]) -> Vec<StrokeStyle> {
        keys.iter()
            .filter_map(|&key| Some(self.strokes.get(key)?.clone()))
            .collect::<Vec<StrokeStyle>>()
    }

    pub fn insert_vectorimage_bytes_threaded(
        &mut self,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
        renderer: Arc<RwLock<Renderer>>,
    ) {
        let tasks_tx = self.tasks_tx.clone();

        self.threadpool.spawn(move || {
                match String::from_utf8(bytes) {
                    Ok(svg) => {
                        match VectorImage::import_from_svg_data(svg.as_str(), pos, None, renderer) {
                            Ok(vectorimage) => {
                                let vectorimage = StrokeStyle::VectorImage(vectorimage);

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

        self.threadpool.spawn(move || {
                match BitmapImage::import_from_image_bytes(&bytes, pos) {
                    Ok(bitmapimage) => {
                        let bitmapimage = StrokeStyle::BitmapImage(bitmapimage);

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
        renderer: Arc<RwLock<Renderer>>,
    ) {
        let tasks_tx = self.tasks_tx.clone();

        self.threadpool.spawn(move || {
                match VectorImage::import_from_pdf_bytes(&bytes, pos, page_width, renderer) {
                    Ok(images) => {
                        for image in images {
                            let image = StrokeStyle::VectorImage(image);

                            tasks_tx.unbounded_send(StateTask::InsertStroke {
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

    pub fn insert_pdf_bytes_as_bitmap_threaded(
        &mut self,
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
        bytes: Vec<u8>,
    ) {
        let tasks_tx = self.tasks_tx.clone();

        self.threadpool.spawn(move || {
                match BitmapImage::import_from_pdf_bytes(&bytes, pos, page_width) {
                    Ok(images) => {
                        for image in images {
                            let image = StrokeStyle::BitmapImage(image);

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

    pub fn import_state(&mut self, strokes_state: &Self) {
        self.clear();
        self.chrono_counter = strokes_state.chrono_counter;

        self.strokes = strokes_state.strokes.clone();
        self.trash_components = strokes_state.trash_components.clone();
        self.selection_components = strokes_state.selection_components.clone();
        self.chrono_components = strokes_state.chrono_components.clone();
        self.render_components = strokes_state.render_components.clone();
    }

    pub fn update_geometry_for_stroke(&mut self, key: StrokeKey) {
        if let Some(stroke) = self.strokes.get_mut(key) {
            match stroke {
                StrokeStyle::BrushStroke(ref mut brushstroke) => {
                    brushstroke.update_geometry();
                }
                StrokeStyle::ShapeStroke(shapestroke) => {
                    shapestroke.update_geometry();
                }
                StrokeStyle::VectorImage(ref mut vectorimage) => {
                    vectorimage.update_geometry();
                }
                StrokeStyle::BitmapImage(ref mut bitmapimage) => {
                    bitmapimage.update_geometry();
                }
            }

            // set flag for rendering regeneration
            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.regenerate_flag = true;
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
        let keys: Vec<StrokeKey> = self.selection_keys_as_rendered();

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

    pub fn strokes_bounds(&self, keys: &[StrokeKey]) -> Vec<AABB> {
        keys.iter()
            .filter_map(|&key| Some(self.strokes.get(key)?.bounds()))
            .collect::<Vec<AABB>>()
    }

    pub fn gen_svgs_for_bounds(&self, bounds: AABB) -> Vec<render::Svg> {
        let keys = self.keys_as_rendered();

        keys.iter()
            .filter_map(|&key| {
                let stroke = self.strokes.get(key)?;
                if !stroke.bounds().intersects(&bounds) {
                    return None;
                }

                match stroke.gen_svgs(na::vector![0.0, 0.0]) {
                    Ok(svgs) => Some(svgs),
                    Err(e) => {
                        log::error!(
                            "stroke.gen_svgs() failed in gen_svg_for_bounds() with Err {}",
                            e
                        );
                        None
                    }
                }
            })
            .flatten()
            .collect::<Vec<render::Svg>>()
    }

    /// Generates a Svg for all strokes as drawn onto the canvas without xml headers or svg roots. Does not include the selection.
    pub fn gen_svgs_all_strokes(&self) -> Vec<render::Svg> {
        let keys = self.keys_as_rendered();

        keys.iter()
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
            .collect::<Vec<render::Svg>>()
    }

    /// Translate the strokes with the offset
    pub fn translate_strokes(
        &mut self,
        strokes: &[StrokeKey],
        offset: na::Vector2<f64>,
        zoom: f64,
    ) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = self.strokes.get_mut(key) {
                stroke.translate(offset);

                if let Some(render_comp) = self.render_components.get_mut(key) {
                    for image in render_comp.images.iter_mut() {
                        image.bounds = image.bounds.translate(offset);
                    }

                    match render::images_to_rendernode(&render_comp.images, zoom) {
                        Ok(Some(rendernode)) => {
                            render_comp.rendernode = Some(rendernode);
                        }
                        Ok(None) => {}
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
    pub fn rotate_strokes(
        &mut self,
        strokes: &[StrokeKey],
        angle: f64,
        center: na::Point2<f64>,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = self.strokes.get_mut(key) {
                stroke.rotate(angle, center);

                self.regenerate_rendering_for_stroke(key, Arc::clone(&renderer), zoom);
            }
        });
    }

    // Resizes the strokes to new bounds
    pub fn resize_strokes(
        &mut self,
        strokes: &[StrokeKey],
        old_bounds: AABB,
        new_bounds: AABB,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        strokes.iter().for_each(|&key| {
            if let Some(stroke) = self.strokes.get_mut(key) {
                let old_stroke_bounds = stroke.bounds();
                let new_stroke_bounds = geometry::scale_inner_bounds_to_new_outer_bounds(
                    stroke.bounds(),
                    old_bounds,
                    new_bounds,
                );

                let offset = new_stroke_bounds.center() - old_stroke_bounds.center();
                let scale = new_stroke_bounds
                    .extents()
                    .component_div(&old_stroke_bounds.extents());

                stroke.translate(offset);
                stroke.scale(scale);

                self.regenerate_rendering_for_stroke(key, Arc::clone(&renderer), zoom);
            }
        });
    }

    /// Returns all strokes below the y_pos
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

    pub fn drag_strokes_proximity(
        &mut self,
        drag_proximity_tool: &DragProximityTool,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        let sphere = BoundingSphere {
            center: na::Point2::from(drag_proximity_tool.pos),
            radius: drag_proximity_tool.radius,
        };
        let tool_bounds = AABB::new_positive(
            na::point![
                drag_proximity_tool.pos[0] - drag_proximity_tool.radius,
                drag_proximity_tool.pos[1] - drag_proximity_tool.radius
            ],
            na::point![
                drag_proximity_tool.pos[0] + drag_proximity_tool.radius,
                drag_proximity_tool.pos[1] + drag_proximity_tool.radius
            ],
        );

        self.strokes
            .iter_mut()
            .par_bridge()
            .filter_map(|(key, stroke)| match stroke {
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
                self.regenerate_rendering_for_stroke(key, Arc::clone(&renderer), zoom);
            });
    }
}
