use std::error::Error;

use gtk4::{
    gdk, gdk_pixbuf, gio, glib, graphene,
    gsk::{self, IsRenderNode},
};

pub enum RendererBackend {
    Librsvg,
    Resvg,
}
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

    pub fn gen_rendernode_backend_resvg(
        &self,
        bounds: p2d::bounding_volume::AABB,
        scalefactor: f64,
        svg: &str,
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
        let node_bounds = graphene::Rect::new(
            (bounds.mins[0] * scalefactor).floor() as f32,
            (bounds.mins[1] * scalefactor).floor() as f32,
            ((bounds.maxs[0] - bounds.mins[0]) * scalefactor).ceil() as f32,
            ((bounds.maxs[1] - bounds.mins[1]) * scalefactor).ceil() as f32,
        );

        let rtree = usvg::Tree::from_data(svg.as_bytes(), &self.usvg_options.to_ref())?;

        //let pixmap_size = rtree.svg_node().size.to_screen_size();
        let mut pixmap = tiny_skia::Pixmap::new(
            node_bounds.width().floor() as u32,
            node_bounds.height().floor() as u32,
        )
        .unwrap();

        resvg::render(
            &rtree,
            usvg::FitTo::Size(
                node_bounds.width().floor() as u32,
                node_bounds.height().floor() as u32,
            ),
            pixmap.as_mut(),
        )
        .unwrap();

        //pixmap.save_png(&PathBuf::from("./tests/output/stroke_resvg.png"))?;
        let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
            &glib::Bytes::from(pixmap.data()),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            node_bounds.width().floor() as i32,
            node_bounds.height().floor() as i32,
            4 * node_bounds.width().floor() as i32,
        );
        let texture = gdk::Texture::for_pixbuf(&pixbuf);

        Ok(gsk::TextureNode::new(&texture, &node_bounds).upcast())
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
}

pub fn default_rendernode() -> gsk::RenderNode {
    let bounds = graphene::Rect::new(0.0, 0.0, 0.0, 0.0);
    gsk::CairoNode::new(&bounds).upcast()
}

pub fn gen_cairosurface(
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

    cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);

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
