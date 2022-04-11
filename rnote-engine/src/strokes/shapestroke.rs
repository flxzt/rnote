use super::StrokeBehaviour;
use crate::pens::shaper::ShaperStyle;
use crate::pens::Shaper;
use crate::DrawBehaviour;
use rnote_compose::penpath::Element;
use rnote_compose::shapes::Line;
use rnote_compose::shapes::Rectangle;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::shapes::ShapeType;
use rnote_compose::shapes::{Ellipse, Shape};
use rnote_compose::style::Composer;
use rnote_compose::transform::Transform;
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::Style;

use p2d::bounding_volume::AABB;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "shapestroke")]
pub struct ShapeStroke {
    #[serde(rename = "shape")]
    pub shape: Shape,
    #[serde(rename = "style")]
    pub style: Style,
}

impl Default for ShapeStroke {
    fn default() -> Self {
        Self::new(Element::default(), &Shaper::default())
    }
}

impl StrokeBehaviour for ShapeStroke {}

impl DrawBehaviour for ShapeStroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        match self.shape {
            Shape::Line(ref line) => match &self.style {
                Style::Smooth(options) => {
                    line.draw_composed(cx, options);
                }
                Style::Rough(options) => {
                    line.draw_composed(cx, options);
                }
                Style::Textured(_) => {}
            },
            Shape::Rectangle(ref rectangle) => match &self.style {
                Style::Smooth(options) => {
                    rectangle.draw_composed(cx, options);
                }
                Style::Rough(options) => {
                    rectangle.draw_composed(cx, options);
                }
                Style::Textured(_) => {}
            },
            Shape::Ellipse(ref ellipse) => match &self.style {
                Style::Smooth(options) => {
                    ellipse.draw_composed(cx, options);
                }
                Style::Rough(options) => {
                    ellipse.draw_composed(cx, options);
                }
                Style::Textured(_) => {}
            },
        };

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
    pub fn new(element: Element, shaper: &Shaper) -> Self {
        let seed = Some(rand_pcg::Pcg64::from_entropy().gen());

        let shape = match shaper.shape_type {
            ShapeType::Line => Shape::Line(Line {
                start: element.pos,
                end: element.pos,
            }),
            ShapeType::Rectangle => Shape::Rectangle(Rectangle {
                cuboid: p2d::shape::Cuboid::new(na::vector![0.0, 0.0]),
                transform: Transform::new_w_isometry(na::Isometry2::new(element.pos, 0.0)),
            }),
            ShapeType::Ellipse => Shape::Ellipse(Ellipse {
                radii: na::vector![0.0, 0.0],
                transform: Transform::new_w_isometry(na::Isometry2::<f64>::new(element.pos, 0.0)),
            }),
        };
        let drawstyle = match shaper.style {
            ShaperStyle::Smooth => {
                let options = shaper.smooth_options;

                Style::Smooth(options)
            }
            ShaperStyle::Rough => {
                let mut options = shaper.rough_options.clone();
                options.seed = seed;

                Style::Rough(options)
            }
        };

        Self {
            shape,
            style: drawstyle,
        }
    }

    pub fn update_shape(&mut self, shaper: &mut Shaper, element: Element) {
        match self.shape {
            Shape::Line(ref mut line) => {
                line.end = element.pos;
            }
            Shape::Rectangle(ref mut rectangle) => {
                let diff = element.pos - shaper.rect_current;

                rectangle.cuboid.half_extents = ((element.pos - shaper.rect_start) / 2.0).abs();
                rectangle.transform.affine =
                    na::Translation2::from(diff / 2.0) * rectangle.transform.affine;

                shaper.rect_current = element.pos;
            }
            Shape::Ellipse(ref mut ellipse) => {
                let center = ellipse
                    .transform
                    .affine
                    .transform_point(&na::point![0.0, 0.0]);

                let diff = element.pos - center.coords;

                ellipse.radii = diff.abs();
            }
        }
    }
}
