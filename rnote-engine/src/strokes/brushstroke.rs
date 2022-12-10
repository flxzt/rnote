use super::strokebehaviour::GeneratedStrokeImages;
use super::StrokeBehaviour;
use crate::render::{self};
use crate::DrawBehaviour;
use piet::RenderContext;
use rnote_compose::helpers::Vector2Helpers;
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::Composer;
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::{PenPath, Style};

use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "brushstroke")]
pub struct BrushStroke {
    #[serde(rename = "path")]
    pub path: PenPath,
    #[serde(default, rename = "style")]
    pub style: Style,
    #[serde(skip)]
    // since the path can have many hitboxes, we store them for faster queries and update them when the stroke geometry changes
    hitboxes: Vec<AABB>,
}

impl StrokeBehaviour for BrushStroke {
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error> {
        let bounds = self.bounds();

        render::Svg::gen_with_piet_cairo_backend(
            |cx| {
                cx.transform(kurbo::Affine::translate(-bounds.mins.coords.to_kurbo_vec()));
                self.draw(cx, 1.0)
            },
            bounds,
        )
    }

    fn gen_images(
        &self,
        viewport: AABB,
        image_scale: f64,
    ) -> Result<GeneratedStrokeImages, anyhow::Error> {
        let bounds = self.bounds();
        let (bounds, partial) = if viewport.contains(&bounds) {
            (bounds, false)
        } else {
            (viewport, true)
        };

        let images = if bounds.extents()[0] < Self::IMAGES_SEGMENTS_THRESHOLD / image_scale
            && bounds.extents()[1] < Self::IMAGES_SEGMENTS_THRESHOLD / image_scale
        {
            // generate a single image when bounds are below threshold
            match &self.style {
                Style::Smooth(options) => {
                    let image = render::Image::gen_with_piet(
                        |piet_cx| {
                            self.path.draw_composed(piet_cx, options);
                            Ok(())
                        },
                        bounds,
                        image_scale,
                    );

                    match image {
                        Ok(image) => vec![image],
                        Err(e) => {
                            log::error!("gen_images() in brushstroke failed with Err: {e:?}");
                            vec![]
                        }
                    }
                }
                Style::Rough(_options) => {
                    // Unsupported
                    vec![]
                }
                Style::Textured(options) => {
                    let image = render::Image::gen_with_piet(
                        |piet_cx| {
                            self.path.draw_composed(piet_cx, options);
                            Ok(())
                        },
                        bounds,
                        image_scale,
                    );

                    match image {
                        Ok(image) => vec![image],
                        Err(e) => {
                            log::error!("gen_images() in brushstroke failed with Err: {e:?}");
                            vec![]
                        }
                    }
                }
            }
        } else {
            match &self.style {
                Style::Smooth(options) => {
                    let mut images = Vec::with_capacity(self.path.segments.len());

                    let mut prev = self.path.start;
                    for seg in self.path.segments.iter() {
                        let seg_path = PenPath::new_w_segments(prev, [*seg]);

                        match render::Image::gen_with_piet(
                            |piet_cx| {
                                seg_path.draw_composed(piet_cx, options);
                                Ok(())
                            },
                            seg_path.composed_bounds(options),
                            image_scale,
                        ) {
                            Ok(image) => images.push(image),
                            Err(e) => {
                                log::error!("gen_images() in brushstroke failed with Err: {e:?}")
                            }
                        }

                        prev = seg.end();
                    }

                    images
                }
                Style::Rough(_) => {
                    // Unsupported
                    vec![]
                }
                Style::Textured(options) => {
                    let mut options = options.clone();
                    let mut images = Vec::with_capacity(self.path.segments.len());

                    let mut prev = self.path.start;
                    for seg in self.path.segments.iter() {
                        let seg_path = PenPath::new_w_segments(prev, [*seg]);

                        match render::Image::gen_with_piet(
                            |piet_cx| {
                                seg_path.draw_composed(piet_cx, &options);
                                Ok(())
                            },
                            seg_path.composed_bounds(&options),
                            image_scale,
                        ) {
                            Ok(image) => images.push(image),
                            Err(e) => {
                                log::error!("gen_images() in brushstroke failed with Err: {e:?}")
                            }
                        }

                        options.advance_seed();

                        prev = seg.end();
                    }

                    images
                }
            }
        };

        if partial {
            Ok(GeneratedStrokeImages::Partial { images, viewport })
        } else {
            Ok(GeneratedStrokeImages::Full(images))
        }
    }
}

