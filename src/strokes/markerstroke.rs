use std::error::Error;

use crate::{
    pens::marker::Marker,
    strokes,
    strokes::InputData,
    strokes::{compose, render, Element},
};
use gtk4::gsk;
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use svg::node::element;

use super::StrokeBehaviour;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerStroke {
    pub elements: Vec<Element>,
    pub marker: Marker,
    pub bounds: p2d::bounding_volume::AABB,
    #[serde(skip)]
    pub hitbox: Vec<p2d::bounding_volume::AABB>,
    #[serde(skip, default = "render::default_rendernode")]
    pub rendernode: gsk::RenderNode,
}

impl Default for MarkerStroke {
    fn default() -> Self {
        Self::new(InputData::default(), Marker::default())
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
        self.bounds = new_bounds.clone();
        self.hitbox = self.gen_hitbox();
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        let mut svg = String::new();
        let mut data = element::path::Data::new();

        for (i, element) in self.elements.iter().enumerate() {
            if i == 0 {
                data = data.move_to((
                    element.inputdata.pos()[0] + offset[0],
                    element.inputdata.pos()[1] + offset[1],
                ));
            } else {
                data = data.line_to((
                    element.inputdata.pos()[0] + offset[0],
                    element.inputdata.pos()[1] + offset[1],
                ));
            }
        }

        let svg_path = element::Path::new()
            .set("d", data)
            .set("stroke", self.marker.color().to_css_color())
            .set("stroke-width", self.marker.width())
            .set("fill", "None");

        svg += rough_rs::node_to_string(&svg_path)?.as_str();

        Ok(svg)
    }

    fn update_rendernode(&mut self, scalefactor: f64, renderer: &render::Renderer) {
        if let Ok(rendernode) = self.gen_rendernode(scalefactor, renderer) {
            self.rendernode = rendernode;
        } else {
            log::error!("failed to gen_rendernode() in update_rendernode() of markerstroke");
        }
    }

    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
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

impl MarkerStroke {
    pub const HITBOX_DEFAULT: f64 = 10.0;

    pub fn new(inputdata: InputData, marker: Marker) -> Self {
        let elements = Vec::with_capacity(20);
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![inputdata.pos()[0], inputdata.pos()[1]],
            na::point![inputdata.pos()[0] + 1.0, inputdata.pos()[1] + 1.0],
        );
        let hitbox: Vec<p2d::bounding_volume::AABB> = Vec::new();

        let mut markerstroke = Self {
            elements,
            marker,
            bounds,
            hitbox,
            rendernode: render::default_rendernode(),
        };

        markerstroke.push_elem(inputdata);

        markerstroke
    }

    pub fn push_elem(&mut self, inputdata: InputData) {
        self.elements.push(Element::new(inputdata));

        self.update_bounds_to_last_elem();
    }

    pub fn complete_stroke(&mut self) {
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

            p2d::bounding_volume::AABB::new(
                na::Point2::from(first - na::vector![marker_x / 2.0, marker_y / 2.0]),
                na::Point2::from(first + delta + na::vector![marker_x / 2.0, marker_y / 2.0]),
            )
        } else {
            p2d::bounding_volume::AABB::new(
                na::Point2::from(
                    first
                        - na::vector![
                            (Self::HITBOX_DEFAULT + marker_width) / 2.0,
                            (Self::HITBOX_DEFAULT + marker_width / 2.0)
                        ],
                ),
                na::Point2::from(
                    first
                        + na::vector![
                            Self::HITBOX_DEFAULT + marker_width,
                            Self::HITBOX_DEFAULT + marker_width
                        ],
                ),
            )
        }
    }

    pub fn import_from_svg(_svg: &str) -> Vec<strokes::StrokeStyle> {
        let strokes: Vec<strokes::StrokeStyle> = Vec::new();

        strokes
    }

    pub fn export_to_svg(&self, xml_header: bool) -> Result<String, Box<dyn Error>> {
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
