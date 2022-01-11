use crate::drawbehaviour::DrawBehaviour;
use crate::{compose, render, utils};
use crate::compose::geometry;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use super::strokebehaviour::StrokeBehaviour;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VectorImage {
    pub bounds: p2d::bounding_volume::AABB,
    pub intrinsic_size: na::Vector2<f64>,
    pub svg_data: String,
}

impl Default for VectorImage {
    fn default() -> Self {
        Self {
            bounds: geometry::aabb_new_zero(),
            intrinsic_size: na::vector![0.0, 0.0],
            svg_data: String::default(),
        }
    }
}

impl DrawBehaviour for VectorImage {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: p2d::bounding_volume::AABB) {
        self.bounds = bounds;
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        let bounds = geometry::aabb_translate(self.bounds, offset);
        let intrinsic_bounds = p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![self.intrinsic_size[0], self.intrinsic_size[1]],
        );

        let svg_data = compose::wrap_svg_root(
            self.svg_data.as_str(),
            Some(bounds),
            Some(intrinsic_bounds),
            false,
            false,
        );
        let svg = render::Svg { bounds, svg_data };

        Ok(vec![svg])
    }
}

impl StrokeBehaviour for VectorImage {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.bounds = geometry::aabb_translate(self.bounds, offset);
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        self.bounds = new_bounds;
    }
}

impl VectorImage {
    pub const SIZE_X_DEFAULT: f64 = 500.0;
    pub const SIZE_Y_DEFAULT: f64 = 500.0;
    pub const OFFSET_X_DEFAULT: f64 = 28.0;
    pub const OFFSET_Y_DEFAULT: f64 = 28.0;

    pub fn import_from_svg_data(
        svg_data: &str,
        pos: na::Vector2<f64>,
        size: Option<na::Vector2<f64>>,
    ) -> Result<Self, anyhow::Error> {
        let intrinsic_size = utils::svg_intrinsic_size(svg_data).unwrap_or_else(|| {
            na::vector![VectorImage::SIZE_X_DEFAULT, VectorImage::SIZE_Y_DEFAULT]
        });

        let bounds = size.map_or_else(
            || {
                p2d::bounding_volume::AABB::new(
                    na::Point2::from(pos),
                    na::Point2::from(intrinsic_size + pos),
                )
            },
            |size| {
                p2d::bounding_volume::AABB::new(na::Point2::from(pos), na::Point2::from(size + pos))
            },
        );

        let svg_data = compose::remove_xml_header(svg_data);

        let vector_image = Self {
            bounds,
            intrinsic_size,
            svg_data,
        };

        Ok(vector_image)
    }

    pub fn import_from_pdf_bytes(
        to_be_read: &[u8],
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let doc = poppler::Document::from_data(to_be_read, None)?;

        let mut images = Vec::new();

        for i in 0..doc.n_pages() {
            if let Some(page) = doc.page(i) {
                let intrinsic_size = page.size();

                let (width, height, _zoom) = if let Some(page_width) = page_width {
                    let zoom = f64::from(page_width) / intrinsic_size.0;

                    (f64::from(page_width), (intrinsic_size.1 * zoom), zoom)
                } else {
                    (intrinsic_size.0, intrinsic_size.1, 1.0)
                };

                let x = pos[0];
                let y = pos[1] + f64::from(i) * (height + Self::OFFSET_Y_DEFAULT / 2.0);

                let svg_stream: Vec<u8> = vec![];

                let surface =
                    cairo::SvgSurface::for_stream(intrinsic_size.0, intrinsic_size.1, svg_stream)
                        .map_err(|e| {
                        anyhow::anyhow!(
                            "create SvgSurface with dimensions ({}, {}) failed, {}",
                            intrinsic_size.0,
                            intrinsic_size.1,
                            e
                        )
                    })?;

                {
                    let cx = cairo::Context::new(&surface).context("new cairo::Context failed")?;

                    // Set margin to white
                    cx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    cx.paint()?;

                    page.render(&cx);

                    // Draw outline around page
                    cx.set_source_rgba(0.7, 0.5, 0.5, 1.0);

                    let line_width = 1.0;
                    cx.set_line_width(line_width);
                    cx.rectangle(
                        line_width / 2.0,
                        line_width / 2.0,
                        intrinsic_size.0 - line_width,
                        intrinsic_size.1 - line_width,
                    );
                    cx.stroke()?;
                }
                let svg_data = match surface.finish_output_stream() {
                    Ok(file_content) => match file_content.downcast::<Vec<u8>>() {
                        Ok(file_content) => *file_content,
                        Err(_) => {
                            log::error!("file_content.downcast() in VectorImage::import_from_pdf_bytes() failed");
                            continue;
                        }
                    },
                    Err(e) => {
                        log::error!("surface.finish_output_stream() in VectorImage::import_from_pdf_bytes() failed with Err {}", e);
                        continue;
                    }
                };
                let svg_data = String::from_utf8_lossy(&svg_data);

                images.push(Self::import_from_svg_data(
                    &svg_data.to_string(),
                    na::vector![x, y],
                    Some(na::vector![width, height]),
                )?);
            }
        }

        Ok(images)
    }
}
