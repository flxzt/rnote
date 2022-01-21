use gtk4::gdk;
use notetakingfileformats::xoppformat;
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

pub mod curves;
pub mod geometry;
pub mod rough;
pub mod shapes;
pub mod solid;
pub mod textured;

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
    const SVG_WRAP_TEMPL_STR: &str = r#"
<svg
  x="{{x}}"
  y="{{y}}"
  width="{{width}}"
  height="{{height}}"
  {{viewbox}}
  preserveAspectRatio="{{preserve_aspectratio}}"
  xmlns="http://www.w3.org/2000/svg"
  xmlns:svg="http://www.w3.org/2000/svg"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  >
  {{data}}
</svg>
"#;
    let mut cx = tera::Context::new();

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
            "viewBox=\"{:.3} {:.3} {:.3} {:.3}\"",
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

    cx.insert("data", data);
    cx.insert("x", &x);
    cx.insert("y", &y);
    cx.insert("width", &width);
    cx.insert("height", &height);
    cx.insert("viewbox", &viewbox);
    cx.insert("preserve_aspectratio", &preserve_aspectratio);

    tera::Tera::one_off(SVG_WRAP_TEMPL_STR, &cx, false).expect("failed to create svg from template")
}

pub fn strip_svg_root(svg: &str) -> String {
    let re = regex::Regex::new(SVG_ROOT_REGEX).unwrap();
    String::from(re.replace_all(svg, ""))
}

/// patterns are rendered rather slow, so this should be used carefully!
pub fn wrap_svg_pattern(data: &str, id: &str, bounds: AABB) -> String {
    const SVG_PATTERN_TEMPL_STR: &str = r#"
<defs>
    <pattern
        id="{{id}}"
        x="{{x}}"
        y="{{y}}"
        width="{{width}}"
        height="{{height}}"
        patternUnits="userSpaceOnUse"
        >
        {{data}}
    </pattern>
</defs>
"#;
    let mut cx = tera::Context::new();
    let x = format!("{:3}", bounds.mins[0]);
    let y = format!("{:3}", bounds.mins[1]);
    let width = format!("{:3}", bounds.extents()[0]);
    let height = format!("{:3}", bounds.extents()[1]);
    cx.insert("id", &id);
    cx.insert("x", &x);
    cx.insert("y", &y);
    cx.insert("width", &width);
    cx.insert("height", &height);
    cx.insert("data", &data);

    tera::Tera::one_off(SVG_PATTERN_TEMPL_STR, &cx, false)
        .expect("failed to create svg from template")
}

/// wraps the data in a group, and translates and scales them with the transform attribute
pub fn wrap_svg_group(
    data: &str,
    offset: na::Vector2<f64>,
    scalevector: na::Vector2<f64>,
) -> String {
    const SVG_GROUP_TEMPL_STR: &str = r#"
<g transform="
  translate({{translate_x}} {{translate_y}})
  scale({{scale_x}} {{scale_y}})
  "
>
  {{data}}
</g>
"#;
    let mut cx = tera::Context::new();
    let translate_x = format!("{:3}", offset[0]);
    let translate_y = format!("{:3}", offset[1]);
    let scale_x = format!("{:3}", scalevector[0]);
    let scale_y = format!("{:3}", scalevector[1]);
    cx.insert("translate_x", &translate_x);
    cx.insert("translate_y", &translate_y);
    cx.insert("scale_x", &scale_x);
    cx.insert("scale_y", &scale_y);
    cx.insert("data", data);

    tera::Tera::one_off(SVG_GROUP_TEMPL_STR, &cx, false)
        .expect("failed to create svg from template")
}

/// Converting a svg::Node to a String
pub fn node_to_string<N>(node: &N) -> Result<String, anyhow::Error>
where
    N: svg::Node,
{
    let mut document_buffer = Vec::<u8>::new();
    svg::write(&mut document_buffer, node)?;
    Ok(String::from_utf8(document_buffer)?)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Color {
    pub r: f64, // between 0.0 and 1.0
    pub g: f64, // between 0.0 and 1.0
    pub b: f64, // between 0.0 and 1.0
    pub a: f64, // between 0.0 and 1.0
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn r(&self) -> f64 {
        self.r
    }

    pub fn g(&self) -> f64 {
        self.g
    }

    pub fn b(&self) -> f64 {
        self.b
    }

    pub fn a(&self) -> f64 {
        self.a
    }

    pub fn to_css_color(self) -> String {
        format!(
            "rgb({:03},{:03},{:03},{:.3})",
            (self.r * 255.0) as i32,
            (self.g * 255.0) as i32,
            (self.b * 255.0) as i32,
            ((1000.0 * self.a).round() / 1000.0),
        )
    }

    pub fn to_gdk(&self) -> gdk::RGBA {
        gdk::RGBA::new(self.r as f32, self.g as f32, self.b as f32, self.a as f32)
    }

    pub fn to_u32(&self) -> u32 {
        ((((self.r * 255.0).round() as u32) & 0xff) << 24)
            | ((((self.g * 255.0).round() as u32) & 0xff) << 16)
            | ((((self.b * 255.0).round() as u32) & 0xff) << 8)
            | (((self.a * 255.0).round() as u32) & 0xff)
    }
}

impl From<gdk::RGBA> for Color {
    fn from(gdk_color: gdk::RGBA) -> Self {
        Self {
            r: f64::from(gdk_color.red()),
            g: f64::from(gdk_color.green()),
            b: f64::from(gdk_color.blue()),
            a: f64::from(gdk_color.alpha()),
        }
    }
}

/// u32 encoded as RGBA
impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self {
            r: f64::from((value >> 24) & 0xff) / 255.0,
            g: f64::from((value >> 16) & 0xff) / 255.0,
            b: f64::from((value >> 8) & 0xff) / 255.0,
            a: f64::from((value) & 0xff) / 255.0,
        }
    }
}

/// From XoppColor
impl From<xoppformat::XoppColor> for Color {
    fn from(xopp_color: xoppformat::XoppColor) -> Self {
        Self {
            r: f64::from(xopp_color.red) / 255.0,
            g: f64::from(xopp_color.green) / 255.0,
            b: f64::from(xopp_color.blue) / 255.0,
            a: f64::from(xopp_color.alpha) / 255.0,
        }
    }
}

/// Into XoppColor
impl Into<xoppformat::XoppColor> for Color {
    fn into(self) -> xoppformat::XoppColor {
        xoppformat::XoppColor {
            red: (self.r * 255.0).floor() as u8,
            green: (self.g * 255.0).floor() as u8,
            blue: (self.b * 255.0).floor() as u8,
            alpha: (self.a * 255.0).floor() as u8,
        }
    }
}
