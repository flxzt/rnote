use std::io;
use std::ops::Deref;

use anyhow::Context;
use gtk4::{gdk, gio, glib, gsk, prelude::*, Snapshot};
use image::io::Reader;
use image::GenericImageView;
use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;
use serde::{Deserialize, Serialize};

use crate::utils::base64;
use crate::DrawBehaviour;
use rnote_compose::helpers::{AABBHelpers, Vector2Helpers};

lazy_static! {
    pub static ref USVG_OPTIONS: usvg::Options = {
        let mut usvg_options = usvg::Options::default();
        usvg_options.fontdb.load_system_fonts();

        usvg_options
    };
}

pub const USVG_XML_OPTIONS: usvg::XmlOptions = usvg::XmlOptions {
    id_prefix: None,
    writer_opts: xmlwriter::Options {
        use_single_quote: false,
        indent: xmlwriter::Indent::None,
        attributes_indent: xmlwriter::Indent::None,
    },
};
/// The maximum tile size (unzoomed)
pub const MAX_TILE_SIZE: na::Vector2<f64> = na::vector![1024.0, 1024.0];
// the maximum size for svgs are joined together for rendering
pub const MAX_JOIN_SIZE: f64 = 1024.0;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ImageMemoryFormat {
    R8g8b8a8Premultiplied,
    B8g8r8a8Premultiplied,
}

impl Default for ImageMemoryFormat {
    fn default() -> Self {
        Self::R8g8b8a8Premultiplied
    }
}

impl TryFrom<gdk::MemoryFormat> for ImageMemoryFormat {
    type Error = anyhow::Error;
    fn try_from(gdk_memory_format: gdk::MemoryFormat) -> Result<Self, Self::Error> {
        match gdk_memory_format {
            gdk::MemoryFormat::R8g8b8a8Premultiplied => Ok(Self::R8g8b8a8Premultiplied),
            gdk::MemoryFormat::B8g8r8a8Premultiplied => Ok(Self::B8g8r8a8Premultiplied),
            _ => Err(anyhow::anyhow!(
                "ImageMemoryFormat try_from() failed, unsupported MemoryFormat `{:?}`",
                gdk_memory_format
            )),
        }
    }
}

/// From impl ImageMemoryFormat into gdk::MemoryFormat
impl From<ImageMemoryFormat> for gdk::MemoryFormat {
    fn from(format: ImageMemoryFormat) -> Self {
        match format {
            ImageMemoryFormat::R8g8b8a8Premultiplied => gdk::MemoryFormat::R8g8b8a8Premultiplied,
            ImageMemoryFormat::B8g8r8a8Premultiplied => gdk::MemoryFormat::B8g8r8a8Premultiplied,
        }
    }
}

impl TryFrom<ImageMemoryFormat> for piet::ImageFormat {
    type Error = anyhow::Error;

