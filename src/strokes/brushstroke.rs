use crate::compose::geometry;
use crate::compose::smooth::SmoothOptions;
use crate::compose::textured::TexturedOptions;
use crate::compose::transformable::Transformable;
use crate::compose::{self, curves, smooth, textured};
use crate::drawbehaviour::DrawBehaviour;
use crate::pens::brush::BrushStyle;
use crate::strokes::strokestyle::Element;
use crate::utils;
use crate::{pens::brush::Brush, render};

use p2d::bounding_volume::{BoundingVolume, AABB};
use rand::{Rng, SeedableRng};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use svg::node::element::path;

use super::strokestyle::InputData;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "brushstroke_style")]
pub enum BrushStrokeStyle {
    #[serde(rename = "smooth")]
    Solid {
        #[serde(rename = "options")]
        options: SmoothOptions,
    },
    #[serde(rename = "textured")]
    Textured {
        #[serde(rename = "options")]
        options: TexturedOptions,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "brushstroke")]
pub struct BrushStroke {
    #[serde(rename = "elements")]
    pub elements: Vec<Element>,
    #[serde(rename = "style")]
    pub style: BrushStrokeStyle,
    #[serde(rename = "bounds")]
    pub bounds: AABB,
    #[serde(skip)]
    pub hitboxes: Vec<AABB>,
}

impl Default for BrushStroke {
    fn default() -> Self {
        Self::new(Element::new(InputData::default()), &Brush::default())
    }
}

impl DrawBehaviour for BrushStroke {
    fn bounds(&self) -> AABB {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: AABB) {
        self.bounds = bounds;
    }

    fn gen_bounds(&self) -> Option<AABB> {
        if let Some(&first) = self.elements.iter().peekable().peek() {
            let mut bounds = AABB::new_invalid();
            bounds.take_point(na::Point2::from(first.inputdata.pos()));

            bounds.merge(
                &self
                    .elements
                    .par_iter()
                    .zip(self.elements.par_iter().skip(1))
                    .zip(self.elements.par_iter().skip(2))
                    .zip(self.elements.par_iter().skip(3))
                    .filter_map(|(((first, second), third), forth)| {
                        let mut bounds = AABB::new_invalid();

                        let width = match &self.style {
                            BrushStrokeStyle::Solid { options } => options.width(),
                            BrushStrokeStyle::Textured { options } => options.width(),
                        };

                        if let Some(cubbez) = curves::gen_cubbez_w_catmull_rom(
                            first.inputdata.pos(),
                            second.inputdata.pos(),
                            third.inputdata.pos(),
                            forth.inputdata.pos(),
                        ) {
                            // Bounds are definitely inside the polygon of the control points. (Could be improved with the second derivative of the bezier curve)
                            bounds.take_point(na::Point2::from(cubbez.start));
                            bounds.take_point(na::Point2::from(cubbez.cp1));
                            bounds.take_point(na::Point2::from(cubbez.cp2));
                            bounds.take_point(na::Point2::from(cubbez.end));
                            bounds.loosen(width);

                            Some(bounds)
                        } else if let Some(line) =
                            curves::gen_line(second.inputdata.pos(), third.inputdata.pos())
                        {
                            bounds.take_point(na::Point2::from(line.start));
                            bounds.take_point(na::Point2::from(line.end));
                            bounds.loosen(width);

                            Some(bounds)
                        } else {
                            None
                        }
                    })
                    .reduce(AABB::new_invalid, |i, next| i.merged(&next)),
            );
            Some(bounds)
        } else {
            None
        }
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        let svg_root = false;

        match self.style {
            BrushStrokeStyle::Solid { options } => self.gen_svgs_solid(options, offset, svg_root),
            BrushStrokeStyle::Textured { options } => {
                self.gen_svgs_textured(options, offset, svg_root)
            }
        }
    }
}

impl Transformable for BrushStroke {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.elements.iter_mut().for_each(|element| {
            element.inputdata.set_pos(element.inputdata.pos() + offset);
        });
        self.update_geometry();
    }
    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.elements.iter_mut().for_each(|element| {
            element
                .inputdata
                .set_pos((isometry * na::Point2::from(element.inputdata.pos())).coords);
        });
        self.update_geometry();
    }
    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        let center = self.bounds.center().coords;

        self.elements.iter_mut().for_each(|element| {
            element
                .inputdata
                .set_pos(((element.inputdata.pos() - center).component_mul(&scale)) + center);
        });
        self.update_geometry();
    }
}

impl BrushStroke {
    pub const HITBOX_DEFAULT: f64 = 10.0;

    pub fn new(element: Element, brush: &Brush) -> Self {
        let seed = Some(rand_pcg::Pcg64::from_entropy().gen());

        let style = match brush.style {
            BrushStyle::Solid => {
                let mut options = brush.smooth_options;
                options.set_seed(seed);

                BrushStrokeStyle::Solid { options }
            }
            BrushStyle::Textured => {
                let mut options = brush.textured_options;
                options.set_seed(seed);

                BrushStrokeStyle::Textured { options }
            }
        };
        let elements = Vec::with_capacity(4);
        let bounds = AABB::new(
            na::point![element.inputdata.pos()[0], element.inputdata.pos()[1]],
            na::point![element.inputdata.pos()[0], element.inputdata.pos()[1]],
        );
        let hitbox = Vec::new();

        let mut brushstroke = Self {
            elements,
            style,
            bounds,
            hitboxes: hitbox,
        };

        // Pushing with push_elem() instead filling vector, because bounds are getting updated there too
        brushstroke.push_elem(element);

        brushstroke
    }

