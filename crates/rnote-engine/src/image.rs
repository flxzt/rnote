// Imports
use crate::Drawable;
use anyhow::Context;
use core::fmt::Debug;
use image::ImageReader;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::math::Vector2;
use rnote_compose::Transformable;
use rnote_compose::ext::{AabbExt, DAffine2Ext, Vector2Ext};
use rnote_compose::shapes::{Rectangle, Shapeable};
use serde::{Deserialize, Serialize};
use std::io::{self, Cursor};
use std::sync::Arc;
use vello_cpu::{RenderContext, peniko};

/// Px unit (96 DPI ) to Point unit ( 72 DPI ) conversion factor.
pub const PX_TO_POINT_CONV_FACTOR: f64 = 96.0 / 72.0;
/// Point unit ( 72 DPI ) to Px unit (96 DPI ) conversion factor.
pub const POINT_TO_PX_CONV_FACTOR: f64 = 72.0 / 96.0;
/// The factor for which the rendering for the current viewport is extended by.
/// For example:: 1.0 means the viewport is extended by its own extents on all sides.
///
/// Used when checking rendering for new zooms or a moved viewport.
/// There is a trade off: a larger value will consume more memory, a smaller value will mean more stuttering on zooms and when moving the view.
pub const VIEWPORT_EXTENTS_MARGIN_FACTOR: f64 = 0.4;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ImageMemoryFormat {
    R8g8b8a8Premultiplied,
}

impl Default for ImageMemoryFormat {
    fn default() -> Self {
        Self::R8g8b8a8Premultiplied
    }
}

impl From<ImageMemoryFormat> for peniko::ImageFormat {
    fn from(value: ImageMemoryFormat) -> Self {
        match value {
            ImageMemoryFormat::R8g8b8a8Premultiplied => Self::Rgba8,
        }
    }
}

#[cfg(feature = "ui")]
impl TryFrom<gtk4::gdk::MemoryFormat> for ImageMemoryFormat {
    type Error = anyhow::Error;
    fn try_from(value: gtk4::gdk::MemoryFormat) -> Result<Self, Self::Error> {
        match value {
            gtk4::gdk::MemoryFormat::R8g8b8a8Premultiplied => Ok(Self::R8g8b8a8Premultiplied),
            _ => Err(anyhow::anyhow!(
                "ImageMemoryFormat try_from() gdk::MemoryFormat failed, unsupported MemoryFormat `{:?}`",
                value
            )),
        }
    }
}

#[cfg(feature = "ui")]
impl From<ImageMemoryFormat> for gtk4::gdk::MemoryFormat {
    fn from(value: ImageMemoryFormat) -> Self {
        match value {
            ImageMemoryFormat::R8g8b8a8Premultiplied => {
                gtk4::gdk::MemoryFormat::R8g8b8a8Premultiplied
            }
        }
    }
}

/// A bitmap image.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename = "image")]
pub struct Image {
    /// The image data.
    ///
    /// Is (de)serialized with base64 encoding.
    #[serde(rename = "data", with = "crate::utils::glib_bytes_base64")]
    pub data: glib::Bytes,
    /// The target rect in the coordinate space of the document.
    #[serde(rename = "rectangle")]
    pub rectangle: Rectangle,
    /// Width of the image data.
    #[serde(rename = "pixel_width")]
    pub pixel_width: u32,
    /// Height of the image data.
    #[serde(rename = "pixel_height")]
    pub pixel_height: u32,
    /// Memory format.
    #[serde(rename = "memory_format")]
    pub memory_format: ImageMemoryFormat,
}

impl Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("data", &String::from("{.. no debug impl ..}"))
            .field("rect", &self.rectangle)
            .field("pixel_width", &self.pixel_width)
            .field("pixel_height", &self.pixel_height)
            .field("memory_format", &self.memory_format)
            .finish()
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            data: glib::Bytes::from_owned(Vec::new()),
            rectangle: Rectangle::default(),
            pixel_width: 0,
            pixel_height: 0,
            memory_format: ImageMemoryFormat::default(),
        }
    }
}