    fn try_from(format: ImageMemoryFormat) -> Result<Self, Self::Error> {
        match format {
            ImageMemoryFormat::R8g8b8a8Premultiplied => Ok(piet::ImageFormat::RgbaPremul),
            _ => Err(anyhow::anyhow!("unsupported memory format {:?}", format)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A pixel image
pub struct Image {
    #[serde(rename = "data", with = "base64")]
    pub data: Vec<u8>,
    /// bounds in the coordinate space of the sheet
    #[serde(rename = "bounds")]
    pub bounds: AABB,
    /// width of the data
    #[serde(rename = "pixel_width")]
    pub pixel_width: u32,
    /// height of the data
    #[serde(rename = "pixel_height")]
    pub pixel_height: u32,
    /// the memory format
    #[serde(rename = "memory_format")]
    pub memory_format: ImageMemoryFormat,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            data: Default::default(),
            bounds: AABB::new_zero(),
            pixel_width: Default::default(),
            pixel_height: Default::default(),
            memory_format: Default::default(),
        }
    }
}

impl From<image::DynamicImage> for Image {
    fn from(dynamic_image: image::DynamicImage) -> Self {
        let pixel_width = dynamic_image.width();
        let pixel_height = dynamic_image.height();
        let memory_format = ImageMemoryFormat::R8g8b8a8Premultiplied;
        let data = dynamic_image.into_rgba8().to_vec();

        let bounds = AABB::new(
            na::point![0.0, 0.0],
            na::point![f64::from(pixel_width), f64::from(pixel_height)],
        );

        Self {
            data,
            bounds,
            pixel_width,
            pixel_height,
            memory_format,
        }
    }
}

impl Image {
    pub fn try_from_encoded_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let reader = Reader::new(io::Cursor::new(bytes)).with_guessed_format()?;
        Ok(Image::from(reader.decode()?))
    }

    pub fn convert_to_rgba8pre_inplace(&mut self) -> Result<(), anyhow::Error> {
        match self.memory_format {
            ImageMemoryFormat::R8g8b8a8Premultiplied => {
                // Already in the correct format
                return Ok(());
            }
            ImageMemoryFormat::B8g8r8a8Premultiplied => {
                let imgbuf_bgra8 = image::ImageBuffer::<image::Bgra<u8>, Vec<u8>>::from_vec(
                    self.pixel_width,
                    self.pixel_height,
                    self.data.clone(),
                )
                .ok_or(anyhow::anyhow!(
                    "RgbaImage::from_vec() failed in Image to_imgbuf() for image with Format {:?}",
                    self.memory_format
                ))?;

                let dynamic_image = image::DynamicImage::ImageBgra8(imgbuf_bgra8).into_rgba8();

                *self = Self {
                    pixel_width: self.pixel_width,
                    pixel_height: self.pixel_height,
                    data: dynamic_image.into_vec(),
                    bounds: self.bounds,
                    memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
                };
            }
        }

        Ok(())
    }

    pub fn to_imgbuf(self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, anyhow::Error> {
        match self.memory_format {
            ImageMemoryFormat::R8g8b8a8Premultiplied => {
                image::RgbaImage::from_vec(self.pixel_width, self.pixel_height, self.data).ok_or(
                    anyhow::anyhow!(
                    "RgbaImage::from_vec() failed in Image to_imgbuf() for image with Format {:?}",
                    self.memory_format
                ),
                )
            }
            ImageMemoryFormat::B8g8r8a8Premultiplied => {
                let imgbuf_bgra8 = image::ImageBuffer::<image::Bgra<u8>, Vec<u8>>::from_vec(
                    self.pixel_width,
                    self.pixel_height,
                    self.data,
                )
                .ok_or(anyhow::anyhow!(
                    "RgbaImage::from_vec() failed in Image to_imgbuf() for image with Format {:?}",
                    self.memory_format
                ))?;

                Ok(image::DynamicImage::ImageBgra8(imgbuf_bgra8).into_rgba8())
            }
        }
    }

    pub fn assert_valid(&self) -> Result<(), anyhow::Error> {
        self.bounds.assert_valid()?;

        if self.pixel_width == 0
            || self.pixel_width == 0
            || self.data.len() as u32 != 4 * self.pixel_width * self.pixel_height
        {
            Err(anyhow::anyhow!(
                "assert_image() failed, invalid size or data"
            ))
        } else {
            Ok(())
        }
    }

    pub fn into_encoded_bytes(
        self,
        format: image::ImageOutputFormat,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let mut bytes_buf: Vec<u8> = vec![];

        let dynamic_image = image::DynamicImage::ImageRgba8(
            self.to_imgbuf()
                .context("image.to_imgbuf() failed in image_to_bytes()")?,
        );
        dynamic_image
            .write_to(&mut bytes_buf, format)
            .context("dynamic_image.write_to() failed in image_to_bytes()")?;

        Ok(bytes_buf)
    }

    pub fn to_memtexture(&self) -> Result<gdk::MemoryTexture, anyhow::Error> {
        self.assert_valid()?;

        let bytes = self.data.deref();

        Ok(gdk::MemoryTexture::new(
            self.pixel_width as i32,
            self.pixel_height as i32,
            self.memory_format.into(),
            &glib::Bytes::from(bytes),
            (self.pixel_width * 4) as usize,
        ))
    }

    pub fn to_rendernode(&self) -> Result<gsk::RenderNode, anyhow::Error> {
        self.assert_valid()?;

        let memtexture = self.to_memtexture()?;

        let rendernode =
            gsk::TextureNode::new(&memtexture, &self.bounds.to_graphene_rect()).upcast();
        Ok(rendernode)
    }

    /// images to rendernode. Returns Ok(None) when no images in slice
    pub fn images_to_rendernode(images: &[Self]) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        let snapshot = Snapshot::new();

        for image in images {
            snapshot.append_node(
                &image
                    .to_rendernode()
                    .context("images_to_rendernode failed")?,
            );
        }

        Ok(snapshot.to_node())
    }

    pub fn append_images_to_rendernode(
        images: &[Self],
        rendernode: Option<&gsk::RenderNode>,
    ) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        let snapshot = Snapshot::new();

        if let Some(rendernode) = rendernode {
            snapshot.append_node(rendernode);
        }

        for image in images {
            image.assert_valid()?;

            snapshot.append_node(
                &image
                    .to_rendernode()
                    .context("image_to_rendernode() failed in append_images_to_rendernode()")?,
            );
        }

        Ok(snapshot.to_node())
    }

