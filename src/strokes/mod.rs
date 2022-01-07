pub mod bitmapimage;
pub mod brushstroke;
pub mod markerstroke;
pub mod shapestroke;
pub mod strokebehaviour;
pub mod strokestyle;
pub mod vectorimage;

pub mod chrono_comp;
pub mod render_comp;
pub mod selection_comp;
pub mod trash_comp;

use std::sync::{Arc, RwLock};

use crate::compose;
use crate::drawbehaviour::DrawBehaviour;
use crate::ui::appwindow::RnoteAppWindow;
use crate::{pens::PenStyle, pens::Pens, render};
use chrono_comp::ChronoComponent;
use render_comp::RenderComponent;
use selection_comp::SelectionComponent;
use trash_comp::TrashComponent;

use self::strokebehaviour::StrokeBehaviour;
use self::strokestyle::{Element, StrokeStyle};
use self::{brushstroke::BrushStroke, markerstroke::MarkerStroke, shapestroke::ShapeStroke};

use gtk4::{glib, glib::clone, prelude::*};
use p2d::bounding_volume::BoundingVolume;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
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
            glib::MainContext::channel::<StateTask>(glib::PRIORITY_DEFAULT);

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

impl Drop for StrokesState {
    fn drop(&mut self) {
        //let _ = self.render_tx.send(Command::Quit);
        if let Some(source) = self.channel_source.take() {
            source.destroy();
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
                    StateTask::Quit => return glib::Continue(false),
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

    pub fn new_stroke(
        &mut self,
        element: Element,
        current_pen: PenStyle,
        pens: &Pens,
    ) -> Option<StrokeKey> {
        match current_pen {
            PenStyle::Marker => {
                let markerstroke =
                    StrokeStyle::MarkerStroke(MarkerStroke::new(element, pens.marker.clone()));

                Some(self.insert_stroke(markerstroke))
            }
            PenStyle::Brush => {
                let brushstroke =
                    StrokeStyle::BrushStroke(BrushStroke::new(element, pens.brush.clone()));

                Some(self.insert_stroke(brushstroke))
            }
            PenStyle::Shaper => {
                let shapestroke =
                    StrokeStyle::ShapeStroke(ShapeStroke::new(element, pens.shaper.clone()));

                Some(self.insert_stroke(shapestroke))
            }
            PenStyle::Eraser | PenStyle::Selector | PenStyle::Tools | PenStyle::Unknown => {
                log::warn!("new_stroke() failed, current_pen is a unsupported PenStyle");
                None
            }
        }
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

        self.append_rendering_new_elem_threaded(key);
        Some(key)
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

    pub fn import_state(&mut self, strokes_state: &Self) {
        self.clear();
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
        let keys: Vec<StrokeKey> = self.keys_selection();
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
                        log::error!(
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

    pub fn gen_svg_all_strokes(&self) -> Result<String, anyhow::Error> {
        let strokes = &self.strokes;

        let keys = self
            .render_components
            .iter()
            .filter_map(|(key, render_comp)| {
                if render_comp.render && !self.trashed(key).unwrap_or(true) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>();

        if keys.len() < 1 {
            return Ok(String::from(""));
        }
        let bounds = if let Some(bounds) = self.gen_bounds(&keys) {
            bounds
        } else {
            return Ok(String::from(""));
        };

        let data: String = keys
            .par_iter()
            .filter_map(|&key| {
                if let Some(stroke) = strokes.get(key) {
                    match stroke.gen_svgs(na::vector![0.0, 0.0]) {
                        Ok(svgs) => return Some(svgs),
                        Err(e) => {
                            log::error!(
                                "stroke.gen_svgs() failed in gen_svg_all_strokes() with Err {}",
                                e
                            );
                        }
                    }
                }
                None
            })
            .flatten()
            .map(|svg| svg.svg_data)
            .collect::<Vec<String>>()
            .join("\n");

        let data = compose::wrap_svg(data.as_str(), Some(bounds), Some(bounds), true, false);

        Ok(data)
    }

    /// Translate all strokes below the y_threshold with the offset in the y-axis
    pub fn translate_strokes_threshold_vertical(&mut self, y_threshold: f64, offset: f64) {
        self.strokes
            .iter_mut()
            .par_bridge()
            .filter_map(|(key, stroke)| {
                if stroke.bounds().mins[1] > y_threshold {
                    stroke.translate(na::vector![0.0, offset]);
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
            .iter()
            .for_each(|&key| {
                self.update_geometry_for_stroke(key);
                self.regenerate_rendering_for_stroke_threaded(key);
            })
    }
}