impl From<image::DynamicImage> for Image {
    fn from(dynamic_image: image::DynamicImage) -> Self {
        let pixel_width = dynamic_image.width();
        let pixel_height = dynamic_image.height();
        let memory_format = ImageMemoryFormat::R8g8b8a8Premultiplied;
        let data = glib::Bytes::from_owned(dynamic_image.into_rgba8().to_vec());
        let bounds = Aabb::new(
            Vector2::ZERO,
            Vector2::new(pixel_width as f64, pixel_height as f64),
        );

        Self {
            data,
            rectangle: Rectangle::from_p2d_aabb(bounds),
            pixel_width,
            pixel_height,
            memory_format,
        }
    }
}

impl Drawable for Image {
    /// Draw itself on a [RenderContext].
    ///
    /// `image_scale` has no meaning here, because the pixmap is already provided.
    fn draw(&self, cx: &mut RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        let state = cx.save_current_state();
        let bounds = self.rectangle.cuboid.local_aabb();

        cx.set_transform(self.rectangle.affine.to_kurbo());
        cx.set_paint(vello_cpu::Image {
            image: vello_cpu::ImageSource::Pixmap(Arc::new(self.to_vello_pixmap())),
            sampler: peniko::ImageSampler::default(),
        });
        cx.fill_rect(&bounds.to_kurbo_rect());

        cx.restore_state(state);
        Ok(())
    }
}

impl Transformable for Image {
    fn translate(&mut self, offset: Vector2) {
        self.rectangle.translate(offset)
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        self.rectangle.rotate(angle, center)
    }

    fn scale(&mut self, scale: Vector2) {
        self.rectangle.scale(scale)
    }
}

impl Image {
    pub fn assert_valid(&self) -> anyhow::Result<()> {
        self.rectangle.bounds().assert_valid()?;
        if self.pixel_width == 0
            || self.pixel_height == 0
            || self.data.len() as u32 != 4 * self.pixel_width * self.pixel_height
        {
            return Err(anyhow::anyhow!(
                "Asserting image validity failed, invalid size or data."
            ));
        }
        Ok(())
    }

