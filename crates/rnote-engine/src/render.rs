// Imports
use crate::Drawable;
use anyhow::Context;
use core::fmt::Debug;
use image::io::Reader;
use once_cell::sync::Lazy;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::ext::AabbExt;
use rnote_compose::shapes::{Rectangle, Shapeable};
use rnote_compose::transform::Transformable;
use serde::{Deserialize, Serialize};
use std::io::{self, Cursor};
use std::sync::Arc;
use svg::Node;

/// Usvg font database
pub static USVG_FONTDB: Lazy<Arc<usvg::fontdb::Database>> = Lazy::new(|| {
    let mut db = usvg::fontdb::Database::new();
    db.load_system_fonts();
    Arc::new(db)
});

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

impl From<ImageMemoryFormat> for piet::ImageFormat {
    fn from(value: ImageMemoryFormat) -> Self {
        match value {
            ImageMemoryFormat::R8g8b8a8Premultiplied => piet::ImageFormat::RgbaPremul,
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
    pub rect: Rectangle,
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
            .field("data", &String::from("- no debug impl -"))
            .field("rect", &self.rect)
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
            rect: Rectangle::default(),
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
            na::point![0.0, 0.0],
            na::point![f64::from(pixel_width), f64::from(pixel_height)],
        );

        Self {
            data,
            rect: Rectangle::from_p2d_aabb(bounds),
            pixel_width,
            pixel_height,
            memory_format,
        }
    }
}

impl Drawable for Image {
    /// Draw itself on a [piet::RenderContext].
    ///
    /// Expects image to be in rgba8-premultiplied format, else drawing will fail.
    ///
    /// `image_scale` has no meaning here, because the bitamp is already provided.
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        let piet_image_format = piet::ImageFormat::from(self.memory_format);

        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let piet_image = cx
            .make_image(
                self.pixel_width as usize,
                self.pixel_height as usize,
                &self.data,
                piet_image_format,
            )
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        cx.transform(self.rect.transform.to_kurbo());
        cx.draw_image(
            &piet_image,
            self.rect.cuboid.local_aabb().to_kurbo_rect(),
            piet::InterpolationMode::Bilinear,
        );
        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl Transformable for Image {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.rect.translate(offset)
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.rect.rotate(angle, center)
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.rect.scale(scale)
    }
}

impl Image {
    pub fn assert_valid(&self) -> anyhow::Result<()> {
        self.rect.bounds().assert_valid()?;

        if self.pixel_width == 0
            || self.pixel_height == 0
            || self.data.len() as u32 != 4 * self.pixel_width * self.pixel_height
        {
            Err(anyhow::anyhow!(
                "Asserting image validity failed, invalid size or data."
            ))
        } else {
            Ok(())
        }
    }

