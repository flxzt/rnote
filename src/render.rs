use std::ops::Deref;

use anyhow::Context;
use gtk4::{gdk, gio, glib, gsk, prelude::*, Native, Snapshot, Widget};
use p2d::bounding_volume::AABB;

use crate::compose::{self, geometry};

#[derive(Debug, Clone, Copy, glib::Enum)]
#[repr(u32)]
#[enum_type(name = "RendererBackend")]
pub enum RendererBackend {
    #[enum_value(name = "Librsvg", nick = "librsvg")]
    Librsvg,
    #[enum_value(name = "Resvg", nick = "resvg")]
    Resvg,
}

#[derive(Debug, Copy, Clone)]
pub enum ImageMemoryFormat {
    R8g8b8a8Premultiplied,
    B8g8r8a8Premultiplied,
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

#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    /// bounds in the coordinate space of the sheet
    pub bounds: AABB,
    /// width of the data
    pub pixel_width: u32,
    /// height of the data
    pub pixel_height: u32,
    /// the memory format
    pub memory_format: ImageMemoryFormat,
}

impl Image {
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
}

#[derive(Debug, Clone)]
pub struct Svg {
    pub svg_data: String,
    pub bounds: AABB,
}

#[derive(Debug, Clone)]
pub struct Renderer {
    pub backend: RendererBackend,
    pub usvg_options: usvg::Options,
    pub usvg_xml_options: usvg::XmlOptions,
}

impl Default for Renderer {
    fn default() -> Self {
        let mut usvg_options = usvg::Options::default();
        usvg_options.fontdb.load_system_fonts();

        let usvg_xml_options = usvg::XmlOptions {
            id_prefix: None,
            writer_opts: xmlwriter::Options {
                use_single_quote: false,
                indent: xmlwriter::Indent::None,
                attributes_indent: xmlwriter::Indent::None,
            },
        };

        Self {
            backend: RendererBackend::Librsvg,
            usvg_options,
            usvg_xml_options,
        }
    }
}

impl Renderer {
    /// generates images from SVGs. bounds are in coordinate space of the sheet, (not zoomed)
    /// expects the svgs to be raw svg tags, no svg root or xml header needed
    pub fn gen_image(
        &self,
        zoom: f64,
        svgs: &[Svg],
        bounds: AABB,
    ) -> Result<Option<Image>, anyhow::Error> {
        if svgs.is_empty() {
            return Ok(None);
        }

        assert_bounds(bounds)?;

        match self.backend {
            RendererBackend::Librsvg => self.gen_image_librsvg(zoom, svgs, bounds),
            RendererBackend::Resvg => self.gen_image_resvg(zoom, svgs, bounds),
        }
    }

    fn gen_image_librsvg(
        &self,
        zoom: f64,
        svgs: &[Svg],
        bounds: AABB,
    ) -> Result<Option<Image>, anyhow::Error> {
        let width_scaled = ((bounds.extents()[0]) * zoom).round() as u32;
        let height_scaled = ((bounds.extents()[1]) * zoom).round() as u32;

        let mut surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            width_scaled as i32,
            height_scaled as i32,
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "create ImageSurface with dimensions ({}, {}) failed, {}",
                width_scaled,
                height_scaled,
                e
            )
        })?;

        let mut svg_data = svgs
            .iter()
            .map(|svg| svg.svg_data.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        svg_data = compose::wrap_svg_root(svg_data.as_str(), Some(bounds), Some(bounds), true);

        // Context in new scope, else accessing the surface data fails with a borrow error
        {
            let cx = cairo::Context::new(&surface).context("new cairo::Context failed")?;

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
                        x: 0.0,
                        y: 0.0,
                        width: f64::from(width_scaled),
                        height: f64::from(height_scaled),
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

        Ok(Some(Image {
            data,
            bounds,
            pixel_width: width_scaled,
            pixel_height: height_scaled,
            memory_format: ImageMemoryFormat::B8g8r8a8Premultiplied,
        }))
    }

    fn gen_image_resvg(
        &self,
        zoom: f64,
        svgs: &[Svg],
        bounds: AABB,
    ) -> Result<Option<Image>, anyhow::Error> {
        let bounds = geometry::aabb_ceil(bounds);
        let width_scaled = ((bounds.extents()[0]) * zoom).round() as u32;
        let height_scaled = ((bounds.extents()[1]) * zoom).round() as u32;

        if width_scaled == 0 || height_scaled == 0 {
            return Ok(None);
        }

        let mut svg_data = svgs
            .iter()
            .map(|svg| svg.svg_data.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        svg_data = compose::wrap_svg_root(svg_data.as_str(), Some(bounds), Some(bounds), true);

        let mut pixmap = tiny_skia::Pixmap::new(width_scaled, height_scaled).ok_or_else(|| {
            anyhow::Error::msg("tiny_skia::Pixmap::new() failed in gen_image_resvg()")
        })?;

        let rtree = usvg::Tree::from_data(svg_data.as_bytes(), &self.usvg_options.to_ref())?;

        resvg::render(&rtree, usvg::FitTo::Zoom(zoom as f32), pixmap.as_mut())
            .ok_or_else(|| anyhow::Error::msg("resvg::render failed in gen_image_resvg."))?;

        let data = pixmap.data().to_vec();

        Ok(Some(Image {
            data,
            bounds,
            pixel_width: width_scaled,
            pixel_height: height_scaled,
            memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
        }))
    }
}

pub fn image_into_bytes(
    image: Image,
    format: image::ImageOutputFormat,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut bytes_buf: Vec<u8> = vec![];

    let dynamic_image = image::DynamicImage::ImageRgba8(
        image
            .to_imgbuf()
            .context("image.to_imgbuf() failed in image_to_bytes()")?,
    );
    dynamic_image
        .write_to(&mut bytes_buf, format)
        .context("dynamic_image.write_to() failed in image_to_bytes()")?;

    Ok(bytes_buf)
}

pub fn image_to_memtexture(image: &Image) -> Result<gdk::MemoryTexture, anyhow::Error> {
    assert_image(image)?;

    let bytes = image.data.deref();

    Ok(gdk::MemoryTexture::new(
        image.pixel_width as i32,
        image.pixel_height as i32,
        image.memory_format.into(),
        &glib::Bytes::from(bytes),
        (image.pixel_width * 4) as usize,
    ))
}

pub fn image_to_rendernode(image: &Image, zoom: f64) -> Result<gsk::RenderNode, anyhow::Error> {
    assert_image(image)?;

    let memtexture = image_to_memtexture(image)?;

    let scaled_bounds = geometry::aabb_scale(image.bounds, zoom);
    assert_bounds(scaled_bounds)?;

    let rendernode =
        gsk::TextureNode::new(&memtexture, &geometry::aabb_to_graphene_rect(scaled_bounds))
            .upcast();
    Ok(rendernode)
}

/// images to rendernode. Returns Ok(None) when no images in slice
pub fn images_to_rendernode(
    images: &[Image],
    zoom: f64,
) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
    let snapshot = Snapshot::new();

    for image in images {
        snapshot
            .append_node(&image_to_rendernode(image, zoom).context("images_to_rendernode failed")?);
    }

    Ok(snapshot.to_node())
}

