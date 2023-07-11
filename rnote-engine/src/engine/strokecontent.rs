// Imports
use crate::document::Background;
use crate::render::Svg;
use crate::strokes::Stroke;
use crate::{DrawBehaviour, RnoteEngine};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::helpers::AabbHelpers;
use rnote_compose::shapes::ShapeBehaviour;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Stroke content.
///
/// Used when exporting and pasting/copying/cutting from/into the clipboard.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "stroke_content")]
pub struct StrokeContent {
    #[serde(rename = "strokes")]
    pub strokes: Vec<Arc<Stroke>>,
    #[serde(rename = "bounds")]
    pub bounds: Option<Aabb>,
    #[serde(rename = "background")]
    pub background: Option<Background>,
}

impl StrokeContent {
    pub const MIME_TYPE: &str = "application/rnote-stroke-content";
    pub const CLIPBOARD_EXPORT_MARGIN: f64 = 6.0;

    pub fn with_bounds(mut self, bounds: Option<Aabb>) -> Self {
        self.bounds = bounds;
        self
    }

    pub fn with_strokes(mut self, strokes: Vec<Arc<Stroke>>) -> Self {
        self.strokes = strokes;
        self
    }

    pub fn with_background(mut self, background: Option<Background>) -> Self {
        self.background = background;
        self
    }