    pub fn try_from_encoded_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let reader = Reader::new(io::Cursor::new(bytes)).with_guessed_format()?;
        Ok(Image::from(reader.decode()?))
    }

    pub fn try_from_cairo_surface(
        mut surface: cairo::ImageSurface,
        bounds: Aabb,
    ) -> anyhow::Result<Self> {
        let width = surface.width() as u32;
        let height = surface.height() as u32;
        let data = surface.data()?.to_vec();

        Ok(Image {
            data: glib::Bytes::from_owned(convert_image_bgra_to_rgba(width, height, data)),
            rect: Rectangle::from_p2d_aabb(bounds),
            pixel_width: width,
            pixel_height: height,
            // cairo renders to bgra8-premultiplied, but we convert it to rgba8-premultiplied
            memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
        })
    }

    pub fn into_imgbuf(
        self,
    ) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, anyhow::Error> {
        self.assert_valid()?;

        match self.memory_format {
            ImageMemoryFormat::R8g8b8a8Premultiplied => {
                image::RgbaImage::from_vec(self.pixel_width, self.pixel_height, self.data.to_vec())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                    "Creating RgbaImage from data failed for image with memory-format {:?}.",
                    self.memory_format
                )
                    })
            }
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
            &graphene::Rect::from_p2d_aabb(self.rect.cuboid.local_aabb()),
        )
        .upcast();
        let transform_node = gsk::TransformNode::new(
            &texture_node,
            &crate::utils::transform_to_gsk(&self.rect.transform),
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

    /// Generates an image with a provided closure that draws onto a [cairo::Context].
    pub fn gen_with_cairo<F>(
        draw_func: F,
        mut bounds: Aabb,
        image_scale: f64,
    ) -> anyhow::Result<Self>
    where
        F: FnOnce(&cairo::Context) -> anyhow::Result<()>,
    {
        bounds.ensure_positive();
        bounds.loosen(1.0);
        bounds.assert_valid()?;

        let width_scaled = ((bounds.extents()[0]) * image_scale).round() as u32;
        let height_scaled = ((bounds.extents()[1]) * image_scale).round() as u32;

        let mut image_surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            width_scaled as i32,
            height_scaled as i32,
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "creating image surface with dimensions ({}, {}) failed, Err: {e:?}",
                width_scaled,
                height_scaled,
            )
        })?;

        {
            let cairo_cx = cairo::Context::new(&image_surface)?;
            cairo_cx.scale(image_scale, image_scale);
            cairo_cx.translate(-bounds.mins[0], -bounds.mins[1]);
            // Apply the draw function
            draw_func(&cairo_cx)?;
        }

        let data = image_surface
            .data()
            .map_err(|e| anyhow::anyhow!("accessing image surface data failed, Err: {e:?}"))?
            .to_vec();

        Ok(Image {
            data: glib::Bytes::from_owned(convert_image_bgra_to_rgba(
                width_scaled,
                height_scaled,
                data,
            )),
            rect: Rectangle::from_p2d_aabb(bounds),
            pixel_width: width_scaled,
            pixel_height: height_scaled,
            // cairo renders to bgra8-premultiplied, but we convert it to rgba8-premultiplied
            memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
        })
    }

    /// Generates an image with a provided closure that draws onto a [piet_cairo::CairoRenderContext].
    pub fn gen_with_piet<F>(draw_func: F, bounds: Aabb, image_scale: f64) -> anyhow::Result<Self>
    where
        F: FnOnce(&mut piet_cairo::CairoRenderContext) -> anyhow::Result<()>,
    {
        let cairo_draw_fn = move |cairo_cx: &cairo::Context| -> anyhow::Result<()> {
            let mut piet_cx = piet_cairo::CairoRenderContext::new(cairo_cx);
            // Apply the draw function
            draw_func(&mut piet_cx)?;
            piet_cx
                .finish()
                .map_err(|e| anyhow::anyhow!("finishing piet context failed, Err: {e:?}"))?;
            Ok(())
        };

        Self::gen_with_cairo(cairo_draw_fn, bounds, image_scale)
    }
}

/// A Svg image.
#[derive(Debug, Clone)]
pub struct Svg {
    /// Svg data String.
    pub svg_data: String,
    /// Bounds of the Svg.
    pub bounds: Aabb,
}

impl Svg {
    pub const MIME_TYPE: &'static str = "image/svg+xml";

    pub fn merge<T>(&mut self, other: T)
    where
        T: IntoIterator<Item = Self>,
    {
        for svg in other {
            self.svg_data += format!("\n{}", svg.svg_data).as_str();
            self.bounds.merge(&svg.bounds);
        }
    }

    pub fn wrap_svg_root(
        &mut self,
        bounds: Option<Aabb>,
        viewbox: Option<Aabb>,
        preserve_aspectratio: bool,
    ) {
        self.svg_data = rnote_compose::utils::wrap_svg_root(
            self.svg_data.as_str(),
            bounds,
            viewbox,
            preserve_aspectratio,
        );
        if let Some(bounds) = bounds {
            self.bounds = bounds
        }
    }

    pub fn add_xml_header(&mut self) {
        self.svg_data = rnote_compose::utils::add_xml_header(&self.svg_data);
    }

    pub fn remove_xml_header(&mut self) {
        self.svg_data = rnote_compose::utils::remove_xml_header(&self.svg_data);
    }

