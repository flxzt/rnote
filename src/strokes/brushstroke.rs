use std::error::Error;

use crate::{
    pens::brush::{self, Brush},
    strokes::{compose, render, Element},
};
use gtk4::gsk;
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use svg::node::element::path;

use super::{curves, StrokeBehaviour};

// Struct field names are also used in brushstroke template, reminder to be careful when renaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeraElement {
    // Pressure from 0.0 to 1.0
    pressure: f64,
    // Position in format `x y` as integer values
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushStroke {
    pub elements: Vec<Element>,
    pub brush: Brush,
    pub bounds: p2d::bounding_volume::AABB,
    #[serde(skip)]
    pub hitbox: Vec<p2d::bounding_volume::AABB>,
    #[serde(skip, default = "render::default_rendernode")]
    pub rendernode: gsk::RenderNode,
}

impl Default for BrushStroke {
    fn default() -> Self {
        Self::new(Element::default(), Brush::default())
    }
}

impl StrokeBehaviour for BrushStroke {
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

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        match self.brush.current_style {
            brush::BrushStyle::Linear => self.linear_svg_data(offset),
            brush::BrushStyle::CubicBezier => self.catmull_rom_svg_data(offset),
            brush::BrushStyle::CustomTemplate(_) => self.templates_svg_data(offset),
            brush::BrushStyle::Experimental => self.experimental_svg_data(offset),
        }
    }

    fn update_rendernode(&mut self, scalefactor: f64, renderer: &render::Renderer) {
        if let Ok(rendernode) = self.gen_rendernode(scalefactor, renderer) {
            self.rendernode = rendernode;
        } else {
            log::error!("failed to gen_rendernode() in update_rendernode() of brushstroke");
        }
    }

    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
        let svg = compose::wrap_svg(
            self.gen_svg_data(na::vector![0.0, 0.0]).unwrap().as_str(),
            Some(self.bounds),
            Some(self.bounds),
            true,
            false,
        );

        renderer.gen_rendernode(self.bounds, scalefactor, svg.as_str())
    }
}

impl BrushStroke {
    pub const HITBOX_DEFAULT: f64 = 10.0;

    pub fn new(element: Element, brush: Brush) -> Self {
        let elements = Vec::with_capacity(20);
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![element.inputdata.pos()[0], element.inputdata.pos()[1]],
            na::point![element.inputdata.pos()[0], element.inputdata.pos()[1]],
        );
        let hitbox = Vec::new();

        let mut brushstroke = Self {
            elements,
            brush,
            bounds,
            hitbox,
            rendernode: render::default_rendernode(),
        };

        // Pushing with push_elem() instead filling vector, because bounds are getting updated there too
        brushstroke.push_elem(element);