    pub fn try_from_encoded_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let reader = ImageReader::new(io::Cursor::new(bytes)).with_guessed_format()?;
        Ok(Image::from(reader.decode()?))
    }

    pub fn try_from_cairo_surface_argb(
        mut surface: cairo::ImageSurface,
        bounds: Aabb,
    ) -> anyhow::Result<Self> {
        let width = surface.width() as u32;
        let height = surface.height() as u32;
        let data = surface.data()?.to_vec();

        Ok(Image {
            data: glib::Bytes::from_owned(convert_pixmap_argb_to_rgba(data)),
            rectangle: Rectangle::from_p2d_aabb(bounds),
            pixel_width: width,
            pixel_height: height,
            // cairo renders to argb-premultiplied, but we convert it to rgba-premultiplied
            memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
        })
    }

    pub fn from_vello_pixmap(value: vello_cpu::Pixmap, bounds: Aabb) -> Self {
        let width = value.width();
        let height = value.height();
        let bytes: Vec<u8> = value
            .take()
            .into_iter()
            .flat_map(|p| [p.r, p.g, p.b, p.a])
            .collect();
        let rectangle = Rectangle::from_p2d_aabb(bounds);
        Self {
            data: glib::Bytes::from_owned(bytes),
            rectangle,
            pixel_width: width as u32,
            pixel_height: height as u32,
            memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
        }
    }

    pub fn to_vello_pixmap(&self) -> vello_cpu::Pixmap {
        let width = self.pixel_width as u16;
        let height = self.pixel_height as u16;
        let data = self
            .data
            .chunks_exact(4)
            .map(|d| peniko::color::PremulRgba8 {
                r: d[0],
                g: d[1],
                b: d[2],
                a: d[3],
            })
            .collect();
        vello_cpu::Pixmap::from_parts(data, width, height)
    }

    pub fn into_imgbuf(
        self,
    ) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, anyhow::Error> {
        self.assert_valid()?;

        match self.memory_format {
            ImageMemoryFormat::R8g8b8a8Premultiplied => image::RgbaImage::from_vec(
                self.pixel_width,
                self.pixel_height,
                self.data.to_vec(),
            )
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Creating RgbaImage from data failed for image with memory-format {:?}.",
                    self.memory_format
                )
            }),
        }
    }

    /// Encodes the image into the provided format.
    ///
    /// When the format is `Jpeg`, the quality should be provided, but falls back to 93 if it is None.
    pub fn into_encoded_bytes(
        self,
        format: image::ImageFormat,
        quality: Option<u8>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        const QUALITY_FALLBACK: u8 = 93;

        self.assert_valid()?;
        let mut bytes_buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let dynamic_image = image::DynamicImage::ImageRgba8(
            self.into_imgbuf()
                .context("Converting image to image::ImageBuffer failed.")?,
        );
        match format {
            image::ImageFormat::Jpeg => {
                image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut bytes_buf,
                    quality.map(|q| q.clamp(0, 100)).unwrap_or(QUALITY_FALLBACK),
                )
                .encode_image(&dynamic_image)
                .context("Encode dynamic image to jpeg failed.")?;
            }
            format => {
                dynamic_image
                    .write_to(&mut bytes_buf, format)
                    .context("Encode dynamic image to format '{format}' failed.")?;
            }
        }

        Ok(bytes_buf.into_inner())
    }

    #[cfg(feature = "ui")]
    pub fn to_memtexture(&self) -> Result<gtk4::gdk::MemoryTexture, anyhow::Error> {
        self.assert_valid()?;

        Ok(gtk4::gdk::MemoryTexture::new(
            self.pixel_width as i32,
            self.pixel_height as i32,
            self.memory_format.into(),
            &self.data,
            (self.pixel_width * 4) as usize,
        ))
    }

    #[cfg(feature = "ui")]
    pub fn to_rendernode(&self) -> Result<gtk4::gsk::RenderNode, anyhow::Error> {
        use crate::ext::GrapheneRectExt;
        use gtk4::{graphene, gsk, prelude::*};

        self.assert_valid()?;

        let memtexture = self.to_memtexture()?;
        let texture_node = gsk::TextureNode::new(
            &memtexture,
            &graphene::Rect::from_p2d_aabb(self.rectangle.cuboid.local_aabb()),
        )
        .upcast();
        let transform_node = gsk::TransformNode::new(
            &texture_node,
            Some(&crate::utils::affine_to_gsk(&self.rectangle.affine)),
        )
        .upcast();

        Ok(transform_node)
    }

    #[cfg(feature = "ui")]
    pub fn images_to_rendernodes<'a>(
        images: impl IntoIterator<Item = &'a Self>,
    ) -> Result<Vec<gtk4::gsk::RenderNode>, anyhow::Error> {
        images.into_iter().map(|img| img.to_rendernode()).collect()
    }

    /// Generates an image with a provided closure that draws onto a [vello_cpu::RenderContext].
    pub fn gen_with_vello<F>(
        draw_func: F,
        mut bounds: Aabb,
        image_scale: f64,
    ) -> anyhow::Result<Self>
    where
        F: FnOnce(&mut vello_cpu::RenderContext) -> anyhow::Result<()>,
    {
        bounds.ensure_positive();
        bounds.loosen(1.);
        bounds.assert_valid()?;
        let width = (bounds.extents().x * image_scale).ceil() as u16;
        let height = (bounds.extents().y * image_scale).ceil() as u16;
        let mut vello_target = vello_cpu::Pixmap::new(width, height);
        let mut vello_resources = vello_cpu::Resources::new();
        let mut vello_cx = RenderContext::new(width, height);

        vello_cx.set_transform(
            kurbo::Affine::scale(image_scale)
                * kurbo::Affine::translate(-bounds.mins.to_kurbo_vec())
                * kurbo::Affine::IDENTITY,
        );
        draw_func(&mut vello_cx)?;
        vello_cx.flush();
        vello_cx.render(&mut vello_target, &mut vello_resources);

        Ok(Self::from_vello_pixmap(vello_target, bounds))
    }
}

pub(super) fn convert_pixmap_argb_to_rgba(mut bytes: Vec<u8>) -> Vec<u8> {
    for src in bytes.as_chunks_mut::<4>().0 {
        let (alpha, red, green, blue) = (src[0], src[1], src[2], src[3]);
        src[0] = red;
        src[1] = green;
        src[2] = blue;
        src[3] = alpha;
    }
    bytes
}
