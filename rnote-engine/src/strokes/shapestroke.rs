use crate::compose;
use crate::compose::geometry::AABBHelpers;
use crate::compose::rough::roughoptions::RoughOptions;
use crate::compose::smooth::SmoothOptions;
use crate::compose::transformable::{Transform, Transformable};
use crate::compose::{curves, rough, shapes};
use crate::drawbehaviour::DrawBehaviour;
use crate::pens::shaper::ShaperDrawStyle;
use crate::render;
use crate::strokes::element::Element;
use crate::strokes::inputdata::InputData;
use crate::{pens::shaper::Shaper, pens::shaper::ShaperStyle};

use p2d::bounding_volume::{BoundingVolume, AABB};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shape_drawstyle")]
pub enum ShapeDrawStyle {
    #[serde(rename = "smooth")]
    Smooth {
        #[serde(rename = "options")]
        options: SmoothOptions
    },
    #[serde(rename = "rough")]
    Rough {
        #[serde(rename = "options")]
        options: RoughOptions
    },
}

impl Transformable for Shape {
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
    #[serde(rename = "drawstyle")]
    pub drawstyle: ShapeDrawStyle,
    #[serde(rename = "bounds")]
    pub bounds: AABB,
}

impl Default for ShapeStroke {
    fn default() -> Self {
        Self::new(Element::new(InputData::default()), &Shaper::default())
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
        match &self.drawstyle {
            ShapeDrawStyle::Smooth { options } => {
                let width = options.width;
                Some(
                    self.shape
                        .bounds()
                        .loosened(width * 0.5 + ShaperDrawStyle::SMOOTH_MARGIN),
                )
            }
            ShapeDrawStyle::Rough { options } => {
                let width = options.stroke_width();
                Some(
                    self.shape
                        .bounds()
                        .loosened(width * 0.5 + ShaperDrawStyle::ROUGH_MARGIN),
                )
            }
        }
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        let element: svg::node::element::Element = match self.shape {
            Shape::Line(ref line) => {
                let line = curves::Line {
                    start: line.start + offset,
                    end: line.end + offset,
                };

                match &self.drawstyle {
                    ShapeDrawStyle::Smooth { options } => {
                        let color = if let Some(color) = options.stroke_color {
                            color.to_css_color()
                        } else {
                            String::from("none")
                        };

                        let fill = if let Some(fill) = options.fill_color {
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
                            .set("stroke-width", options.width)
                            .set("fill", fill)
                            .into()
                    }
                    ShapeDrawStyle::Rough { options } => svg::node::element::Group::new()
                        .add(rough::line(line, options))
                        .into(),
                }
            }
            Shape::Rectangle(ref rectangle) => {
                let mut rectangle = rectangle.clone();
                rectangle.transform.append_translation_mut(offset);

                match &self.drawstyle {
                    ShapeDrawStyle::Smooth { options } => {
                        compose::smooth::compose_rectangle(rectangle, &options)
                    }
                    ShapeDrawStyle::Rough { options } => {
                        rough::rectangle(rectangle, options).into()
                    }
                }
            }
            Shape::Ellipse(ref ellipse) => {
                let mut ellipse = ellipse.clone();
                ellipse.transform.append_translation_mut(offset);

                match &self.drawstyle {
                    ShapeDrawStyle::Smooth { options } => {
                        compose::smooth::compose_ellipse(ellipse, &options)
                    }
                    ShapeDrawStyle::Rough { options } => rough::ellipse(ellipse, options).into(),
                }
            }
        };

        let svg_data = compose::svg_node_to_string(&element).map_err(|e| {
            anyhow::anyhow!(
                "node_to_string() failed in gen_svg_data() for a shapestroke, {}",
                e
            )
        })?;

        let svg = render::Svg {
            bounds: self.bounds.translate(offset),
            svg_data,
        };
        Ok(vec![svg])
    }
}

impl Transformable for ShapeStroke {
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

        let mut shapestroke = Self {
            shape,
            drawstyle,
            bounds,
            seed,
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
                let diff = element.inputdata.pos() - shaper.rect_current;

                rectangle.cuboid.half_extents =
                    ((element.inputdata.pos() - shaper.rect_start) / 2.0).abs();
                rectangle.transform.transform =
                    na::Translation2::from(diff / 2.0) * rectangle.transform.transform;

                shaper.rect_current = element.inputdata.pos();
            }
            Shape::Ellipse(ref mut ellipse) => {
                let center = ellipse
                    .transform
                    .transform
                    .transform_point(&na::point![0.0, 0.0]);

                let diff = element.inputdata.pos() - center.coords;

                ellipse.radii = diff.abs();
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
