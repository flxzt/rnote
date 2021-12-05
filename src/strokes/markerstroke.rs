use crate::{
    compose, curves, geometry,
    pens::marker::Marker,
    render,
    strokes::{self, Element},
};
use gtk4::gsk;
use p2d::bounding_volume::BoundingVolume;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use svg::node::element::path;

use crate::strokes::strokestyle::StrokeBehaviour;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarkerStroke {
    pub elements: Vec<Element>,
    pub marker: Marker,
    pub bounds: p2d::bounding_volume::AABB,
    #[serde(skip)]
    pub hitbox: Vec<p2d::bounding_volume::AABB>,
}

impl Default for MarkerStroke {
    fn default() -> Self {
        Self::new(Element::default(), Marker::default())
    }
}

impl StrokeBehaviour for MarkerStroke {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        self.bounds
    }

    fn translate(&mut self, offset: na::Vector2<f64>) {
        let new_elements: Vec<Element> = self
            .elements
            .iter()
            .map(|element| {
                let mut new_element = element.clone();
                new_element
                    .inputdata
                    .set_pos(element.inputdata.pos() + offset);
                new_element
            })
            .collect();

        self.elements = new_elements;
        self.update_bounds();
        self.hitbox = self.gen_hitbox();
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        let offset = na::vector![
            new_bounds.mins[0] - self.bounds.mins[0],
            new_bounds.mins[1] - self.bounds.mins[1]
        ];

        let scalevector = na::vector![
            (new_bounds.maxs[0] - new_bounds.mins[0]) / (self.bounds.maxs[0] - self.bounds.mins[0]),
            (new_bounds.maxs[1] - new_bounds.mins[1]) / (self.bounds.maxs[1] - self.bounds.mins[1])
        ];

        let new_elements: Vec<Element> = self
            .elements
            .iter()
            .map(|element| {
                let mut new_element = element.clone();
                let top_left = na::vector![self.bounds.mins[0], self.bounds.mins[1]];

                new_element.inputdata.set_pos(
                    ((element.inputdata.pos() - top_left).component_mul(&scalevector))
                        + top_left
                        + offset,
                );

                new_element
            })
            .collect();

        self.elements = new_elements;
        self.bounds = new_bounds;
        self.hitbox = self.gen_hitbox();
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, anyhow::Error> {
        if self.elements.len() <= 1 {
            return Ok(String::from(""));
        }

        let commands: Vec<path::Command> = self
            .elements
            .par_iter()
            .zip(self.elements.par_iter().skip(1))
            .zip(self.elements.par_iter().skip(2))
            .zip(self.elements.par_iter().skip(3))
            .enumerate()
            .map(|(i, (((first, second), third), forth))| {
                let mut commands = Vec::new();
                if let Some(mut cubic_bezier) =
                    curves::gen_cubbez_w_catmull_rom(first, second, third, forth)
                {
                    cubic_bezier.start += offset;
                    cubic_bezier.cp1 += offset;
                    cubic_bezier.cp2 += offset;
                    cubic_bezier.end += offset;

                    if i == 0 {
                        commands.push(path::Command::Move(
                            path::Position::Absolute,
                            path::Parameters::from((cubic_bezier.start[0], cubic_bezier.start[1])),
                        ));
                    } else {
                        commands.push(path::Command::CubicCurve(
                            path::Position::Absolute,
                            path::Parameters::from((
                                (cubic_bezier.cp1[0], cubic_bezier.cp1[1]),
                                (cubic_bezier.cp2[0], cubic_bezier.cp2[1]),
                                (cubic_bezier.end[0], cubic_bezier.end[1]),
                            )),
                        ));
                    }
                } else {
                    if i == 0 {
                        commands.push(path::Command::Move(
                            path::Position::Absolute,
                            path::Parameters::from((
                                second.inputdata.pos()[0],
                                second.inputdata.pos()[1],
                            )),
                        ));
                    } else {
                        commands.push(path::Command::Line(
                            path::Position::Absolute,
                            path::Parameters::from((
                                third.inputdata.pos()[0],
                                third.inputdata.pos()[1],
                            )),
                        ));
                    }
                }

                commands
            })
            .flatten()
            .collect();

        let svg = if !commands.is_empty() {
            let path = svg::node::element::Path::new()
                .set("stroke", self.marker.color.to_css_color())
                .set("stroke-width", self.marker.width())
                .set("stroke-linejoin", "round")
                .set("stroke-linecap", "round")
                .set("fill", "none")
                .set("d", path::Data::from(commands));
            rough_rs::node_to_string(&path)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "rough_rs::node_to_string failed in gen_svg_data() for a markerstroke, {}",
                        e
                    )
                })?
                .to_string()
        } else {
            String::from("")
        };

        Ok(svg)
    }

    fn gen_rendernode(
        &self,
        zoom: f64,
        renderer: &render::Renderer,
    ) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        let svg = compose::wrap_svg(
            self.gen_svg_data(na::vector![0.0, 0.0])?.as_str(),
            Some(self.bounds),
            Some(self.bounds),
            true,
            false,
        );
        Ok(Some(renderer.gen_rendernode(
            self.bounds,
            zoom,
            svg.as_str(),
        )?))
    }
}

