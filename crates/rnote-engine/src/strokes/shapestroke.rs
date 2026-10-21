// Imports
use super::Content;
use crate::{Drawable, strokes::content};
use kurbo::Shape as _;
use kurbo::{Cap, Join, Stroke};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::math::Vector2;
use rnote_compose::Style;
use rnote_compose::Transformable;
use rnote_compose::ext::AabbExt;
use rnote_compose::shapes::Shape;
use rnote_compose::shapes::Shapeable;
use rnote_compose::style::Composer;
use serde::{Deserialize, Serialize};
use vello_cpu::RenderContext;

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
    fn draw_highlight(&self, cx: &mut RenderContext, total_zoom: f64) -> anyhow::Result<()> {
        const PATH_HIGHLIGHT_MIN_STROKE_WIDTH: f64 = 5.0;
        const DRAW_BOUNDS_THRESHOLD_AREA: f64 = 10_u32.pow(2) as f64;
        let bounds = self.bounds();
        let bez_path = self.shape.outline_path();

        if bounds.scale(total_zoom).volume() < DRAW_BOUNDS_THRESHOLD_AREA {
            cx.set_paint(content::CONTENT_HIGHLIGHT_COLOR);
            cx.fill_path(&bounds.to_kurbo_rect().into_path(0.1));
        } else {
            cx.set_paint(content::CONTENT_HIGHLIGHT_COLOR);
            cx.set_stroke(
                Stroke::new(
                    (PATH_HIGHLIGHT_MIN_STROKE_WIDTH / total_zoom)
                        .max(self.style.stroke_width() + 10.0 / total_zoom),
                )
                .with_join(Join::Round)
                .with_caps(Cap::Round),
            );
            cx.stroke_path(&bez_path);
        }

        Ok(())
    }

    fn update_geometry(&mut self) {
        self.hitboxes = self.gen_hitboxes_int();
    }
}

impl Drawable for ShapeStroke {
    fn draw(&self, cx: &mut RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        self.shape.composed_draw(cx, &self.style);
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
    fn translate(&mut self, offset: Vector2) {
        self.shape.translate(offset);
    }
    fn rotate(&mut self, angle: f64, center: Vector2) {
        self.shape.rotate(angle, center);
    }
    fn scale(&mut self, scale: Vector2) {
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
