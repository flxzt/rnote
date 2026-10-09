// Imports
use crate::Drawable;
use crate::Svg;
use crate::document::Background;
use crate::store::chrono_comp::StrokeLayer;
use crate::strokes::Stroke;
use anyhow::Context;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::math::Vector2;
use rnote_compose::shapes::Shapeable;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

    pub fn size(&self) -> Option<Vector2> {
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
        let Some(bounds) = self.bounds() else {
            return Ok(None);
        };
        let bounds_loosened = bounds.loosened(margin);

        // The background and all strokes that are not on the highlighter layer.
        let mut svg = Svg::gen_with_cairo(
            |cairo_cx| {
                self.draw_back_to_cairo(
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

        svg.simplify()
            .context("simplifying the non-highlighter Svg fragment failed")?;

        let image_bounds = self.image_bounds();
        let highlighter_strokes = self
            .strokes
            .iter()
            .filter(|s| s.layer == StrokeLayer::Highlighter)
            .collect::<Vec<_>>();

        if !highlighter_strokes.is_empty() {
            // Cairo's Svg backend emulates blend operators with `feImage`-based filters that reference groups inside `<defs>`.
            // Barely any Svg renderer is able to display that construct, and the usvg simplification below is unable to preserve it,
            // which turns the exported Svg into a black/blank image in most applications.
            //
            // To work around this, the highlighter strokes are drawn into a separate Svg and blended multiplicatively with the content
            // below through `mix-blend-mode`. This is widely supported instead and falls back to plain source-over compositing where it isn't.

            for stroke in highlighter_strokes {
                let mut fragment = Svg::gen_with_cairo(
                    |cairo_cx| {
                        cairo_cx.save()?;
                        Self::clip(cairo_cx, bounds);
                        let draw_res = Self::draw_stroke(
                            stroke,
                            &image_bounds,
                            cairo_cx,
                            1.0,
                            optimize_printing,
                        );
                        cairo_cx.restore()?;
                        draw_res
                    },
                    bounds_loosened,
                )?;

                // Each usvg pass assigns its own random id prefix (so IDs can't collide) and puts everything into the same coordinate space.
                fragment
                    .simplify()
                    .context("simplifying a highlighter stroke Svg fragment failed")?;

                svg.svg_data.push_str(&format!(
                    "\n<g style=\"mix-blend-mode:multiply\">{}</g>",
                    fragment.svg_data
                ));
            }
        }

        Ok(Some(svg))
    }

    /// Draw the content to a cairo context.
    pub fn draw_to_cairo(
        &self,
        cairo_cx: &cairo::Context,
        draw_background: bool,
        draw_pattern: bool,
        optimize_printing: bool,
        margin: f64,
        image_scale: f64,
    ) -> anyhow::Result<()> {
        self.draw_back_to_cairo(
            cairo_cx,
            draw_background,
            draw_pattern,
            optimize_printing,
            margin,
            image_scale,
        )?;

        let Some(bounds) = self.bounds() else {
            return Ok(());
        };

        cairo_cx.save()?;
        Self::clip(cairo_cx, bounds);
        cairo_cx.set_operator(cairo::Operator::Multiply);

        let image_bounds = self.image_bounds();
        for stroke in self
            .strokes
            .iter()
            .filter(|s| s.layer == StrokeLayer::Highlighter)
        {
            Self::draw_stroke(
                stroke,
                &image_bounds,
                cairo_cx,
                image_scale,
                optimize_printing,
            )?;
        }

        cairo_cx.restore()?;

        Ok(())
    }

    /// Draw the background and all strokes that are not on the highlighter layer.
    fn draw_back_to_cairo(
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
        Self::clip(cairo_cx, bounds_loosened);

        if draw_background && let Some(background) = &self.background {
            background.draw_to_cairo(cairo_cx, bounds_loosened, draw_pattern, optimize_printing)?;
        }

        cairo_cx.restore()?;
        cairo_cx.save()?;
        Self::clip(cairo_cx, bounds);

        let image_bounds = self.image_bounds();
        for stroke in self
            .strokes
            .iter()
            .filter(|s| s.layer != StrokeLayer::Highlighter)
        {
            Self::draw_stroke(
                stroke,
                &image_bounds,
                cairo_cx,
                image_scale,
                optimize_printing,
            )?;
        }

        cairo_cx.restore()?;

        Ok(())
    }

    /// Clip to the given bounds.
    fn clip(cairo_cx: &cairo::Context, bounds: Aabb) {
        cairo_cx.rectangle(
            bounds.mins[0],
            bounds.mins[1],
            bounds.extents()[0],
            bounds.extents()[1],
        );
        cairo_cx.clip();
    }

    /// Bounds of the bitmap / vector image strokes.
    fn image_bounds(&self) -> Vec<Aabb> {
        self.strokes
            .iter()
            .filter_map(
                |stroke_content_stroke| match stroke_content_stroke.stroke.as_ref() {
                    Stroke::BitmapImage(image) => Some(image.rectangle.bounds()),
                    Stroke::VectorImage(image) => Some(image.rectangle.bounds()),
                    _ => None,
                },
            )
            .collect::<Vec<Aabb>>()
    }

    fn draw_stroke(
        stroke_content_stroke: &LayeredStroke,
        image_bounds: &[Aabb],
        cairo_cx: &cairo::Context,
        image_scale: f64,
        optimize_printing: bool,
    ) -> anyhow::Result<()> {
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
    }
}
