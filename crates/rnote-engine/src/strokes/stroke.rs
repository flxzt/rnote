// Imports
use super::bitmapimage::BitmapImage;
use super::brushstroke::BrushStroke;
use super::content::GeneratedContentImages;
use super::shapestroke::ShapeStroke;
use super::vectorimage::VectorImage;
use super::{Content, TextStroke};
use crate::fileformats::xoppformat::{self, XoppColor};
use crate::store::chrono_comp::StrokeLayer;
use crate::{render, Engine};
use crate::{utils, Drawable};
use p2d::bounding_volume::Aabb;
use rnote_compose::ext::AabbExt;
use rnote_compose::penpath::Element;
use rnote_compose::shapes::{Rectangle, Shapeable};
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::transform::Transform;
use rnote_compose::transform::Transformable;
use rnote_compose::{Color, PenPath, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "stroke")]
pub enum Stroke {
    #[serde(rename = "brushstroke")]
    BrushStroke(BrushStroke),
    #[serde(rename = "shapestroke")]
    ShapeStroke(ShapeStroke),
    #[serde(rename = "textstroke")]
    TextStroke(TextStroke),
    #[serde(rename = "vectorimage")]
    VectorImage(VectorImage),
    #[serde(rename = "bitmapimage")]
    BitmapImage(BitmapImage),
}

impl Content for Stroke {
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error> {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.gen_svg(),
            Stroke::ShapeStroke(shapestroke) => shapestroke.gen_svg(),
            Stroke::TextStroke(textstroke) => textstroke.gen_svg(),
            Stroke::VectorImage(vectorimage) => vectorimage.gen_svg(),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.gen_svg(),
        }
    }

    fn gen_images(
        &self,
        viewport: Aabb,
        image_scale: f64,
    ) -> Result<GeneratedContentImages, anyhow::Error> {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.gen_images(viewport, image_scale),
            Stroke::ShapeStroke(shapestroke) => shapestroke.gen_images(viewport, image_scale),
            Stroke::TextStroke(textstroke) => textstroke.gen_images(viewport, image_scale),
            Stroke::VectorImage(vectorimage) => vectorimage.gen_images(viewport, image_scale),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.gen_images(viewport, image_scale),
        }
    }

    fn draw_highlight(
        &self,
        cx: &mut impl piet::RenderContext,
        total_zoom: f64,
    ) -> anyhow::Result<()> {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.draw_highlight(cx, total_zoom),
            Stroke::ShapeStroke(shapestroke) => shapestroke.draw_highlight(cx, total_zoom),
            Stroke::TextStroke(textstroke) => textstroke.draw_highlight(cx, total_zoom),
            Stroke::VectorImage(vectorimage) => vectorimage.draw_highlight(cx, total_zoom),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.draw_highlight(cx, total_zoom),
        }
    }

    fn update_geometry(&mut self) {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.update_geometry(),
            Stroke::ShapeStroke(shapestroke) => shapestroke.update_geometry(),
            Stroke::TextStroke(textstroke) => textstroke.update_geometry(),
            Stroke::VectorImage(vectorimage) => vectorimage.update_geometry(),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.update_geometry(),
        }
    }
}

impl Drawable for Stroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, image_scale: f64) -> anyhow::Result<()> {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.draw(cx, image_scale),
            Stroke::ShapeStroke(shapestroke) => shapestroke.draw(cx, image_scale),
            Stroke::TextStroke(textstroke) => textstroke.draw(cx, image_scale),
            Stroke::VectorImage(vectorimage) => vectorimage.draw(cx, image_scale),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.draw(cx, image_scale),
        }
    }

    fn draw_to_cairo(&self, cx: &cairo::Context, image_scale: f64) -> anyhow::Result<()> {
        match self {
            Stroke::BrushStroke(brushstroke) => brushstroke.draw_to_cairo(cx, image_scale),
            Stroke::ShapeStroke(shapestroke) => shapestroke.draw_to_cairo(cx, image_scale),
            Stroke::TextStroke(textstroke) => textstroke.draw_to_cairo(cx, image_scale),
            Stroke::VectorImage(vectorimage) => vectorimage.draw_to_cairo(cx, image_scale),
            Stroke::BitmapImage(bitmapimage) => bitmapimage.draw_to_cairo(cx, image_scale),
        }
    }
}

