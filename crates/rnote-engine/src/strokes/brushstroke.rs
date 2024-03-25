// Imports
use super::content::GeneratedContentImages;
use super::Content;
use crate::Drawable;
use crate::{
    render::{self},
    strokes::content,
};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::ext::AabbExt;
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::shapes::Shapeable;
use rnote_compose::style::Composer;
use rnote_compose::transform::Transformable;
use rnote_compose::{PenPath, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "brushstroke")]
pub struct BrushStroke {
    #[serde(rename = "path")]
    pub path: PenPath,
    #[serde(default, rename = "style")]
    pub style: Style,
    // since the path can have many hitboxes, we store them here and update them when the stroke geometry changes
    #[serde(skip)]
    hitboxes: Vec<Aabb>,
}

impl Content for BrushStroke {
    fn gen_images(
        &self,
        viewport: Aabb,
        image_scale: f64,
    ) -> Result<GeneratedContentImages, anyhow::Error> {
        /// The threshold of the image size on either axis.
        /// When below it the stroke is rendered as a single image
        const IMAGES_SIZE_THRESHOLD: f64 = 1000.0;
        /// The threshold of the stroke width in relation to either bound axis.
        /// When above it the stroke is rendered as a single image
        const IMAGES_STROKE_WIDTH_BOUNDS_THRESHOLD: f64 = 0.2;

        let bounds = self.bounds();
        let partial = !viewport.contains(&bounds);
        let Some(bounds) = viewport.intersection(&bounds) else {
            return Ok(GeneratedContentImages::Partial {
                images: vec![],
                viewport,
            });
        };
        let bounds_extents = bounds.extents();

        let image_size_condition = bounds_extents[0] < IMAGES_SIZE_THRESHOLD / image_scale
            && bounds_extents[1] < IMAGES_SIZE_THRESHOLD / image_scale;

        let stroke_width_condition = self.style.stroke_width()
            > IMAGES_STROKE_WIDTH_BOUNDS_THRESHOLD * bounds_extents[0]
            || self.style.stroke_width() > IMAGES_STROKE_WIDTH_BOUNDS_THRESHOLD * bounds_extents[1];

        // if these conditions evaluate true the stroke is rendered as a single image
        let images = if image_size_condition || stroke_width_condition {
            // generate a single image when bounds are smaller than threshold
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
                            tracing::error!(
                                "Generating images for brushstroke failed , Err: {e:?}"
                            );
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
                            tracing::error!(
                                "Generating images for brushstroke failed , Err: {e:?}"
                            );
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
                        let image = render::Image::gen_with_piet(
                            |piet_cx| {
                                seg_path.draw_composed(piet_cx, options);
                                Ok(())
                            },
                            seg_path.composed_bounds(options),
                            image_scale,
                        );
                        match image {
                            Ok(image) => images.push(image),
                            Err(e) => {
                                tracing::error!(
                                    "generating images for brushstroke failed , Err: {e:?}"
                                );
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
                        let image = render::Image::gen_with_piet(
                            |piet_cx| {
                                seg_path.draw_composed(piet_cx, &options);
                                Ok(())
                            },
                            seg_path.composed_bounds(&options),
                            image_scale,
                        );
                        match image {
                            Ok(image) => images.push(image),
                            Err(e) => {
                                tracing::error!(
                                    "generating images for brushstroke failed , Err: {e:?}"
                                );
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
            Ok(GeneratedContentImages::Partial { images, viewport })
        } else {
            Ok(GeneratedContentImages::Full(images))
        }
    }

    fn draw_highlight(
        &self,
        cx: &mut impl piet::RenderContext,
        total_zoom: f64,
    ) -> anyhow::Result<()> {
        const PATH_HIGHLIGHT_MIN_STROKE_WIDTH: f64 = 5.0;
        const DRAW_BOUNDS_THRESHOLD_AREA: f64 = 10_u32.pow(2) as f64;

        let bounds = self.bounds();

        if bounds.scale(total_zoom).volume() < DRAW_BOUNDS_THRESHOLD_AREA {
            cx.fill(bounds.to_kurbo_rect(), &content::CONTENT_HIGHLIGHT_COLOR);
        } else {
            cx.stroke_styled(
                self.outline_path(),
                &content::CONTENT_HIGHLIGHT_COLOR,
                (PATH_HIGHLIGHT_MIN_STROKE_WIDTH / total_zoom)
                    .max(self.style.stroke_width() + 3.0 / total_zoom),
                &piet::StrokeStyle::new()
                    .line_join(piet::LineJoin::Round)
                    .line_cap(piet::LineCap::Round),
            );
        }
        Ok(())
    }

    fn update_geometry(&mut self) {
        self.hitboxes = self.gen_hitboxes_int();
    }
}

impl Drawable for BrushStroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        match &self.style {
            Style::Smooth(options) => self.path.draw_composed(cx, options),
            Style::Rough(_) => {
                // Rough style currently unsupported for pen paths
                unimplemented!()
            }
            Style::Textured(options) => self.path.draw_composed(cx, options),
        };

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl Shapeable for BrushStroke {
    fn bounds(&self) -> Aabb {
        match &self.style {
            Style::Smooth(options) => self.path.composed_bounds(options),
            Style::Rough(_options) => unimplemented!(),
            Style::Textured(options) => self.path.composed_bounds(options),
        }
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        self.hitboxes.clone()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        self.path.outline_path()
    }
}

impl Transformable for BrushStroke {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.path.translate(offset);
    }
    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.path.rotate(angle, center);
    }
    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.path.scale(scale);
        // Using the geometric mean behaves the best when scaling non-uniformly.
        let scale_scalar = (scale[0] * scale[1]).sqrt();
        self.style
            .set_stroke_width(self.style.stroke_width() * scale_scalar);
    }
}

impl BrushStroke {
    pub fn new(start: Element, style: Style) -> Self {
        let path = PenPath::new(start);

        Self::from_penpath(path, style)
    }

    /// Construct from the given pen path and style.
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

    /// Replace the current path with the given new one. the new path must not be empty.
    pub fn replace_path(&mut self, path: PenPath) {
        self.path = path;
        self.update_geometry();
    }

    // internal method generating the current hitboxes.
    fn gen_hitboxes_int(&self) -> Vec<Aabb> {
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
