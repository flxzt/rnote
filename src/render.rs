use std::ops::Deref;

use anyhow::Context;
use gtk4::{
    gdk, gio, glib, graphene,
    gsk::{self, IsRenderNode},
    prelude::*,
    Native, Snapshot, Widget,
};
use rayon::prelude::*;

use crate::{geometry, strokes::StrokeKey};

#[derive(Debug, Clone)]
pub enum RendererBackend {
    Librsvg,
    Resvg,
}

#[derive(Debug, Clone)]
pub enum RenderTask {
    UpdateStrokeWithImage {
        key: StrokeKey,
        image: Image,
        zoom: f64,
    },
    Quit,
}

#[derive(Debug, Clone)]
pub struct Image {
    data: Vec<u8>,
    /// bounds in the coordinate space of the sheet
    bounds: p2d::bounding_volume::AABB,
    /// width of the data
    data_width: i32,
    /// height of the data
    data_height: i32,
    /// the memory format
    memory_format: gdk::MemoryFormat,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            bounds: p2d::bounding_volume::AABB::new(na::point![0.0, 0.0], na::point![0.0, 0.0]),
            data_width: 0,
            data_height: 0,
            memory_format: gdk::MemoryFormat::R8g8b8a8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Svg {
    pub svg_data: String,
    pub bounds: p2d::bounding_volume::AABB,
}

#[derive(Debug, Clone)]
pub struct Renderer {
    pub backend: RendererBackend,
    pub usvg_options: usvg::Options,
}

impl Default for Renderer {
    fn default() -> Self {
        let mut usvg_options = usvg::Options::default();
        usvg_options.fontdb.load_system_fonts();

        Self {
            usvg_options,
            backend: RendererBackend::Librsvg,
        }
    }
}

impl Renderer {
    pub fn gen_rendernode(&self, zoom: f64, svg: &Svg) -> Result<gsk::RenderNode, anyhow::Error> {
        let image = self.gen_image(zoom, svg)?;

        Ok(image_to_texturenode(&image, zoom).upcast())
    }

    pub fn gen_rendernode_par(
        &self,
        zoom: f64,
        svgs: &[Svg],
    ) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        let images = self.gen_images_par(zoom, svgs);

        let snapshot = Snapshot::new();

        // Rendernodes are not Sync or Send, so sequentially iterating here
        images.iter().for_each(|image| {
            snapshot.append_node(&image_to_texturenode(image, zoom));
        });

        Ok(snapshot.to_node())
    }

    pub fn gen_image(&self, zoom: f64, svg: &Svg) -> Result<Image, anyhow::Error> {
        match self.backend {
            RendererBackend::Librsvg => self.gen_image_librsvg(zoom, svg),
            RendererBackend::Resvg => self.gen_image_resvg(zoom, svg),
        }
    }

    pub fn gen_images_par(&self, zoom: f64, svgs: &[Svg]) -> Vec<Image> {
        // Parallel iteration to generate the texture images
        svgs.par_iter()
            .filter_map(|svg| match self.gen_image(zoom, &svg) {
                Ok(image) => Some(image),
                Err(e) => {
                    log::error!(
                        "gen_image_librsvg() in gen_rendernode_par_librsvg() failed, {}",
                        e
                    );
                    None
                }
            })
            .collect::<Vec<Image>>()
    }

    fn gen_image_librsvg(&self, zoom: f64, svg: &Svg) -> Result<Image, anyhow::Error> {
        let width_scaled = ((svg.bounds.extents()[0]) * zoom).round() as i32;
        let height_scaled = ((svg.bounds.extents()[1]) * zoom).round() as i32;

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

        // Context in new scope, else accessing the surface data fails with a borrow error
        {
            let cx = cairo::Context::new(&surface).context("new cairo::Context failed")?;
            //cx.scale(zoom, zoom);

            let stream =
                gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.svg_data.as_bytes()));
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
                .context("librsvg render document failed")?;

