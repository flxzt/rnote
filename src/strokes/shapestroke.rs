use super::{Element, StrokeBehaviour};
use crate::pens::shaper::{self, DrawStyle};
use crate::strokes::compose;
use crate::utils;
use crate::{pens::shaper::CurrentShape, pens::shaper::Shaper, strokes::render};

use gtk4::gsk;
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeStyle {
    Line {
        start: na::Vector2<f64>, // The position of the line start
        end: na::Vector2<f64>,   // The position of the line end
    },
    Rectangle {
        start: na::Vector2<f64>, // The position of the rect start
        end: na::Vector2<f64>,   // The position of the rect end
    },
    Ellipse {
        pos: na::Vector2<f64>, // The center position
        radius_x: f64,         // The radius
        radius_y: f64,         // The radius
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeStroke {
    pub shape_style: ShapeStyle,
    pub shaper: Shaper,
    pub bounds: p2d::bounding_volume::AABB,
    pub seed: Option<u64>,
    #[serde(skip, default = "render::default_rendernode")]
    pub rendernode: gsk::RenderNode,
}

impl Default for ShapeStroke {
    fn default() -> Self {
        Self::new(Element::default(), Shaper::default())
    }
}

impl StrokeBehaviour for ShapeStroke {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        self.bounds
    }

    fn translate(&mut self, offset: na::Vector2<f64>) {
        match self.shape_style {
            ShapeStyle::Line {
                ref mut start,
                ref mut end,
            } => {
                *start = *start + offset;
                *end = *end + offset;
            }
            ShapeStyle::Rectangle {
                ref mut start,
                ref mut end,
            } => {
                *start = *start + offset;
                *end = *end + offset;
            }
            ShapeStyle::Ellipse {
                ref mut pos,
                radius_x: _,
                radius_y: _,
            } => {
                *pos = *pos + offset;
            }
        }

        self.update_bounds();
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        match self.shape_style {
            ShapeStyle::Line {
                ref mut start,
                ref mut end,
            } => {
                let offset = na::vector![
                    new_bounds.mins[0] - self.bounds.mins[0],
                    new_bounds.mins[1] - self.bounds.mins[1]
                ];

                let scalevector = na::vector![
                    (new_bounds.maxs[0] - new_bounds.mins[0])
                        / (self.bounds.maxs[0] - self.bounds.mins[0]),
                    (new_bounds.maxs[1] - new_bounds.mins[1])
                        / (self.bounds.maxs[1] - self.bounds.mins[1])
                ];
                let top_left = na::vector![self.bounds.mins[0], self.bounds.mins[1]];

                *start = (*start - top_left).component_mul(&scalevector) + top_left + offset;
                *end = (*end - top_left).component_mul(&scalevector) + top_left + offset;
            }
            ShapeStyle::Rectangle {
                ref mut start,
                ref mut end,
            } => {
                *start = na::vector![new_bounds.mins[0], new_bounds.mins[1]]
                    + na::Vector2::<f64>::from_element(self.shaper.rectangle_config.width());
                *end = na::vector![new_bounds.maxs[0], new_bounds.maxs[1]]
                    - na::Vector2::<f64>::from_element(self.shaper.rectangle_config.width());
            }
            ShapeStyle::Ellipse {
                ref mut pos,
                ref mut radius_x,
                ref mut radius_y,
            } => {
                let center = na::vector![
                    new_bounds.mins[0] + (new_bounds.maxs[0] - new_bounds.mins[0]) / 2.0,
                    new_bounds.mins[1] + (new_bounds.maxs[1] - new_bounds.mins[1]) / 2.0
                ];
                *pos = center;

                *radius_x = (new_bounds.maxs[0] - new_bounds.mins[0]) / 2.0
                    - self.shaper.ellipse_config.width();
                *radius_y = (new_bounds.maxs[1] - new_bounds.mins[1]) / 2.0
                    - self.shaper.ellipse_config.width();
            }
        }

        self.bounds = new_bounds;
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn std::error::Error>> {
        let mut svg = String::new();

        let element: svg::node::element::Element = match self.shape_style {
            ShapeStyle::Line { ref start, ref end } => match self.shaper.drawstyle {
                shaper::DrawStyle::Smooth => {
                    let color = if let Some(color) = self.shaper.line_config.color {
                        color.to_css_color()
                    } else {
                        String::from("none")
                    };

                    let fill = if let Some(fill) = self.shaper.line_config.fill {
                        fill.to_css_color()
                    } else {
                        String::from("none")
                    };

                    svg::node::element::Line::new()
                        .set("x1", start[0] + offset[0])
                        .set("y1", start[1] + offset[1])
                        .set("x2", end[0] + offset[0])
                        .set("y2", end[1] + offset[1])
                        .set("stroke", color)
                        .set("stroke-width", self.shaper.line_config.width())
                        .set("fill", fill)
                        .into()
                }
                shaper::DrawStyle::Rough => {
                    let mut rough_config = self.shaper.roughconfig.clone();

                    if let Some(color) = self.shaper.line_config.color {
                        rough_config.stroke = Some(rough_rs::utils::Color::new(
                            color.r, color.g, color.b, color.a,
                        ));
                    }
                    if let Some(fill) = self.shaper.line_config.fill {
                        rough_config.fill =
                            Some(rough_rs::utils::Color::new(fill.r, fill.g, fill.b, fill.a));
                    }

                    rough_config.stroke_width = self.shaper.line_config.width();
                    rough_config.seed = self.seed;

                    let mut rough_generator =
                        rough_rs::generator::RoughGenerator::new(Some(rough_config));

                    svg::node::element::Group::new()
                        .add(rough_generator.line(start + offset, end + offset))
                        .into()
                }
            },
            ShapeStyle::Rectangle { ref start, ref end } => match self.shaper.drawstyle {
                shaper::DrawStyle::Smooth => {
                    let color = if let Some(color) = self.shaper.rectangle_config.color {
                        color.to_css_color()
                    } else {
                        String::from("none")
                    };
                    let fill = if let Some(fill) = self.shaper.rectangle_config.fill {
                        fill.to_css_color()
                    } else {
                        String::from("none")
                    };

                    let (mins, maxs) = utils::vec2_mins_maxs(*start, *end);

                    svg::node::element::Rectangle::new()
                        .set("x", mins[0] + offset[0])
                        .set("y", mins[1] + offset[1])
                        .set("width", maxs[0] - mins[0])
                        .set("height", maxs[1] - mins[1])
                        .set("stroke", color)
                        .set("stroke-width", self.shaper.rectangle_config.width())
                        .set("fill", fill)
                        .into()
                }
                shaper::DrawStyle::Rough => {
                    let mut rough_config = self.shaper.roughconfig.clone();

                    if let Some(color) = self.shaper.rectangle_config.color {
                        rough_config.stroke = Some(rough_rs::utils::Color::new(
                            color.r, color.g, color.b, color.a,
                        ));
                    }
                    if let Some(fill) = self.shaper.rectangle_config.fill {
                        rough_config.fill =
                            Some(rough_rs::utils::Color::new(fill.r, fill.g, fill.b, fill.a));
                    }

                    rough_config.stroke_width = self.shaper.rectangle_config.width();
                    rough_config.seed = self.seed;

                    let mut rough_generator =
                        rough_rs::generator::RoughGenerator::new(Some(rough_config));

                    rough_generator
                        .rectangle(start + offset, end + offset)
                        .into()
                }
            },
            ShapeStyle::Ellipse {
                ref pos,
                ref radius_x,
                ref radius_y,
            } => match self.shaper.drawstyle {
                shaper::DrawStyle::Smooth => {
                    let color = if let Some(color) = self.shaper.ellipse_config.color {
                        color.to_css_color()
                    } else {
                        String::from("none")
                    };
                    let fill = if let Some(fill) = self.shaper.ellipse_config.fill {
                        fill.to_css_color()
                    } else {
                        String::from("none")
                    };

                    svg::node::element::Ellipse::new()
                        .set("cx", pos[0] + offset[0])
                        .set("cy", pos[1] + offset[1])
                        .set("rx", *radius_x)
                        .set("ry", *radius_y)
                        .set("stroke", color)
                        .set("stroke-width", self.shaper.ellipse_config.width())
                        .set("fill", fill)
                        .into()
                }
                shaper::DrawStyle::Rough => {
                    let mut rough_config = self.shaper.roughconfig.clone();

                    if let Some(color) = self.shaper.ellipse_config.color {
                        rough_config.stroke = Some(rough_rs::utils::Color::new(
                            color.r, color.g, color.b, color.a,
                        ));
                    }
                    if let Some(fill) = self.shaper.ellipse_config.fill {
                        rough_config.fill =
                            Some(rough_rs::utils::Color::new(fill.r, fill.g, fill.b, fill.a));
                    }

                    rough_config.stroke_width = self.shaper.ellipse_config.width();
                    rough_config.seed = self.seed;

                    let mut rough_generator =
                        rough_rs::generator::RoughGenerator::new(Some(rough_config));

                    rough_generator
                        .ellipse(pos + offset, *radius_x, *radius_y)
                        .into()
                }
            },
        };

        svg += rough_rs::node_to_string(&element)?.as_str();
        //println!("{}", svg);
        Ok(svg)
    }

    fn update_rendernode(&mut self, scalefactor: f64, renderer: &render::Renderer) {
        if let Ok(rendernode) = self.gen_rendernode(scalefactor, renderer) {
            self.rendernode = rendernode;
        } else {
            log::error!("failed to gen_rendernode() in update_rendernode() of shapestroke");
        }
    }

    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn std::error::Error>> {
        let svg = compose::wrap_svg(
            self.gen_svg_data(na::vector![0.0, 0.0])?.as_str(),
            Some(self.bounds),
            Some(self.bounds),
            true,
            false,
        );
        renderer.gen_rendernode(self.bounds, scalefactor, svg.as_str())
    }
}

impl ShapeStroke {
    pub fn new(element: Element, shaper: Shaper) -> Self {
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![element.inputdata.pos()[0], element.inputdata.pos()[1]],
            na::point![
                element.inputdata.pos()[0] + 1.0,
                element.inputdata.pos()[1] + 1.0
            ],
        );

        let seed = Some(rough_rs::utils::random_u64_full(None));

        let shape_style = match shaper.current_shape {
            CurrentShape::Line => ShapeStyle::Line {
                start: element.inputdata.pos(),
                end: element.inputdata.pos(),
            },
            CurrentShape::Rectangle => ShapeStyle::Rectangle {
                start: element.inputdata.pos(),
                end: element.inputdata.pos(),
            },
            CurrentShape::Ellipse => ShapeStyle::Ellipse {
                pos: element.inputdata.pos(),
                radius_x: 0.0,
                radius_y: 0.0,
            },
        };

        let mut shapestroke = Self {
            shape_style,
            shaper,
            bounds,
            seed,
            rendernode: render::default_rendernode(),
        };

        shapestroke.update_bounds();

        shapestroke
    }

    pub fn update_shape(&mut self, element: Element) {
        match self.shape_style {
            ShapeStyle::Line {
                start: _,
                ref mut end,
            } => {
                *end = element.inputdata.pos();
            }
            ShapeStyle::Rectangle {
                start: _,
                ref mut end,
            } => {
                *end = element.inputdata.pos();
            }
            ShapeStyle::Ellipse {
                ref pos,
                ref mut radius_x,
                ref mut radius_y,
            } => {
                let delta = element.inputdata.pos() - *pos;
                *radius_x = delta[0].abs();
                *radius_y = delta[1].abs();
            }
        }

        self.update_bounds();
    }

    pub fn update_bounds(&mut self) {
        match self.shape_style {
            ShapeStyle::Line { ref start, ref end } => match self.shaper.drawstyle {
                shaper::DrawStyle::Smooth => {
                    self.bounds = utils::aabb_new_positive(*start, *end)
                        .loosened(self.shaper.line_config.width() * 0.5);
                }
                shaper::DrawStyle::Rough => {
                    self.bounds = utils::aabb_new_positive(*start, *end)
                        // TODO what are the actual bounds for a rough line?
                        .loosened(self.shaper.line_config.width() * 0.5 + DrawStyle::ROUGH_MARGIN * 2.0);
                }
            },
            ShapeStyle::Rectangle { ref start, ref end } => {
                match self.shaper.drawstyle {
                    shaper::DrawStyle::Smooth => {
                        self.bounds = utils::aabb_new_positive(*start, *end)
                            .loosened(self.shaper.rectangle_config.width() * 0.5);
                    }
                    shaper::DrawStyle::Rough => {
                        self.bounds = utils::aabb_new_positive(*start, *end)
                            // TODO what are the actual bounds for a rough rect?
                            .loosened(self.shaper.rectangle_config.width() * 0.5 + DrawStyle::ROUGH_MARGIN * 2.0);
                    }
                };
            }
            ShapeStyle::Ellipse {
                ref pos,
                ref radius_x,
                ref radius_y,
            } => match self.shaper.drawstyle {
                shaper::DrawStyle::Smooth => {
                    self.bounds = utils::aabb_new_positive(
                        na::vector![pos[0] - radius_x, pos[1] - radius_y],
                        na::vector![pos[0] + radius_x, pos[1] + radius_y],
                    )
                    .loosened(self.shaper.ellipse_config.width());
                }
                shaper::DrawStyle::Rough => {
                    self.bounds = utils::aabb_new_positive(
                        na::vector![pos[0] - radius_x, pos[1] - radius_y],
                        na::vector![pos[0] + radius_x, pos[1] + radius_y],
                    )
                    // TODO what are the actual bounds for a rough ellipse?
                    .loosened(self.shaper.ellipse_config.width() + DrawStyle::ROUGH_MARGIN * 2.0);
                }
            },
        }
    }

    pub fn complete_stroke(&mut self) {
        self.update_bounds();
    }
}