    /// Simplify the Svg by passing it through [usvg].
    ///
    /// Also moves the bounds to mins: [0., 0.], maxs: extents
    pub fn simplify(&mut self) -> anyhow::Result<()> {
        const COORDINATES_PREC: u8 = 3;
        const TRANSFORMS_PREC: u8 = 4;

        let xml_options = usvg::WriteOptions {
            id_prefix: Some(rnote_compose::utils::svg_random_id_prefix()),
            preserve_text: true,
            coordinates_precision: COORDINATES_PREC,
            transforms_precision: TRANSFORMS_PREC,
            use_single_quote: false,
            indent: xmlwriter::Indent::None,
            attributes_indent: xmlwriter::Indent::None,
        };
        let bounds_simplified = Aabb::new(na::point![0.0, 0.0], self.bounds.extents().into());
        let svg_data_wrapped = rnote_compose::utils::wrap_svg_root(
            &rnote_compose::utils::remove_xml_header(&self.svg_data),
            Some(bounds_simplified),
            Some(self.bounds),
            false,
        );

        let usvg_tree = usvg::Tree::from_str(
            &svg_data_wrapped,
            &usvg::Options {
                fontdb: Arc::clone(&USVG_FONTDB),
                ..Default::default()
            },
        )?;

        self.svg_data = usvg_tree.to_string(&xml_options);
        self.bounds = bounds_simplified;

        Ok(())
    }

    /// Generate an Svg through cairo's SvgSurface.
    pub fn gen_with_cairo<F>(draw_func: F, mut bounds: Aabb) -> anyhow::Result<Self>
    where
        F: FnOnce(&cairo::Context) -> anyhow::Result<()>,
    {
        bounds.ensure_positive();
        bounds.assert_valid()?;

        let width = bounds.extents()[0];
        let height = bounds.extents()[1];
        let mut svg_surface =
            cairo::SvgSurface::for_stream(width, height, Vec::new()).map_err(|e| {
                anyhow::anyhow!(
                    "Creating svg surface with dimensions ({width}, {height}) failed, Err: {e:?}"
                )
            })?;
        svg_surface.set_document_unit(cairo::SvgUnit::Px);

        {
            let cairo_cx = cairo::Context::new(&svg_surface)?;
            // cairo only draws elements with positive coordinates, so we need to translate the content here
            cairo_cx.translate(-bounds.mins[0], -bounds.mins[1]);
            // apply the draw function
            draw_func(&cairo_cx)?;
        }

        let content = String::from_utf8(
            *svg_surface
                .finish_output_stream()
                .map_err(|e| {
                    anyhow::anyhow!("Finishing Svg surface output stream failed, Err: {e:?}")
                })?
                .downcast::<Vec<u8>>()
                .map_err(|e| {
                    anyhow::anyhow!("Downcasting Svg surface content failed, Err: {e:?}")
                })?,
        )?;
        let svg_data = rnote_compose::utils::remove_xml_header(&content);
        let mut group = svg::node::element::Group::new().add(svg::node::Blob::new(svg_data));
        // translate the content back to it's original position
        group.assign(
            "transform",
            format!("translate({} {})", bounds.mins[0], bounds.mins[1]),
        );

        Ok(Self {
            svg_data: rnote_compose::utils::svg_node_to_string(&group)?,
            bounds,
        })
    }

    /// Generate an Svg with piet, using the `piet_cairo` backend and cairo's SvgSurface.
    ///
    /// This might be preferable to the `piet_svg` backend, because especially text alignment and sizes can be different
    /// with it.
    pub fn gen_with_piet_cairo_backend<F>(draw_func: F, bounds: Aabb) -> anyhow::Result<Self>
    where
        F: FnOnce(&mut piet_cairo::CairoRenderContext) -> anyhow::Result<()>,
    {
        let cairo_draw_fn = |cairo_cx: &cairo::Context| {
            let mut piet_cx = piet_cairo::CairoRenderContext::new(cairo_cx);
            // Apply the draw function
            draw_func(&mut piet_cx)?;
            piet_cx
                .finish()
                .map_err(|e| anyhow::anyhow!("finishing piet context failed, Err: {e:?}"))
        };

        Self::gen_with_cairo(cairo_draw_fn, bounds)
    }