            cx.stroke()
                .context("cairo stroke() for rendered context failed")?;
        }

        let data = surface
            .data()
            .context("accessing imagesurface data failed")?;
        return Ok(Image {
            data: data.to_vec(),
            bounds: svg.bounds,
            data_width: width_scaled,
            data_height: height_scaled,
            memory_format: gdk::MemoryFormat::B8g8r8a8Premultiplied,
        });
    }

    fn gen_image_resvg(&self, zoom: f64, svg: &Svg) -> Result<Image, anyhow::Error> {
        let width_scaled = ((svg.bounds.extents()[0]) * zoom).round() as i32;
        let height_scaled = ((svg.bounds.extents()[1]) * zoom).round() as i32;

        let rtree = usvg::Tree::from_data(svg.svg_data.as_bytes(), &self.usvg_options.to_ref())?;

        let mut pixmap = tiny_skia::Pixmap::new(width_scaled as u32, height_scaled as u32)
            .ok_or_else(|| {
                anyhow::Error::msg("tiny_skia::Pixmap::new() failed in gen_image_resvg()")
            })?;

        resvg::render(
            &rtree,
            usvg::FitTo::Size(width_scaled as u32, height_scaled as u32),
            pixmap.as_mut(),
        )
        .ok_or_else(|| anyhow::Error::msg("resvg::render failed in gen_image_resvg."))?;

        let bytes = pixmap.data();

        return Ok(Image {
            data: bytes.to_vec(),
            bounds: svg.bounds,
            data_width: width_scaled,
            data_height: height_scaled,
            memory_format: gdk::MemoryFormat::R8g8b8a8Premultiplied,
        });
    }
}

pub fn default_rendernode() -> gsk::RenderNode {
    let bounds = graphene::Rect::new(0.0, 0.0, 0.0, 0.0);
    gsk::CairoNode::new(&bounds).upcast()
}

pub fn default_render_threadpool() -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::default()
        .build()
        .unwrap_or_else(|e| {
            log::error!("default_render_threadpool() failed with Err {}", e);
            panic!()
        })
}

pub fn image_to_memtexture(image: &Image) -> gdk::MemoryTexture {
    let bytes = image.data.deref();

    gdk::MemoryTexture::new(
        image.data_width,
        image.data_height,
        image.memory_format,
        &glib::Bytes::from(bytes),
        (image.data_width * 4) as usize,
    )
}

pub fn image_to_texturenode(image: &Image, zoom: f64) -> gsk::TextureNode {
    let memtexture = image_to_memtexture(image);

    gsk::TextureNode::new(
        &memtexture,
        &geometry::aabb_to_graphene_rect(geometry::aabb_scale(image.bounds, zoom)),
    )
}

pub fn rendernode_to_texture(
    active_widget: &Widget,
    node: &gsk::RenderNode,
    viewport: Option<p2d::bounding_volume::AABB>,
) -> Result<Option<gdk::Texture>, anyhow::Error> {
    let viewport = if let Some(viewport) = viewport {
        Some(geometry::aabb_to_graphene_rect(viewport))
    } else {
        None
    };

    if let Some(root) = active_widget.root() {
        if let Some(root_renderer) = root.upcast::<Native>().renderer() {
            let texture = root_renderer.render_texture(node, viewport.as_ref());
            return Ok(texture);
        }
    }

    Ok(None)
}

#[allow(dead_code)]
fn gen_caironode_librsvg(zoom: f64, svg: &Svg) -> Result<gsk::CairoNode, anyhow::Error> {
    if svg.bounds.extents()[0] < 0.0 || svg.bounds.extents()[1] < 0.0 {
        return Err(anyhow::anyhow!(
            "gen_rendernode_librsvg() failed, bounds width/ height is < 0.0"
        ));
    }

    let caironode_bounds = graphene::Rect::new(
        (svg.bounds.mins[0] * zoom).floor() as f32,
        (svg.bounds.mins[1] * zoom).floor() as f32,
        ((svg.bounds.extents()[0]) * zoom).ceil() as f32,
        ((svg.bounds.extents()[1]) * zoom).ceil() as f32,
    );

    let new_caironode = gsk::CairoNode::new(&caironode_bounds);
    let cx = new_caironode
        .draw_context()
        .context("failed to get cairo draw_context() from new_caironode")?;

    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.svg_data.as_bytes()));

    let librsvg_handle = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)?;

    let librsvg_renderer = librsvg::CairoRenderer::new(&librsvg_handle);
    librsvg_renderer.render_document(
        &cx,
        &cairo::Rectangle {
            x: (svg.bounds.mins[0].floor() * zoom),
            y: (svg.bounds.mins[1].floor() * zoom),
            width: ((svg.bounds.extents()[0]).ceil() * zoom),
            height: ((svg.bounds.extents()[1]).ceil() * zoom),
        },
    )?;
    Ok(new_caironode)
}