impl DrawBehaviour for BrushStroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        match &self.style {
            Style::Smooth(options) => self.path.draw_composed(cx, options),
            Style::Rough(_) => {
                // Rough style currently unsupported for pen paths
            }
            Style::Textured(options) => self.path.draw_composed(cx, options),
        };

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
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

    fn hitboxes(&self) -> Vec<AABB> {
        self.hitboxes.clone()
    }
}

impl TransformBehaviour for BrushStroke {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.path.translate(offset);
    }
    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.path.rotate(angle, center);
    }
    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.path.scale(scale);
    }
}

impl BrushStroke {
    /// when one of the extents of the stroke is above this threshold, images are generated separately for each stroke segment (to avoid very large images)
    pub const IMAGES_SEGMENTS_THRESHOLD: f64 = 1000.0;

    pub fn new(start: Element, style: Style) -> Self {
        let path = PenPath::new(start);

        Self::from_penpath(path, style)
    }

    /// New from pen path.
    pub fn from_penpath(path: PenPath, style: Style) -> Self {
        let mut new_brushstroke = Self {
            path,
            style,
            hitboxes: vec![],
        };
        new_brushstroke.update_geometry();

        new_brushstroke
    }

    pub fn push_segment(&mut self, segment: Segment) {
        self.path.segments.push(segment);
    }

    pub fn extend_w_segments(&mut self, segments: impl IntoIterator<Item = Segment>) {
        self.path.extend(segments);
    }

    pub fn update_geometry(&mut self) {
        self.hitboxes = self.gen_hitboxes();
    }

    /// Replacing the current path with a new one. the new path must not be empty.
    pub fn replace_path(&mut self, path: PenPath) {
        self.path = path;
        self.update_geometry();
    }

    // internal method generating the current hitboxes.
    fn gen_hitboxes(&self) -> Vec<AABB> {
        let stroke_width = self.style.stroke_width();

        self.path
            .hitboxes()
            .into_iter()
            .map(|hb| hb.loosened(stroke_width * 0.5))
            .collect()
    }

    pub fn gen_image_for_last_segments(
        &self,
        n_last_segments: usize,
        image_scale: f64,
    ) -> Result<Option<render::Image>, anyhow::Error> {
        let image = match &self.style {
            Style::Smooth(options) => {
                let path_len = self.path.segments.len();

                let start_el = self
                    .path
                    .segments
                    .get(path_len.saturating_sub(n_last_segments).saturating_sub(1))
                    .map(|s| s.end())
                    .unwrap_or(self.path.start);

                let range_path = PenPath::new_w_segments(
                    start_el,
                    self.path.segments[path_len.saturating_sub(n_last_segments)..]
                        .iter()
                        .copied(),
                );

                let image = render::Image::gen_with_piet(
                    |piet_cx| {
                        range_path.draw_composed(piet_cx, options);
                        Ok(())
                    },
                    range_path.composed_bounds(options),
                    image_scale,
                )?;

                Some(image)
            }
            Style::Rough(_) => None,
            Style::Textured(options) => {
                let mut options = options.clone();
                let path_len = self.path.segments.len();

                (0..path_len.saturating_sub(n_last_segments)).for_each(|_| {
                    options.advance_seed();
                });

                let start_el = self
                    .path
                    .segments
                    .get(path_len.saturating_sub(n_last_segments).saturating_sub(1))
                    .map(|s| s.end())
                    .unwrap_or(self.path.start);

                let range_path = PenPath::new_w_segments(
                    start_el,
                    self.path.segments[path_len.saturating_sub(n_last_segments)..]
                        .iter()
                        .copied(),
                );

                let image = render::Image::gen_with_piet(
                    |piet_cx| {
                        range_path.draw_composed(piet_cx, &options);
                        Ok(())
                    },
                    range_path.composed_bounds(&options),
                    image_scale,
                )?;

                Some(image)
            }
        };

        Ok(image)
    }
}
