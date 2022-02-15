use std::sync::{Arc, RwLock};

use crate::drawbehaviour::DrawBehaviour;
use crate::render::Renderer;
use crate::{render, utils};

use chrono::Utc;
use notetakingfileformats::xoppformat::{self, XoppColor};
use p2d::bounding_volume::AABB;
use rand::distributions::Uniform;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use super::bitmapimage::BitmapImage;
use super::brushstroke::{BrushStroke, BrushStrokeStyle};
use super::markerstroke::MarkerStroke;
use super::shapestroke::ShapeStroke;
use super::vectorimage::VectorImage;
use crate::compose::transformable::Transformable;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "strokestyle")]
pub enum StrokeStyle {
    #[serde(rename = "markerstroke")]
    MarkerStroke(MarkerStroke),
    #[serde(rename = "brushstroke")]
    BrushStroke(BrushStroke),
    #[serde(rename = "shapestroke")]
    ShapeStroke(ShapeStroke),
    #[serde(rename = "vectorimage")]
    VectorImage(VectorImage),
    #[serde(rename = "bitmapimage")]
    BitmapImage(BitmapImage),
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::MarkerStroke(MarkerStroke::default())
    }
}

impl DrawBehaviour for StrokeStyle {
    fn bounds(&self) -> AABB {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.bounds(),
            Self::BrushStroke(brushstroke) => brushstroke.bounds(),
            Self::ShapeStroke(shapestroke) => shapestroke.bounds(),
            Self::VectorImage(vectorimage) => vectorimage.bounds(),
            Self::BitmapImage(bitmapimage) => bitmapimage.bounds(),
        }
    }

    fn set_bounds(&mut self, bounds: AABB) {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.set_bounds(bounds),
            Self::BrushStroke(brushstroke) => brushstroke.set_bounds(bounds),
            Self::ShapeStroke(shapestroke) => shapestroke.set_bounds(bounds),
            Self::VectorImage(vectorimage) => vectorimage.set_bounds(bounds),
            Self::BitmapImage(bitmapimage) => bitmapimage.set_bounds(bounds),
        }
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        match self {
            Self::MarkerStroke(markerstroke) => markerstroke.gen_svgs(offset),
            Self::BrushStroke(brushstroke) => brushstroke.gen_svgs(offset),
            Self::ShapeStroke(shapestroke) => shapestroke.gen_svgs(offset),
            Self::VectorImage(vectorimage) => vectorimage.gen_svgs(offset),
            Self::BitmapImage(bitmapimage) => bitmapimage.gen_svgs(offset),
        }
    }
}

impl Transformable for StrokeStyle {
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

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        match self {
            Self::MarkerStroke(markerstroke) => {
                markerstroke.rotate(angle, center);
            }
            Self::BrushStroke(brushstroke) => {
                brushstroke.rotate(angle, center);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.rotate(angle, center);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.rotate(angle, center);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.rotate(angle, center);
            }
        }
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        match self {
            Self::MarkerStroke(markerstroke) => {
                markerstroke.scale(scale);
            }
            Self::BrushStroke(brushstroke) => {
                brushstroke.scale(scale);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.scale(scale);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.scale(scale);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.scale(scale);
            }
        }
    }
}