    pub fn new_w_elements(
        mut elements: impl Iterator<Item = Element>,
        brush: &Brush,
    ) -> Option<Self> {
        if let Some(first) = elements.next() {
            let mut brushstroke = Self::new(first, brush);

            for element in elements {
                brushstroke.elements.push(element);
            }
            brushstroke.update_geometry();

            Some(brushstroke)
        } else {
            None
        }
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
        self.hitboxes = self.gen_hitboxes();
    }

    fn update_bounds_to_last_elem(&mut self) {
        if let Some(last) = self.elements.last() {
            let width = match self.style {
                BrushStrokeStyle::Solid { options } => options.width(),
                BrushStrokeStyle::Textured { options } => options.width(),
            };

            self.bounds.merge(&AABB::new(
                na::Point2::from(last.inputdata.pos() - na::Vector2::from_element(width)),
                na::Point2::from(last.inputdata.pos() + na::Vector2::from_element(width)),
            ));
        }
    }

    fn gen_hitboxes(&self) -> Vec<AABB> {
        let mut hitbox: Vec<AABB> = Vec::with_capacity(self.elements.len() as usize);
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

    fn gen_hitbox_for_elems(&self, first: &Element, second: Option<&Element>) -> AABB {
        let width = match self.style {
            BrushStrokeStyle::Solid { options } => options.width(),
            BrushStrokeStyle::Textured { options } => options.width(),
        };

        let first = first.inputdata.pos();
        if let Some(second) = second {
            let second = second.inputdata.pos();

            let delta = second - first;
            let brush_x = if delta[0] < 0.0 { -width } else { width };
            let brush_y = if delta[1] < 0.0 { -width } else { width };

            geometry::aabb_new_positive(
                na::Point2::from(first - na::vector![brush_x / 2.0, brush_y / 2.0]),
                na::Point2::from(first + delta + na::vector![brush_x / 2.0, brush_y / 2.0]),
            )
        } else {
            geometry::aabb_new_positive(
                na::Point2::from(
                    first
                        - na::vector![
                            (Self::HITBOX_DEFAULT + width) / 2.0,
                            (Self::HITBOX_DEFAULT + width / 2.0)
                        ],
                ),
                na::Point2::from(
                    first + na::vector![Self::HITBOX_DEFAULT + width, Self::HITBOX_DEFAULT + width],
                ),
            )
        }
    }

    pub fn gen_svg_for_elems(
        &self,
        elements: (&Element, &Element, &Element, &Element),
        offset: na::Vector2<f64>,
        svg_root: bool,
    ) -> Result<Option<render::Svg>, anyhow::Error> {
        match self.style {
            BrushStrokeStyle::Solid { mut options } => {
                let mut seed = options.seed();
                // Advance the seed (skip first three elements) so that stroke keeps generating the same patterns
                for _ in 3..self.elements.len() {
                    seed = seed.map(|seed| utils::seed_advance(seed));
                }
                options.set_seed(seed);

                Ok(Self::gen_svg_elem_solid(
                    &options, elements, offset, svg_root,
                ))
            }
            BrushStrokeStyle::Textured { mut options } => {
                let mut seed = options.seed();
                // Advance the seed (skip first three elements) so that stroke keeps generating the same patterns
                for _ in 3..self.elements.len() {
                    seed = seed.map(|seed| utils::seed_advance(seed));
                }
                options.set_seed(seed);

                Ok(Self::gen_svg_elem_textured(
                    &options, elements, offset, svg_root,
                ))
            }
        }
    }

    pub fn gen_svg_elem_solid(
        options: &SmoothOptions,
        elements: (&Element, &Element, &Element, &Element),
        offset: na::Vector2<f64>,
        svg_root: bool,
    ) -> Option<render::Svg> {
        let mut commands = Vec::new();

        let start_width = elements.1.inputdata.pressure() * options.width();
        let end_width = elements.2.inputdata.pressure() * options.width();

        let mut bounds = AABB::new_invalid();

        if let Some(mut cubbez) = curves::gen_cubbez_w_catmull_rom(
            elements.0.inputdata.pos(),
            elements.1.inputdata.pos(),
            elements.2.inputdata.pos(),
            elements.3.inputdata.pos(),
        ) {
            cubbez.start += offset;
            cubbez.cp1 += offset;
            cubbez.cp2 += offset;
            cubbez.end += offset;

            // Bounds are definitely inside the polygon of the control points. (Could be improved with the second derivative of the bezier curve)
            bounds.take_point(na::Point2::from(cubbez.start));
            bounds.take_point(na::Point2::from(cubbez.cp1));
            bounds.take_point(na::Point2::from(cubbez.cp2));
            bounds.take_point(na::Point2::from(cubbez.end));

            let n_splits = 5;
            // Number of splits for the bezier curve approximation
            let lines = curves::approx_cubbez_with_lines(cubbez, n_splits);
            let n_lines = lines.len() as i32;

            for (i, line) in lines.iter().enumerate() {
                // splitted line start / end widths are a linear interpolation between the start and end width / n splits.
                // Not mathematically correct, TODO to carry the t of the splits through approx_offsetted_cubbez_with_lines_w_subdivion()
                let line_start_width = start_width
                    + (end_width - start_width) * (f64::from(i as i32) / f64::from(n_lines));
                let line_end_width = start_width
                    + (end_width - start_width) * (f64::from(i as i32 + 1) / f64::from(n_lines));

                commands.append(&mut smooth::compose_line_variable_width(
                    *line,
                    line_start_width,
                    line_end_width,
                    true,
                    options,
                ));
            }
        } else if let Some(mut line) =
            curves::gen_line(elements.1.inputdata.pos(), elements.2.inputdata.pos())
        {
            line.start += offset;
            line.end += offset;

            bounds.take_point(na::Point2::from(line.start));
            bounds.take_point(na::Point2::from(line.end));

            commands.append(&mut smooth::compose_line_variable_width(
                line,
                start_width,
                end_width,
                true,
                options,
            ));
        } else {
            return None;
        }

        bounds.loosen(start_width.max(end_width));

        let fill = options
            .color()
            .map_or(String::from(""), |color| color.to_css_color());

        let path = svg::node::element::Path::new()
            .set("stroke", "none")
            //.set("stroke", self.brush.color.to_css_color())
            //.set("stroke-width", 1.0)
            .set("fill", fill)
            .set("d", path::Data::from(commands));

        let mut svg_data = compose::svg_node_to_string(&path)
            .map_err(|e| {
                anyhow::anyhow!(
                    "node_to_string() failed in gen_svg_elem_solid() of brushstroke with Err `{}`",
                    e
                )
            })
            .ok()?;

        if svg_root {
            svg_data = compose::wrap_svg_root(&svg_data, Some(bounds), Some(bounds), true);
        }
        Some(render::Svg { svg_data, bounds })
    }

    pub fn gen_svgs_solid(
        &self,
        options: SmoothOptions,
        offset: na::Vector2<f64>,
        svg_root: bool,
    ) -> Result<Vec<render::Svg>, anyhow::Error> {
        let svgs: Vec<render::Svg> = self
            .elements
            .iter()
            .zip(self.elements.iter().skip(1))
            .zip(self.elements.iter().skip(2))
            .zip(self.elements.iter().skip(3))
            .filter_map(|(((first, second), third), forth)| {
                Self::gen_svg_elem_solid(&options, (first, second, third, forth), offset, svg_root)
            })
            .collect();

        Ok(svgs)
    }

    pub fn gen_svg_elem_textured(
        options: &TexturedOptions,
        elements: (&Element, &Element, &Element, &Element),
        offset: na::Vector2<f64>,
        svg_root: bool,
    ) -> Option<render::Svg> {
        let start_width = elements.1.inputdata.pressure() * options.width();
        let end_width = elements.2.inputdata.pressure() * options.width();
        let mid_width = (start_width + end_width) * 0.5;

        let mut bounds = AABB::new_invalid();

        // Configure the textured Configuration
        let element = if let Some(mut line) =
            curves::gen_line(elements.1.inputdata.pos(), elements.2.inputdata.pos())
        {
            line.start += offset;
            line.end += offset;

            bounds.take_point(na::Point2::from(line.start));
            bounds.take_point(na::Point2::from(line.end));

            textured::compose_line(line, mid_width, &options)
        } else {
            return None;
        };

        bounds.loosen(options.width());

        let mut svg_data = compose::svg_node_to_string(&element)
            .map_err(|e| {
                anyhow::anyhow!(
                    "node_to_string() failed in gen_svg_elem_textured() of brushstroke with Err `{}`",
                    e
                )
            })
            .ok()?;

        if svg_root {
            svg_data = compose::wrap_svg_root(&svg_data, Some(bounds), Some(bounds), true);
        }

        Some(render::Svg { svg_data, bounds })
    }

    pub fn gen_svgs_textured(
        &self,
        mut options: TexturedOptions,
        offset: na::Vector2<f64>,
        svg_root: bool,
    ) -> Result<Vec<render::Svg>, anyhow::Error> {
        let mut seed = options.seed();

        let svgs: Vec<render::Svg> = self
            .elements
            .iter()
            .zip(self.elements.iter().skip(1))
            .zip(self.elements.iter().skip(2))
            .zip(self.elements.iter().skip(3))
            .filter_map(|(((first, second), third), forth)| {
                seed = seed.map(|seed| utils::seed_advance(seed));
                options.set_seed(seed);

                Self::gen_svg_elem_textured(
                    &options,
                    (first, second, third, forth),
                    offset,
                    svg_root,
                )
            })
            .collect();

        Ok(svgs)
    }
}
