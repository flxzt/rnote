use std::io;

use crate::{compose, render};
use gtk4::gsk;
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

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, anyhow::Error> {
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

        let svg = compose::wrap_svg(
            svg.as_str(),
            Some(bounds),
            Some(intrinsic_bounds),
            false,
            false,
        );

        Ok(svg)
    }

    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        Ok(Some(renderer.gen_rendernode(
            self.bounds,
            scalefactor,
            compose::add_xml_header(self.gen_svg_data(na::vector![0.0, 0.0])?.as_str()).as_str(),
        )?))
    }
}

impl BitmapImage {
    pub const SIZE_X_DEFAULT: f64 = 500.0;
    pub const SIZE_Y_DEFAULT: f64 = 500.0;
    pub const OFFSET_X_DEFAULT: f64 = 28.0;
    pub const OFFSET_Y_DEFAULT: f64 = 28.0;

    pub fn import_from_image_bytes<P>(
        to_be_read: P,
        pos: na::Vector2<f64>,
    ) -> Result<Self, anyhow::Error>
    where
        P: AsRef<[u8]>,
    {
        let reader = Reader::new(io::Cursor::new(&to_be_read)).with_guessed_format()?;
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
}
