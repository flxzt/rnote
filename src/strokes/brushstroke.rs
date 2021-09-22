use std::error::Error;

use crate::{
    pens::brush::{self, Brush},
    strokes::InputData,
    strokes::{compose, render, Element},
};
use gtk4::gsk;
use p2d::bounding_volume::BoundingVolume;
use rough_rs::generator::RoughGenerator;
use serde::{Deserialize, Serialize};

use super::StrokeBehaviour;

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
        Self::new(InputData::default(), Brush::default())
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
        //self.update_bounds();
        self.bounds = new_bounds.clone();
        self.hitbox = self.gen_hitbox();
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        match self.brush.current_template {
            brush::TemplateType::Linear
            | brush::TemplateType::CubicBezier
            | brush::TemplateType::Custom(_) => self.templates_svg_data(offset),
            brush::TemplateType::Experimental => self.experimental_svg_data(offset),
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

        //println!("{}", svg);

        renderer.gen_rendernode(self.bounds, scalefactor, svg.as_str())
    }
}

impl BrushStroke {
    pub const HITBOX_DEFAULT: f64 = 10.0;

    pub fn new(inputdata: InputData, brush: Brush) -> Self {
        let elements = Vec::with_capacity(20);
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![inputdata.pos()[0], inputdata.pos()[1]],
            na::point![inputdata.pos()[0], inputdata.pos()[1]],
        );
        let hitbox = Vec::new();

        let mut brushstroke = Self {
            elements,
            brush,
            bounds,
            hitbox,
            rendernode: render::default_rendernode(),
        };

        log::info!("current template: {:?}", brushstroke.brush.current_template);
        brushstroke.push_elem(inputdata);

        brushstroke
    }

    pub fn validation_stroke(data_entries: &Vec<InputData>, brush: &Brush) -> Option<Self> {
        let mut data_entries_iter = data_entries.iter();
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

    pub fn templates_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        let mut cx = tera::Context::new();

        let color = self.brush.color().to_css_color();
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
                        x: f64::from(first.inputdata.pos()[0] + offset[0]),
                        y: f64::from(first.inputdata.pos()[1] + offset[1]),
                    },
                    TeraElement {
                        pressure: second.inputdata.pressure(),
                        x: f64::from(second.inputdata.pos()[0] + offset[0]),
                        y: f64::from(second.inputdata.pos()[1] + offset[1]),
                    },
                    TeraElement {
                        pressure: third.inputdata.pressure(),
                        x: f64::from(third.inputdata.pos()[0] + offset[0]),
                        y: f64::from(third.inputdata.pos()[1] + offset[1]),
                    },
                    TeraElement {
                        pressure: fourth.inputdata.pressure(),
                        x: f64::from(fourth.inputdata.pos()[0] + offset[0]),
                        y: f64::from(fourth.inputdata.pos()[1] + offset[1]),
                    },
                )
            })
            .collect::<Vec<(TeraElement, TeraElement, TeraElement, TeraElement)>>();

        cx.insert("color", &color);
        cx.insert("width", &width);
        cx.insert("sensitivity", &sensitivity);
        cx.insert("attributes", "");
        cx.insert("elements", &teraelements);

        let svg = self
            .brush
            .templates
            .borrow()
            .render(self.brush.current_template.template_name().as_str(), &cx)?;

        Ok(svg)
    }

    pub fn experimental_svg_data(
        &self,
        offset: na::Vector2<f64>,
    ) -> Result<String, Box<dyn Error>> {
        let mut rough_generator = RoughGenerator::new(None);
        rough_generator.config.roughness = 0.4;
        rough_generator.config.seed = Some(1);
        rough_generator.config.preserve_vertices = true;
        rough_generator.config.stroke = Some(rough_rs::utils::Color::new(
            self.brush.color().r,
            self.brush.color().g,
            self.brush.color().b,
            self.brush.color().a,
        ));

        let mut svg = String::new();

        for (((element_start, element_one), element_two), element_end) in self
            .elements
            .iter()
            .zip(self.elements.iter().skip(1))
            .zip(self.elements.iter().skip(2))
            .zip(self.elements.iter().skip(3))
            // Overlapping the elements make the strokes actually look nicer
            .step_by(1)
        {
            rough_generator.config.stroke_width =
                self.brush.width() * element_one.inputdata.pressure();

            let svg_path = rough_generator.cubic_bezier(
                element_start.inputdata.pos() + offset,
                (element_one.inputdata.pos().scale(2.0) - element_start.inputdata.pos()) + offset,
                element_two.inputdata.pos() + offset,
                element_end.inputdata.pos() + offset,
                None,
            );
            svg += rough_rs::node_to_string(&svg_path)?.as_str();
        }

        //println!("{}", svg);
        Ok(svg)
    }
}
