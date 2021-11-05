use std::{error::Error, ops::Deref};

use gtk4::{
    gdk, gio, glib, graphene,
    gsk::{self, IsRenderNode},
    prelude::*,
    Native, Widget,
};

use crate::utils;

#[derive(Debug)]
pub enum RendererBackend {
    Librsvg,
    Resvg,
}

#[derive(Debug)]
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
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
        match self.backend {
            RendererBackend::Librsvg => {
                self.gen_rendernode_backend_librsvg(bounds, scalefactor, svg)
            }
            RendererBackend::Resvg => self.gen_rendernode_backend_resvg(bounds, scalefactor, svg),
        }
    }

    pub fn gen_rendernode_backend_librsvg(
        &self,
        bounds: p2d::bounding_volume::AABB,
        scalefactor: f64,
        svg: &str,
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
        let caironode_bounds = graphene::Rect::new(
            (bounds.mins[0] * scalefactor).floor() as f32,
            (bounds.mins[1] * scalefactor).floor() as f32,
            ((bounds.maxs[0] - bounds.mins[0]) * scalefactor).ceil() as f32,
            ((bounds.maxs[1] - bounds.mins[1]) * scalefactor).ceil() as f32,
        );

        let new_caironode = gsk::CairoNode::new(&caironode_bounds);
        let cx = new_caironode
            .draw_context()
            .expect("failed to get cairo draw_context() from new_caironode");

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

    pub fn gen_rendernode_backend_resvg(
        &self,
        svg_bounds: p2d::bounding_volume::AABB,
        scalefactor: f64,
        svg: &str,
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
        let node_bounds = graphene::Rect::new(
            (svg_bounds.mins[0].floor() * scalefactor) as f32,
            (svg_bounds.mins[1].floor() * scalefactor) as f32,
            ((svg_bounds.maxs[0] - svg_bounds.mins[0]).ceil() * scalefactor) as f32,
            ((svg_bounds.maxs[1] - svg_bounds.mins[1]).ceil() * scalefactor) as f32,
        );
        let width = ((svg_bounds.maxs[0] - svg_bounds.mins[0]).ceil() * scalefactor).round() as i32;
        let height = ((svg_bounds.maxs[1] - svg_bounds.mins[1]).ceil() * scalefactor).round() as i32;
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
}

pub fn default_rendernode() -> gsk::RenderNode {
    let bounds = graphene::Rect::new(0.0, 0.0, 0.0, 0.0);
    gsk::CairoNode::new(&bounds).upcast()
}

pub fn gen_cairosurface_librsvg(
    bounds: &p2d::bounding_volume::AABB,
    scalefactor: f64,
    svg: &str,
) -> Result<cairo::ImageSurface, Box<dyn Error>> {
    let width_scaled = (scalefactor * (bounds.maxs[0] - bounds.mins[0])).round() as i32;
    let height_scaled = (scalefactor * (bounds.maxs[1] - bounds.mins[1])).round() as i32;

    let surface =
        cairo::ImageSurface::create(cairo::Format::ARgb32, width_scaled, height_scaled).unwrap();

    // the ImageSurface has scaled size. Draw onto it in the unscaled, original coordinates, and will get scaled with this method .set_device_scale()
    surface.set_device_scale(scalefactor, scalefactor);

    let cx = cairo::Context::new(&surface).expect("Failed to create a cairo context");

    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));
    let handle = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)
        .expect("failed to parse xml into librsvg");
    let renderer = librsvg::CairoRenderer::new(&handle);
    renderer.render_document(
        &cx,
        &cairo::Rectangle {
            x: 0.0,
            y: 0.0,
            width: bounds.maxs[0] - bounds.mins[0],
            height: bounds.maxs[1] - bounds.mins[1],
        },
    )?;

    cx.stroke()
        .expect("failed to stroke() cairo context onto cairo surface.");

    Ok(surface)
}

/// Expects Imagesurface in ARgb32 premultiplied Format !
pub fn cairosurface_to_memtexture(
    mut surface: cairo::ImageSurface,
) -> Result<gdk::MemoryTexture, Box<dyn Error>> {
    let width = surface.width();
    let height = surface.height();
    let stride = surface.stride();

    let data = surface.data()?;
    let bytes = data.deref();

    // switch bytes around
    let bytes: Vec<u8> = bytes
        .iter()
        .zip(bytes.iter().skip(1))
        .zip(bytes.iter().skip(2))
        .zip(bytes.iter().skip(3))
        .step_by(4)
        .map(|(((first, second), third), forth)| [*first, *second, *third, *forth])
        .flatten()
        .collect();

    Ok(gdk::MemoryTexture::new(
        width,
        height,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &glib::Bytes::from(&bytes),
        stride as usize,
    ))
}

pub fn render_node_to_texture(
    active_widget: &Widget,
    node: &gsk::RenderNode,
    viewport: p2d::bounding_volume::AABB,
) -> Result<Option<gdk::Texture>, Box<dyn Error>> {
    if let Some(root) = active_widget.root() {
        if let Some(root_renderer) = root.upcast::<Native>().renderer() {
            let texture =
                root_renderer.render_texture(node, Some(&utils::aabb_to_graphene_rect(viewport)));
            return Ok(texture);
        }
    }

    Ok(None)
}
