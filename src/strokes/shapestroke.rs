use crate::compose::{curves, geometry, rough, shapes};
use crate::drawbehaviour::DrawBehaviour;
use crate::pens::shaper::{self, DrawStyle};
use crate::strokes::strokebehaviour::StrokeBehaviour;
use crate::strokes::strokestyle::Element;
use crate::{compose, render};
use crate::{pens::shaper::ShapeStyle, pens::shaper::Shaper};

use p2d::bounding_volume::{BoundingVolume, AABB};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use super::strokebehaviour;
use super::strokestyle::InputData;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shape")]
pub enum Shape {
    #[serde(rename = "line")]
    Line(curves::Line),
    #[serde(rename = "rectangle")]
    Rectangle(shapes::Rectangle),
    #[serde(rename = "ellipse")]
    Ellipse(shapes::Ellipse),
}

impl StrokeBehaviour for Shape {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        match self {
            Self::Line(line) => {
                line.translate(offset);
            }
            Self::Rectangle(rectangle) => {
                rectangle.translate(offset);
            }
            Self::Ellipse(ellipse) => {
                ellipse.translate(offset);
            }
        }
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        match self {
            Self::Line(line) => {
                line.rotate(angle, center);
            }
            Self::Rectangle(rectangle) => {
                rectangle.rotate(angle, center);
            }
            Self::Ellipse(ellipse) => {
                ellipse.rotate(angle, center);
            }
        }
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        match self {
            Self::Line(line) => {
                line.scale(scale);
            }
            Self::Rectangle(rectangle) => {
                rectangle.scale(scale);
            }
            Self::Ellipse(ellipse) => {
                ellipse.scale(scale);
            }
        }
    }
}

impl Shape {
    pub fn bounds(&self) -> AABB {
        match self {
            Self::Line(line) => line.global_aabb(),
            Self::Rectangle(rectangle) => rectangle.global_aabb(),
            Self::Ellipse(ellipse) => ellipse.global_aabb(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "shapestroke")]
pub struct ShapeStroke {
    #[serde(rename = "seed")]
    pub seed: Option<u64>,
    #[serde(rename = "shape")]
    pub shape: Shape,
    #[serde(rename = "shaper")]
    pub shaper: Shaper,
    #[serde(rename = "bounds")]
    pub bounds: AABB,
}

impl Default for ShapeStroke {
    fn default() -> Self {
        Self::new(Element::new(InputData::default()), Shaper::default())
    }
}

impl DrawBehaviour for ShapeStroke {
    fn bounds(&self) -> AABB {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: AABB) {
        self.bounds = bounds;
    }

    fn gen_bounds(&self) -> Option<AABB> {
        let new_bounds = match self.shaper.drawstyle() {
            shaper::DrawStyle::Smooth => self.shape.bounds().loosened(self.shaper.width() * 0.5),
            shaper::DrawStyle::Rough => {
                self.shape
                    .bounds()
                    // TODO what are the actual bounds for the rough shapes?
                    .loosened(self.shaper.width() * 0.5 + DrawStyle::ROUGH_MARGIN)
            }
        };

        Some(new_bounds)
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        let element: svg::node::element::Element = match self.shape {
            Shape::Line(ref line) => {
                let line = curves::Line {
                    start: line.start + offset,
                    end: line.end + offset,
                };

                match self.shaper.drawstyle() {
                    shaper::DrawStyle::Smooth => {
                        let color = if let Some(color) = self.shaper.color() {
                            color.to_css_color()
                        } else {
                            String::from("none")
                        };

                        let fill = if let Some(fill) = self.shaper.fill() {
                            fill.to_css_color()
                        } else {
                            String::from("none")
                        };

                        svg::node::element::Line::new()
                            .set("x1", line.start[0])
                            .set("y1", line.start[1])
                            .set("x2", line.end[0])
                            .set("y2", line.end[1])
                            .set("stroke", color)
                            .set("stroke-width", self.shaper.width())
                            .set("fill", fill)
                            .into()
                    }
                    shaper::DrawStyle::Rough => {
                        let mut rough_options = self.shaper.rough_config.clone();

                        if let Some(color) = self.shaper.color() {
                            rough_options.stroke =
                                Some(compose::Color::new(color.r, color.g, color.b, color.a));
                        }
                        if let Some(fill) = self.shaper.fill() {
                            rough_options.fill =
                                Some(compose::Color::new(fill.r, fill.g, fill.b, fill.a));
                        }

                        rough_options.stroke_width = self.shaper.width();
                        rough_options.seed = self.seed;

                        svg::node::element::Group::new()
                            .add(rough::line(&mut rough_options, line))
                            .into()
                    }
                }
            }
            Shape::Rectangle(ref rectangle) => {
                let mut rectangle = rectangle.clone();
                rectangle.transform.append_translation_mut(offset);

                match self.shaper.drawstyle() {
                    shaper::DrawStyle::Smooth => {
                        compose::solid::compose_rectangle(rectangle, &self.shaper)
                    }
                    shaper::DrawStyle::Rough => {
                        let mut rough_options = self.shaper.rough_config.clone();

                        if let Some(color) = self.shaper.color() {
                            rough_options.stroke =
                                Some(compose::Color::new(color.r, color.g, color.b, color.a));
                        }
                        if let Some(fill) = self.shaper.fill() {
                            rough_options.fill =
                                Some(compose::Color::new(fill.r, fill.g, fill.b, fill.a));
                        }

                        rough_options.stroke_width = self.shaper.width();
                        rough_options.seed = self.seed;

                        rough::rectangle(&mut rough_options, rectangle).into()
                    }
                }
            }
            Shape::Ellipse(ref ellipse) => {
                let mut ellipse = ellipse.clone();
                ellipse.transform.append_translation_mut(offset);

                match self.shaper.drawstyle() {
                    shaper::DrawStyle::Smooth => {
                        compose::solid::compose_ellipse(ellipse, &self.shaper)
                    }
                    shaper::DrawStyle::Rough => {
                        let mut rough_options = self.shaper.rough_config.clone();

                        if let Some(color) = self.shaper.color() {
                            rough_options.stroke =
                                Some(compose::Color::new(color.r, color.g, color.b, color.a));
                        }
                        if let Some(fill) = self.shaper.fill() {
                            rough_options.fill =
                                Some(compose::Color::new(fill.r, fill.g, fill.b, fill.a));
                        }

                        rough_options.stroke_width = self.shaper.width();
                        rough_options.seed = self.seed;

                        rough::ellipse(&mut rough_options, ellipse).into()
                    }
                }
            }
        };

        let svg_data = compose::node_to_string(&element).map_err(|e| {
            anyhow::anyhow!(
                "node_to_string() failed in gen_svg_data() for a shapestroke, {}",
                e
            )
        })?;

        let svg = render::Svg {
            bounds: geometry::aabb_translate(self.bounds, offset),
            svg_data,
        };
        Ok(vec![svg])
    }
}

impl StrokeBehaviour for ShapeStroke {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.shape.translate(offset);
        self.update_geometry();
    }
    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.shape.rotate(angle, center);
        self.update_geometry();
    }
    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.shape.scale(scale);
        self.update_geometry();
    }
}