    pub fn concat_images(
        images: Vec<Self>,
        bounds: AABB,
        image_scale: f64,
    ) -> Result<Image, anyhow::Error> {
        let mut target_image = image::RgbaImage::new(
            (bounds.extents()[0] * image_scale).round() as u32,
            (bounds.extents()[1] * image_scale).round() as u32,
        );

        for image in images.into_iter() {
            let offset = (image.bounds.mins.coords - bounds.mins.coords) * image_scale;

            let mut image_buf = image.to_imgbuf()?;
            image::imageops::overlay(
                &mut target_image,
                &mut image_buf,
                offset[0].round() as u32,
                offset[1].round() as u32,
            );
        }

        let pixel_width = target_image.width();
        let pixel_height = target_image.height();

        Ok(Image {
            data: target_image.into_vec(),
            pixel_width,
            pixel_height,
            bounds,
            memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
        })
    }

    // Public method
    pub fn gen_images(
        svgs: Vec<Svg>,
        bounds: AABB,
        image_scale: f64,
    ) -> Result<Vec<Self>, anyhow::Error> {
        Self::gen_images_librsvg(svgs, bounds, image_scale)
    }

    // With librsvg
    fn gen_images_librsvg(
        mut svgs: Vec<Svg>,
        mut bounds: AABB,
        image_scale: f64,
    ) -> Result<Vec<Self>, anyhow::Error> {
        bounds.ensure_positive();
        bounds.assert_valid()?;

        // joining svgs for sizes that are not worth
        if bounds.extents()[0] < MAX_JOIN_SIZE && bounds.extents()[1] < MAX_JOIN_SIZE {
            let svg_data = svgs
                .into_iter()
                .map(|svg| svg.svg_data)
                .collect::<Vec<String>>()
                .join("\n");

            svgs = vec![Svg { svg_data, bounds }];
        }

        let mut images = vec![];

        for svg in svgs {
            let svg_data = rnote_compose::utils::wrap_svg_root(
                svg.svg_data.as_str(),
                Some(bounds),
                Some(bounds),
                false,
            );

            for mut splitted_bounds in svg.bounds.split(MAX_TILE_SIZE / image_scale) {
                splitted_bounds.ensure_positive();
                if splitted_bounds.assert_valid().is_err() {
                    continue;
                }
                splitted_bounds.loosen(1.0);

                let splitted_width_scaled =
                    ((splitted_bounds.extents()[0]) * image_scale).round() as u32;
                let splitted_height_scaled =
                    ((splitted_bounds.extents()[1]) * image_scale).round() as u32;

                let mut surface = cairo::ImageSurface::create(
                    cairo::Format::ARgb32,
                    splitted_width_scaled as i32,
                    splitted_height_scaled as i32,
                )
                .map_err(|e| {
                    anyhow::anyhow!(
                        "create ImageSurface with dimensions ({}, {}) failed, {}",
                        splitted_width_scaled,
                        splitted_height_scaled,
                        e
                    )
                })?;

                // Context in new scope, else accessing the surface data fails with a borrow error
                {
                    let cx = cairo::Context::new(&surface).context("new cairo::Context failed")?;
                    cx.scale(image_scale, image_scale);
                    cx.translate(-splitted_bounds.mins[0], -splitted_bounds.mins[1]);

                    /*                 // Debugging bounds
                    cx.set_line_width(1.0);
                    cx.set_source_rgba(1.0, 0.0, 0.0, 1.0);
                    cx.rectangle(splitted_bounds.mins[0], splitted_bounds.mins[1], splitted_bounds.extents()[0], splitted_bounds.extents()[1]);
                    cx.stroke()?;
                    cx.set_source_rgba(0.0, 0.0, 0.0, 0.0); */

                    let stream =
                        gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg_data.as_bytes()));

                    let handle = librsvg::Loader::new()
                        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(
                            &stream, None, None,
                        )
                        .context("read stream to librsvg Loader failed")?;
                    let renderer = librsvg::CairoRenderer::new(&handle);
                    renderer
                        .render_document(
                            &cx,
                            &cairo::Rectangle {
                                x: bounds.mins[0],
                                y: bounds.mins[1],
                                width: bounds.extents()[0],
                                height: bounds.extents()[1],
                            },
                        )
                        .map_err(|e| {
                            anyhow::Error::msg(format!(
                            "librsvg render_document() failed in gen_image_librsvg() with Err {}",
                            e
                        ))
                        })?;
                }
                // Surface needs to be flushed before accessing its data
                surface.flush();

                let data = surface
                    .data()
                    .map_err(|e| {
                        anyhow::Error::msg(format!(
                            "accessing imagesurface data failed in gen_image_librsvg() with Err {}",
                            e
                        ))
                    })?
                    .to_vec();

                images.push(Image {
                    data,
                    bounds: splitted_bounds,
                    pixel_width: splitted_width_scaled,
                    pixel_height: splitted_height_scaled,
                    memory_format: ImageMemoryFormat::B8g8r8a8Premultiplied,
                })
            }
        }

        Ok(images)
    }

    // With resvg
    #[allow(unused)]
    fn gen_images_resvg(
        mut svgs: Vec<Svg>,
        mut bounds: AABB,
        image_scale: f64,
    ) -> Result<Vec<Self>, anyhow::Error> {
        bounds.ensure_positive();
        bounds.assert_valid()?;

        // joining svgs for sizes that are not worth
        if bounds.extents()[0] < MAX_JOIN_SIZE && bounds.extents()[0] < MAX_JOIN_SIZE {
            let svg_data = svgs
                .into_iter()
                .map(|svg| svg.svg_data)
                .collect::<Vec<String>>()
                .join("\n");

            svgs = vec![Svg { svg_data, bounds }];
        }

        let mut images = vec![];

        for svg in svgs {
            let svg_data = rnote_compose::utils::wrap_svg_root(
                svg.svg_data.as_str(),
                Some(bounds),
                Some(bounds),
                false,
            );
            let svg_tree = usvg::Tree::from_data(svg_data.as_bytes(), &USVG_OPTIONS.to_ref())?;

            for mut splitted_bounds in bounds.split(MAX_TILE_SIZE / image_scale) {
                splitted_bounds.ensure_positive();
                if splitted_bounds.assert_valid().is_err() {
                    continue;
                }
                splitted_bounds.loosen(1.0);

                let splitted_width_scaled =
                    ((splitted_bounds.extents()[0]) * image_scale).round() as u32;
                let splitted_height_scaled =
                    ((splitted_bounds.extents()[1]) * image_scale).round() as u32;
                let offset = splitted_bounds.mins.coords - bounds.mins.coords;

                let mut pixmap =
                    tiny_skia::Pixmap::new(splitted_width_scaled, splitted_height_scaled)
                        .ok_or_else(|| {
                            anyhow::Error::msg(
                                "tiny_skia::Pixmap::new() failed in gen_image_resvg()",
                            )
                        })?;

                resvg::render(
                    &svg_tree,
                    usvg::FitTo::Original,
                    tiny_skia::Transform::from_translate(-offset[0] as f32, -offset[1] as f32)
                        .post_scale(image_scale as f32, image_scale as f32),
                    pixmap.as_mut(),
                )
                .ok_or_else(|| anyhow::Error::msg("resvg::render failed in gen_image_resvg."))?;

                let data = pixmap.data().to_vec();

                images.push(Image {
                    data,
                    bounds: splitted_bounds,
                    pixel_width: splitted_width_scaled,
                    pixel_height: splitted_height_scaled,
                    memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
                });
            }
        }
        Ok(images)
    }

    pub fn gen_image_from_piet(
        to_be_drawn: &impl DrawBehaviour,
        bounds: AABB,
        image_scale: f64,
    ) -> Result<Self, anyhow::Error> {
        let width = (bounds.extents()[0] * image_scale).ceil() as u32;
        let height = (bounds.extents()[1] * image_scale).ceil() as u32;
        let mut image_surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32)?;

        {
            let cairo_cx = cairo::Context::new(&image_surface)?;
            let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

            piet_cx.transform(kurbo::Affine::scale(image_scale));
            piet_cx.transform(kurbo::Affine::translate(-bounds.mins.coords.to_kurbo_vec()));

            to_be_drawn.draw(&mut piet_cx, image_scale)?;

            piet_cx.finish().map_err(|e| {
                anyhow::anyhow!("piet_cx.finish() failed in gen_images() with Err {}", e)
            })?;
        }
        // Surface needs to be flushed before accessing its data
        image_surface.flush();

        let data = image_surface
            .data()
            .map_err(|e| {
                anyhow::Error::msg(format!(
                "accessing imagesurface data failed in strokebehaviour gen_images() with Err {}",
                e
            ))
            })?
            .to_vec();

        Ok(Image {
            data,
            bounds,
            pixel_width: width,
            pixel_height: height,
            memory_format: ImageMemoryFormat::B8g8r8a8Premultiplied,
        })
    }
}

