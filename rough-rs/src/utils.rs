use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use rand::{Rng, SeedableRng};

use crate::options::Options;

pub(crate) fn merge<T>(this: Option<T>, other: Option<T>) -> Option<T> {
    match this {
        Some(t) => Some(t),
        None => other,
    }
}

/// representing a RGBA color
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
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

/// Random u64 with full number range
pub fn random_u64_full(seed: Option<u64>) -> u64 {
    let mut rng = if let Some(seed) = seed {
        rand_pcg::Pcg64::seed_from_u64(seed)
    } else {
        rand_pcg::Pcg64::from_entropy()
    };
    rng.gen()
}

/// Random i32 with full number range
pub fn random_i32_full(seed: Option<u64>) -> i32 {
    let mut rng = if let Some(seed) = seed {
        rand::rngs::StdRng::seed_from_u64(seed)
    } else {
        rand::rngs::StdRng::from_entropy()
    };
    rng.gen()
}

/// Random f64 between 0.0 and 1.0
fn random_f64_0to1(seed: Option<u64>) -> f64 {
    let mut rng = if let Some(seed) = seed {
        rand::rngs::StdRng::seed_from_u64(seed)
    } else {
        rand::rngs::StdRng::from_entropy()
    };
    rng.gen_range(0.0..1.0)
}

/// returning random f64 from 0.0 to 1.0 created from seed and advancing it
pub fn rand_f64_0to1_next(options: &mut Options) -> f64 {
    if let Some(ref mut seed) = options.seed {
        *seed = random_u64_full(Some(*seed));
    };
    random_f64_0to1(options.seed)
}