impl ShapeStroke {
    pub fn new(element: Element, shaper: Shaper) -> Self {
        let seed = Some(rand_pcg::Pcg64::from_entropy().gen());

        let shape = match shaper.shapestyle() {
            ShapeStyle::Line => Shape::Line(curves::Line {
                start: element.inputdata.pos(),
                end: element.inputdata.pos(),
            }),
            ShapeStyle::Rectangle => Shape::Rectangle(shapes::Rectangle {
                cuboid: p2d::shape::Cuboid::new(na::vector![0.0, 0.0]),
                transform: strokebehaviour::StrokeTransform::new_w_isometry(na::Isometry2::new(
                    element.inputdata.pos(),
                    0.0,
                )),
            }),
            ShapeStyle::Ellipse => Shape::Ellipse(shapes::Ellipse {
                radii: na::vector![0.0, 0.0],
                transform: strokebehaviour::StrokeTransform::new_w_isometry(
                    na::Isometry2::<f64>::new(element.inputdata.pos(), 0.0),
                ),
            }),
        };
        let bounds = shape.bounds();

        let mut shapestroke = Self {
            shape,
            shaper,
            bounds,
            seed,
        };

        if let Some(new_bounds) = shapestroke.gen_bounds() {
            shapestroke.bounds = new_bounds;
        }

        shapestroke
    }

    pub fn update_shape(&mut self, element: Element) {
        match self.shape {
            Shape::Line(ref mut line) => {
                line.end = element.inputdata.pos();
            }
            Shape::Rectangle(ref mut rectangle) => {
                let offset = element.inputdata.pos()
                    - rectangle
                        .transform
                        .transform_point(na::point![0.0, 0.0])
                        .coords;

                if offset[0] > 0.0 {
                    rectangle.cuboid.half_extents[0] = offset[0];
                }
                if offset[1] > 0.0 {
                    rectangle.cuboid.half_extents[1] = offset[1];
                }
            }
            Shape::Ellipse(ref mut ellipse) => {
                let offset = element.inputdata.pos()
                    - ellipse
                        .transform
                        .transform_point(na::point![0.0, 0.0])
                        .coords;

                if offset[0] > 0.0 {
                    ellipse.radii[0] = offset[0];
                }
                if offset[1] > 0.0 {
                    ellipse.radii[1] = offset[1];
                }
            }
        }

        self.update_geometry();
    }

    pub fn update_geometry(&mut self) {
        if let Some(new_bounds) = self.gen_bounds() {
            self.bounds = new_bounds;
        }
    }
}
