use super::strokebehaviour::GeneratedStrokeImages;
use super::StrokeBehaviour;
use crate::{render, DrawBehaviour};
use rnote_compose::shapes::Shape;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::Composer;
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::Style;

use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "shapestroke")]
pub struct ShapeStroke {
    #[serde(rename = "shape")]
    pub shape: Shape,
    #[serde(rename = "style")]
    pub style: Style,
    #[serde(skip)]
    // since the path can have many hitboxes, we store them for faster queries and update them when we the stroke geometry changes
    pub hitboxes: Vec<AABB>,
}

impl StrokeBehaviour for ShapeStroke {
    fn gen_svg(&self) -> Result<crate::render::Svg, anyhow::Error> {
        let bounds = self.bounds();
        let mut cx = piet_svg::RenderContext::new_no_text(kurbo::Size::new(
            bounds.extents()[0],
            bounds.extents()[1],
        ));

        self.draw(&mut cx, 1.0)?;
        let svg_data = rnote_compose::utils::piet_svg_cx_to_svg(cx)?;

        Ok(render::Svg { svg_data, bounds })
    }

    fn gen_images(
        &self,
        viewport: AABB,
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
        } else {
            Ok(GeneratedStrokeImages::Partial {
                images: vec![render::Image::gen_with_piet(
                    |piet_cx| self.draw(piet_cx, image_scale),
                    viewport,
                    image_scale,
                )?],
                viewport,
            })
        }
    }
}

impl DrawBehaviour for ShapeStroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        self.shape.draw_composed(cx, &self.style);

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

impl ShapeBehaviour for ShapeStroke {
    fn bounds(&self) -> AABB {
        match &self.style {
            Style::Smooth(options) => self.shape.composed_bounds(options),
            Style::Rough(options) => self.shape.composed_bounds(options),
            Style::Textured(_) => self.shape.bounds(),
        }
    }

    fn hitboxes(&self) -> Vec<AABB> {
        self.hitboxes.clone()
    }
}

impl TransformBehaviour for ShapeStroke {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.shape.translate(offset);
    }
    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.shape.rotate(angle, center);
    }
    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
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

    pub fn update_geometry(&mut self) {
        self.hitboxes = self.gen_hitboxes();
    }

    fn gen_hitboxes(&self) -> Vec<AABB> {
        let width = self.style.stroke_width();

        self.shape
            .hitboxes()
            .into_iter()
            .map(|hitbox| hitbox.loosened(width / 2.0))
            .collect()
    }

    /*
    pub fn update_shape(&mut self, shaper: &mut Shaper, element: Element) {
        match self.shape {
            Shape::Line(ref mut line) => {
                line.end = element.inputdata.pos();
            }
            Shape::Rectangle(ref mut rectangle) => {
                let relative_pos = element.inputdata.pos() - shaper.rect_start;
                let constrained_relative_pos = Self::constrain(relative_pos, shaper.ratio);

                rectangle.cuboid.half_extents = (constrained_relative_pos / 2.0).abs();

                let diff = constrained_relative_pos - shaper.rect_current + shaper.rect_start;
                rectangle.transform.transform *= na::Translation2::from(diff / 2.0);

                shaper.rect_current = shaper.rect_start + constrained_relative_pos;
            }
            Shape::Ellipse(ref mut ellipse) => {
                let center = ellipse
                    .transform
                    .transform
                    .transform_point(&na::point![0.0, 0.0]);

                let diff = element.inputdata.pos() - center.coords;
                ellipse.radii = Self::constrain(diff.abs(), shaper.ratio);
            }
        }

        self.update_geometry();
    }
    */
}
