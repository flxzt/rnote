use std::io;

use crate::{compose, render};
use anyhow::Context;
use image::{io::Reader, GenericImageView};
use serde::{Deserialize, Serialize};

use crate::strokes::strokestyle::StrokeBehaviour;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Format {
    Png,
    Jpeg,
}

impl Format {
    pub fn as_mime_type(&self) -> String {
        match self {
            Format::Png => String::from("image/png"),
            Format::Jpeg => String::from("image/jpeg"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BitmapImage {
    pub data_base64: String,
    pub format: Format,
    pub bounds: p2d::bounding_volume::AABB,
    pub intrinsic_size: na::Vector2<f64>,
}

impl Default for BitmapImage {
    fn default() -> Self {
        Self {
            data_base64: String::default(),
            format: Format::Png,
            bounds: p2d::bounding_volume::AABB::new_invalid(),
            intrinsic_size: na::vector![0.0, 0.0],
        }
    }
}

pub const BITMAPIMAGE_TEMPL_STR: &str = r#"
<image x="{{x}}" y="{{y}}" width="{{width}}" height="{{height}}" href="data:{{mime_type}};base64,{{data_base64}}"/>
"#;

impl StrokeBehaviour for BitmapImage {
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

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        let mut cx = tera::Context::new();

        let x = 0.0;
        let y = 0.0;
        let width = self.intrinsic_size[0];
        let height = self.intrinsic_size[1];

        cx.insert("x", &x);
        cx.insert("y", &y);
        cx.insert("width", &width);
        cx.insert("height", &height);
        cx.insert("data_base64", &self.data_base64);
        cx.insert("mime_type", &self.format.as_mime_type());

        let svg = tera::Tera::one_off(BITMAPIMAGE_TEMPL_STR, &cx, false)?;

        let intrinsic_bounds = p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![self.intrinsic_size[0], self.intrinsic_size[1]],
        );

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

        let svg_data = compose::wrap_svg(
            svg.as_str(),
            Some(bounds),
            Some(intrinsic_bounds),
            false,
            false,
        );
        let svg = render::Svg { bounds, svg_data };

        Ok(vec![svg])
    }
}

impl BitmapImage {
    pub const SIZE_X_DEFAULT: f64 = 500.0;
    pub const SIZE_Y_DEFAULT: f64 = 500.0;
    pub const OFFSET_X_DEFAULT: f64 = 32.0;
    pub const OFFSET_Y_DEFAULT: f64 = 32.0;

    pub fn import_from_image_bytes<P>(
        to_be_read: P,
        pos: na::Vector2<f64>,
    ) -> Result<Self, anyhow::Error>
    where
        P: AsRef<[u8]>,
    {
        let reader = Reader::new(io::Cursor::new(&to_be_read)).with_guessed_format()?;
        log::debug!("BitmapImage detected format: {:?}", reader.format());

        let format = match reader.format() {
            Some(image::ImageFormat::Png) => Format::Png,
            Some(image::ImageFormat::Jpeg) => Format::Jpeg,
            _ => {
                return Err(anyhow::Error::msg("unsupported format."));
            }
        };

        let bitmap_data = reader.decode()?;
        let dimensions = bitmap_data.dimensions();
        let intrinsic_size = na::vector![f64::from(dimensions.0), f64::from(dimensions.1)];

        let bounds = p2d::bounding_volume::AABB::new(
            na::Point2::from(pos),
            na::Point2::from(intrinsic_size + pos),
        );
        let data_base64 = base64::encode(&to_be_read);

        Ok(Self {
            data_base64,
            format,
            bounds,
            intrinsic_size,
        })
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
                let (width, height, zoom) = if let Some(page_width) = page_width {
                    let zoom = f64::from(page_width) / intrinsic_size.0;

                    (page_width, (intrinsic_size.1 * zoom).round() as i32, zoom)
                } else {
                    (
                        intrinsic_size.0.round() as i32,
                        intrinsic_size.1.round() as i32,
                        1.0,
                    )
                };

                let x = pos[0];
                let y = pos[1]
                    + f64::from(i) * (f64::from(height) + Self::OFFSET_Y_DEFAULT.round() / 2.0);

                let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "create ImageSurface with dimensions ({}, {}) failed, {}",
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

                let mut png_data: Vec<u8> = Vec::new();
                surface.write_to_png(&mut png_data)?;

                images.push(Self::import_from_image_bytes(&png_data, na::vector![x, y])?);
            }
        }

        Ok(images)
    }
}