pub fn append_images_to_rendernode(
    rendernode: Option<&gsk::RenderNode>,
    images: &[Image],
    zoom: f64,
) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
    let snapshot = Snapshot::new();

    if let Some(rendernode) = rendernode {
        snapshot.append_node(rendernode);
    }

    for image in images {
        assert_image(image)?;

        snapshot.append_node(
            &image_to_rendernode(image, zoom)
                .context("image_to_rendernode() failed in append_images_to_rendernode()")?,
        );
    }

    Ok(snapshot.to_node())
}

pub fn rendernode_to_texture(
    active_widget: &Widget,
    node: &gsk::RenderNode,
    viewport: Option<AABB>,
) -> Result<Option<gdk::Texture>, anyhow::Error> {
    if let Some(viewport) = viewport {
        assert_bounds(viewport)?;
    }

    let viewport = viewport.map(geometry::aabb_to_graphene_rect);

    if let Some(root) = active_widget.root() {
        let texture = root
            .upcast::<Native>()
            .renderer()
            .render_texture(node, viewport.as_ref());
        return Ok(Some(texture));
    }

    Ok(None)
}

pub fn draw_svgs_to_cairo_context(
    zoom: f64,
    svgs: &[Svg],
    bounds: AABB,
    cx: &cairo::Context,
) -> Result<(), anyhow::Error> {
    let mut svg_data = svgs
        .iter()
        .map(|svg| svg.svg_data.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    svg_data = compose::wrap_svg_root(svg_data.as_str(), Some(bounds), Some(bounds), true);

    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg_data.as_bytes()));

    let librsvg_handle = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)?;

    let librsvg_renderer = librsvg::CairoRenderer::new(&librsvg_handle);
    librsvg_renderer.render_document(
        cx,
        &cairo::Rectangle {
            x: (bounds.mins[0].floor() * zoom),
            y: (bounds.mins[1].floor() * zoom),
            width: ((bounds.extents()[0]).ceil() * zoom),
            height: ((bounds.extents()[1]).ceil() * zoom),
        },
    )?;

    Ok(())
}

fn gen_caironode_librsvg(zoom: f64, svg: &Svg) -> Result<gsk::CairoNode, anyhow::Error> {
    if svg.bounds.extents()[0] < 0.0 || svg.bounds.extents()[1] < 0.0 {
        return Err(anyhow::anyhow!(
            "gen_rendernode_librsvg() failed, bounds width/ height is < 0.0"
        ));
    }

    let caironode_bounds = geometry::aabb_scale(geometry::aabb_ceil(svg.bounds), zoom);

    let new_caironode = gsk::CairoNode::new(&geometry::aabb_to_graphene_rect(caironode_bounds));
    let cx = new_caironode.draw_context();

    draw_svgs_to_cairo_context(zoom, &[svg.to_owned()], caironode_bounds, &cx)?;

    Ok(new_caironode)
}

pub fn assert_bounds(bounds: AABB) -> Result<(), anyhow::Error> {
    if bounds.extents()[0] < 0.0
        || bounds.extents()[1] < 0.0
        || bounds.maxs[0] < bounds.mins[0]
        || bounds.maxs[1] < bounds.mins[1]
    {
        Err(anyhow::anyhow!(
            "assert_bounds() failed, invalid bounds `{:?}`",
            bounds,
        ))
    } else {
        Ok(())
    }
}

pub fn assert_image(image: &Image) -> Result<(), anyhow::Error> {
    assert_bounds(image.bounds)?;

    if image.pixel_width == 0
        || image.pixel_width == 0
        || image.data.len() as u32 != 4 * image.pixel_width * image.pixel_height
    {
        Err(anyhow::anyhow!("assert_image() failed, invalid image data"))
    } else {
        Ok(())
    }
}
