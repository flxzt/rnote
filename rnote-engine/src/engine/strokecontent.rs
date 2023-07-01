// Imports
use crate::document::Background;
use crate::render::Svg;
use crate::strokes::Stroke;
use crate::{DrawBehaviour, RnoteEngine};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::shapes::ShapeBehaviour;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Stroke content. Used when copying/cutting/pasting a selection into/from the clipboard.
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

    pub fn size(&self) -> Option<na::Vector2<f64>> {
        self.bounds().map(|b| b.extents())
    }

    pub fn bounds(&self) -> Option<Aabb> {
        if self.strokes.is_empty() {
            return None;
        }
        if self.bounds.is_some() {
            return self.bounds;
        }
        Some(
            self.strokes
                .iter()
                .map(|s| s.bounds())
                .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x)),
        )
    }

    /// Generate a Svg from the content.
    pub fn generate_svg(&self) -> anyhow::Result<Option<Svg>> {
        if self.strokes.is_empty() {
            return Ok(None);
        }
        let Some(bounds) = self.bounds() else {
            return Ok(None)
        };
        let mut svg = Svg {
            svg_data: String::new(),
            bounds,
        };
        svg.merge([Svg::gen_with_piet_cairo_backend(
            |piet_cx| {
                for stroke in self.strokes.iter() {
                    stroke.draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                }
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
        image_scale: f64,
    ) -> anyhow::Result<()> {
        let Some(bounds) = self.bounds() else { return Ok(()) };
        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

        if draw_background {
            if let Some(background) = &self.background {
                background.draw_to_cairo(&cairo_cx, bounds, draw_pattern)?;
            }
        }
        for stroke in self.strokes.iter() {
            stroke.draw(&mut piet_cx, image_scale)?;
        }
        Ok(())
    }
}
