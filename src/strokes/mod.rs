pub mod bitmapimage;
pub mod brushstroke;
pub mod markerstroke;
pub mod shapestroke;
pub mod vectorimage;

pub mod chrono_comp;
pub mod render_comp;
pub mod selection_comp;
pub mod trash_comp;

/*
Conventions:
Coordinates in 2d space: origin is thought of in top-left corner of the screen.
Vectors / Matrices in 2D space:
    Vector2: first element is the x-axis, second element is the y-axis
    Matrix2: representing bounds / a rectangle, the coordinate (0,0) is the x-axis of the upper-left corner, (0,1) is the y-axis of the upper-left corner,
        (1,0) is the x-axis of the bottom-right corner, (1,1) is the y-axis of the bottom-right corner.
*/

use crate::{pens::PenStyle, pens::Pens, render};
use chrono_comp::ChronoComponent;
use render_comp::RenderComponent;
use selection_comp::SelectionComponent;
use trash_comp::TrashComponent;

use self::{
    bitmapimage::BitmapImage, brushstroke::BrushStroke, markerstroke::MarkerStroke,
    shapestroke::ShapeStroke, vectorimage::VectorImage,
};

use gtk4::gsk;
use p2d::bounding_volume::BoundingVolume;
use rand::{distributions::Uniform, prelude::Distribution};
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};

slotmap::new_key_type! {
    pub struct StrokeKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokesState {
    // Components
    strokes: HopSlotMap<StrokeKey, StrokeStyle>,
    render_components: SecondaryMap<StrokeKey, Option<RenderComponent>>,
    trash_components: SecondaryMap<StrokeKey, Option<TrashComponent>>,
    selection_components: SecondaryMap<StrokeKey, Option<SelectionComponent>>,
    chrono_components: SecondaryMap<StrokeKey, Option<ChronoComponent>>,

    // Other state
    /// value is equal chrono_component of the newest inserted stroke.
    chrono_counter: u64,
    #[serde(skip)]
    pub scalefactor: f64, // changes with the canvas scalefactor
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
            render_components: SecondaryMap::new(),
            chrono_components: SecondaryMap::new(),

            chrono_counter: 0,
            scalefactor: 1.0,
            renderer: render::Renderer::default(),
            selection_bounds: None,
        }
    }
}

impl StrokesState {
    pub fn new() -> Self {
        Self::default()
    }

