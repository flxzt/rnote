// Imports
use super::strokebehaviour::GeneratedStrokeImages;
use super::StrokeBehaviour;
use crate::{render, strokes::strokebehaviour, DrawBehaviour};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::helpers::{AabbHelpers, Vector2Helpers};
use rnote_compose::shapes::Shape;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::Composer;
use rnote_compose::transform::TransformBehaviour;
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

impl StrokeBehaviour for ShapeStroke {
    fn gen_svg(&self) -> Result<crate::render::Svg, anyhow::Error> {
        let bounds = self.bounds();

        render::Svg::gen_with_piet_cairo_backend(
            |cx| {
                cx.transform(kurbo::Affine::translate(-bounds.mins.coords.to_kurbo_vec()));
                self.draw(cx, 1.0)
            },
            bounds,
        )
    }

    fn gen_images(
        &self,
        viewport: Aabb,
        image_scale: f64,
    ) -> Result<GeneratedStrokeImages, anyhow::Error> {
        let bounds = self.bounds();

        if viewport.contains(&bounds) {
            Ok(GeneratedStrokeImages::Full(vec![
                render::Image::gen_with_piet(
                    |piet_cx| self.draw(piet_cx, image_scale),
                    bounds,
                    image_scale,
                )?,
            ]))
        } else if let Some(intersection_bounds) = viewport.intersection(&bounds) {
            Ok(GeneratedStrokeImages::Partial {
                images: vec![render::Image::gen_with_piet(
                    |piet_cx| self.draw(piet_cx, image_scale),
                    intersection_bounds,
                    image_scale,
                )?],
                viewport,
            })
        } else {
            Ok(GeneratedStrokeImages::Partial {
                images: vec![],
                viewport,
            })
        }
    }

    fn draw_highlight(
        &self,
        cx: &mut impl piet::RenderContext,
        total_zoom: f64,
    ) -> anyhow::Result<()> {
        const HIGHLIGHT_STROKE_WIDTH: f64 = 1.5;
        cx.stroke(
            self.bounds().to_kurbo_rect(),
            &*strokebehaviour::STROKE_HIGHLIGHT_COLOR,
            HIGHLIGHT_STROKE_WIDTH / total_zoom,
        );
        Ok(())
    }

    fn update_geometry(&mut self) {
        self.hitboxes = self.gen_hitboxes_int();
    }
}

impl DrawBehaviour for ShapeStroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        self.shape.draw_composed(cx, &self.style);

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl ShapeBehaviour for ShapeStroke {
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
}

impl TransformBehaviour for ShapeStroke {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.shape.translate(offset);
    }
    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.shape.rotate(angle, center);
    }
    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.shape.scale(scale);
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