#[derive(Debug, Clone)]
/// A svg image
pub struct Svg {
    /// the svg data as String
    pub svg_data: String,
    /// the bounds of the svg
    pub bounds: AABB,
}

impl Svg {
    pub fn concat(svgs: Vec<Self>) -> Option<Self> {
        if svgs.is_empty() {
            return None;
        }

        Some(svgs.into_iter().fold(
            Self {
                svg_data: String::from(""),
                bounds: AABB::new_invalid(),
            },
            |acc, x| Svg {
                svg_data: acc.svg_data + "\n" + x.svg_data.as_str(),
                bounds: acc.bounds.merged(&x.bounds),
            },
        ))
    }

    pub fn draw_svgs_to_cairo_context(
        svgs: &[Self],
        mut bounds: AABB,
        cx: &cairo::Context,
    ) -> Result<(), anyhow::Error> {
        bounds.ensure_positive();
        bounds.assert_valid()?;

        for svg in svgs {
            let svg_data = rnote_compose::utils::wrap_svg_root(
                svg.svg_data.as_str(),
                Some(svg.bounds),
                Some(svg.bounds),
                false,
            );

            let stream =
                gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg_data.as_bytes()));

            let handle = librsvg::Loader::new()
                .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(
                    &stream, None, None,
                )
                .context("read stream to librsvg Loader failed")?;
            let renderer = librsvg::CairoRenderer::new(&handle);
            renderer
                .render_document(
                    &cx,
                    &cairo::Rectangle {
                        x: svg.bounds.mins[0],
                        y: svg.bounds.mins[1],
                        width: svg.bounds.extents()[0],
                        height: svg.bounds.extents()[1],
                    },
                )
                .map_err(|e| {
                    anyhow::Error::msg(format!(
                    "librsvg render_document() failed in draw_svgs_to_cairo_context() with Err {}",
                    e
                ))
                })?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn render_svg_to_caironode(&self) -> Result<gsk::CairoNode, anyhow::Error> {
        if self.bounds.extents()[0] < 0.0 || self.bounds.extents()[1] < 0.0 {
            return Err(anyhow::anyhow!(
                "gen_rendernode_librsvg() failed, bounds width/ height is < 0.0"
            ));
        }

        let new_caironode = gsk::CairoNode::new(&self.bounds.to_graphene_rect());
        let cx = new_caironode.draw_context();

        Svg::draw_svgs_to_cairo_context(&[self.to_owned()], self.bounds, &cx)?;

        Ok(new_caironode)
    }
}
