pub mod bitmapimage;
pub mod brushstroke;
pub mod compose;
pub mod curves;
pub mod markerstroke;
pub mod render;
pub mod shapestroke;
pub mod vectorimage;

use crate::{pens::PenStyle, pens::Pens};

use self::{
    bitmapimage::BitmapImage, brushstroke::BrushStroke, markerstroke::MarkerStroke,
    shapestroke::ShapeStroke, vectorimage::VectorImage,
};

use std::error::Error;

use gtk4::{gsk, Snapshot};
use p2d::bounding_volume::BoundingVolume;
use rand::{distributions::Uniform, prelude::Distribution};
use serde::{Deserialize, Serialize};

pub trait StrokeBehaviour {
    // returns the bounds of the type
    fn bounds(&self) -> p2d::bounding_volume::AABB;
    // translates (as in moves) the type for offset
    fn translate(&mut self, offset: na::Vector2<f64>);
    // resizes the type to the desired new_bounds
    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB);
    // gen_svg_data() generates the svg elements as a String, without the xml header or the svg root.
    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>>;
    // updates the rendernodes for the type implementing this trait
    fn update_rendernode(&mut self, scalefactor: f64, renderer: &render::Renderer);
    // generates and returns the rendernode for this type
    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn Error>>;
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

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.gen_svg_data(offset),
            Self::BrushStroke(brushstroke) => brushstroke.gen_svg_data(offset),
            Self::ShapeStroke(shapestroke) => shapestroke.gen_svg_data(offset),
            Self::VectorImage(vectorimage) => vectorimage.gen_svg_data(offset),
            Self::BitmapImage(bitmapimage) => bitmapimage.gen_svg_data(offset),
        }
    }

    fn update_rendernode(&mut self, scalefactor: f64, renderer: &render::Renderer) {
        match self {
            Self::MarkerStroke(markerstroke) => {
                markerstroke.update_rendernode(scalefactor, renderer);
            }
            Self::BrushStroke(brushstroke) => {
                brushstroke.update_rendernode(scalefactor, renderer);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.update_rendernode(scalefactor, renderer);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.update_rendernode(scalefactor, renderer);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.update_rendernode(scalefactor, renderer);
            }
        }
    }

    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
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

impl StrokeStyle {
    pub fn complete_stroke(&mut self) {
        match self {
            StrokeStyle::MarkerStroke(markerstroke) => {
                markerstroke.complete_stroke();
            }
            StrokeStyle::BrushStroke(brushstroke) => {
                brushstroke.complete_stroke();
            }
            StrokeStyle::ShapeStroke(shapestroke) => {
                shapestroke.complete_stroke();
            }
            StrokeStyle::VectorImage(_vectorimage) => {}
            StrokeStyle::BitmapImage(_bitmapimage) => {}
        }
    }

    pub fn complete_all_strokes(strokes: &mut Vec<Self>) {
        for stroke in strokes {
            stroke.complete_stroke();
        }
    }

    pub fn gen_bounds(strokes: &[Self]) -> Option<p2d::bounding_volume::AABB> {
        let mut strokes_iter = strokes.iter();

        if let Some(first) = strokes_iter.next() {
            let mut bounds = match first {
                StrokeStyle::MarkerStroke(markerstroke) => markerstroke.bounds,
                StrokeStyle::BrushStroke(brushstroke) => brushstroke.bounds,
                StrokeStyle::ShapeStroke(shapestroke) => shapestroke.bounds,
                StrokeStyle::VectorImage(vectorimage) => vectorimage.bounds,
                StrokeStyle::BitmapImage(bitmapimage) => bitmapimage.bounds,
            };

            for stroke in strokes_iter {
                match stroke {
                    StrokeStyle::MarkerStroke(markerstroke) => {
                        bounds.merge(&markerstroke.bounds);
                    }
                    StrokeStyle::BrushStroke(brushstroke) => {
                        bounds.merge(&brushstroke.bounds);
                    }
                    StrokeStyle::ShapeStroke(shapestroke) => {
                        bounds.merge(&shapestroke.bounds);
                    }
                    StrokeStyle::VectorImage(vectorimage) => {
                        bounds.merge(&vectorimage.bounds);
                    }
                    StrokeStyle::BitmapImage(bitmapimage) => {
                        bounds.merge(&bitmapimage.bounds);
                    }
                }
            }
            Some(bounds)
        } else {
            None
        }
    }

    // returns true if resizing is needed
    pub fn new_stroke(
        strokes: &mut Vec<Self>,
        element: Element,
        current_pen: PenStyle,
        pens: &Pens,
    ) {
        match current_pen {
            PenStyle::Marker => {
                strokes.push(StrokeStyle::MarkerStroke(MarkerStroke::new(
                    element,
                    pens.marker.clone(),
                )));
            }
            PenStyle::Brush => {
                strokes.push(StrokeStyle::BrushStroke(BrushStroke::new(
                    element,
                    pens.brush.clone(),
                )));
            }
            PenStyle::Shaper => {
                strokes.push(StrokeStyle::ShapeStroke(ShapeStroke::new(
                    element,
                    pens.shaper.clone(),
                )));
            }
            PenStyle::Eraser | PenStyle::Selector | PenStyle::Unkown => {}
        }
    }

    #[allow(dead_code)]
    pub fn remove_from_strokes(strokes: &mut Vec<Self>, indices: Vec<usize>) {
        for (to_remove_index, i) in indices.iter().enumerate() {
            strokes.remove(i - to_remove_index);
        }
    }

    // returns true if resizing is needed
    pub fn add_to_last_stroke(strokes: &mut Vec<Self>, element: Element, pens: &Pens) {
        if let Some(strokes) = strokes.last_mut() {
            match strokes {
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
        } else {
            strokes.push(StrokeStyle::BrushStroke(BrushStroke::new(
                element,
                pens.brush.clone(),
            )));
        }
    }

    pub fn update_all_rendernodes(
        strokes: &mut Vec<Self>,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) {
        for stroke in strokes {
            stroke.update_rendernode(scalefactor, renderer);
        }
    }

    pub fn draw_strokes(strokes: &[Self], snapshot: &Snapshot) {
        for stroke in strokes.iter() {
            match stroke {
                StrokeStyle::MarkerStroke(markerstroke) => {
                    snapshot.append_node(&markerstroke.rendernode);
                }
                StrokeStyle::BrushStroke(brushstroke) => {
                    snapshot.append_node(&brushstroke.rendernode);
                }
                StrokeStyle::ShapeStroke(shapestroke) => {
                    snapshot.append_node(&shapestroke.rendernode);
                }
                StrokeStyle::VectorImage(vectorimage) => {
                    snapshot.append_node(&vectorimage.rendernode);
                }
                StrokeStyle::BitmapImage(bitmapimage) => {
                    snapshot.append_node(&bitmapimage.rendernode);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
