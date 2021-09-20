pub mod bitmapimage;
pub mod brushstroke;
pub mod markerstroke;
pub mod vectorimage;

use crate::{config, pens::PenStyle, pens::Pens};

use self::{
    bitmapimage::BitmapImage, brushstroke::BrushStroke, markerstroke::MarkerStroke,
    vectorimage::VectorImage,
};

use std::{error::Error, ops::Deref};

use gtk4::{gio, glib, graphene, gsk, Snapshot};
use p2d::bounding_volume::BoundingVolume;
use rand::{distributions::Uniform, prelude::Distribution};
use serde::{Deserialize, Serialize};

pub trait StrokeBehaviour {
    fn bounds(&self) -> p2d::bounding_volume::AABB;
    fn translate(&mut self, offset: na::Vector2<f64>);
    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB);
    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>>;
    fn update_caironode(&mut self, scalefactor: f64);
    fn gen_caironode(&self, scalefactor: f64) -> Result<gsk::CairoNode, Box<dyn Error>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrokeStyle {
    MarkerStroke(MarkerStroke),
    BrushStroke(BrushStroke),
    VectorImage(VectorImage),
    BitmapImage(BitmapImage),
}

impl StrokeBehaviour for StrokeStyle {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.bounds,
            Self::BrushStroke(brushstroke) => brushstroke.bounds,
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
            Self::VectorImage(vectorimage) => vectorimage.gen_svg_data(offset),
            Self::BitmapImage(bitmapimage) => bitmapimage.gen_svg_data(offset),
        }
    }

    fn update_caironode(&mut self, scalefactor: f64) {
        match self {
            Self::MarkerStroke(markerstroke) => {
                markerstroke.update_caironode(scalefactor);
            }
            Self::BrushStroke(brushstroke) => {
                brushstroke.update_caironode(scalefactor);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.update_caironode(scalefactor);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.update_caironode(scalefactor);
            }
        }
    }

    fn gen_caironode(&self, scalefactor: f64) -> Result<gsk::CairoNode, Box<dyn Error>> {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.gen_caironode(scalefactor),
            Self::BrushStroke(brushstroke) => brushstroke.gen_caironode(scalefactor),
            Self::VectorImage(vectorimage) => vectorimage.gen_caironode(scalefactor),
            Self::BitmapImage(bitmapimage) => bitmapimage.gen_caironode(scalefactor),
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
            StrokeStyle::VectorImage(_vectorimage) => {}
            StrokeStyle::BitmapImage(_bitmapimage) => {}
        }
    }

    pub fn complete_all_strokes(strokes: &mut Vec<Self>) {
        for stroke in strokes {
            stroke.complete_stroke();
        }
    }

    pub fn gen_bounds(strokes: &Vec<Self>) -> Option<p2d::bounding_volume::AABB> {
        let mut strokes_iter = strokes.iter();

        if let Some(first) = strokes_iter.next() {
            let mut bounds = match first {
                StrokeStyle::MarkerStroke(markerstroke) => markerstroke.bounds,
                StrokeStyle::BrushStroke(brushstroke) => brushstroke.bounds,
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
        inputdata: InputData,
        current_pen: PenStyle,
        pens: &Pens,
    ) {
        match current_pen {
            PenStyle::Marker => {
                strokes.push(StrokeStyle::MarkerStroke(MarkerStroke::new(
                    inputdata,
                    pens.marker.clone(),
                )));
            }
            PenStyle::Brush => {
                strokes.push(StrokeStyle::BrushStroke(BrushStroke::new(
                    inputdata,
                    pens.brush.clone(),
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
    pub fn add_to_last_stroke(strokes: &mut Vec<Self>, inputdata: InputData, pens: &Pens) {
        if let Some(strokes) = strokes.last_mut() {
            match strokes {
                StrokeStyle::MarkerStroke(ref mut markerstroke) => {
                    markerstroke.push_elem(inputdata.clone());
                }
                StrokeStyle::BrushStroke(ref mut brushstroke) => {
                    brushstroke.push_elem(inputdata.clone());
                }
                StrokeStyle::VectorImage(_vectorimage) => {}
                StrokeStyle::BitmapImage(_bitmapimage) => {}
            }
        } else {
            strokes.push(StrokeStyle::BrushStroke(BrushStroke::new(
                inputdata.clone(),
                pens.brush.clone(),
            )));
        }
    }

    pub fn update_all_caironodes(strokes: &mut Vec<Self>, scalefactor: f64) {
        for stroke in strokes {
            stroke.update_caironode(scalefactor);
        }
    }

    pub fn register_custom_templates(strokes: &mut Vec<Self>) -> Result<(), Box<dyn Error>> {
        for stroke in strokes {
            match stroke {
                StrokeStyle::MarkerStroke(_markerstroke) => {}
                StrokeStyle::BrushStroke(brushstroke) => {
                    brushstroke.brush.register_custom_template()?;
                }
                StrokeStyle::VectorImage(_vectorimage) => {}
                StrokeStyle::BitmapImage(_bitmapimage) => {}
            }
        }
        Ok(())
    }

    pub fn draw_strokes(strokes: &Vec<Self>, snapshot: &Snapshot) {
        for stroke in strokes.iter() {
            match stroke {
                StrokeStyle::MarkerStroke(markerstroke) => {
                    snapshot.append_node(&markerstroke.caironode);
                }
                StrokeStyle::BrushStroke(brushstroke) => {
                    snapshot.append_node(&brushstroke.caironode);
                }
                StrokeStyle::VectorImage(vectorimage) => {
                    snapshot.append_node(&vectorimage.caironode);
                }
                StrokeStyle::BitmapImage(bitmapimage) => {
                    snapshot.append_node(&bitmapimage.caironode);
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

    pub fn validation_data(bounds: p2d::bounding_volume::AABB) -> Vec<Self> {
        let mut rng = rand::thread_rng();
        let data_entries_uniform = Uniform::from(0..=20);
        let x_uniform = Uniform::from(bounds.mins[0]..=bounds.maxs[0]);
        let y_uniform = Uniform::from(bounds.mins[1]..=bounds.maxs[1]);
        let pressure_uniform = Uniform::from(0_f64..=1_f64);

        let mut data_entries: Vec<Self> = Vec::new();

        for _i in 0..=data_entries_uniform.sample(&mut rng) {
            data_entries.push(Self::new(
                na::vector![x_uniform.sample(&mut rng), y_uniform.sample(&mut rng)],
                pressure_uniform.sample(&mut rng),
            ));
        }

        data_entries
    }
}

// Represents a single Stroke Element
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[allow(dead_code)]
pub fn add_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(r#"<\?xml[^\?>]*\?>"#).unwrap();
    if !re.is_match(svg) {
        let mut string = String::from(r#"<?xml version="1.0" standalone="no"?>"#);
        string.push_str("\n");
        string.push_str(svg);
        string
    } else {
        String::from(svg)
    }
}

pub fn remove_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(r#"<\?xml[^\?>]*\?>"#).unwrap();
    String::from(re.replace_all(svg, ""))
}

#[allow(dead_code)]
pub fn strip_svg_root(svg: &str) -> String {
    let re = regex::Regex::new(r#"<svg[^>]*>|<[^/svg]*/svg>"#).unwrap();
    String::from(re.replace_all(svg, ""))
}

pub fn wrap_svg(
    data: &str,
    bounds: Option<p2d::bounding_volume::AABB>,
    viewbox: Option<p2d::bounding_volume::AABB>,
    xml_header: bool,
    preserve_aspectratio: bool,
) -> String {
    let mut cx = tera::Context::new();

    let (x, y, width, height) = if let Some(bounds) = bounds {
        let x = format!("{}", bounds.mins[0].floor() as i32);
        let y = format!("{}", bounds.mins[1].floor() as i32);
        let width = format!("{}", (bounds.maxs[0] - bounds.mins[0]).ceil() as i32);
        let height = format!("{}", (bounds.maxs[1] - bounds.mins[1]).ceil() as i32);

        (x, y, width, height)
    } else {
        (
            String::from("0"),
            String::from("0"),
            String::from("100%"),
            String::from("100%"),
        )
    };

    let viewbox = if let Some(viewbox) = viewbox {
        format!(
            "viewBox=\"{} {} {} {}\"",
            viewbox.mins[0].floor() as i32,
            viewbox.mins[1].floor() as i32,
            (viewbox.maxs[0] - viewbox.mins[0]).ceil() as i32,
            (viewbox.maxs[1] - viewbox.mins[1]).ceil() as i32
        )
    } else {
        String::from("")
    };
    let preserve_aspectratio = if preserve_aspectratio {
        String::from("xMidyMid")
    } else {
        String::from("none")
    };

    cx.insert("xml_header", &xml_header);
    cx.insert("data", data);
    cx.insert("x", &x);
    cx.insert("y", &y);
    cx.insert("width", &width);
    cx.insert("height", &height);
    cx.insert("viewbox", &viewbox);
    cx.insert("preserve_aspectratio", &preserve_aspectratio);

    let templ = String::from_utf8(
        gio::resources_lookup_data(
            (String::from(config::APP_IDPATH) + "templates/svg_wrap.svg.templ").as_str(),
            gio::ResourceLookupFlags::NONE,
        )
        .unwrap()
        .deref()
        .to_vec(),
    )
    .unwrap();
    let output = tera::Tera::one_off(templ.as_str(), &cx, false)
        .expect("failed to create svg from template");

    output
}

pub fn svg_intrinsic_size(svg: &str) -> Option<na::Vector2<f64>> {
    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));
    if let Ok(handle) = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)
    {
        let renderer = librsvg::CairoRenderer::new(&handle);

        let intrinsic_size = if let Some(size) = renderer.intrinsic_size_in_pixels() {
            Some(na::vector![size.0, size.1])
        } else {
            log::warn!("intrinsic_size_in_pixels() failed in svg_intrinsic_size()");
            None
        };

        return intrinsic_size;
    } else {
        return None;
    }
}

pub fn gen_caironode_for_svg(
    bounds: p2d::bounding_volume::AABB,
    scalefactor: f64,
    svg: &str,
) -> Result<gsk::CairoNode, Box<dyn Error>> {
    let caironode_bounds = graphene::Rect::new(
        (bounds.mins[0] * scalefactor).floor() as f32,
        (bounds.mins[1] * scalefactor).floor() as f32,
        ((bounds.maxs[0] - bounds.mins[0]) * scalefactor).ceil() as f32,
        ((bounds.maxs[1] - bounds.mins[1]) * scalefactor).ceil() as f32,
    );

    let new_node = gsk::CairoNode::new(&caironode_bounds);
    let cx = new_node
        .draw_context()
        .expect("failed to get cairo draw_context() from caironode");

    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));
    let handle = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)?;
    let renderer = librsvg::CairoRenderer::new(&handle);

    renderer.render_document(
        &cx,
        &cairo::Rectangle {
            x: (bounds.mins[0].floor() * scalefactor),
            y: (bounds.mins[1].floor() * scalefactor),
            width: ((bounds.maxs[0] - bounds.mins[0]).ceil() * scalefactor),
            height: ((bounds.maxs[1] - bounds.mins[1]).ceil() * scalefactor),
        },
    )?;
    Ok(new_node)
}

pub fn gen_cairosurface(
    bounds: &p2d::bounding_volume::AABB,
    scalefactor: f64,
    svg: &str,
) -> Result<cairo::ImageSurface, Box<dyn Error>> {
    let width_scaled = (scalefactor * (bounds.maxs[0] - bounds.mins[0])).round() as i32;
    let height_scaled = (scalefactor * (bounds.maxs[1] - bounds.mins[1])).round() as i32;

    let surface =
        cairo::ImageSurface::create(cairo::Format::ARgb32, width_scaled, height_scaled).unwrap();

    // the ImageSurface has scaled size. Draw onto it in the unscaled, original coordinates, and will get scaled with this method .set_device_scale()
    surface.set_device_scale(scalefactor, scalefactor);

    let cx = cairo::Context::new(&surface).expect("Failed to create a cairo context");

    cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);

    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));
    let handle = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)
        .expect("failed to parse xml into librsvg");
    let renderer = librsvg::CairoRenderer::new(&handle);
    renderer.render_document(
        &cx,
        &cairo::Rectangle {
            x: 0.0,
            y: 0.0,
            width: bounds.maxs[0] - bounds.mins[0],
            height: bounds.maxs[1] - bounds.mins[1],
        },
    )?;

    cx.stroke()
        .expect("failed to stroke() cairo context onto cairo surface.");

    Ok(surface)
}
