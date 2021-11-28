use std::ops::Deref;

use anyhow::Context;
use gtk4::{
    gdk, gio, glib, graphene,
    gsk::{self, IsRenderNode},
    prelude::*,
    Native, Snapshot, Widget,
};
use rayon::prelude::*;

use crate::geometry;

#[derive(Debug, Clone)]
pub enum RendererBackend {
    Librsvg,
    Resvg,
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
    pub fn gen_rendernode(
        &self,
        bounds: p2d::bounding_volume::AABB,
        scalefactor: f64,
        svg: &str,
    ) -> Result<gsk::RenderNode, anyhow::Error> {
        match self.backend {
            RendererBackend::Librsvg => self.gen_rendernode_librsvg(bounds, scalefactor, svg),
            RendererBackend::Resvg => self.gen_rendernode_resvg(bounds, scalefactor, svg),
        }
    }

    pub fn gen_rendernode_par(
        &self,
        scalefactor: f64,
        svgs: &[Svg],
    ) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        let images = self.gen_images_par_librsvg(scalefactor, svgs);

        let snapshot = Snapshot::new();

        // Rendernodes are not Sync or Send, so sequentially iterating here
        images.iter().for_each(|image| {
            snapshot.append_node(&image_to_texturenode(image, scalefactor));
        });

        Ok(snapshot.to_node())
    }

    pub fn gen_rendernode_librsvg(
        &self,
        bounds: p2d::bounding_volume::AABB,
        scalefactor: f64,
        svg: &str,
    ) -> Result<gsk::RenderNode, anyhow::Error> {
        if bounds.maxs[0] - bounds.mins[0] < 0.0 || bounds.maxs[1] - bounds.mins[1] < 0.0 {
            return Err(anyhow::anyhow!(
                "gen_rendernode_librsvg() failed, bounds width/ height is < 0.0"
            ));
        }

        let caironode_bounds = graphene::Rect::new(
            (bounds.mins[0] * scalefactor).floor() as f32,
            (bounds.mins[1] * scalefactor).floor() as f32,
            ((bounds.maxs[0] - bounds.mins[0]) * scalefactor).ceil() as f32,
            ((bounds.maxs[1] - bounds.mins[1]) * scalefactor).ceil() as f32,
        );

        let new_caironode = gsk::CairoNode::new(&caironode_bounds);
        let cx = new_caironode
            .draw_context()
            .context("failed to get cairo draw_context() from new_caironode")?;

        let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));

        let librsvg_handle = librsvg::Loader::new()
            .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(
                &stream, None, None,
            )?;

        let librsvg_renderer = librsvg::CairoRenderer::new(&librsvg_handle);
        librsvg_renderer.render_document(
            &cx,
            &cairo::Rectangle {
                x: (bounds.mins[0].floor() * scalefactor),
                y: (bounds.mins[1].floor() * scalefactor),
                width: ((bounds.maxs[0] - bounds.mins[0]).ceil() * scalefactor),
                height: ((bounds.maxs[1] - bounds.mins[1]).ceil() * scalefactor),
            },
        )?;
        Ok(new_caironode.upcast())
    }

    pub fn gen_rendernode_resvg(
        &self,
        svg_bounds: p2d::bounding_volume::AABB,
        scalefactor: f64,
        svg: &str,
    ) -> Result<gsk::RenderNode, anyhow::Error> {
        let node_bounds = graphene::Rect::new(
            (svg_bounds.mins[0].floor() * scalefactor) as f32,
            (svg_bounds.mins[1].floor() * scalefactor) as f32,
            ((svg_bounds.maxs[0] - svg_bounds.mins[0]).ceil() * scalefactor) as f32,
            ((svg_bounds.maxs[1] - svg_bounds.mins[1]).ceil() * scalefactor) as f32,
        );
        let width = ((svg_bounds.maxs[0] - svg_bounds.mins[0]).ceil() * scalefactor).round() as i32;
        let height =
            ((svg_bounds.maxs[1] - svg_bounds.mins[1]).ceil() * scalefactor).round() as i32;
        let stride = 4 * width as usize;

        let rtree = usvg::Tree::from_data(svg.as_bytes(), &self.usvg_options.to_ref())?;

        let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();

        resvg::render(
            &rtree,
            usvg::FitTo::Size(width as u32, height as u32),
            pixmap.as_mut(),
        )
        .unwrap();

        let bytes = pixmap.data();

        let memtexture = gdk::MemoryTexture::new(
            width,
            height,
            gdk::MemoryFormat::R8g8b8a8Premultiplied,
            &glib::Bytes::from(&bytes),
            stride,
        );
        Ok(gsk::TextureNode::new(&memtexture, &node_bounds).upcast())
    }

    pub fn gen_images_par_librsvg(&self, scalefactor: f64, svgs: &[Svg]) -> Vec<Image> {
        // Parallel iteration to generate the texture images
        svgs.par_iter()
            .filter_map(
                |svg| match gen_image_librsvg(svg.bounds, scalefactor, &svg.svg_data) {
                    Ok(image) => Some(image),
                    Err(e) => {
                        //println!("{}\n", svg.svg_data.as_str());
                        log::error!(
                            "gen_image_librsvg() in gen_rendernode_par_librsvg() failed, {}",
                            e
                        );
                        None
                    }
                },
            )
            .collect::<Vec<Image>>()
    }
}

