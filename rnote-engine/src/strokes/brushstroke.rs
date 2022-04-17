use super::StrokeBehaviour;
use crate::render::{self};
use crate::DrawBehaviour;
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::Composer;
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::{PenPath, Style};

use p2d::bounding_volume::{BoundingVolume, AABB};
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
            Style::default(),
        )
    }
}

impl StrokeBehaviour for BrushStroke {
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error> {
        let bounds = self.bounds();
        let mut cx = piet_svg::RenderContext::new_no_text(kurbo::Size::new(
            bounds.extents()[0],
            bounds.extents()[1],
        ));

        self.draw(&mut cx, 1.0)?;
        let svg_data = rnote_compose::utils::piet_svg_cx_to_svg(cx)?;

        Ok(render::Svg { svg_data, bounds })
    }

    fn gen_images(&self, image_scale: f64) -> Result<Vec<render::Image>, anyhow::Error> {
        let images = match &self.style {
            Style::Smooth(options) => self
                .path
                .iter()
                .filter_map(|segment| {
                    let image = render::Image::gen_with_piet(
                        |piet_cx| {
                            segment.draw_composed(piet_cx, options);
                            Ok(())
                        },
                        segment.composed_bounds(options),
                        image_scale,
                    );

                    match image {
                        Ok(image) => Some(image),
                        Err(e) => {
                            log::error!("gen_images() failed with Err {}", e);
                            None
                        }
                    }
                })
                .flatten()
                .collect::<Vec<render::Image>>(),
            Style::Rough(_) => vec![],
            Style::Textured(options) => {
                let mut options = options.clone();

                self.path
                    .iter()
                    .filter_map(|segment| {
                        options.seed = options
                            .seed
                            .map(|seed| rnote_compose::utils::seed_advance(seed));

                        let image = render::Image::gen_with_piet(
                            |piet_cx| {
                                segment.draw_composed(piet_cx, &options);
                                Ok(())
                            },
                            segment.composed_bounds(&options),
                            image_scale,
                        );

                        match image {
                            Ok(image) => Some(image),
                            Err(e) => {
                                log::error!("gen_images() failed with Err {}", e);
                                None
                            }
                        }
                    })
                    .flatten()
                    .collect::<Vec<render::Image>>()
            }
        };

        Ok(images)
    }
}

impl DrawBehaviour for BrushStroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
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

    pub fn new(segment: Segment, style: Style) -> Self {
        let path = PenPath::new_w_segment(segment);

        Self::from_penpath(path, style)
    }

    pub fn from_penpath(path: PenPath, style: Style) -> Self {
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
            Style::Smooth(options) => Some(options.stroke_width),
            Style::Rough(_) => None,
            Style::Textured(options) => Some(options.stroke_width),
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

    pub fn gen_images_for_last_segments(
        &self,
        no_last_segments: usize,
        image_scale: f64,
    ) -> Result<Vec<render::Image>, anyhow::Error> {
        let images = match &self.style {
            Style::Smooth(options) => self
                .path
                .iter()
                .rev()
                .take(no_last_segments)
                .rev()
                .filter_map(|segment| {
                    let image = render::Image::gen_with_piet(
                        |piet_cx| {
                            segment.draw_composed(piet_cx, options);
                            Ok(())
                        },
                        segment.composed_bounds(options),
                        image_scale,
                    );

                    match image {
                        Ok(image) => Some(image),
                        Err(e) => {
                            log::error!("gen_images_for_last_segments() failed with Err {}", e);
                            None
                        }
                    }
                })
                .flatten()
                .collect::<Vec<render::Image>>(),
            Style::Rough(_) => vec![],
            Style::Textured(options) => self
                .path
                .iter()
                .enumerate()
                .rev()
                .take(no_last_segments)
                .rev()
                .filter_map(|(i, segment)| {
                    let mut options = options.clone();
                    (0..=i).for_each(|_| {
                        options.seed = options
                            .seed
                            .map(|seed| rnote_compose::utils::seed_advance(seed))
                    });

                    let image = render::Image::gen_with_piet(
                        |piet_cx| {
                            segment.draw_composed(piet_cx, &options);
                            Ok(())
                        },
                        segment.composed_bounds(&options),
                        image_scale,
                    );

                    match image {
                        Ok(image) => Some(image),
                        Err(e) => {
                            log::error!("gen_images_for_last_segments() failed with Err {}", e);
                            None
                        }
                    }
                })
                .flatten()
                .collect::<Vec<render::Image>>(),
        };

        Ok(images)
    }
}
