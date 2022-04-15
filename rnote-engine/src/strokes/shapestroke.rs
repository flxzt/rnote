use super::StrokeBehaviour;
use crate::{render, DrawBehaviour};
use rnote_compose::shapes::Shape;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::Composer;
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::Style;

use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shapestroke")]
pub struct ShapeStroke {
    #[serde(rename = "shape")]
    pub shape: Shape,
    #[serde(rename = "style")]
    pub style: Style,
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

    fn gen_images(&self, image_scale: f64) -> Result<Vec<render::Image>, anyhow::Error> {
        Ok(render::Image::gen_images_from_drawable(
            self,
            self.bounds(),
            image_scale,
        )?)
    }
}

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
<<<<<<< HEAD
    pub fn new(element: Element, shaper: &Shaper) -> Self {
        let seed = Some(rand_pcg::Pcg64::from_entropy().gen());

        let shape = match shaper.style {
            ShaperStyle::Line => Shape::Line(curves::Line {
                start: element.inputdata.pos(),
                end: element.inputdata.pos(),
            }),
            ShaperStyle::Rectangle => Shape::Rectangle(shapes::Rectangle {
                cuboid: p2d::shape::Cuboid::new(na::vector![0.0, 0.0]),
                transform: Transform::new_w_isometry(na::Isometry2::new(
                    element.inputdata.pos(),
                    0.0,
                )),
            }),
            ShaperStyle::Ellipse => Shape::Ellipse(shapes::Ellipse {
                radii: na::vector![0.0, 0.0],
                transform: Transform::new_w_isometry(na::Isometry2::<f64>::new(
                    element.inputdata.pos(),
                    0.0,
                )),
            }),
        };
        let bounds = shape.bounds();
        let drawstyle = match shaper.drawstyle {
            ShaperDrawStyle::Smooth => {
                let mut options = shaper.smooth_options;
                options.seed = seed;

                ShapeDrawStyle::Smooth { options }
            }
            ShaperDrawStyle::Rough => {
                let mut options = shaper.rough_options.clone();
                options.seed = seed;

                ShapeDrawStyle::Rough { options }
            }
        };
        let ratio = shaper.ratio;

        let mut shapestroke = Self {
            shape,
            drawstyle,
            bounds,
            seed,
            ratio,
        };

        if let Some(new_bounds) = shapestroke.gen_bounds() {
            shapestroke.bounds = new_bounds;
        }

        shapestroke
    }

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

    pub fn update_geometry(&mut self) {
        if let Some(new_bounds) = self.gen_bounds() {
            self.bounds = new_bounds;
        }
=======
    pub fn new(shape: Shape, style: Style) -> Self {
        Self { shape, style }
>>>>>>> 26b5fbea4fb1225b97449ca1e1a6726cc071e1d8
    }

    fn constrain(pos: na::Vector2<f64>, ratio: ShaperConstraintRatio) -> na::Vector2<f64> {
        let max = pos.max();
        let dx = *pos.index((0, 0));
        let dy = *pos.index((1, 0));
        match ratio {
            ShaperConstraintRatio::Disabled => pos,
            ShaperConstraintRatio::OneToOne => na::vector![max, max],
            ShaperConstraintRatio::ThreeToTwo => {
                if dx > dy {
                    na::vector![dx, dx / 1.5]
                } else {
                    na::vector![dy / 1.5, dy]
                }
            }
            ShaperConstraintRatio::Golden => {
                if dx > dy {
                    na::vector![dx, dx / 1.618]
                } else {
                    na::vector![dy / 1.618, dy]
                }
            }
        }
    }
}
