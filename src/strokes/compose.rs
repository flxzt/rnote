use gtk4::{gio, glib};
use std::ops::Deref;

use crate::{config};

#[allow(dead_code)]
pub fn add_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(r#"<\?xml[^\?>]*\?>"#).unwrap();
    if !re.is_match(svg) {
        let mut string = String::from(r#"<?xml version="1.0" standalone="no"?>"#);
        string.push_str("\n");
        string.push_str(svg);
        string
    } else {
        String::from(svg)
    }
}

pub fn remove_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(r#"<\?xml[^\?>]*\?>"#).unwrap();
    String::from(re.replace_all(svg, ""))
}

#[allow(dead_code)]
pub fn strip_svg_root(svg: &str) -> String {
    let re = regex::Regex::new(r#"<svg[^>]*>|<[^/svg]*/svg>"#).unwrap();
    String::from(re.replace_all(svg, ""))
}

pub fn wrap_svg(
    data: &str,
    bounds: Option<p2d::bounding_volume::AABB>,
    viewbox: Option<p2d::bounding_volume::AABB>,
    xml_header: bool,
    preserve_aspectratio: bool,
) -> String {
    let mut cx = tera::Context::new();

    let (x, y, width, height) = if let Some(bounds) = bounds {
        let x = format!("{}", bounds.mins[0].floor() as i32);
        let y = format!("{}", bounds.mins[1].floor() as i32);
        let width = format!("{}", (bounds.maxs[0] - bounds.mins[0]).ceil() as i32);
        let height = format!("{}", (bounds.maxs[1] - bounds.mins[1]).ceil() as i32);

        (x, y, width, height)
    } else {
        (
            String::from("0"),
            String::from("0"),
            String::from("100%"),
            String::from("100%"),
        )
    };

    let viewbox = if let Some(viewbox) = viewbox {
        format!(
            "viewBox=\"{} {} {} {}\"",
            viewbox.mins[0].floor() as i32,
            viewbox.mins[1].floor() as i32,
            (viewbox.maxs[0] - viewbox.mins[0]).ceil() as i32,
            (viewbox.maxs[1] - viewbox.mins[1]).ceil() as i32
        )
    } else {
        String::from("")
    };
    let preserve_aspectratio = if preserve_aspectratio {
        String::from("xMidyMid")
    } else {
        String::from("none")
    };

    cx.insert("xml_header", &xml_header);
    cx.insert("data", data);
    cx.insert("x", &x);
    cx.insert("y", &y);
    cx.insert("width", &width);
    cx.insert("height", &height);
    cx.insert("viewbox", &viewbox);
    cx.insert("preserve_aspectratio", &preserve_aspectratio);

    let templ = String::from_utf8(
        gio::resources_lookup_data(
            (String::from(config::APP_IDPATH) + "templates/svg_wrap.svg.templ").as_str(),
            gio::ResourceLookupFlags::NONE,
        )
        .unwrap()
        .deref()
        .to_vec(),
    )
    .unwrap();
    let output = tera::Tera::one_off(templ.as_str(), &cx, false)
        .expect("failed to create svg from template");

    output
}

pub fn svg_intrinsic_size(svg: &str) -> Option<na::Vector2<f64>> {
    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));
    if let Ok(handle) = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)
    {
        let renderer = librsvg::CairoRenderer::new(&handle);

        let intrinsic_size = if let Some(size) = renderer.intrinsic_size_in_pixels() {
            Some(na::vector![size.0, size.1])
        } else {
            log::warn!("intrinsic_size_in_pixels() failed in svg_intrinsic_size()");
            None
        };

        return intrinsic_size;
    } else {
        return None;
    }
}