impl Shapeable for Stroke {
    fn bounds(&self) -> Aabb {
        match self {
            Self::BrushStroke(brushstroke) => brushstroke.bounds(),
            Self::ShapeStroke(shapestroke) => shapestroke.bounds(),
            Self::TextStroke(textstroke) => textstroke.bounds(),
            Self::VectorImage(vectorimage) => vectorimage.bounds(),
            Self::BitmapImage(bitmapimage) => bitmapimage.bounds(),
        }
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        match self {
            Self::BrushStroke(brushstroke) => brushstroke.hitboxes(),
            Self::ShapeStroke(shapestroke) => shapestroke.hitboxes(),
            Self::TextStroke(textstroke) => textstroke.hitboxes(),
            Self::VectorImage(vectorimage) => vectorimage.hitboxes(),
            Self::BitmapImage(bitmapimage) => bitmapimage.hitboxes(),
        }
    }

    fn outline_path(&self) -> kurbo::BezPath {
        match self {
            Self::BrushStroke(brushstroke) => brushstroke.outline_path(),
            Self::ShapeStroke(shapestroke) => shapestroke.outline_path(),
            Self::TextStroke(textstroke) => textstroke.outline_path(),
            Self::VectorImage(vectorimage) => vectorimage.outline_path(),
            Self::BitmapImage(bitmapimage) => bitmapimage.outline_path(),
        }
    }
}

