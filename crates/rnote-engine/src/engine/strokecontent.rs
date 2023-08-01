use crate::Drawable;
// Imports
use crate::document::Background;
use crate::render::Svg;
use crate::strokes::{Content, Stroke};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::shapes::Shapeable;
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
        margin: f64,
    ) -> anyhow::Result<Option<Svg>> {
        let Some(bounds) = self.bounds() else {
            return Ok(None);
        };
        let bounds_loosened = bounds.loosened(margin);
        let mut content_svg = match (with_background, self.background) {
            (true, Some(background)) => background.gen_svg(bounds_loosened, with_pattern)?,
            _ => Svg {
                svg_data: String::new(),
                bounds,
            },
        };

        for stroke in &self.strokes {
            content_svg.merge(std::iter::once(stroke.gen_svg()?));
        }

        // The simplification also moves the bounds to mins: [0.0, 0.0], maxs: extents
        if let Err(e) = content_svg.simplify() {
            log::warn!("simplifying Svg while generating StrokeContent Svg failed, Err: {e:?}");
        };
        Ok(Some(content_svg))
    }

    pub fn draw_to_cairo(
        &self,
        cairo_cx: &cairo::Context,
        draw_background: bool,
        draw_pattern: bool,
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
                background.draw_to_cairo(cairo_cx, bounds_loosened, draw_pattern)?;
            }
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

        for stroke in self.strokes.iter() {
            stroke.draw_to_cairo(cairo_cx, image_scale)?;
        }

        cairo_cx.restore()?;

        Ok(())
    }
}
