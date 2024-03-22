// Imports
use super::Content;
use crate::{strokes::content, Drawable};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::ext::AabbExt;
use rnote_compose::shapes::Shape;
use rnote_compose::shapes::Shapeable;
use rnote_compose::style::Composer;
use rnote_compose::transform::Transformable;
use rnote_compose::Style;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "shapestroke")]
pub struct ShapeStroke {
    #[serde(rename = "shape")]
    pub shape: Shape,
    #[serde(rename = "style")]
    pub style: Style,
    #[serde(skip)]
    // since the shape can have many hitboxes, we store them and update them when the stroke geometry changes
    hitboxes: Vec<Aabb>,
}

impl Content for ShapeStroke {
    fn draw_highlight(
        &self,
        cx: &mut impl piet::RenderContext,
        total_zoom: f64,
    ) -> anyhow::Result<()> {
        const PATH_HIGHLIGHT_MIN_STROKE_WIDTH: f64 = 5.0;
        const DRAW_BOUNDS_THRESHOLD_AREA: f64 = 10_u32.pow(2) as f64;
        let bounds = self.bounds();
        let bez_path = self.shape.outline_path();

        if bounds.scale(total_zoom).volume() < DRAW_BOUNDS_THRESHOLD_AREA {
            cx.fill(bounds.to_kurbo_rect(), &content::CONTENT_HIGHLIGHT_COLOR);
        } else {
            cx.stroke_styled(
                bez_path,
                &content::CONTENT_HIGHLIGHT_COLOR,
                (PATH_HIGHLIGHT_MIN_STROKE_WIDTH / total_zoom)
                    .max(self.style.stroke_width() + 10.0 / total_zoom),
                &piet::StrokeStyle::new()
                    .line_join(piet::LineJoin::Round)
                    .line_cap(piet::LineCap::Round),
            );
        }

        Ok(())
    }

    fn update_geometry(&mut self) {
        self.hitboxes = self.gen_hitboxes_int();
    }
}

impl Drawable for ShapeStroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        self.shape.draw_composed(cx, &self.style);

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl Shapeable for ShapeStroke {
    fn bounds(&self) -> Aabb {
        match &self.style {
            Style::Smooth(options) => self.shape.composed_bounds(options),
            Style::Rough(options) => self.shape.composed_bounds(options),
            Style::Textured(_) => self.shape.bounds(),
        }
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        self.hitboxes.clone()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        self.shape.outline_path()
    }
}

impl Transformable for ShapeStroke {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.shape.translate(offset);
    }
    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.shape.rotate(angle, center);
    }
    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.shape.scale(scale);
        // Using the geometric mean behaves the best when scaling non-uniformly.
        let scale_scalar = (scale[0] * scale[1]).sqrt();
        self.style
            .set_stroke_width(self.style.stroke_width() * scale_scalar);
    }
}

impl ShapeStroke {
    pub fn new(shape: Shape, style: Style) -> Self {
        let mut shapestroke = Self {
            shape,
            style,
            hitboxes: vec![],
        };
        shapestroke.update_geometry();

        shapestroke
    }

    fn gen_hitboxes_int(&self) -> Vec<Aabb> {
        let width = self.style.stroke_width();

        self.shape
            .hitboxes()
            .into_iter()
            .map(|hitbox| hitbox.loosened(width * 0.5))
            .collect()
    }
}
