pub mod bitmapimage;
pub mod brushstroke;
pub mod markerstroke;
pub mod shapestroke;
pub mod strokestyle;
pub mod vectorimage;

pub mod chrono_comp;
pub mod render_comp;
pub mod selection_comp;
pub mod trash_comp;

use crate::{pens::PenStyle, pens::Pens, render};
use chrono_comp::ChronoComponent;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use render_comp::RenderComponent;
use selection_comp::SelectionComponent;
use trash_comp::TrashComponent;

use self::strokestyle::{Element, StrokeBehaviour, StrokeStyle};
use self::{brushstroke::BrushStroke, markerstroke::MarkerStroke, shapestroke::ShapeStroke};

use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};

slotmap::new_key_type! {
    pub struct StrokeKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(skip)]
    pub zoom: f64, // changes with the canvas zoom
    #[serde(skip)]
    pub renderer: render::Renderer,
    pub selection_bounds: Option<p2d::bounding_volume::AABB>,
}

impl Default for StrokesState {
    fn default() -> Self {
        Self {
            strokes: HopSlotMap::with_key(),
            trash_components: SecondaryMap::new(),
            selection_components: SecondaryMap::new(),
            chrono_components: SecondaryMap::new(),
            render_components: SecondaryMap::new(),

            chrono_counter: 0,
            zoom: 1.0,
            renderer: render::Renderer::default(),
            selection_bounds: None,
        }
    }
}

impl StrokesState {
    pub fn new() -> Self {
        Self::default()
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
            PenStyle::Eraser | PenStyle::Selector | PenStyle::Unkown => {
                log::warn!("tried inserting a new stroke with an unsupported PenStyle");
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

        self.update_rendering_for_stroke(key);
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
    pub fn add_to_last_stroke(&mut self, element: Element) -> Option<StrokeKey> {
        if let Some(key) = self.last_stroke_key() {
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

            self.update_rendering_for_stroke(key);
            Some(key)
        } else {
            log::warn!("tried add_to_last_stroke() while there is no last stroke");
            None
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
    }

    pub fn import_state(&mut self, strokes_state: &Self) {
        self.clear();
        self.strokes = strokes_state.strokes.clone();
        self.trash_components = strokes_state.trash_components.clone();
        self.selection_components = strokes_state.selection_components.clone();
        self.chrono_components = strokes_state.chrono_components.clone();
        self.render_components = strokes_state.render_components.clone();

        self.update_rendering_current_view(None, true);
    }

    /// Returns the key to the completed stroke
    pub fn complete_stroke(&mut self, key: StrokeKey) {
        if let Some(stroke) = self.strokes.get_mut(key) {
            match stroke {
                StrokeStyle::MarkerStroke(ref mut markerstroke) => {
                    markerstroke.complete_stroke();
                }
                StrokeStyle::BrushStroke(ref mut brushstroke) => {
                    brushstroke.complete_stroke();
                }
                StrokeStyle::ShapeStroke(shapestroke) => {
                    shapestroke.complete_stroke();
                }
                StrokeStyle::VectorImage(ref mut _vectorimage) => {}
                StrokeStyle::BitmapImage(ref mut _bitmapimage) => {}
            }
        }
    }

    pub fn complete_all_strokes(&mut self) {
        let keys: Vec<StrokeKey> = self.strokes.keys().collect();
        keys.iter().for_each(|key| {
            self.complete_stroke(*key);
        });
    }

    /// Calculates the height needed to fit all strokes
    pub fn calc_height(&self) -> i32 {
        let new_height = if let Some(stroke) = self
            .strokes
            .values()
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
    pub fn gen_bounds<'a>(
        &self,
        mut keys: impl Iterator<Item = &'a StrokeKey>,
    ) -> Option<p2d::bounding_volume::AABB> {
        if let Some(first_key) = keys.next() {
            if let Some(first) = self.strokes.get(*first_key) {
                let mut bounds = match first {
                    StrokeStyle::MarkerStroke(markerstroke) => markerstroke.bounds,
                    StrokeStyle::BrushStroke(brushstroke) => brushstroke.bounds,
                    StrokeStyle::ShapeStroke(shapestroke) => shapestroke.bounds,
                    StrokeStyle::VectorImage(vectorimage) => vectorimage.bounds,
                    StrokeStyle::BitmapImage(bitmapimage) => bitmapimage.bounds,
                };

                keys.for_each(|key| match self.strokes.get(*key) {
                    Some(StrokeStyle::MarkerStroke(markerstroke)) => {
                        bounds.merge(&markerstroke.bounds);
                    }
                    Some(StrokeStyle::BrushStroke(brushstroke)) => {
                        bounds.merge(&brushstroke.bounds);
                    }
                    Some(StrokeStyle::ShapeStroke(shapestroke)) => {
                        bounds.merge(&shapestroke.bounds);
                    }
                    Some(StrokeStyle::VectorImage(vectorimage)) => {
                        bounds.merge(&vectorimage.bounds);
                    }
                    Some(StrokeStyle::BitmapImage(bitmapimage)) => {
                        bounds.merge(&bitmapimage.bounds);
                    }
                    None => {}
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
                if render_comp.render && !self.trashed(key).unwrap_or_else(|| true) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>();

        let data: String = keys.par_iter()
            .map(|&key| {
                if let Some(stroke) = strokes.get(key) {
                    match stroke.gen_svg_data(na::vector![0.0, 0.0]) {
                        Ok(data_entry) => {
                            return data_entry
                        }
                        Err(e) => {
                            log::error!("gen_svg_data() failed for stroke with key {:?} in gen_svg_from_strokes(), {}", key, e);
                        }
                    }
                }
                String::new()
            }).collect::<Vec<String>>().join("\n");

        Ok(data)
    }
}
