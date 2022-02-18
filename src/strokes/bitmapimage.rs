use std::io;
use std::sync::{Arc, RwLock};

use crate::compose::geometry::AABBHelpers;
use crate::compose::shapes;
use crate::drawbehaviour::DrawBehaviour;
use crate::render::Renderer;
use crate::{compose, render};
use anyhow::Context;
use gtk4::cairo;
use image::{io::Reader, GenericImageView};
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};
use svg::node::element;

use crate::compose::transformable::{Transform, Transformable};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "bitmapimage_format")]
pub enum BitmapImageFormat {
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "jpeg")]
    Jpeg,
}

impl BitmapImageFormat {
    pub fn as_mime_type(&self) -> String {
        match self {
            BitmapImageFormat::Png => String::from("image/png"),
            BitmapImageFormat::Jpeg => String::from("image/jpeg"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "bitmapimage")]
pub struct BitmapImage {
    #[serde(rename = "data_base64")]
    pub data_base64: String,
    #[serde(rename = "format")]
    pub format: BitmapImageFormat,
    #[serde(rename = "intrinsic_size")]
    pub intrinsic_size: na::Vector2<f64>,
    #[serde(rename = "rectangle")]
    pub rectangle: shapes::Rectangle,
    #[serde(rename = "bounds")]
    pub bounds: AABB,
}

impl Default for BitmapImage {
    fn default() -> Self {
        Self {
            data_base64: String::default(),
            format: BitmapImageFormat::Png,
            intrinsic_size: na::vector![0.0, 0.0],
            rectangle: shapes::Rectangle::default(),
            bounds: AABB::new_zero(),
        }
    }
}

impl DrawBehaviour for BitmapImage {
    fn bounds(&self) -> AABB {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: AABB) {
        self.bounds = bounds;
    }

    fn gen_bounds(&self) -> Option<AABB> {
        Some(self.rectangle.global_aabb())
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        let mut rectangle = self.rectangle.clone();
        rectangle.transform.append_translation_mut(offset);

        let transform_string = rectangle.transform.transform_as_svg_transform_attr();

        let svg_root = element::Image::new()
            .set("x", -self.rectangle.cuboid.half_extents[0])
            .set("y", -self.rectangle.cuboid.half_extents[1])
            .set("width", 2.0 * self.rectangle.cuboid.half_extents[0])
            .set("height", 2.0 * self.rectangle.cuboid.half_extents[1])
            .set(
                "viewBox",
                format!(
                    "{:.3} {:.3} {:.3} {:.3}",
                    0.0, 0.0, self.intrinsic_size[0], self.intrinsic_size[1]
                ),
            )
            .set("preserveAspectRatio", "none")
            .set("transform", transform_string)
            .set(
                "href",
                format!(
                    "data:{mime_type};base64,{data_base64}",
                    mime_type = &self.format.as_mime_type(),
                    data_base64 = &self.data_base64
                ),
            );

        let svg_data = compose::svg_node_to_string(&svg_root)?;
        let svg = render::Svg {
            bounds: self.bounds.translate(offset),
            svg_data,
        };

        Ok(vec![svg])
    }
}

impl Transformable for BitmapImage {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.rectangle.translate(offset);
        self.update_geometry();
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.rectangle.rotate(angle, center);
        self.update_geometry();
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.rectangle.scale(scale);
        self.update_geometry();
    }
}

impl BitmapImage {
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
            Some(image::ImageFormat::Png) => BitmapImageFormat::Png,
            Some(image::ImageFormat::Jpeg) => BitmapImageFormat::Jpeg,
            _ => {
                return Err(anyhow::Error::msg("unsupported format."));
            }
        };

        let data_base64 = base64::encode(&to_be_read);

        let intrinsic_size = extract_dimensions(&to_be_read)?;

        let rectangle = shapes::Rectangle {
            cuboid: p2d::shape::Cuboid::new(intrinsic_size / 2.0),
            transform: Transform::new_w_isometry(na::Isometry2::new(
                pos + intrinsic_size / 2.0,
                0.0,
            )),
        };

        let mut bitmapimage = Self {
            data_base64,
            format,
            intrinsic_size,
            rectangle,
            bounds: AABB::new_zero(),
        };
        bitmapimage.update_geometry();

        Ok(bitmapimage)
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
                    + f64::from(i) * (f64::from(height) + f64::from(Self::OFFSET_Y_DEFAULT) / 2.0);

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
                        line_width / 2.0,
                        line_width / 2.0,
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

    pub fn update_geometry(&mut self) {
        if let Some(new_bounds) = self.gen_bounds() {
            self.set_bounds(new_bounds);
        }
    }

    pub fn export_as_image_bytes(
        &self,
        zoom: f64,
        format: image::ImageOutputFormat,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let export_bounds = self.bounds.translate(-self.bounds().mins.coords);
        let mut export_svg_data = self
            .gen_svgs(-self.bounds().mins.coords)
            .context("gen_svgs() failed in BitmapImage export_as_bytes()")?
            .iter()
            .map(|svg| svg.svg_data.clone())
            .collect::<Vec<String>>()
            .join("\n");
        export_svg_data = compose::wrap_svg_root(
            export_svg_data.as_str(),
            Some(export_bounds),
            Some(export_bounds),
            false,
        );
        let export_svg = render::Svg {
            bounds: export_bounds,
            svg_data: export_svg_data,
        };

        let image_raw = render::concat_images(
            renderer
                .read()
                .unwrap()
                .gen_images(zoom, vec![export_svg], export_bounds)?,
            export_bounds,
            zoom,
        )?;

        Ok(render::image_into_encoded_bytes(image_raw, format)?)
    }
}

pub fn extract_dimensions<P>(to_be_read: P) -> Result<na::Vector2<f64>, anyhow::Error>
where
    P: AsRef<[u8]>,
{
    let reader = Reader::new(io::Cursor::new(&to_be_read)).with_guessed_format()?;

    let bitmap_data = reader.decode()?;
    let dimensions = bitmap_data.dimensions();

    Ok(na::vector![
        f64::from(dimensions.0),
        f64::from(dimensions.1)
    ])
}