        brushstroke
    }

    pub fn validation_stroke(elements: &[Element], brush: &Brush) -> Option<Self> {
        let mut data_entries_iter = elements.iter();
        let mut stroke = if let Some(first_entry) = data_entries_iter.next() {
            Self::new(first_entry.clone(), brush.clone())
        } else {
            return None;
        };

        for data_entry in data_entries_iter {
            stroke.push_elem(data_entry.clone());
        }
        stroke.complete_stroke();

        Some(stroke)
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
                    last.inputdata.pos() - na::vector![self.brush.width(), self.brush.width()],
                ),
                na::Point2::from(
                    last.inputdata.pos() + na::vector![self.brush.width(), self.brush.width()],
                ),
            ));
        }
    }

    fn update_bounds(&mut self) {
        let mut elements_iter = self.elements.iter();
        if let Some(first) = elements_iter.next() {
            self.bounds = p2d::bounding_volume::AABB::new_invalid();

            self.bounds.merge(&p2d::bounding_volume::AABB::new(
                na::Point2::from(
                    first.inputdata.pos() - na::vector![self.brush.width(), self.brush.width()],
                ),
                na::Point2::from(
                    first.inputdata.pos() + na::vector![self.brush.width(), self.brush.width()],
                ),
            ));
            for element in elements_iter {
                self.bounds.merge(&p2d::bounding_volume::AABB::new(
                    na::Point2::from(
                        element.inputdata.pos()
                            - na::vector![self.brush.width(), self.brush.width()],
                    ),
                    na::Point2::from(
                        element.inputdata.pos()
                            + na::vector![self.brush.width(), self.brush.width()],
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
        let brush_width = self.brush.width();

        let first = first.inputdata.pos();
        if let Some(second) = second {
            let second = second.inputdata.pos();

            let delta = second - first;
            let marker_x = if delta[0] < 0.0 {
                -brush_width
            } else {
                brush_width
            };
            let marker_y = if delta[1] < 0.0 {
                -brush_width
            } else {
                brush_width
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
                            (Self::HITBOX_DEFAULT + brush_width) / 2.0,
                            (Self::HITBOX_DEFAULT + brush_width / 2.0)
                        ],
                ),
                na::Point2::from(
                    first
                        + na::vector![
                            Self::HITBOX_DEFAULT + brush_width,
                            Self::HITBOX_DEFAULT + brush_width
                        ],
                ),
            )
        }
    }

    pub fn linear_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        let mut commands = Vec::new();

        for (i, (first, second)) in self
            .elements
            .iter()
            .zip(self.elements.iter().skip(1))
            .enumerate()
        {
            let line = curves::gen_line(first, second, offset);

            let start_width = first.inputdata.pressure() * self.brush.width();
            let end_width = second.inputdata.pressure() * self.brush.width();

            if let Some(line) = line {
                if i == 0 {
                    commands.push(path::Command::Move(
                        path::Position::Absolute,
                        path::Parameters::from((line.start[0], line.start[1])),
                    ));
                }
                commands.append(&mut compose::svg_linear_variable_width(
                    line,
                    start_width,
                    end_width,
                ));
            }
        }

        let path = svg::node::element::Path::new()
            .set("stroke", "none")
            .set("fill", self.brush.color.to_css_color())
            .set("d", path::Data::from(commands));
        let svg = rough_rs::node_to_string(&path)?.to_string();

        Ok(svg)
    }

    pub fn catmull_rom_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        let mut commands = Vec::new();

        for (((first, second), third), forth) in self
            .elements
            .iter()
            .zip(self.elements.iter().skip(1))
            .zip(self.elements.iter().skip(2))
            .zip(self.elements.iter().skip(3))
        {
            let width_start = second.inputdata.pressure() * self.brush.width();
            let width_end = third.inputdata.pressure() * self.brush.width();

            let mut cubic_bezier =
                curves::gen_cubic_bezier_w_catmull_rom(first, second, third, forth);
            cubic_bezier.start += offset;
            cubic_bezier.cp1 += offset;
            cubic_bezier.cp2 += offset;
            cubic_bezier.end += offset;

            commands.append(&mut compose::svg_cubic_bezier_variable_width(
                cubic_bezier,
                width_start,
                width_end,
                true,
            ));

            // Debugging
            /*             let end_offset_dist = width_end / 2.0;
            let end_unit_norm = compose::vector2_unit_norm(cubic_bezier.end - cubic_bezier.cp2);
            let end_offset = end_unit_norm * end_offset_dist;

            commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((
                    (cubic_bezier.end + 1.2 * end_offset)[0],
                    (cubic_bezier.end + 1.2 * end_offset)[1],
                )),
            ));
            commands.push(path::Command::Line(
                path::Position::Absolute,
                path::Parameters::from((
                    (cubic_bezier.end + 2.2 * end_offset)[0],
                    (cubic_bezier.end + 2.2 * end_offset)[1],
                )),
            ));
            commands.push(path::Command::Close); */
        }

        let svg = if !commands.is_empty() {
            let path = svg::node::element::Path::new()
                .set("stroke", "none")
/*                 .set(
                    "stroke",
                    crate::utils::Color::new(0.0, 0.5, 0.5, 1.0).to_css_color(),
                ) */
                //.set("stroke-width", 1.0)
                //.set("fill", "none")
                .set("fill", self.brush.color.to_css_color())
                .set("d", path::Data::from(commands));
            rough_rs::node_to_string(&path)?.to_string()
        } else {
            String::from("")
        };

        //println!("{}", svg);
        Ok(svg)
    }

    pub fn experimental_svg_data(
        &self,
        _offset: na::Vector2<f64>,
    ) -> Result<String, Box<dyn Error>> {
        Ok(String::from(""))
    }

    pub fn templates_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        let mut cx = tera::Context::new();

        let color = self.brush.color.to_css_color();
        let width = self.brush.width();
        let sensitivity = self.brush.sensitivity();

        let teraelements = self
            .elements
            .iter()
            .zip(self.elements.iter().skip(1))
            .zip(self.elements.iter().skip(2))
            .zip(self.elements.iter().skip(3))
            .map(|(((first, second), third), fourth)| {
                (
                    TeraElement {
                        pressure: first.inputdata.pressure(),
                        x: first.inputdata.pos()[0] + offset[0],
                        y: first.inputdata.pos()[1] + offset[1],
                    },
                    TeraElement {
                        pressure: second.inputdata.pressure(),
                        x: second.inputdata.pos()[0] + offset[0],
                        y: second.inputdata.pos()[1] + offset[1],
                    },
                    TeraElement {
                        pressure: third.inputdata.pressure(),
                        x: third.inputdata.pos()[0] + offset[0],
                        y: third.inputdata.pos()[1] + offset[1],
                    },
                    TeraElement {
                        pressure: fourth.inputdata.pressure(),
                        x: fourth.inputdata.pos()[0] + offset[0],
                        y: fourth.inputdata.pos()[1] + offset[1],
                    },
                )
            })
            .collect::<Vec<(TeraElement, TeraElement, TeraElement, TeraElement)>>();

        cx.insert("color", &color);
        cx.insert("width", &width);
        cx.insert("sensitivity", &sensitivity);
        cx.insert("attributes", "");
        cx.insert("elements", &teraelements);

        let svg = if let brush::BrushStyle::CustomTemplate(templ) = &self.brush.current_style {
            tera::Tera::one_off(templ.as_str(), &cx, false)?
        } else {
            log::error!("template_svg_data() called, but brush is not BrushStyle::CustomTemplate");
            String::from("")
        };

        Ok(svg)
    }
}
