// Imports
use crate::Drawable;
use crate::Svg;
use crate::document::Background;
use crate::store::chrono_comp::StrokeLayer;
use crate::strokes::Stroke;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::shapes::Shapeable;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "layered_stroke")]
pub struct LayeredStroke {
    #[serde(rename = "stroke")]
    pub stroke: Arc<Stroke>,
    #[serde(rename = "layer")]
    pub layer: StrokeLayer,
}

/// Stroke content.
///
/// Used when exporting and pasting/copying/cutting from/into the clipboard.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "stroke_content")]
pub struct StrokeContent {
    #[serde(rename = "strokes")]
    pub strokes: Vec<LayeredStroke>,
    #[serde(rename = "bounds")]
    pub bounds: Option<Aabb>,
    #[serde(rename = "background")]
    pub background: Option<Background>,
}
impl StrokeContent {
    pub const MIME_TYPE: &'static str = "application/rnote-stroke-content";
    pub const CLIPBOARD_EXPORT_MARGIN: f64 = 6.0;

    pub fn with_bounds(mut self, bounds: Aabb) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn with_strokes(mut self, strokes: Vec<LayeredStroke>) -> Self {
        self.strokes = strokes;
        self
    }

    pub fn with_background(mut self, background: Background) -> Self {
        self.background = Some(background);
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
                .map(|stroke_content_stroke| stroke_content_stroke.stroke.bounds())
                .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x)),
        )
    }

    pub fn size(&self) -> Option<na::Vector2<f64>> {
        self.bounds().map(|b| b.extents())
    }

    /// Generate a Svg from the content.
    ///
    /// Moves the bounds to mins: [0.0, 0.0], maxs: extents.
    ///
    /// Returns Ok(None) if there is no content stored.
    pub fn gen_svg(
        &self,
        draw_background: bool,
        draw_pattern: bool,
        optimize_printing: bool,
        margin: f64,
    ) -> anyhow::Result<Option<Svg>> {
        let Some(bounds_loosened) = self.bounds().map(|b| b.loosened(margin)) else {
            return Ok(None);
        };
        let mut svg = Svg::gen_with_cairo(
            |cairo_cx| {
                self.draw_to_cairo(
                    cairo_cx,
                    draw_background,
                    draw_pattern,
                    optimize_printing,
                    margin,
                    1.0,
                )
            },
            bounds_loosened,
        )?;
        // The simplification also moves the bounds to mins: [0.0, 0.0], maxs: extents
        if let Err(e) = svg.simplify() {
            warn!("Simplifying Svg while generating StrokeContent Svg failed, Err: {e:?}");
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
        let Some(bounds) = self.bounds() else {
            return Ok(());
        };
        let bounds_loosened = bounds.loosened(margin);

        cairo_cx.save()?;
        cairo_cx.rectangle(
            bounds_loosened.mins[0],
            bounds_loosened.mins[1],
            bounds_loosened.extents()[0],
            bounds_loosened.extents()[1],
        );
        cairo_cx.clip();

        if draw_background && let Some(background) = &self.background {
            background.draw_to_cairo(cairo_cx, bounds_loosened, draw_pattern, optimize_printing)?;
        }

        cairo_cx.restore()?;
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
            .filter_map(
                |stroke_content_stroke| match stroke_content_stroke.stroke.as_ref() {
                    Stroke::BitmapImage(image) => Some(image.rectangle.bounds()),
                    Stroke::VectorImage(image) => Some(image.rectangle.bounds()),
                    _ => None,
                },
            )
            .collect::<Vec<Aabb>>();

        let draw_stroke = |stroke_content_stroke: &LayeredStroke,
                           cairo_cx: &cairo::Context|
         -> anyhow::Result<()> {
            let stroke_bounds = stroke_content_stroke.stroke.bounds();

            if optimize_printing
                && image_bounds
                    .iter()
                    .all(|bounds| !bounds.contains(&stroke_bounds))
            {
                // Using the stroke's bounds instead of hitboxes works for inclusion.
                // If this is changed to intersection, all hitboxes must be checked individually.

                let mut darkest_color_stroke = stroke_content_stroke.stroke.as_ref().clone();
                darkest_color_stroke.set_to_darkest_color();

                darkest_color_stroke.draw_to_cairo(cairo_cx, image_scale)
            } else {
                stroke_content_stroke
                    .stroke
                    .draw_to_cairo(cairo_cx, image_scale)
            }
        };

        let (non_highlighter_strokes, highlighter_strokes): (Vec<_>, Vec<_>) = self
            .strokes
            .iter()
            .partition(|stroke| stroke.layer != StrokeLayer::Highlighter);

        for stroke in non_highlighter_strokes {
            draw_stroke(stroke, cairo_cx)?;
        }

        if !highlighter_strokes.is_empty() {
            cairo_cx.save()?;
            cairo_cx.set_operator(cairo::Operator::Multiply);

            for stroke in highlighter_strokes {
                draw_stroke(stroke, cairo_cx)?;
            }

            cairo_cx.restore()?;
        }

        cairo_cx.restore()?;

        Ok(())
    }
}
