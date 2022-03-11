use super::bitmapimage::{self, BitmapImage};
use super::brushstroke::{BrushStroke, BrushStrokeStyle};
use super::inputdata::InputData;
use super::shapestroke::ShapeStroke;
use super::vectorimage::VectorImage;
use crate::compose::color::Color;
use crate::compose::geometry::AABBHelpers;
use crate::compose::shapes;
use crate::compose::smooth::SmoothOptions;
use crate::compose::transformable::{Transform, Transformable};
use crate::drawbehaviour::DrawBehaviour;
use crate::pens::brush::{Brush, BrushStyle};
use crate::render::{self, Renderer};
use crate::strokes::element::Element;
use crate::utils;

use std::sync::{Arc, RwLock};

use p2d::bounding_volume::AABB;
use rnote_fileformats::xoppformat::{self, XoppColor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "strokestyle")]
pub enum StrokeStyle {
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
        Self::BrushStroke(BrushStroke::default())
    }
}

impl DrawBehaviour for StrokeStyle {
    fn bounds(&self) -> AABB {
        match self {
            Self::BrushStroke(brushstroke) => brushstroke.bounds(),
            Self::ShapeStroke(shapestroke) => shapestroke.bounds(),
            Self::VectorImage(vectorimage) => vectorimage.bounds(),
            Self::BitmapImage(bitmapimage) => bitmapimage.bounds(),
        }
    }

    fn set_bounds(&mut self, bounds: AABB) {
        match self {
            Self::BrushStroke(brushstroke) => brushstroke.set_bounds(bounds),
            Self::ShapeStroke(shapestroke) => shapestroke.set_bounds(bounds),
            Self::VectorImage(vectorimage) => vectorimage.set_bounds(bounds),
            Self::BitmapImage(bitmapimage) => bitmapimage.set_bounds(bounds),
        }
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        match self {
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
    pub fn from_xoppstroke(
        stroke: xoppformat::XoppStroke,
        offset: na::Vector2<f64>,
    ) -> Result<Self, anyhow::Error> {
        let mut width_iter = stroke.width.iter();

        let mut smooth_options = SmoothOptions::default();
        smooth_options.stroke_color = Some(Color::from(stroke.color));

        // The first element is the absolute width, every following is the relative width (between 0.0 and 1.0)
        if let Some(&width) = width_iter.next() {
            smooth_options.width = width;
        }

        let brush = Brush {
            style: BrushStyle::Solid,
            smooth_options,
            ..Brush::default()
        };

        let elements = stroke.coords.into_iter().map(|mut coords| {
            coords[0] += offset[0];
            coords[1] += offset[1];
            // Defaulting to PRESSURE_DEFAULT if width iterator is shorter than the coords vec
            let pressure = width_iter
                .next()
                .map(|&width| width / smooth_options.width)
                .unwrap_or(InputData::PRESSURE_DEFAULT);

            Element::new(InputData::new(coords, pressure))
        });

        BrushStroke::new_w_elements(elements, &brush)
            .map(|brushstroke| StrokeStyle::BrushStroke(brushstroke))
            .ok_or(anyhow::Error::msg(
                "BrushStroke new_w_elements() failed in from_xoppstroke()",
            ))
    }

    pub fn from_xoppimage(
        image: xoppformat::XoppImage,
        offset: na::Vector2<f64>,
    ) -> Result<Self, anyhow::Error> {
        let bounds = AABB::new(
            na::point![image.left, image.top],
            na::point![image.right, image.bottom],
        )
        .translate(offset);

        let intrinsic_size = bitmapimage::extract_dimensions(&base64::decode(&image.data)?)?;

        let rectangle = shapes::Rectangle {
            cuboid: p2d::shape::Cuboid::new(bounds.half_extents()),
            transform: Transform::new_w_isometry(na::Isometry2::new(bounds.center().coords, 0.0)),
        };

        let mut bitmapimage = BitmapImage {
            data_base64: image.data,
            // Xopp images are always Png
            format: bitmapimage::BitmapImageFormat::Png,
            intrinsic_size,
            rectangle,
            ..BitmapImage::default()
        };
        bitmapimage.update_geometry();

        Ok(StrokeStyle::BitmapImage(bitmapimage))
    }

    pub fn into_xopp(
        self,
        current_dpi: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Option<xoppformat::XoppStrokeStyle> {
        match self {
            StrokeStyle::BrushStroke(brushstroke) => {
                // Xopp expects at least 4 coordinates, so stroke with elements < 2 is not exported
                if brushstroke.elements.len() < 2 {
                    return None;
                }

                let (width, color): (f64, XoppColor) = match brushstroke.style {
                    // Return early if color is None
                    BrushStrokeStyle::Marker { options } => {
                        (options.width, options.stroke_color?.into())
                    }
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
                let shape_image = render::concat_images(
                    shapestroke.gen_images(1.0, renderer).ok()?,
                    shapestroke.bounds(),
                    1.0,
                )
                .ok()?;
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