impl MarkerStroke {
    pub const HITBOX_DEFAULT: f64 = 10.0;

    pub fn new(element: Element, marker: Marker) -> Self {
        let elements = Vec::with_capacity(20);
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![element.inputdata.pos()[0], element.inputdata.pos()[1]],
            na::point![
                element.inputdata.pos()[0] + 1.0,
                element.inputdata.pos()[1] + 1.0
            ],
        );
        let hitbox: Vec<p2d::bounding_volume::AABB> = Vec::new();

        let mut markerstroke = Self {
            elements,
            marker,
            bounds,
            hitbox,
        };

        // Pushing with push_elem() instead filling vector, because bounds are getting updated there too
        markerstroke.push_elem(element);

        markerstroke
    }

    pub fn push_elem(&mut self, element: Element) {
        self.elements.push(element);

        self.update_bounds_to_last_elem();
    }

    pub fn pop_elem(&mut self) -> Option<Element> {
        let element = self.elements.pop();

        self.complete_stroke();
        element
    }

    pub fn complete_stroke(&mut self) {
        self.update_bounds();
        self.hitbox = self.gen_hitbox();
    }

    fn update_bounds_to_last_elem(&mut self) {
        // Making sure bounds are always outside of coord + width
        if let Some(last) = self.elements.last() {
            self.bounds.merge(&p2d::bounding_volume::AABB::new(
                na::Point2::from(
                    last.inputdata.pos() - na::vector![self.marker.width(), self.marker.width()],
                ),
                na::Point2::from(
                    last.inputdata.pos() + na::vector![self.marker.width(), self.marker.width()],
                ),
            ));
        }
    }

    pub fn update_bounds(&mut self) {
        let mut elements_iter = self.elements.iter();
        if let Some(first) = elements_iter.next() {
            self.bounds = p2d::bounding_volume::AABB::new_invalid();

            self.bounds.merge(&p2d::bounding_volume::AABB::new(
                na::Point2::from(
                    first.inputdata.pos() - na::vector![self.marker.width(), self.marker.width()],
                ),
                na::Point2::from(
                    first.inputdata.pos() + na::vector![self.marker.width(), self.marker.width()],
                ),
            ));
            for element in elements_iter {
                self.bounds.merge(&p2d::bounding_volume::AABB::new(
                    na::Point2::from(
                        element.inputdata.pos()
                            - na::vector![self.marker.width(), self.marker.width()],
                    ),
                    na::Point2::from(
                        element.inputdata.pos()
                            + na::vector![self.marker.width(), self.marker.width()],
                    ),
                ));
            }
        }
    }

    fn gen_hitbox(&self) -> Vec<p2d::bounding_volume::AABB> {
        let mut hitbox: Vec<p2d::bounding_volume::AABB> =
            Vec::with_capacity(self.elements.len() as usize);
        let mut elements_iter = self.elements.iter().peekable();
        while let Some(first) = elements_iter.next() {
            let second = if let Some(&second) = elements_iter.peek() {
                Some(second)
            } else {
                None
            };
            hitbox.push(self.gen_last_hitbox(first, second));
        }

        hitbox
    }

    fn gen_last_hitbox(
        &self,
        first: &Element,
        second: Option<&Element>,
    ) -> p2d::bounding_volume::AABB {
        let marker_width = self.marker.width();

        let first = first.inputdata.pos();
        if let Some(second) = second {
            let second = second.inputdata.pos();

            let delta = second - first;
            let marker_x = if delta[0] < 0.0 {
                -marker_width
            } else {
                marker_width
            };
            let marker_y = if delta[1] < 0.0 {
                -marker_width
            } else {
                marker_width
            };

            geometry::aabb_new_positive(
                first - na::vector![marker_x / 2.0, marker_y / 2.0],
                first + delta + na::vector![marker_x / 2.0, marker_y / 2.0],
            )
        } else {
            geometry::aabb_new_positive(
                first
                    - na::vector![
                        (Self::HITBOX_DEFAULT + marker_width) / 2.0,
                        (Self::HITBOX_DEFAULT + marker_width / 2.0)
                    ],
                first
                    + na::vector![
                        Self::HITBOX_DEFAULT + marker_width,
                        Self::HITBOX_DEFAULT + marker_width
                    ],
            )
        }
    }

    pub fn import_from_svg(_svg: &str) -> Vec<strokes::StrokeStyle> {
        let strokes: Vec<strokes::StrokeStyle> = Vec::new();

        strokes
    }

    pub fn export_to_svg(&self, xml_header: bool) -> Result<String, anyhow::Error> {
        let svg = compose::wrap_svg(
            Self::gen_svg_data(self, na::vector![0.0, 0.0])?.as_str(),
            Some(self.bounds),
            Some(self.bounds),
            xml_header,
            false,
        );

        Ok(svg)
    }
}