    // returns true if resizing is needed
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
            PenStyle::Eraser | PenStyle::Selector | PenStyle::Unkown => None,
        }
    }

    pub fn insert_stroke(&mut self, stroke: StrokeStyle) -> StrokeKey {
        let key = self.strokes.insert(stroke);
        self.chrono_counter += 1;

        self.trash_components
            .insert(key, Some(TrashComponent::default()));
        self.selection_components
            .insert(key, Some(SelectionComponent::default()));
        self.render_components
            .insert(key, Some(RenderComponent::default()));
        self.chrono_components
            .insert(key, Some(ChronoComponent::new(self.chrono_counter)));

        self.update_rendering_for_stroke(key);
        key
    }

    pub fn remove_stroke(&mut self, key: StrokeKey) -> Option<StrokeStyle> {
        self.trash_components.remove(key);
        self.selection_components.remove(key);
        self.render_components.remove(key);
        self.chrono_components.remove(key);

        self.strokes.remove(key)
    }

    pub fn last_stroke_key(&self) -> Option<StrokeKey> {
        let mut sorted = self
            .chrono_components
            .iter()
            .filter_map(|(key, chrono_comp)| {
                if let (Some(Some(trash_comp)), Some(chrono_comp)) =
                    (self.trash_components.get(key), chrono_comp)
                {
                    if !trash_comp.trashed {
                        return Some((key, chrono_comp.t));
                    }
                }
                None
            })
            .collect::<Vec<(StrokeKey, u64)>>();
        sorted.sort_unstable_by(|first, second| first.1.cmp(&second.1));

        let last_stroke_key = sorted.last().copied();
        if let Some(last_stroke_key) = last_stroke_key {
            Some(last_stroke_key.0)
        } else {
            None
        }
    }

    /// returns key to last stroke
    pub fn add_to_last_stroke(&mut self, element: Element, pens: &Pens) -> StrokeKey {
        let key = if let Some((key, stroke)) = self.strokes.iter_mut().last() {
            match stroke {
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
            key
        } else {
            self.insert_stroke(StrokeStyle::BrushStroke(BrushStroke::new(
                element,
                pens.brush.clone(),
            )))
        };

        self.update_rendering_for_stroke(key);

        key
    }

    /// Clears every stroke and every component
    pub fn clear(&mut self) {
        self.chrono_counter = 0;

        self.strokes.clear();
        self.trash_components.clear();
        self.selection_components.clear();
        self.render_components.clear();
        self.chrono_components.clear();
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

        self.update_rendering_for_stroke(key);
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
}

pub trait StrokeBehaviour {
    // returns the bounds of the type
    fn bounds(&self) -> p2d::bounding_volume::AABB;
    // translates (as in moves) the type for offset
    fn translate(&mut self, offset: na::Vector2<f64>);
    // resizes the type to the desired new_bounds
    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB);
    // gen_svg_data() generates the svg elements as a String, without the xml header or the svg root.
    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn std::error::Error>>;
    // generates and returns the rendernode for this type
    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrokeStyle {
    MarkerStroke(MarkerStroke),
    BrushStroke(BrushStroke),
    ShapeStroke(ShapeStroke),
    VectorImage(VectorImage),
    BitmapImage(BitmapImage),
}

impl StrokeBehaviour for StrokeStyle {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.bounds,
            Self::BrushStroke(brushstroke) => brushstroke.bounds,
            Self::ShapeStroke(shapestroke) => shapestroke.bounds,
            Self::VectorImage(vectorimage) => vectorimage.bounds,
            Self::BitmapImage(bitmapimage) => bitmapimage.bounds,
        }
    }
    fn translate(&mut self, offset: na::Vector2<f64>) {
        match self {
            Self::MarkerStroke(markerstroke) => {
                markerstroke.translate(offset);
            }
            Self::BrushStroke(brushstroke) => {
                brushstroke.translate(offset);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.translate(offset);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.translate(offset);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.translate(offset);
            }
        }
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        match self {
            Self::MarkerStroke(markerstroke) => {
                markerstroke.resize(new_bounds);
            }
            Self::BrushStroke(brushstroke) => {
                brushstroke.resize(new_bounds);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.resize(new_bounds);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.resize(new_bounds);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.resize(new_bounds);
            }
        }
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.gen_svg_data(offset),
            Self::BrushStroke(brushstroke) => brushstroke.gen_svg_data(offset),
            Self::ShapeStroke(shapestroke) => shapestroke.gen_svg_data(offset),
            Self::VectorImage(vectorimage) => vectorimage.gen_svg_data(offset),
            Self::BitmapImage(bitmapimage) => bitmapimage.gen_svg_data(offset),
        }
    }

    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn std::error::Error>> {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.gen_rendernode(scalefactor, renderer),
            Self::BrushStroke(brushstroke) => brushstroke.gen_rendernode(scalefactor, renderer),
            Self::ShapeStroke(shapestroke) => shapestroke.gen_rendernode(scalefactor, renderer),
            Self::VectorImage(vectorimage) => vectorimage.gen_rendernode(scalefactor, renderer),
            Self::BitmapImage(bitmapimage) => bitmapimage.gen_rendernode(scalefactor, renderer),
        }
    }
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::MarkerStroke(MarkerStroke::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputData {
    pos: na::Vector2<f64>,
    pressure: f64,
}

impl Default for InputData {
    fn default() -> Self {
        Self {
            pos: na::vector![0.0, 0.0],
            pressure: Self::PRESSURE_DEFAULT,
        }
    }
}

impl InputData {
    pub const PRESSURE_DEFAULT: f64 = 0.5;

    pub fn new(pos: na::Vector2<f64>, pressure: f64) -> Self {
        let mut inputdata = Self::default();
        inputdata.set_pos(pos);
        inputdata.set_pressure(pressure);

        inputdata
    }

    pub fn pos(&self) -> na::Vector2<f64> {
        self.pos
    }

    pub fn set_pos(&mut self, pos: na::Vector2<f64>) {
        self.pos = pos;
    }

    pub fn pressure(&self) -> f64 {
        self.pressure
    }

    pub fn set_pressure(&mut self, pressure: f64) {
        self.pressure = pressure.clamp(0.0, 1.0);
    }
}

// Represents a single Stroke Element
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Element {
    inputdata: InputData,
}

impl Element {
    pub fn new(inputdata: InputData) -> Self {
        Self { inputdata }
    }

    pub fn inputdata(&self) -> InputData {
        self.inputdata.clone()
    }

    pub fn set_inputdata(&mut self, inputdata: InputData) {
        self.inputdata = inputdata;
    }

    pub fn validation_data(bounds: p2d::bounding_volume::AABB) -> Vec<Self> {
        let mut rng = rand::thread_rng();
        let data_entries_uniform = Uniform::from(0..=20);
        let x_uniform = Uniform::from(bounds.mins[0]..=bounds.maxs[0]);
        let y_uniform = Uniform::from(bounds.mins[1]..=bounds.maxs[1]);
        let pressure_uniform = Uniform::from(0_f64..=1_f64);

        let mut data_entries: Vec<Self> = Vec::new();

        for _i in 0..=data_entries_uniform.sample(&mut rng) {
            data_entries.push(Self::new(InputData::new(
                na::vector![x_uniform.sample(&mut rng), y_uniform.sample(&mut rng)],
                pressure_uniform.sample(&mut rng),
            )));
        }

        data_entries
    }
}
