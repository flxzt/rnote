use crate::{
    compose, curves, drawbehaviour::DrawBehaviour, geometry, pens::marker::Marker, render,
    strokes::strokestyle::Element,
};
use p2d::bounding_volume::BoundingVolume;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use svg::node::element::path;

use crate::strokes::strokebehaviour::StrokeBehaviour;

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
        Self {
            elements: vec![],
            marker: Marker::default(),
            bounds: geometry::aabb_new_zero(),
            hitbox: vec![],
        }
    }
}

impl DrawBehaviour for MarkerStroke {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: p2d::bounding_volume::AABB) {
        self.bounds = bounds;
    }

    fn gen_bounds(&self) -> Option<p2d::bounding_volume::AABB> {
        if let Some(&first) = self.elements.iter().peekable().peek() {
            let mut bounds = p2d::bounding_volume::AABB::new_invalid();
            bounds.take_point(na::Point2::<f64>::from(first.inputdata.pos()));

            bounds.merge(
                &self
                    .elements
                    .par_iter()
                    .zip(self.elements.par_iter().skip(1))
                    .zip(self.elements.par_iter().skip(2))
                    .zip(self.elements.par_iter().skip(3))
                    .filter_map(|(((first, second), third), forth)| {
                        let marker_width = self.marker.width();

                        let mut bounds = p2d::bounding_volume::AABB::new_invalid();

                        if let Some(cubbez) =
                            curves::gen_cubbez_w_catmull_rom(first, second, third, forth)
                        {
                            // Bounds are definitely inside the polygon of the control points. (Could be improved with the second derivative of the bezier curve)
                            bounds.take_point(na::Point2::<f64>::from(cubbez.start));
                            bounds.take_point(na::Point2::<f64>::from(cubbez.cp1));
                            bounds.take_point(na::Point2::<f64>::from(cubbez.cp2));
                            bounds.take_point(na::Point2::<f64>::from(cubbez.end));

                            bounds.loosen(marker_width);
                            // Ceil to nearest integers to avoid subpixel placement errors between stroke elements.
                            bounds = geometry::aabb_ceil(bounds);
                            Some(bounds)
                        } else if let Some(line) = curves::gen_line(second, third) {
                            bounds.take_point(na::Point2::<f64>::from(line.start));
                            bounds.take_point(na::Point2::<f64>::from(line.end));

                            bounds.loosen(marker_width);
                            // Ceil to nearest integers to avoid subpixel placement errors between stroke elements.
                            bounds = geometry::aabb_ceil(bounds);

                            Some(bounds)
                        } else {
                            None
                        }
                    })
                    .reduce(p2d::bounding_volume::AABB::new_invalid, |i, next| {
                        i.merged(&next)
                    }),
            );
            Some(bounds)
        } else {
            None
        }
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
                self.gen_svg_elem((first, second, third, forth), offset, svg_root)
            })
            .collect();

        Ok(svgs)
    }
}

impl StrokeBehaviour for MarkerStroke {
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
}

impl MarkerStroke {
    pub const HITBOX_DEFAULT: f64 = 10.0;

    pub fn new(element: Element, marker: Marker) -> Self {
        let elements = Vec::with_capacity(20);
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![element.inputdata.pos()[0], element.inputdata.pos()[1]],
            na::point![element.inputdata.pos()[0], element.inputdata.pos()[1]],
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

        self.update_geometry();
        element
    }

    pub fn update_geometry(&mut self) {
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

    pub fn gen_svg_elem(
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

            // Bounds are definitely inside the polygon of the control points. (Could be improved with the second derivative of the bezier curve)
            bounds.take_point(na::Point2::<f64>::from(cubbez.start));
            bounds.take_point(na::Point2::<f64>::from(cubbez.cp1));
            bounds.take_point(na::Point2::<f64>::from(cubbez.cp2));
            bounds.take_point(na::Point2::<f64>::from(cubbez.end));
            bounds.loosen(marker_width);

            // Ceil to nearest integers to avoid subpixel placement errors between stroke elements.
            bounds = geometry::aabb_ceil(bounds);

            commands.append(&mut compose::compose_cubbez(cubbez, true));
        } else if let Some(mut line) = curves::gen_line(elements.1, elements.2) {
            line.start += offset;
            line.end += offset;

            bounds.take_point(na::Point2::<f64>::from(line.start));
            bounds.take_point(na::Point2::<f64>::from(line.end));
            bounds.loosen(marker_width);

            commands.append(&mut compose::compose_line(line, true));
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
}
