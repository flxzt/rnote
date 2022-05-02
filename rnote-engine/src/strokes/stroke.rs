use super::bitmapimage::BitmapImage;
use super::brushstroke::BrushStroke;
use super::shapestroke::ShapeStroke;
use super::strokebehaviour::GeneratedStrokeImages;
use super::vectorimage::VectorImage;
use super::StrokeBehaviour;
use crate::pens::brush::BrushStyle;
use crate::pens::Brush;
use crate::render;
use crate::{utils, DrawBehaviour};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::shapes::{Rectangle, ShapeBehaviour};
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::transform::Transform;
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::{Color, PenPath, Style};

use p2d::bounding_volume::AABB;
use rnote_fileformats::xoppformat::{self, XoppColor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "stroke")]
pub enum Stroke {
    #[serde(rename = "brushstroke")]
    BrushStroke(BrushStroke),
    #[serde(rename = "shapestroke")]
    ShapeStroke(ShapeStroke),
    #[serde(rename = "vectorimage")]
    VectorImage(VectorImage),
    #[serde(rename = "bitmapimage")]
    BitmapImage(BitmapImage),
}

impl Default for Stroke {
    fn default() -> Self {
        Self::BrushStroke(BrushStroke::default())
    }
}

impl StrokeBehaviour for Stroke {
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error> {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.gen_svg(),
            Stroke::ShapeStroke(shapestroke) => shapestroke.gen_svg(),
            Stroke::VectorImage(vectorimage) => vectorimage.gen_svg(),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.gen_svg(),
        }
    }

    fn gen_images(
        &self,
        viewport: AABB,
        image_scale: f64,
    ) -> Result<GeneratedStrokeImages, anyhow::Error> {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.gen_images(viewport, image_scale),
            Stroke::ShapeStroke(shapestroke) => shapestroke.gen_images(viewport, image_scale),
            Stroke::VectorImage(vectorimage) => vectorimage.gen_images(viewport, image_scale),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.gen_images(viewport, image_scale),
        }
    }
}

impl DrawBehaviour for Stroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, image_scale: f64) -> anyhow::Result<()> {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.draw(cx, image_scale),
            Stroke::ShapeStroke(shapestroke) => shapestroke.draw(cx, image_scale),
            Stroke::VectorImage(vectorimage) => vectorimage.draw(cx, image_scale),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.draw(cx, image_scale),
        }
    }
}

impl ShapeBehaviour for Stroke {
    fn bounds(&self) -> AABB {
        match self {
            Self::BrushStroke(brushstroke) => brushstroke.bounds(),
            Self::ShapeStroke(shapestroke) => shapestroke.bounds(),
            Self::VectorImage(vectorimage) => vectorimage.bounds(),
            Self::BitmapImage(bitmapimage) => bitmapimage.bounds(),
        }
    }

    fn hitboxes(&self) -> Vec<AABB> {
        match self {
            Self::BrushStroke(brushstroke) => brushstroke.hitboxes(),
            Self::ShapeStroke(shapestroke) => shapestroke.hitboxes(),
            Self::VectorImage(vectorimage) => vectorimage.hitboxes(),
            Self::BitmapImage(bitmapimage) => bitmapimage.hitboxes(),
        }
    }
}

impl TransformBehaviour for Stroke {
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

impl Stroke {
    pub fn from_xoppstroke(
        stroke: xoppformat::XoppStroke,
        offset: na::Vector2<f64>,
    ) -> Result<Self, anyhow::Error> {
        let mut width_iter = stroke.width.into_iter();

        let mut smooth_options = SmoothOptions::default();
        smooth_options.stroke_color = Some(Color::from(stroke.color));

        // The first element is the absolute width, every following is the relative width (between 0.0 and 1.0)
        if let Some(width) = width_iter.next() {
            smooth_options.stroke_width = width;
        }

        let mut brush = Brush::default();
        brush.style = BrushStyle::Solid;
        brush.smooth_options = smooth_options;

        let absolute_width = brush.smooth_options.stroke_width;

        let elements = stroke
            .coords
            .into_iter()
            .map(|mut coords| {
                coords[0] += offset[0];
                coords[1] += offset[1];
                // Defaulting to PRESSURE_DEFAULT if width iterator is shorter than the coords vec
                let pressure = width_iter
                    .next()
                    .map(|width| width / absolute_width)
                    .unwrap_or(Element::PRESSURE_DEFAULT);

                Element::new(coords, pressure)
            })
            .collect::<Vec<Element>>();

        let penpath = elements
            .iter()
            .zip(elements.iter().skip(1))
            .map(|(&start, &end)| Segment::Line { start, end })
            .collect::<PenPath>();

        let brushstroke = BrushStroke::from_penpath(penpath, brush.gen_style_for_current_options())
            .ok_or(anyhow::anyhow!(
                "creating brushstroke from penpath in from_xoppstroke() failed."
            ))?;

        Ok(Stroke::BrushStroke(brushstroke))
    }