    pub fn bounds(&self) -> Option<Aabb> {
        if self.bounds.is_some() {
            return self.bounds;
        }
        if self.strokes.is_empty() {
            return None;
        }
        Some(
            self.strokes
                .iter()
                .map(|s| s.bounds())
                .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x)),
        )
    }

    pub fn size(&self) -> Option<na::Vector2<f64>> {
        self.bounds().map(|b| b.extents())
    }

    /// Generate a Svg from the content.
    ///
    // Moves the bounds to mins: [0.0, 0.0], maxs: extents.
    pub fn gen_svg(
        &self,
        with_background: bool,
        with_pattern: bool,
        optimize_printing: bool,
        margin: f64,
    ) -> anyhow::Result<Option<Svg>> {
        let Some(bounds) = self.bounds() else {
            return Ok(None);
        };
        let bounds_loosened = bounds.loosened(margin);
        let mut svg = match (with_background, self.background) {
            (true, Some(background)) => {
                background.gen_svg(bounds_loosened, with_pattern, optimize_printing)?
            }
            _ => Svg {
                svg_data: String::new(),
                bounds,
            },
        };

        let image_bounds = self
            .strokes
            .iter()
            .filter_map(|stroke| match stroke.as_ref() {
                Stroke::BitmapImage(image) => Some(image.rectangle.bounds()),
                Stroke::VectorImage(image) => Some(image.rectangle.bounds()),
                _ => None,
            })
            .collect::<Vec<Aabb>>();

        svg.merge([Svg::gen_with_piet_cairo_backend(
            |piet_cx| {
                piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
                piet_cx.clip(bounds.to_kurbo_rect());
                for stroke in self.strokes.iter() {
                    let stroke_bounds = stroke.bounds();

                    stroke.draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;

                    if optimize_printing
                        && image_bounds
                            .iter()
                            .all(|bounds| !bounds.contains(&stroke_bounds))
                    {
                        // Using the stroke's bounds instead of hitboxes works for inclusion.
                        // If this is changed to intersection, all hitboxes must be checked individually.

                        match stroke.as_ref() {
                            Stroke::BrushStroke(brush_stroke) => {
                                let mut modified_brush_stroke = brush_stroke.clone();

                                if let Some(color) = modified_brush_stroke.style.stroke_color() {
                                    modified_brush_stroke
                                        .style
                                        .set_stroke_color(color.to_darkest_color());
                                }

                                if let Some(color) = modified_brush_stroke.style.fill_color() {
                                    modified_brush_stroke
                                        .style
                                        .set_fill_color(color.to_darkest_color());
                                }

                                modified_brush_stroke
                                    .draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                            }
                            Stroke::ShapeStroke(shape_stroke) => {
                                let mut modified_shape_stroke = shape_stroke.clone();

                                if let Some(color) = modified_shape_stroke.style.stroke_color() {
                                    modified_shape_stroke
                                        .style
                                        .set_stroke_color(color.to_darkest_color());
                                }

                                if let Some(color) = modified_shape_stroke.style.fill_color() {
                                    modified_shape_stroke
                                        .style
                                        .set_fill_color(color.to_darkest_color());
                                }

                                modified_shape_stroke
                                    .draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                            }
                            Stroke::TextStroke(text_stroke) => {
                                let mut modified_text_stroke = text_stroke.clone();

                                modified_text_stroke.text_style.color =
                                    modified_text_stroke.text_style.color.to_darkest_color();

                                modified_text_stroke
                                    .draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                            }
                            _ => stroke.draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?,
                        };
                    } else {
                        stroke.draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                    }
                }
                piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
                Ok(())
            },
            bounds,
        )?]);
        // The simplification also moves the bounds to mins: [0.0, 0.0], maxs: extents
        if let Err(e) = svg.simplify() {
            log::warn!("simplifying Svg while exporting StrokeContent failed, Err: {e:?}");
        };
        Ok(Some(svg))
    }

    pub fn draw_to_cairo(
        &self,
        cairo_cx: &cairo::Context,
        draw_background: bool,
        draw_pattern: bool,
        optimize_printing: bool,
        margin: f64,
        image_scale: f64,
    ) -> anyhow::Result<()> {
        let Some(bounds) = self.bounds() else { return Ok(()) };
        let bounds_loosened = bounds.loosened(margin);

        cairo_cx.save()?;
        cairo_cx.rectangle(
            bounds_loosened.mins[0],
            bounds_loosened.mins[1],
            bounds_loosened.extents()[0],
            bounds_loosened.extents()[1],
        );
        cairo_cx.clip();

        if draw_background {
            if let Some(background) = &self.background {
                background.draw_to_cairo(
                    cairo_cx,
                    bounds_loosened,
                    draw_pattern,
                    optimize_printing,
                )?;
            }
        }

        cairo_cx.save()?;
        cairo_cx.rectangle(
            bounds.mins[0],
            bounds.mins[1],
            bounds.extents()[0],
            bounds.extents()[1],
        );
        cairo_cx.clip();

        let image_bounds = self
            .strokes
            .iter()
            .filter_map(|stroke| match stroke.as_ref() {
                Stroke::BitmapImage(image) => Some(image.rectangle.bounds()),
                Stroke::VectorImage(image) => Some(image.rectangle.bounds()),
                _ => None,
            })
            .collect::<Vec<Aabb>>();

        let mut piet_cx = piet_cairo::CairoRenderContext::new(cairo_cx);
        for stroke in self.strokes.iter() {
            let stroke_bounds = stroke.bounds();

            // if optimize_printing
            //     && image_bounds
            //         .iter()
            //         .all(|bounds| !bounds.contains(&stroke_bounds))
            // {
            //     // Using the stroke's bounds instead of hitboxes works for inclusion.
            //     // If this is changed to intersection, all hitboxes must be checked individually.

            //     let mut stroke = stroke.as_ref().clone();

            //     match &mut stroke {
            //         Stroke::BrushStroke(brush_stroke) => {
            //             if let Some(color) = brush_stroke.style.stroke_color() {
            //                 brush_stroke
            //                     .style
            //                     .set_stroke_color(color.to_darkest_color());
            //             }

            //             if let Some(color) = brush_stroke.style.fill_color() {
            //                 brush_stroke.style.set_fill_color(color.to_darkest_color());
            //             }
            //         }
            //         Stroke::ShapeStroke(shape_stroke) => {
            //             if let Some(color) = shape_stroke.style.stroke_color() {
            //                 shape_stroke
            //                     .style
            //                     .set_stroke_color(color.to_darkest_color());
            //             }

            //             if let Some(color) = shape_stroke.style.fill_color() {
            //                 shape_stroke.style.set_fill_color(color.to_darkest_color());
            //             }
            //         }
            //         Stroke::TextStroke(text_stroke) => {
            //             text_stroke.text_style.color =
            //                 text_stroke.text_style.color.to_darkest_color();
            //         }
            //         _ => {}
            //     };

            //     stroke.draw(&mut piet_cx, image_scale)?;
            // } else {
            //     stroke.draw(&mut piet_cx, image_scale)?;
            // }

            if optimize_printing
                && image_bounds
                    .iter()
                    .all(|bounds| !bounds.contains(&stroke_bounds))
            {
                // Using the stroke's bounds instead of hitboxes works for inclusion.
                // If this is changed to intersection, all hitboxes must be checked individually.

                match stroke.as_ref() {
                    Stroke::BrushStroke(brush_stroke) => {
                        let mut modified_brush_stroke = brush_stroke.clone();

                        if let Some(color) = modified_brush_stroke.style.stroke_color() {
                            modified_brush_stroke
                                .style
                                .set_stroke_color(color.to_darkest_color());
                        }

                        if let Some(color) = modified_brush_stroke.style.fill_color() {
                            modified_brush_stroke
                                .style
                                .set_fill_color(color.to_darkest_color());
                        }

                        modified_brush_stroke.draw(&mut piet_cx, image_scale)?;
                    }
                    Stroke::ShapeStroke(shape_stroke) => {
                        let mut modified_shape_stroke = shape_stroke.clone();

                        if let Some(color) = modified_shape_stroke.style.stroke_color() {
                            modified_shape_stroke
                                .style
                                .set_stroke_color(color.to_darkest_color());
                        }

                        if let Some(color) = modified_shape_stroke.style.fill_color() {
                            modified_shape_stroke
                                .style
                                .set_fill_color(color.to_darkest_color());
                        }

                        modified_shape_stroke.draw(&mut piet_cx, image_scale)?;
                    }
                    Stroke::TextStroke(text_stroke) => {
                        let mut modified_text_stroke = text_stroke.clone();

                        modified_text_stroke.text_style.color =
                            modified_text_stroke.text_style.color.to_darkest_color();

                        modified_text_stroke.draw(&mut piet_cx, image_scale)?;
                    }
                    _ => stroke.draw(&mut piet_cx, image_scale)?,
                };
            } else {
                stroke.draw(&mut piet_cx, image_scale)?;
            }
        }
        cairo_cx.restore()?;
        cairo_cx.restore()?;
        Ok(())
    }
}
