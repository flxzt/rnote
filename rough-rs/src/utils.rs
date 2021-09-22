use std::fmt::Debug;

#[allow(dead_code)]
pub(crate) fn merge<T>(this: Option<T>, other: Option<T>) -> Option<T> {
    match this {
        Some(t) => return Some(t),
        None => match other {
            Some(o) => return Some(o),
            None => return None,
        },
    }
}

/// representing a RGBA color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: f32, // between 0.0 and 1.0
    g: f32, // between 0.0 and 1.0
    b: f32, // between 0.0 and 1.0
    a: f32, // between 0.0 and 1.0
}

impl Default for Color {
    fn default() -> Self {
        Self::black()
    }
}

impl Color {
    /// Creating a new Color
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }
    /// exporting a color to a css compliant String in form rgba(red, green, blue, alpha)
    pub fn to_css_color(self) -> String {
        format!(
            "rgb({:03},{:03},{:03},{:.3})",
            (self.r * 255.0) as i32,
            (self.g * 255.0) as i32,
            (self.b * 255.0) as i32,
            ((1000.0 * self.a).round() / 1000.0),
        )
    }

    /// the transparent Color
    pub fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// the black Color
    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}