    pub fn from_xoppimage(
        xopp_image: xoppformat::XoppImage,
        offset: na::Vector2<f64>,
    ) -> Result<Self, anyhow::Error> {
        let bounds = AABB::new(
            na::point![xopp_image.left, xopp_image.top],
            na::point![xopp_image.right, xopp_image.bottom],
        )
        .translate(offset);

        let bytes = base64::decode(&xopp_image.data)?;

        let rectangle = Rectangle {
            cuboid: p2d::shape::Cuboid::new(bounds.half_extents()),
            transform: Transform::new_w_isometry(na::Isometry2::new(bounds.center().coords, 0.0)),
        };
        let image = render::Image::try_from_encoded_bytes(&bytes)?;

        Ok(Stroke::BitmapImage(BitmapImage { image, rectangle }))
    }

    pub fn into_xopp(self, current_dpi: f64) -> Option<xoppformat::XoppStrokeType> {
        let image_scale = 3.0;

        match self {
            Stroke::BrushStroke(brushstroke) => {
                let (width, color): (f64, XoppColor) = match brushstroke.style {
                    // Return early if color is None
                    Style::Smooth(options) => (options.stroke_width, options.stroke_color?.into()),
                    Style::Rough(options) => (options.stroke_width, options.stroke_color?.into()),
                    Style::Textured(options) => {
                        (options.stroke_width, options.stroke_color?.into())
                    }
                };

                let tool = xoppformat::XoppTool::Pen;
                let elements_vec = brushstroke.path.into_elements();

                // The first width element is the absolute width of the stroke
                let stroke_width =
                    utils::convert_value_dpi(width, current_dpi, xoppformat::XoppFile::DPI);

                let mut width_vec = vec![stroke_width];

                // the rest are pressures between 0.0 and 1.0
                let mut pressures = elements_vec
                    .iter()
                    .map(|element| stroke_width * element.pressure)
                    .collect::<Vec<f64>>();
                width_vec.append(&mut pressures);

                // Xopp expects at least 4 coordinates, so stroke with elements < 2 is not exported
                if elements_vec.len() < 2 {
                    return None;
                }

                let coords = elements_vec
                    .iter()
                    .map(|element| {
                        utils::convert_coord_dpi(
                            element.pos,
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        )
                    })
                    .collect::<Vec<na::Vector2<f64>>>();

                Some(xoppformat::XoppStrokeType::XoppStroke(
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
            Stroke::ShapeStroke(shapestroke) => {
                let png_data = match shapestroke
                    .export_as_image_bytes(image::ImageOutputFormat::Png, image_scale)
                {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        log::error!("export_as_bytes() failed for shapestroke in stroke to_xopp() with Err `{}`", e);
                        return None;
                    }
                };
                let shapestroke_bounds = shapestroke.bounds();

                Some(xoppformat::XoppStrokeType::XoppImage(
                    xoppformat::XoppImage {
                        left: utils::convert_value_dpi(
                            shapestroke_bounds.mins[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        top: utils::convert_value_dpi(
                            shapestroke_bounds.mins[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        right: utils::convert_value_dpi(
                            shapestroke_bounds.maxs[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        bottom: utils::convert_value_dpi(
                            shapestroke_bounds.maxs[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        data: base64::encode(&png_data),
                    },
                ))
            }
            Stroke::VectorImage(vectorimage) => {
                let png_data = match vectorimage
                    .export_as_image_bytes(image::ImageOutputFormat::Png, image_scale)
                {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        log::error!("export_as_bytes() failed for vectorimage in stroke to_xopp() with Err `{}`", e);
                        return None;
                    }
                };
                let vectorimage_bounds = vectorimage.bounds();

                Some(xoppformat::XoppStrokeType::XoppImage(
                    xoppformat::XoppImage {
                        left: utils::convert_value_dpi(
                            vectorimage_bounds.mins[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        top: utils::convert_value_dpi(
                            vectorimage_bounds.mins[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        right: utils::convert_value_dpi(
                            vectorimage_bounds.maxs[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        bottom: utils::convert_value_dpi(
                            vectorimage_bounds.maxs[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        data: base64::encode(&png_data),
                    },
                ))
            }
            Stroke::BitmapImage(bitmapimage) => {
                let png_data = match bitmapimage
                    .export_as_image_bytes(image::ImageOutputFormat::Png, image_scale)
                {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        log::error!("export_as_bytes() failed for bitmapimage in stroke to_xopp() with Err `{}`", e);
                        return None;
                    }
                };

                let bounds = bitmapimage.bounds();

                Some(xoppformat::XoppStrokeType::XoppImage(
                    xoppformat::XoppImage {
                        left: utils::convert_value_dpi(
                            bounds.mins[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        top: utils::convert_value_dpi(
                            bounds.mins[1],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        right: utils::convert_value_dpi(
                            bounds.maxs[0],
                            current_dpi,
                            xoppformat::XoppFile::DPI,
                        ),
                        bottom: utils::convert_value_dpi(
                            bounds.maxs[1],
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
