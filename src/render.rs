use std::ops::Deref;

use anyhow::Context;
use gtk4::{gdk, glib, graphene, gsk, prelude::*, Native, Snapshot, Widget};
use p2d::bounding_volume::AABB;

use crate::compose::{self, geometry};

#[derive(Debug, Clone)]
pub enum RendererBackend {
    Librsvg,
    Resvg,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    /// bounds in the coordinate space of the sheet
    pub bounds: AABB,
    /// width of the data
    pub data_width: i32,
    /// height of the data
    pub data_height: i32,
    /// the memory format
    pub memory_format: gdk::MemoryFormat,
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
    pub fn gen_image(&self, zoom: f64, svgs: &[Svg], bounds: AABB) -> Result<Image, anyhow::Error> {
        if svgs.is_empty() {
            return Err(anyhow::Error::msg("gen_image() failed, no svg's in slice."));
        }
        if bounds.extents()[0] <= 0.0 || bounds.extents()[1] <= 0.0 {
            return Err(anyhow::Error::msg(
                "gen_image() failed, bounds extents are <= 0.0",
            ));
        }
        /*         match self.backend {
            RendererBackend::Librsvg => self.gen_image_librsvg(zoom, svgs, bounds),
            RendererBackend::Resvg => self.gen_image_resvg(zoom, svgs, bounds),
        } */
        self.gen_image_resvg(zoom, svgs, bounds)
    }
    /*
    fn gen_image_librsvg(
        &self,
        zoom: f64,
        svgs: &[Svg],
        bounds: AABB,
    ) -> Result<Image, anyhow::Error> {
        let width_scaled = ((bounds.extents()[0]) * zoom).round() as i32;
        let height_scaled = ((bounds.extents()[1]) * zoom).round() as i32;

        let mut surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, width_scaled, height_scaled)
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

        Ok(Image {
            data,
            bounds,
            data_width: width_scaled,
            data_height: height_scaled,
            memory_format: gdk::MemoryFormat::B8g8r8a8Premultiplied,
        })
    } */

    fn gen_image_resvg(
        &self,
        zoom: f64,
        svgs: &[Svg],
        bounds: AABB,
    ) -> Result<Image, anyhow::Error> {
        let width_scaled = ((bounds.extents()[0]) * zoom).round() as i32;
        let height_scaled = ((bounds.extents()[1]) * zoom).round() as i32;

        let mut svg_data = svgs
            .iter()
            .map(|svg| svg.svg_data.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        svg_data = compose::wrap_svg_root(svg_data.as_str(), Some(bounds), Some(bounds), true);

        let mut pixmap = tiny_skia::Pixmap::new(width_scaled as u32, height_scaled as u32)
            .ok_or_else(|| {
                anyhow::Error::msg("tiny_skia::Pixmap::new() failed in gen_image_resvg()")
            })?;

        let rtree = usvg::Tree::from_data(svg_data.as_bytes(), &self.usvg_options.to_ref())?;

        resvg::render(&rtree, usvg::FitTo::Zoom(zoom as f32), pixmap.as_mut())
            .ok_or_else(|| anyhow::Error::msg("resvg::render failed in gen_image_resvg."))?;

        let data = pixmap.data().to_vec();

        Ok(Image {
            data,
            bounds,
            data_width: width_scaled,
            data_height: height_scaled,
            memory_format: gdk::MemoryFormat::R8g8b8a8Premultiplied,
        })
    }
}

pub fn default_rendernode() -> gsk::RenderNode {
    let bounds = graphene::Rect::new(0.0, 0.0, 0.0, 0.0);
    gsk::CairoNode::new(&bounds).upcast()
}

pub fn image_to_memtexture(image: &Image) -> Result<gdk::MemoryTexture, anyhow::Error> {
    if image.data_width <= 0 || image.data_height <= 0 || image.data.is_empty() {
        return Err(anyhow::anyhow!(
            "image_to_memtexture() failed, invalid image"
        ));
    }
    let bytes = image.data.deref();

    Ok(gdk::MemoryTexture::new(
        image.data_width,
        image.data_height,
        image.memory_format,
        &glib::Bytes::from(bytes),
        (image.data_width * 4) as usize,
    ))
}

pub fn image_to_rendernode(image: &Image, zoom: f64) -> Result<gsk::RenderNode, anyhow::Error> {
    let memtexture = image_to_memtexture(image)?;

    let rendernode = gsk::TextureNode::new(
        &memtexture,
        &geometry::aabb_to_graphene_rect(geometry::aabb_scale(image.bounds, zoom)),
    )
    .upcast();
    Ok(rendernode)
}

pub fn images_to_rendernode(images: &[Image], zoom: f64) -> Result<gsk::RenderNode, anyhow::Error> {
    let snapshot = Snapshot::new();

    for image in images {
        snapshot
            .append_node(&image_to_rendernode(image, zoom).context("images_to_rendernode failed")?);
    }

    Ok(snapshot.to_node())
}

pub fn append_images_to_rendernode(
    rendernode: &gsk::RenderNode,
    images: &[Image],
    zoom: f64,
) -> Result<gsk::RenderNode, anyhow::Error> {
    let snapshot = Snapshot::new();

    snapshot.append_node(rendernode);
    for image in images {
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

/*
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
    let cx = new_caironode
        .draw_context()
        .context("failed to get cairo draw_context() from new_caironode")?;

    draw_svgs_to_cairo_context(zoom, &[svg.to_owned()], caironode_bounds, &cx)?;

    Ok(new_caironode)
}
 */