    pub fn draw_to_cairo(&self, cx: &cairo::Context) -> anyhow::Result<()> {
        let svg_data = rnote_compose::utils::wrap_svg_root(
            self.svg_data.as_str(),
            Some(self.bounds),
            Some(self.bounds),
            false,
        );
        let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg_data.as_bytes()));
        let handle = rsvg::Loader::new()
            .with_unlimited_size(true)
            .read_stream(&stream, None::<&gio::File>, None::<&gio::Cancellable>)
            .context("reading stream to rsvg loader failed.")?;
        let renderer = rsvg::CairoRenderer::new(&handle);
        renderer
            .render_document(
                cx,
                &cairo::Rectangle::new(
                    self.bounds.mins[0],
                    self.bounds.mins[1],
                    self.bounds.extents()[0],
                    self.bounds.extents()[1],
                ),
            )
            .map_err(|e| anyhow::anyhow!("rendering rsvg document failed, Err: {e:?}"))?;
        Ok(())
    }

    /// Generate an image from an Svg.
    ///
    /// Using rsvg for rendering.
    pub fn gen_image(&self, image_scale: f64) -> Result<Image, anyhow::Error> {
        let mut bounds = self.bounds;
        bounds.ensure_positive();
        bounds.assert_valid()?;

        let svg_data = rnote_compose::utils::wrap_svg_root(
            self.svg_data.as_str(),
            Some(bounds),
            Some(bounds),
            false,
        );
        let width_scaled = ((bounds.extents()[0]) * image_scale).round() as u32;
        let height_scaled = ((bounds.extents()[1]) * image_scale).round() as u32;

        let mut surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32,
                width_scaled as i32,
                height_scaled as i32,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "creating ImageSurface with dimensions ({width_scaled}, {height_scaled}) failed, Err: {e:?}"
                )
            })?;

        // Context in new scope, else accessing the surface data fails with a borrow error
        {
            let cx =
                cairo::Context::new(&surface).context("creating new cairo::Context failed.")?;
            cx.scale(image_scale, image_scale);
            cx.translate(-bounds.mins[0], -bounds.mins[1]);

            let stream =
                gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg_data.as_bytes()));

            let handle = rsvg::Loader::new()
                .with_unlimited_size(true)
                .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(
                    &stream, None, None,
                )
                .context("read stream to rsvg loader failed.")?;

            let renderer = rsvg::CairoRenderer::new(&handle);
            renderer
                .render_document(
                    &cx,
                    &cairo::Rectangle::new(
                        bounds.mins[0],
                        bounds.mins[1],
                        bounds.extents()[0],
                        bounds.extents()[1],
                    ),
                )
                .map_err(|e| anyhow::anyhow!("rendering rsvg document failed, Err: {e:?}"))?;
        }

        let data = surface
            .data()
            .map_err(|e| anyhow::anyhow!("accessing imagesurface data failed, Err: {e:?}"))?
            .to_vec();

        Ok(Image {
            data: glib::Bytes::from_owned(convert_image_bgra_to_rgba(
                width_scaled,
                height_scaled,
                data,
            )),
            rect: Rectangle::from_p2d_aabb(bounds),
            pixel_width: width_scaled,
            pixel_height: height_scaled,
            // cairo renders to bgra8-premultiplied, but we convert it to rgba8-premultiplied
            memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
        })
    }
}

fn convert_image_bgra_to_rgba(_width: u32, _height: u32, mut bytes: Vec<u8>) -> Vec<u8> {
    for src in bytes.chunks_exact_mut(4) {
        let (blue, green, red, alpha) = (src[0], src[1], src[2], src[3]);
        src[0] = red;
        src[1] = green;
        src[2] = blue;
        src[3] = alpha;
    }
    bytes
}