impl StrokeStyle {
    pub fn to_xopp(
        self,
        current_dpi: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Option<xoppformat::XoppStrokeStyle> {
        match self {
            StrokeStyle::MarkerStroke(markerstroke) => {
                // Xopp expects at least 4 coordinates, so stroke with elements < 2 is not exported
                if markerstroke.elements.len() < 2 {
                    return None;
                }

                let color = markerstroke
                    .options
                    .stroke_color
                    .map(|color| color.into())?;
                let tool = xoppformat::XoppTool::Pen;
                let width = vec![utils::convert_value_dpi(
                    markerstroke.options.width,
                    current_dpi,
                    xoppformat::XoppFile::DPI,
                )];
                let coords = markerstroke
                    .elements
                    .iter()
                    .map(|element| {
                        utils::convert_coord_dpi(
                            element.inputdata.pos(),
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        )
                    })
                    .collect::<Vec<na::Vector2<f64>>>();

                Some(xoppformat::XoppStrokeStyle::XoppStroke(
                    xoppformat::XoppStroke {
                        tool,
                        color,
                        width,
                        coords,
                        fill: None,
                        timestamp: None,
                        audio_filename: None,
                    },
                ))
            }
            StrokeStyle::BrushStroke(brushstroke) => {
                // Xopp expects at least 4 coordinates, so stroke with elements < 2 is not exported
                if brushstroke.elements.len() < 2 {
                    return None;
                }

                let (width, color): (f64, XoppColor) = match brushstroke.style {
                    // Return early if color is None
                    BrushStrokeStyle::Solid { options } => {
                        (options.width, options.stroke_color?.into())
                    }
                    BrushStrokeStyle::Textured { options } => {
                        (options.width, options.stroke_color?.into())
                    }
                };

                let tool = xoppformat::XoppTool::Pen;

                // The first width element is the absolute width of the stroke
                let stroke_width =
                    utils::convert_value_dpi(width, current_dpi, xoppformat::XoppFile::DPI);

                let mut width_vec = vec![stroke_width];

                // the rest are pressures between 0.0 and 1.0
                let mut pressures = brushstroke
                    .elements
                    .iter()
                    .map(|element| stroke_width * element.inputdata.pressure())
                    .collect::<Vec<f64>>();
                width_vec.append(&mut pressures);

                let coords = brushstroke
                    .elements
                    .iter()
                    .map(|element| {
                        utils::convert_coord_dpi(
                            element.inputdata.pos(),
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        )
                    })
                    .collect::<Vec<na::Vector2<f64>>>();

                Some(xoppformat::XoppStrokeStyle::XoppStroke(
                    xoppformat::XoppStroke {
                        tool,
                        color,
                        width: width_vec,
                        coords,
                        fill: None,
                        timestamp: None,
                        audio_filename: None,
                    },
                ))
            }
            StrokeStyle::ShapeStroke(shapestroke) => {
                let shape_image = render::concat_images(shapestroke.gen_images(1.0, renderer).ok()?, shapestroke.bounds(), 1.0).ok()?;
                let image_bytes =
                    render::image_into_encoded_bytes(shape_image, image::ImageOutputFormat::Png)
                        .map_err(|e| {
                            log::error!(
                                "image_to_bytes() failed in to_xopp() for shapestroke with Err {}",
                                e
                            )
                        })
                        .ok()?;

                Some(xoppformat::XoppStrokeStyle::XoppImage(
                    xoppformat::XoppImage {
                        left: utils::convert_value_dpi(
                            shapestroke.bounds.mins[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        top: utils::convert_value_dpi(
                            shapestroke.bounds.mins[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        right: utils::convert_value_dpi(
                            shapestroke.bounds.maxs[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        bottom: utils::convert_value_dpi(
                            shapestroke.bounds.maxs[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        data: base64::encode(&image_bytes),
                    },
                ))
                // FIXME: The above is unacceptably slow, needs investigation
                //None
            }
            StrokeStyle::VectorImage(vectorimage) => {
                let png_data = match vectorimage.export_as_image_bytes(
                    1.0,
                    image::ImageOutputFormat::Png,
                    renderer,
                ) {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        log::error!("bitmapimage.export_as_bytes() failed in stroke to_xopp() with Err `{}`", e);
                        return None;
                    }
                };

                Some(xoppformat::XoppStrokeStyle::XoppImage(
                    xoppformat::XoppImage {
                        left: utils::convert_value_dpi(
                            vectorimage.bounds.mins[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        top: utils::convert_value_dpi(
                            vectorimage.bounds.mins[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        right: utils::convert_value_dpi(
                            vectorimage.bounds.maxs[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        bottom: utils::convert_value_dpi(
                            vectorimage.bounds.maxs[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        data: base64::encode(&png_data),
                    },
                ))
            }
            StrokeStyle::BitmapImage(bitmapimage) => {
                let png_data = match bitmapimage.export_as_image_bytes(
                    1.0,
                    image::ImageOutputFormat::Png,
                    renderer,
                ) {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        log::error!("bitmapimage.export_as_bytes() failed in stroke to_xopp() with Err `{}`", e);
                        return None;
                    }
                };

                Some(xoppformat::XoppStrokeStyle::XoppImage(
                    xoppformat::XoppImage {
                        left: utils::convert_value_dpi(
                            bitmapimage.bounds.mins[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        top: utils::convert_value_dpi(
                            bitmapimage.bounds.mins[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        right: utils::convert_value_dpi(
                            bitmapimage.bounds.maxs[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        bottom: utils::convert_value_dpi(
                            bitmapimage.bounds.maxs[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        data: base64::encode(&png_data),
                    },
                ))
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename = "element")]
pub struct Element {
    #[serde(rename = "inputdata")]
    pub inputdata: InputData,
    #[serde(rename = "timestamp")]
    pub timestamp: Option<chrono::DateTime<Utc>>,
}

impl Element {
    pub fn new(inputdata: InputData) -> Self {
        let timestamp = Utc::now();

        Self {
            inputdata,
            timestamp: Some(timestamp),
        }
    }

    pub fn validation_data(bounds: AABB) -> Vec<Self> {
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
