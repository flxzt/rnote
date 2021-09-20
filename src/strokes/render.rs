use std::error::Error;

use gtk4::{gio, glib, graphene, gsk};

pub fn gen_caironode_for_svg(
    bounds: p2d::bounding_volume::AABB,
    scalefactor: f64,
    svg: &str,
) -> Result<gsk::CairoNode, Box<dyn Error>> {
    let caironode_bounds = graphene::Rect::new(
        (bounds.mins[0] * scalefactor).floor() as f32,
        (bounds.mins[1] * scalefactor).floor() as f32,
        ((bounds.maxs[0] - bounds.mins[0]) * scalefactor).ceil() as f32,
        ((bounds.maxs[1] - bounds.mins[1]) * scalefactor).ceil() as f32,
    );

    let new_node = gsk::CairoNode::new(&caironode_bounds);
    let cx = new_node
        .draw_context()
        .expect("failed to get cairo draw_context() from caironode");

    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));
    let handle = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)?;
    let renderer = librsvg::CairoRenderer::new(&handle);

    renderer.render_document(
        &cx,
        &cairo::Rectangle {
            x: (bounds.mins[0].floor() * scalefactor),
            y: (bounds.mins[1].floor() * scalefactor),
            width: ((bounds.maxs[0] - bounds.mins[0]).ceil() * scalefactor),
            height: ((bounds.maxs[1] - bounds.mins[1]).ceil() * scalefactor),
        },
    )?;
    Ok(new_node)
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
