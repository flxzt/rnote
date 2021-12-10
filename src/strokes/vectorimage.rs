use std::path::Path;

use crate::{compose, render};

use anyhow::Context;
use gtk4::gsk;
use serde::{Deserialize, Serialize};

use super::strokestyle::StrokeBehaviour;

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
            bounds: p2d::bounding_volume::AABB::new_invalid(),
            intrinsic_size: na::vector![0.0, 0.0],
            svg_data: String::default(),
        }
    }
}

impl StrokeBehaviour for VectorImage {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        self.bounds
    }

    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.bounds = self
            .bounds
            .transform_by(&na::geometry::Isometry2::new(offset, 0.0));
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        self.bounds = new_bounds;
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, anyhow::Error> {
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![
                self.bounds.mins[0] + offset[0],
                self.bounds.mins[1] + offset[1]
            ],
            na::point![
                self.bounds.maxs[0] + offset[0],
                self.bounds.maxs[1] + offset[1]
            ],
        );

        let intrinsic_bounds = p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![self.intrinsic_size[0], self.intrinsic_size[1]],
        );

        let svg = compose::wrap_svg(
            self.svg_data.as_str(),
            Some(bounds),
            Some(intrinsic_bounds),
            false,
            false,
        );
        Ok(svg)
    }

    fn gen_rendernode(
        &self,
        zoom: f64,
        renderer: &render::Renderer,
    ) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        Ok(Some(renderer.gen_rendernode(
            self.bounds,
            zoom,
            compose::add_xml_header(self.gen_svg_data(na::vector![0.0, 0.0])?.as_str()).as_str(),
        )?))
    }
}

impl VectorImage {
    pub const SIZE_X_DEFAULT: f64 = 500.0;
    pub const SIZE_Y_DEFAULT: f64 = 500.0;
    pub const OFFSET_X_DEFAULT: f64 = 28.0;
    pub const OFFSET_Y_DEFAULT: f64 = 28.0;

    pub fn import_from_svg(
        svg: &str,
        pos: na::Vector2<f64>,
        bounds: Option<p2d::bounding_volume::AABB>,
    ) -> Result<Self, anyhow::Error> {
        let (intrinsic_size, bounds) = if let Some(bounds) = bounds {
            (
                na::vector![
                    bounds.maxs[0] - bounds.mins[0],
                    bounds.maxs[1] - bounds.mins[1]
                ],
                bounds,
            )
        } else {
            let intrinsic_size = compose::svg_intrinsic_size(svg).unwrap_or_else(|| {
                na::vector![VectorImage::SIZE_X_DEFAULT, VectorImage::SIZE_Y_DEFAULT]
            });

            let intrinsic_bounds = p2d::bounding_volume::AABB::new(
                na::Point2::from(pos),
                na::Point2::from(intrinsic_size + pos),
            );

            (intrinsic_size, intrinsic_bounds)
        };

        let svg_data = compose::remove_xml_header(svg);

        let vector_image = Self {
            bounds,
            intrinsic_size,
            svg_data,
        };

        Ok(vector_image)
    }

    pub fn import_from_pdf_bytes(
        to_be_read: &[u8],
        _pos: na::Vector2<f64>,
        page_width: Option<i32>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let doc = poppler::Document::from_data(to_be_read, None)?;

        let images = Vec::new();

        for i in 0..doc.n_pages() {
            if let Some(page) = doc.page(i) {
                let intrinsic_size = page.size();
                let (width, height, zoom) = if let Some(page_width) = page_width {
                    let zoom = f64::from(page_width) / intrinsic_size.0;

                    (f64::from(page_width), (intrinsic_size.1 * zoom), zoom)
                } else {
                    (intrinsic_size.0, intrinsic_size.1, 1.0)
                };
                /*
                let x = pos[0];
                let y = pos[1] + f64::from(i) * (height + Self::OFFSET_Y_DEFAULT / 2.0);

                let bounds = p2d::bounding_volume::AABB::new(
                    na::point![x, y],
                    na::point![x + width, y + height],
                ); */

                let surface =
                    cairo::SvgSurface::new(width, height, None::<&Path>).map_err(|e| {
                        anyhow::anyhow!(
                            "create SvgSurface with dimensions ({}, {}) failed, {}",
                            width,
                            height,
                            e
                        )
                    })?;

                {
                    let cx = cairo::Context::new(&surface).context("new cairo::Context failed")?;
                    cx.scale(zoom, zoom);

                    // Set margin to white
                    cx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    cx.paint()?;

                    page.render(&cx);

                    cx.scale(1.0 / zoom, 1.0 / zoom);

                    // Draw outline around page
                    cx.set_source_rgba(0.7, 0.5, 0.5, 1.0);

                    let line_width = 1.0;
                    cx.set_line_width(line_width);
                    cx.rectangle(
                        f64::from(0) + line_width / 2.0,
                        f64::from(0) + line_width / 2.0,
                        f64::from(width) - line_width,
                        f64::from(height) - line_width,
                    );
                    cx.stroke()?;
                }

                /*                 images.push(Self::import_from_svg(
                    &svg.into_owned(),
                    na::vector![x, y],
                    Some(bounds),
                )?); */
            }
        }

        Ok(images)
    }
}
