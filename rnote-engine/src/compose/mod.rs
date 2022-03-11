use p2d::bounding_volume::AABB;
use rand::Rng;
use rand::SeedableRng;
use svg::node::{self, element};

pub mod color;
pub mod curves;
pub mod geometry;
pub mod rough;
pub mod shapes;
pub mod smooth;
pub mod textured;
pub mod transformable;

// Miscalleneous SVG functions

const XML_HEADER_REGEX: &str = r#"<\?xml[^\?>]*\?>"#;
const SVG_ROOT_REGEX: &str = r#"<svg[^>]*>|<[^/svg]*/svg>"#;

pub fn check_xml_header(svg: &str) -> bool {
    let re = regex::Regex::new(XML_HEADER_REGEX).unwrap();
    re.is_match(svg)
}

pub fn add_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(XML_HEADER_REGEX).unwrap();
    if !re.is_match(svg) {
        String::from(r#"<?xml version="1.0" standalone="no"?>"#) + "\n" + svg
    } else {
        String::from(svg)
    }
}

pub fn remove_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(XML_HEADER_REGEX).unwrap();
    String::from(re.replace_all(svg, ""))
}

pub fn check_svg_root(svg: &str) -> bool {
    let re = regex::Regex::new(SVG_ROOT_REGEX).unwrap();
    re.is_match(svg)
}

pub fn wrap_svg_root(
    data: &str,
    bounds: Option<AABB>,
    viewbox: Option<AABB>,
    preserve_aspectratio: bool,
) -> String {
    let (x, y, width, height) = if let Some(bounds) = bounds {
        let x = format!("{:.3}", bounds.mins[0]);
        let y = format!("{:.3}", bounds.mins[1]);
        let width = format!("{:.3}", bounds.extents()[0]);
        let height = format!("{:.3}", bounds.extents()[1]);

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
            "{:.3} {:.3} {:.3} {:.3}",
            viewbox.mins[0],
            viewbox.mins[1],
            viewbox.extents()[0],
            viewbox.extents()[1]
        )
    } else {
        String::from("")
    };
    let preserve_aspectratio = if preserve_aspectratio {
        String::from("xMidyMid")
    } else {
        String::from("none")
    };

    let svg_root = element::SVG::new()
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("xmlns:svg", "http://www.w3.org/2000/svg")
        .set("xmlns:xlink", "http://www.w3.org/1999/xlink")
        .set("x", x.as_str())
        .set("y", y.as_str())
        .set("width", width.as_str())
        .set("height", height.as_str())
        .set("viewBox", viewbox.as_str())
        .set("preserveAspectRatio", preserve_aspectratio.as_str())
        .add(node::Text::new(data));

    // unwrapping because we know its a valid Svg
    svg_node_to_string(&svg_root).unwrap()
}

pub fn strip_svg_root(svg: &str) -> String {
    let re = regex::Regex::new(SVG_ROOT_REGEX).unwrap();
    String::from(re.replace_all(svg, ""))
}

/// Converting a svg::Node to a String
pub fn svg_node_to_string<N>(node: &N) -> Result<String, anyhow::Error>
where
    N: svg::Node,
{
    let mut document_buffer = Vec::<u8>::new();
    svg::write(&mut document_buffer, node)?;
    Ok(String::from_utf8(document_buffer)?)
}

pub fn new_rng_default_pcg64(seed: Option<u64>) -> rand_pcg::Pcg64 {
    if let Some(seed) = seed {
        rand_pcg::Pcg64::seed_from_u64(seed)
    } else {
        rand_pcg::Pcg64::from_entropy()
    }
}

pub fn random_id_prefix() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>()
}