pub fn default_rendernode() -> gsk::RenderNode {
    let bounds = graphene::Rect::new(0.0, 0.0, 0.0, 0.0);
    gsk::CairoNode::new(&bounds).upcast()
}

pub fn gen_image_librsvg(
    bounds: p2d::bounding_volume::AABB,
    scalefactor: f64,
    svg: &str,
) -> Result<Image, anyhow::Error> {
    let width_scaled = (scalefactor * (bounds.maxs[0] - bounds.mins[0])).round() as i32;
    let height_scaled = (scalefactor * (bounds.maxs[1] - bounds.mins[1])).round() as i32;

    let mut surface =
        cairo::ImageSurface::create(cairo::Format::ARgb32, width_scaled, height_scaled)
            .map_err(|e| anyhow::anyhow!("create ImageSurface with dimensions ({}, {}) failed, {}", width_scaled, height_scaled, e))?;

    // Context in new scope, else accessing the surface data fails with a borrow error
    {
        let cx = cairo::Context::new(&surface).context("new cairo::Context failed")?;
        cx.scale(scalefactor, scalefactor);

        let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));
        let handle = librsvg::Loader::new()
            .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)
            .context("read stream to librsvg Loader failed")?;
        let renderer = librsvg::CairoRenderer::new(&handle);
        renderer
            .render_document(
                &cx,
                &cairo::Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: bounds.maxs[0] - bounds.mins[0],
                    height: bounds.maxs[1] - bounds.mins[1],
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
        bounds,
        data_width: width_scaled,
        data_height: height_scaled,
    });
}

/// Expects Image pixels in ARgb32 premultiplied Format !
pub fn image_to_memtexture(image: &Image) -> gdk::MemoryTexture {
    let bytes = image.data.deref();

    gdk::MemoryTexture::new(
        image.data_width,
        image.data_height,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &glib::Bytes::from(bytes),
        (image.data_width * 4) as usize,
    )
}

/// Expects Image pixels in ARgb32 premultiplied Format !
pub fn image_to_texturenode(image: &Image, scalefactor: f64) -> gsk::TextureNode {
    let image_bounds = image.bounds;
    let memtexture = image_to_memtexture(image);

    gsk::TextureNode::new(
        &memtexture,
        &geometry::aabb_to_graphene_rect(geometry::aabb_scale(image_bounds, scalefactor)),
    )
}

pub fn rendernode_to_texture(
    active_widget: &Widget,
    node: &gsk::RenderNode,
    viewport: p2d::bounding_volume::AABB,
) -> Result<Option<gdk::Texture>, anyhow::Error> {
    if let Some(root) = active_widget.root() {
        if let Some(root_renderer) = root.upcast::<Native>().renderer() {
            let texture = root_renderer
                .render_texture(node, Some(&geometry::aabb_to_graphene_rect(viewport)));
            return Ok(texture);
        }
    }

    Ok(None)
}
