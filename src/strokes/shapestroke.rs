use super::StrokeBehaviour;
use crate::strokes::compose;
use crate::{
    pens::shaper::CurrentShape, pens::shaper::Shaper, strokes::render, strokes::InputData,
};

use gtk4::gsk;
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeStyle {
    Line {
        pos: na::Vector2<f64>, // The position of the line start
    },
    Rectangle {
        shape: p2d::shape::Cuboid,
        pos: na::Vector2<f64>, // The position of the upper left corner
    },
    Ellipse {
        shape: p2d::shape::Ball,
        pos: na::Vector2<f64>, // The center position
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeStroke {
    pub shape_style: ShapeStyle,
    pub shaper: Shaper,
    pub bounds: p2d::bounding_volume::AABB,
    #[serde(skip, default = "render::default_rendernode")]
    pub rendernode: gsk::RenderNode,
}

impl Default for ShapeStroke {
    fn default() -> Self {
        Self::new(InputData::default(), Shaper::default())
    }
}

impl StrokeBehaviour for ShapeStroke {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        self.bounds
    }

    fn translate(&mut self, offset: na::Vector2<f64>) {
        match self.shape_style {
            ShapeStyle::Line { ref mut pos } => {}
            ShapeStyle::Rectangle {
                shape: _,
                ref mut pos,
            } => {
                *pos = *pos + offset;
            }
            ShapeStyle::Ellipse {
                shape: _,
                ref mut pos,
            } => {
                *pos = *pos + offset;
            }
        }

        self.update_bounds();
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        match self.shape_style {
            ShapeStyle::Line { ref mut pos } => {}
            ShapeStyle::Rectangle {
                ref mut shape,
                ref mut pos,
            } => {
                *pos = na::vector![new_bounds.mins[0], new_bounds.mins[1]];
                shape.half_extents = (na::vector![new_bounds.maxs[0], new_bounds.maxs[1]]
                    - na::vector![new_bounds.mins[0], new_bounds.mins[1]])
                .scale(0.5);
            }
            ShapeStyle::Ellipse {
                ref mut shape,
                ref mut pos,
            } => {
                let center = na::vector![
                    new_bounds.mins[0] + (new_bounds.maxs[0] - new_bounds.mins[0]) / 2.0,
                    new_bounds.mins[1] + (new_bounds.maxs[1] - new_bounds.mins[1]) / 2.0
                ];
                *pos = center;

                shape.radius = (new_bounds.maxs[0] - new_bounds.mins[0])
                    .min(new_bounds.maxs[1] - new_bounds.mins[1])
                    / 2.0
                    - self.shaper.rectangle_config.width();
            }
        }

        self.bounds = new_bounds;
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn std::error::Error>> {
        let mut svg = String::new();

        let element: svg::node::element::Element = match self.shape_style {
            ShapeStyle::Line { ref pos } => svg::node::element::Rectangle::new().into(),
            ShapeStyle::Rectangle { ref shape, ref pos } => {
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

                svg::node::element::Rectangle::new()
                    .set("x", pos[0] + offset[0])
                    .set("y", pos[1] + offset[1])
                    .set("width", 2.0 * shape.half_extents[0])
                    .set("height", 2.0 * shape.half_extents[1])
                    .set("stroke", color)
                    .set("stroke-width", self.shaper.rectangle_config.width())
                    .set("fill", fill)
                    .into()
            }
            ShapeStyle::Ellipse { ref shape, ref pos } => {
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
                    .set("rx", shape.radius)
                    .set("ry", shape.radius)
                    .set("stroke", color)
                    .set("stroke-width", self.shaper.ellipse_config.width())
                    .set("fill", fill)
                    .into()
            }
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
    pub fn new(inputdata: InputData, shaper: Shaper) -> Self {
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![inputdata.pos()[0], inputdata.pos()[1]],
            na::point![inputdata.pos()[0] + 1.0, inputdata.pos()[1] + 1.0],
        );

        let shape_style = match shaper.current_shape {
            CurrentShape::Line => ShapeStyle::Line {
                pos: inputdata.pos(),
            },
            CurrentShape::Rectangle => ShapeStyle::Rectangle {
                shape: p2d::shape::Cuboid::new(na::vector![0.0, 0.0]),
                pos: inputdata.pos(),
            },
            CurrentShape::Ellipse => ShapeStyle::Ellipse {
                shape: p2d::shape::Ball::new(1.0),
                pos: inputdata.pos(),
            },
        };

        let mut shapestroke = Self {
            shape_style,
            shaper,
            bounds,
            rendernode: render::default_rendernode(),
        };

        shapestroke.update_bounds();

        shapestroke
    }

    pub fn update_shape(&mut self, inputdata: InputData) {
        match self.shape_style {
            ShapeStyle::Line { ref mut pos } => {}
            ShapeStyle::Rectangle {
                ref mut shape,
                ref mut pos,
            } => {
                let delta = inputdata.pos() - *pos;
                shape.half_extents = delta.scale(0.5).abs();
            }
            ShapeStyle::Ellipse {
                ref mut shape,
                ref pos,
            } => {
                let delta = inputdata.pos() - *pos;
                shape.radius = delta.norm().abs();
            }
        }

        self.update_bounds();
    }

    pub fn update_bounds(&mut self) {
        match self.shape_style {
            ShapeStyle::Line { ref pos } => {}
            ShapeStyle::Rectangle { ref shape, ref pos } => {
                self.bounds = shape
                    .aabb(&na::geometry::Isometry2::new(
                        *pos + shape.half_extents,
                        0.0,
                    ))
                    .loosened(self.shaper.rectangle_config.width());
            }
            ShapeStyle::Ellipse { ref shape, ref pos } => {
                self.bounds = shape
                    .aabb(&na::geometry::Isometry2::new(*pos, 0.0))
                    .loosened(self.shaper.ellipse_config.width());
            }
        }
    }

    pub fn complete_stroke(&mut self) {
        self.update_bounds();
    }
}
