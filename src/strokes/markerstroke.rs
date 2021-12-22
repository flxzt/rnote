use crate::{
    compose, curves, geometry,
    pens::marker::Marker,
    render,
    strokes::{self, Element},
};
use p2d::bounding_volume::BoundingVolume;
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

    fn set_bounds(&mut self, bounds: p2d::bounding_volume::AABB) {
        self.bounds = bounds;
    }

    fn gen_bounds(&self) -> Option<p2d::bounding_volume::AABB> {
        let mut elements_iter = self.elements.iter().peekable();

        if let Some(&first) = elements_iter.peek() {
            let mut bounds = p2d::bounding_volume::AABB::new_invalid();
            bounds.take_point(na::Point2::<f64>::from(first.inputdata.pos()));

            elements_iter
                .zip(self.elements.iter().skip(1))
                .zip(self.elements.iter().skip(2))
                .zip(self.elements.iter().skip(3))
                .for_each(|(((first, second), third), forth)| {
                    let width = self.marker.width();

                    if let Some(cubbez) =
                        curves::gen_cubbez_w_catmull_rom(first, second, third, forth)
                    {
                        // Bounds are definitely inside the polygon of the control points. (Could be improved with the second derivative of the bezier curve)
                        bounds.take_point(na::Point2::<f64>::from(cubbez.start));
                        bounds.take_point(na::Point2::<f64>::from(cubbez.cp1));
                        bounds.take_point(na::Point2::<f64>::from(cubbez.cp2));
                        bounds.take_point(na::Point2::<f64>::from(cubbez.end));

                        bounds.loosen(width);
                        // Ceil to nearest integers to avoid subpixel placement errors between stroke elements.
                        bounds = geometry::aabb_ceil(bounds);
                    } else if let Some(line) = curves::gen_line(second, third) {
                        bounds.take_point(na::Point2::<f64>::from(line.start));
                        bounds.take_point(na::Point2::<f64>::from(line.end));
                    } else {
                        return;
                    }
                });
            Some(bounds)
        } else {
            None
        }
    }

    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.elements.iter_mut().for_each(|element| {
            element.inputdata.set_pos(element.inputdata.pos() + offset);
        });

        self.bounds = geometry::aabb_translate(self.bounds, offset);
        self.hitbox = self.gen_hitbox();
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        let offset = na::vector![
            new_bounds.mins[0] - self.bounds.mins[0],
            new_bounds.mins[1] - self.bounds.mins[1]
        ];

        let scalevector = na::vector![
            (new_bounds.extents()[0]) / (self.bounds().extents()[0]),
            (new_bounds.extents()[1]) / (self.bounds().extents()[1])
        ];

        self.elements.iter_mut().for_each(|element| {
            let top_left = na::vector![self.bounds.mins[0], self.bounds.mins[1]];

            element.inputdata.set_pos(
                ((element.inputdata.pos() - top_left).component_mul(&scalevector))
                    + top_left
                    + offset,
            );
        });

        self.bounds = new_bounds;
        self.hitbox = self.gen_hitbox();
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        let svg_root = false;

        let svgs: Vec<render::Svg> = self
            .elements
            .iter()
            .zip(self.elements.iter().skip(1))
            .zip(self.elements.iter().skip(2))
            .zip(self.elements.iter().skip(3))
            .filter_map(|(((first, second), third), forth)| {
                self.gen_svg_for_elems((first, second, third, forth), offset, svg_root)
            })
            .collect();

        Ok(svgs)
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
        if let Some(new_bounds) = self.gen_bounds() {
            self.set_bounds(new_bounds);
        }
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

            self.bounds = geometry::aabb_ceil(self.bounds);
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

            hitbox.push(self.gen_hitbox_for_elems(first, second));
        }

        hitbox
    }

    fn gen_hitbox_for_elems(
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

    pub fn gen_svg_for_elems(
        &self,
        elements: (&Element, &Element, &Element, &Element),
        offset: na::Vector2<f64>,
        svg_root: bool,
    ) -> Option<render::Svg> {
        let mut commands = Vec::new();
        let marker_width = self.marker.width();

        let mut bounds = p2d::bounding_volume::AABB::new_invalid();

        if let Some(mut cubbez) =
            curves::gen_cubbez_w_catmull_rom(elements.0, elements.1, elements.2, elements.3)
        {
            cubbez.start += offset;
            cubbez.cp1 += offset;
            cubbez.cp2 += offset;
            cubbez.end += offset;

            bounds.take_point(na::Point2::<f64>::from(cubbez.start));
            bounds.take_point(na::Point2::<f64>::from(cubbez.cp1));
            bounds.take_point(na::Point2::<f64>::from(cubbez.cp2));
            bounds.take_point(na::Point2::<f64>::from(cubbez.end));
            // Bounds are definitely inside the polygon of the control points. (Could be improved with the second derivative of the bezier curve)

            bounds.loosen(marker_width);
            // Ceil to nearest integers to avoid subpixel placement errors between stroke elements.
            bounds = geometry::aabb_ceil(bounds);

            commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((cubbez.start[0], cubbez.start[1])),
            ));
            commands.push(path::Command::CubicCurve(
                path::Position::Absolute,
                path::Parameters::from((
                    (cubbez.cp1[0], cubbez.cp1[1]),
                    (cubbez.cp2[0], cubbez.cp2[1]),
                    (cubbez.end[0], cubbez.end[1]),
                )),
            ));
        } else if let Some(mut line) = curves::gen_line(elements.1, elements.2) {
            line.start += offset;
            line.end += offset;

            bounds.take_point(na::Point2::<f64>::from(line.start));
            bounds.take_point(na::Point2::<f64>::from(line.end));

            commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((line.start[0], line.start[1])),
            ));
            commands.push(path::Command::Line(
                path::Position::Absolute,
                path::Parameters::from((line.end[0], line.end[1])),
            ));
        } else {
            return None;
        }

        let path = svg::node::element::Path::new()
            .set("stroke", self.marker.color.to_css_color())
            .set("stroke-width", marker_width)
            .set("stroke-linejoin", "round")
            .set("stroke-linecap", "round")
            .set("fill", "none")
            .set("d", path::Data::from(commands));

        match rough_rs::node_to_string(&path) {
            Ok(mut svg_data) => {
                if svg_root {
                    svg_data =
                        compose::wrap_svg(&svg_data, Some(bounds), Some(bounds), true, false);
                }
                Some(render::Svg { svg_data, bounds })
            }
            Err(e) => {
                log::error!(
                    "rough_rs::node_to_string() failed in gen_svg_elem() of brushstroke, {}",
                    e
                );
                None
            }
        }
    }

    pub fn import_from_svg(_svg: &str) -> Vec<strokes::StrokeStyle> {
        let strokes: Vec<strokes::StrokeStyle> = Vec::new();

        strokes
    }

    pub fn export_to_svg(&self, xml_header: bool) -> Result<String, anyhow::Error> {
        let svgs = Self::gen_svgs(self, na::vector![0.0, 0.0])?;
        let svg_data = svgs
            .iter()
            .map(|svg| svg.svg_data.clone())
            .collect::<Vec<String>>()
            .join("\n");

        let svg = compose::wrap_svg(
            svg_data.as_str(),
            Some(self.bounds),
            Some(self.bounds),
            xml_header,
            false,
        );

        Ok(svg)
    }
}
