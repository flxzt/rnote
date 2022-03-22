use super::StrokeBehaviour;
use crate::pens::brush::BrushStyle;
use crate::pens::Brush;
use crate::render::{self};
use crate::DrawBehaviour;
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::{composer, Composer};
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::{PenPath, Style};

use p2d::bounding_volume::{BoundingVolume, AABB};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "brushstroke")]
pub struct BrushStroke {
    #[serde(rename = "path")]
    pub path: PenPath,
    #[serde(rename = "style")]
    pub style: Style,
    #[serde(skip)]
    pub hitboxes: Vec<AABB>,
}

impl Default for BrushStroke {
    fn default() -> Self {
        Self::new(
            Segment::Dot {
                element: Element::default(),
            },
            &Brush::default(),
        )
    }
}

impl StrokeBehaviour for BrushStroke {}

impl DrawBehaviour for BrushStroke {
    fn draw(
        &self,
        cx: &mut impl piet::RenderContext,
        _image_scale: f64,
    ) -> Result<(), anyhow::Error> {
        match &self.style {
            Style::Smooth(options) => self.path.draw_composed(cx, options),
            Style::Rough(_) => {
                // Rough style currently unsupported for pen paths
            }
            Style::Textured(options) => self.path.draw_composed(cx, options),
        };

        Ok(())
    }
}

impl ShapeBehaviour for BrushStroke {
    fn bounds(&self) -> AABB {
        match &self.style {
            Style::Smooth(options) => self.path.composed_bounds(options),
            // TODO: Needs fixing
            Style::Rough(_options) => self.path.bounds(),
            Style::Textured(options) => self.path.composed_bounds(options),
        }
    }
}

impl TransformBehaviour for BrushStroke {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.path.translate(offset);
        self.update_geometry();
    }
    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.path.rotate(angle, center);
        self.update_geometry();
    }
    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.path.scale(scale);
        self.update_geometry();
    }
}

impl BrushStroke {
    pub const HITBOX_DEFAULT: f64 = 10.0;

    pub fn new(segment: Segment, brush: &Brush) -> Self {
        let path = PenPath::new_w_segment(segment);

        Self::from_penpath(path, brush)
    }

    pub fn from_penpath(path: PenPath, brush: &Brush) -> Self {
        let seed = rand_pcg::Pcg64::from_entropy().gen();

        let style = match brush.style {
            BrushStyle::Marker => {
                let options = brush.smooth_options;

                Style::Smooth(options)
            }
            BrushStyle::Solid => {
                let options = brush.smooth_options;

                Style::Smooth(options)
            }
            BrushStyle::Textured => {
                let mut options = brush.textured_options;
                options.seed = Some(seed);

                Style::Textured(options)
            }
        };

        let hitboxes = Vec::new();

        Self {
            path,
            style,
            hitboxes,
        }
    }

    pub fn push_segment(&mut self, segment: Segment) {
        self.path.push_back(segment);
    }

    pub fn update_geometry(&mut self) {
        self.hitboxes = self.gen_hitboxes();
    }

    fn gen_hitboxes(&self) -> Vec<AABB> {
        let width = match &self.style {
            Style::Smooth(options) => Some(options.width),
            Style::Rough(_) => None,
            Style::Textured(options) => Some(options.width),
        };

        if let Some(width) = width {
            self.path
                .iter()
                .map(|segment| segment.bounds().loosened(width))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn gen_svgs_for_last_segments(
        &self,
        no_last_segments: usize,
        _offset: na::Vector2<f64>,
        svg_root: bool,
    ) -> Result<Vec<render::Svg>, anyhow::Error> {
        let svg_elements = match &self.style {
            Style::Smooth(options) => self
                .path
                .iter()
                .rev()
                .take(no_last_segments)
                .filter_map(|segment| {
                    let bounds = segment.composed_bounds(options);
                    let mut svg_cx = piet_svg::RenderContext::new_no_text(kurbo::Size::new(
                        bounds.extents()[0],
                        bounds.extents()[1],
                    ));

                    segment.draw_composed(&mut svg_cx, options);

                    Some((composer::piet_svg_cx_to_svg(svg_cx).ok()?, bounds))
                })
                .collect::<Vec<(String, AABB)>>(),
            Style::Rough(_) => vec![],
            Style::Textured(options) => {
                let mut options = options.clone();
                // Advancing the seed exactly by one for each element, just like drawing the whole path.
                (0..self.path.len() - no_last_segments).for_each(|_i| {
                    options.seed = options
                        .seed
                        .map(|seed| rnote_compose::utils::seed_advance(seed));
                });

                self.path
                    .iter()
                    .rev()
                    .take(no_last_segments as usize)
                    .rev()
                    .filter_map(|segment| {
                        options.seed = options
                            .seed
                            .map(|seed| rnote_compose::utils::seed_advance(seed));

                        let bounds = segment.composed_bounds(&options);
                        let mut svg_cx = piet_svg::RenderContext::new_no_text(kurbo::Size::new(
                            bounds.extents()[0],
                            bounds.extents()[1],
                        ));

                        segment.draw_composed(&mut svg_cx, &options);

                        Some((composer::piet_svg_cx_to_svg(svg_cx).ok()?, bounds))
                    })
                    .collect::<Vec<(String, AABB)>>()
            }
        };

        Ok(svg_elements
            .into_iter()
            .map(|(mut svg_data, bounds)| {
                if svg_root {
                    svg_data = rnote_compose::utils::wrap_svg_root(
                        &svg_data,
                        Some(bounds),
                        Some(bounds),
                        false,
                    );
                }

                render::Svg { svg_data, bounds }
            })
            .collect::<Vec<render::Svg>>())
    }
}