impl Transformable for Stroke {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        match self {
            Self::BrushStroke(brushstroke) => {
                brushstroke.translate(offset);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.translate(offset);
            }
            Self::TextStroke(textstroke) => {
                textstroke.translate(offset);
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
            Self::TextStroke(textstroke) => {
                textstroke.rotate(angle, center);
            }
            Self::VectorImage(vectorimage) => {
                vectorimage.rotate(angle, center);
            }
            Self::BitmapImage(bitmapimage) => {
                bitmapimage.rotate(angle, center);
            }
        }
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        match self {
            Self::BrushStroke(brushstroke) => {
                brushstroke.scale(scale);
            }
            Self::ShapeStroke(shapestroke) => {
                shapestroke.scale(scale);
            }
            Self::TextStroke(textstroke) => {
                textstroke.scale(scale);
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
    /// The default offset in surface coords when importing a stroke.
    pub const IMPORT_OFFSET_DEFAULT: na::Vector2<f64> = na::vector![32.0, 32.0];

    pub fn extract_default_layer(&self) -> StrokeLayer {
        match self {
            Stroke::BrushStroke(_) => StrokeLayer::UserLayer(0),
            Stroke::ShapeStroke(_) => StrokeLayer::UserLayer(0),
            Stroke::TextStroke(_) => StrokeLayer::UserLayer(0),
            Stroke::VectorImage(_) | Stroke::BitmapImage(_) => StrokeLayer::Image,
        }
    }

    /// Invert the brightness of all colors of the stroke.
    ///
    /// Returns true if the stroke was modified and needs to update its rendering.
    pub fn set_to_inverted_brightness_color(&mut self) -> bool {
        match self {
            Stroke::BrushStroke(brush_stroke) => {
                if let Some(color) = brush_stroke.style.stroke_color() {
                    brush_stroke
                        .style
                        .set_stroke_color(color.to_inverted_brightness_color());
                }

                if let Some(color) = brush_stroke.style.fill_color() {
                    brush_stroke
                        .style
                        .set_fill_color(color.to_inverted_brightness_color());
                }

                true
            }
            Stroke::ShapeStroke(shape_stroke) => {
                if let Some(color) = shape_stroke.style.stroke_color() {
                    shape_stroke
                        .style
                        .set_stroke_color(color.to_inverted_brightness_color());
                }

                if let Some(color) = shape_stroke.style.fill_color() {
                    shape_stroke
                        .style
                        .set_fill_color(color.to_inverted_brightness_color());
                }

                true
            }
            Stroke::TextStroke(text_stroke) => {
                text_stroke.text_style.color =
                    text_stroke.text_style.color.to_inverted_brightness_color();

                true
            }
            Stroke::VectorImage(_) => false,
            Stroke::BitmapImage(_) => false,
        }
    }

    /// Set all colors of the stroke to their darkest variant.
    ///
    /// Returns true if the stroke was modified and needs to update its rendering.
    pub fn set_to_darkest_color(&mut self) -> bool {
        match self {
            Stroke::BrushStroke(brush_stroke) => {
                if let Some(color) = brush_stroke.style.stroke_color() {
                    brush_stroke
                        .style
                        .set_stroke_color(color.to_darkest_color());
                }

                if let Some(color) = brush_stroke.style.fill_color() {
                    brush_stroke.style.set_fill_color(color.to_darkest_color());
                }

                true
            }
            Stroke::ShapeStroke(shape_stroke) => {
                if let Some(color) = shape_stroke.style.stroke_color() {
                    shape_stroke
                        .style
                        .set_stroke_color(color.to_darkest_color());
                }

                if let Some(color) = shape_stroke.style.fill_color() {
                    shape_stroke.style.set_fill_color(color.to_darkest_color());
                }

                true
            }
            Stroke::TextStroke(text_stroke) => {
                text_stroke.text_style.color = text_stroke.text_style.color.to_darkest_color();

                true
            }
            Stroke::VectorImage(_) => false,
            Stroke::BitmapImage(_) => false,
        }
    }

    pub fn from_xoppstroke(
        stroke: xoppformat::XoppStroke,
        offset: na::Vector2<f64>,
        target_dpi: f64,
    ) -> Result<(Self, StrokeLayer), anyhow::Error> {
        let mut widths: Vec<f64> = stroke
            .width
            .into_iter()
            .map(|w| crate::utils::convert_value_dpi(w, xoppformat::XoppFile::DPI, target_dpi))
            .collect();

        let coords: Vec<na::Vector2<f64>> = stroke
            .coords
            .into_iter()
            .map(|c| {
                na::vector![
                    crate::utils::convert_value_dpi(c[0], xoppformat::XoppFile::DPI, target_dpi),
                    crate::utils::convert_value_dpi(c[1], xoppformat::XoppFile::DPI, target_dpi)
                ]
            })
            .collect();

        if widths.is_empty() {
            return Err(anyhow::anyhow!("Stroke has empty widths vector."));
        }

        let mut smooth_options = SmoothOptions::default();

        let layer = match stroke.tool {
            xoppformat::XoppTool::Pen => {
                smooth_options.stroke_color = Some(crate::utils::color_from_xopp(stroke.color));
                StrokeLayer::UserLayer(0)
            }
            xoppformat::XoppTool::Highlighter => {
                let mut color = crate::utils::color_from_xopp(stroke.color);
                // the highlighter always has alpha 0.5
                color.a = 0.5;

                smooth_options.stroke_color = Some(color);
                StrokeLayer::Highlighter
            }
            xoppformat::XoppTool::Eraser => {
                smooth_options.stroke_color = Some(Color::WHITE);
                StrokeLayer::UserLayer(0)
            }
        };

        // remove the first element, which will be the stroke width.
        let mut stroke_width = widths.remove(0);

        // extract the maximum width ( the widths in xournal++'s format are not relative to the stroke width).
        let max_width = widths.iter().cloned().reduce(f64::max);

        if let Some(max_width) = max_width {
            // the stroke width in rnote needs to be the maximum of all widths
            stroke_width = max_width;

            // the coordinate widths are relative to the max width
            widths
                .iter_mut()
                .for_each(|coord_width| *coord_width /= max_width);
        } else {
            // If there are no coordinate widths, we fill the widths vector with pressure 1.0 for a constant width stroke.
            widths = (0..coords.len()).map(|_| 1.0).collect();
        };

        smooth_options.stroke_width = stroke_width;

        let penpath = PenPath::try_from_elements(
            coords
                .into_iter()
                .zip(widths.into_iter())
                .map(|(pos, pressure)| Element::new(pos + offset, pressure)),
        )
        .ok_or_else(|| anyhow::anyhow!("Could not generate pen path from coordinates vector"))?;

        let brushstroke = BrushStroke::from_penpath(penpath, Style::Smooth(smooth_options));

        Ok((Stroke::BrushStroke(brushstroke), layer))
    }

    pub fn from_xoppimage(
        xopp_image: xoppformat::XoppImage,
        offset: na::Vector2<f64>,
        target_dpi: f64,
    ) -> Result<Self, anyhow::Error> {
        let bounds = Aabb::new(
            na::point![
                crate::utils::convert_value_dpi(
                    xopp_image.left,
                    xoppformat::XoppFile::DPI,
                    target_dpi
                ),
                crate::utils::convert_value_dpi(
                    xopp_image.top,
                    xoppformat::XoppFile::DPI,
                    target_dpi
                )
            ],
            na::point![
                crate::utils::convert_value_dpi(
                    xopp_image.right,
                    xoppformat::XoppFile::DPI,
                    target_dpi
                ),
                crate::utils::convert_value_dpi(
                    xopp_image.bottom,
                    xoppformat::XoppFile::DPI,
                    target_dpi
                )
            ],
        )
        .translate(offset);

        let bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &xopp_image.data)?;

        let rectangle = Rectangle {
            cuboid: p2d::shape::Cuboid::new(bounds.half_extents()),
            transform: Transform::new_w_isometry(na::Isometry2::new(bounds.center().coords, 0.0)),
        };
        let image = render::Image::try_from_encoded_bytes(&bytes)?;

        Ok(Stroke::BitmapImage(BitmapImage { image, rectangle }))
    }

    pub fn into_xopp(self, current_dpi: f64) -> Option<xoppformat::XoppStrokeType> {
        match self {
            Stroke::BrushStroke(brushstroke) => {
                let (stroke_width, color): (f64, XoppColor) = match &brushstroke.style {
                    // Return early if color is None
                    Style::Smooth(options) => (
                        options.stroke_width,
                        crate::utils::xoppcolor_from_color(options.stroke_color?),
                    ),
                    Style::Rough(options) => (
                        options.stroke_width,
                        crate::utils::xoppcolor_from_color(options.stroke_color?),
                    ),
                    Style::Textured(options) => (
                        options.stroke_width,
                        crate::utils::xoppcolor_from_color(options.stroke_color?),
                    ),
                };

                let tool = xoppformat::XoppTool::Pen;
                let elements_vec = brushstroke.path.into_elements();
                let stroke_style = &brushstroke.style;
                let stroke_width =
                    utils::convert_value_dpi(stroke_width, current_dpi, xoppformat::XoppFile::DPI);

                // in Xopp's format the first width element is the absolute width of the stroke
                let mut width_vec = vec![stroke_width];

                // the rest are pressures between 0.0 and 1.0
                let mut pressures: Vec<f64> = elements_vec
                    .iter()
                    .map(|element| match &stroke_style {
                        Style::Smooth(options) => {
                            options.pressure_curve.apply(stroke_width, element.pressure)
                        }
                        Style::Rough(_) | Style::Textured(_) => stroke_width * element.pressure,
                    })
                    .collect();
                width_vec.append(&mut pressures);

                // Xopp expects at least 4 coordinates, so strokes with elements < 2 aren't exported
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
                let png_data = match shapestroke.export_to_bitmap_image_bytes(
                    image::ImageFormat::Png,
                    Engine::STROKE_EXPORT_IMAGE_SCALE,
                ) {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        tracing::error!("Converting ShapeStroke to XoppImage failed, Err: {e:?}");
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
                        data: base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            png_data,
                        ),
                    },
                ))
            }
            Stroke::TextStroke(textstroke) => {
                // Xournal++ text strokes do not support affine transformations, so we have to convert on best effort here.
                // The best solution for now seems to be to export them as a bitmap image.
                let png_data = match textstroke.export_to_bitmap_image_bytes(
                    image::ImageFormat::Png,
                    Engine::STROKE_EXPORT_IMAGE_SCALE,
                ) {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        tracing::error!("Converting TextStroke to XoppImage failed, Err: {e:?}");
                        return None;
                    }
                };
                let vectorimage_bounds = textstroke.bounds();

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
                        data: base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            png_data,
                        ),
                    },
                ))
            }
            Stroke::VectorImage(vectorimage) => {
                let png_data = match vectorimage.export_to_bitmap_image_bytes(
                    image::ImageFormat::Png,
                    Engine::STROKE_EXPORT_IMAGE_SCALE,
                ) {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        tracing::error!(
                            "Exporting VectorImage to image bytes failed while converting Stroke to Xopp, Err: {e:?}"
                        );
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
                        data: base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            png_data,
                        ),
                    },
                ))
            }
            Stroke::BitmapImage(bitmapimage) => {
                let png_data = match bitmapimage.export_to_bitmap_image_bytes(
                    image::ImageFormat::Png,
                    Engine::STROKE_EXPORT_IMAGE_SCALE,
                ) {
                    Ok(image_bytes) => image_bytes,
                    Err(e) => {
                        tracing::error!(
                            "Exporting BitmapImage to image bytes failed while converting Stroke to Xopp, Err: {e:?}"
                        );
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
                        data: base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            png_data,
                        ),
                    },
                ))
            }
        }
    }
}
