use std::ops::Range;

use crate::utils;

use super::curves;

use gtk4::glib;
use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use svg::node::element::{self, Element};

/// The distribution for the spread of dots across the width of the textured stroke
#[derive(
    Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize, num_derive::FromPrimitive,
)]
#[repr(u32)]
#[enum_type(name = "TexturedDotsDistribution")]
pub enum TexturedDotsDistribution {
    #[enum_value(name = "Uniform", nick = "uniform")]
    Uniform = 0,
    #[enum_value(name = "Normal", nick = "normal")]
    Normal,
    #[enum_value(name = "Exponential", nick = "exponential")]
    Exponential,
    #[enum_value(name = "ReverseExponential", nick = "reverse-exponential")]
    ReverseExponential,
}

impl Default for TexturedDotsDistribution {
    fn default() -> Self {
        Self::Normal
    }
}

impl TexturedDotsDistribution {
    /// Samples a value for the given range, symmetrical to the mid of the range. For distributions that are open ended, samples are clipped to the range
    fn sample_for_range_symmetrical_clipped<G: rand::Rng + ?Sized>(
        &self,
        rng: &mut G,
        range: Range<f64>,
    ) -> f64 {
        let sample = match self {
            Self::Uniform => rand_distr::Uniform::from(range.clone()).sample(rng),
            Self::Normal => {
                // setting the mean to the mid of the range
                let mean = (range.end + range.start) / 2.0;
                // the standard deviation
                let std_dev = ((range.end - range.start) / 2.0) / 3.0;

                rand_distr::Normal::new(mean, std_dev).unwrap().sample(rng)
            }
            Self::Exponential => {
                let mid = (range.end + range.start) / 2.0;
                let width = (range.end - range.start) / 4.0;
                // The lambda
                let lambda = 1.0;

                let sign: f64 = if rand_distr::Standard.sample(rng) {
                    1.0
                } else {
                    -1.0
                };

                mid + sign * width * rand_distr::Exp::new(lambda).unwrap().sample(rng)
            }
            Self::ReverseExponential => {
                let width = (range.end - range.start) / 4.0;
                // The lambda
                let lambda = 1.0;

                let positive: bool = rand_distr::Standard.sample(rng);
                let sign = if positive { 1.0 } else { -1.0 };
                let offset = if positive { range.start } else { range.end };

                offset + (sign * width * rand_distr::Exp::new(lambda).unwrap().sample(rng))
            }
        };

        if !range.contains(&sample) {
            // Do a uniform distribution as fallback if sample is out of range
            rand_distr::Uniform::from(range.clone()).sample(rng)
        } else {
            sample
        }
    }
}

/// The Configuration of how the textured shape should look

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "textured_config")]
pub struct TexturedConfig {
    /// An optional seed to generate reproducable strokes
    #[serde(rename = "seed")]
    seed: Option<u64>,
    /// The color of the dots
    #[serde(rename = "color")]
    color: utils::Color,
    /// Amount dots per 10x10 area
    #[serde(rename = "density")]
    density: f64,
    /// the radii of the dots
    #[serde(rename = "radii")]
    radii: na::Vector2<f64>,
    /// the distribution type
    #[serde(rename = "distribution")]
    distribution: TexturedDotsDistribution,
}

impl Default for TexturedConfig {
    fn default() -> Self {
        Self {
            seed: None,
            density: Self::DENSITY_DEFAULT,
            color: Self::COLOR_DEFAULT,
            radii: Self::RADII_DEFAULT,
            distribution: TexturedDotsDistribution::default(),
        }
    }
}

impl TexturedConfig {
    /// The default color
    pub const COLOR_DEFAULT: utils::Color = utils::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    /// Density min
    pub const DENSITY_MIN: f64 = 0.0;
    /// Density max
    pub const DENSITY_MAX: f64 = 10.0;
    /// Density default
    pub const DENSITY_DEFAULT: f64 = 5.0;
    /// Radii min
    pub const RADII_MIN: na::Vector2<f64> = na::vector![0.0, 0.0];
    /// Radii max
    pub const RADII_MAX: na::Vector2<f64> = na::vector![100.0, 100.0];
    /// Radii default
    pub const RADII_DEFAULT: na::Vector2<f64> = na::vector![2.0, 0.3];

    pub fn seed(&self) -> Option<u64> {
        self.seed
    }

    pub fn set_seed(&mut self, seed: Option<u64>) {
        self.seed = seed
    }

    pub fn color(&self) -> utils::Color {
        self.color
    }

    pub fn set_color(&mut self, color: utils::Color) {
        self.color = color;
    }

    pub fn density(&self) -> f64 {
        self.density
    }

    pub fn set_density(&mut self, density: f64) {
        self.density = density.clamp(Self::DENSITY_MIN, Self::DENSITY_MAX);
    }

    pub fn radii(&self) -> na::Vector2<f64> {
        self.radii
    }

    pub fn set_radii(&mut self, radii: na::Vector2<f64>) {
        self.radii = na::vector![
            radii[0].clamp(Self::RADII_MIN[0], Self::RADII_MAX[0]),
            radii[1].clamp(Self::RADII_MIN[0], Self::RADII_MAX[1])
        ];
    }

    pub fn distribution(&self) -> TexturedDotsDistribution {
        self.distribution
    }

    pub fn set_distribution(&mut self, distribution: TexturedDotsDistribution) {
        self.distribution = distribution;
    }
}

pub fn compose_line(line: curves::Line, width: f64, config: &mut TexturedConfig) -> Element {
    let rect = line.line_w_width_to_rect(width);
    let area = 4.0 * rect.cuboid.half_extents[0] * rect.cuboid.half_extents[1];

    // Ranges for randomization
    let range_x = -rect.cuboid.half_extents[0]..rect.cuboid.half_extents[0];
    let range_y = -rect.cuboid.half_extents[1]..rect.cuboid.half_extents[1];
    let range_dots_rot = -std::f64::consts::FRAC_PI_8..std::f64::consts::FRAC_PI_8;
    let range_dots_rx = config.radii[0] * 0.8..config.radii[0] * 1.25;
    let range_dots_ry = config.radii[1] * 0.8..config.radii[1] * 1.25;

    let distr_x = Uniform::from(range_x);
    let distr_dots_rot = Uniform::from(range_dots_rot);
    let distr_dots_rx = Uniform::from(range_dots_rx);
    let distr_dots_ry = Uniform::from(range_dots_ry);

    let n_dots = (area * 0.1 * config.density).round() as i32;
    let vec = line.end - line.start;

    let mut rng = if let Some(seed) = config.seed {
        rand_pcg::Pcg64::seed_from_u64(seed)
    } else {
        rand_pcg::Pcg64::from_entropy()
    };

    let mut group = element::Group::new();

    for _ in 0..n_dots {
        let x_pos = distr_x.sample(&mut rng);
        let y_pos = config
            .distribution
            .sample_for_range_symmetrical_clipped(&mut rng, range_y.clone());

        let pos = rect.transform.isometry * na::point![x_pos, y_pos];

        let rotation_angle = na::Rotation2::rotation_between(&na::Vector2::x(), &vec).angle()
            + distr_dots_rot.sample(&mut rng);
        let radii = na::vector![
            distr_dots_rx.sample(&mut rng),
            distr_dots_ry.sample(&mut rng)
        ];

        let ellipse = element::Ellipse::new()
            .set(
                "transform",
                format!(
                    "rotate({},{},{})",
                    rotation_angle.to_degrees(),
                    pos[0],
                    pos[1]
                ),
            )
            .set("cx", pos[0])
            .set("cy", pos[1])
            .set("rx", radii[0])
            .set("ry", radii[1])
            .set("fill", config.color.to_css_color());

        group = group.add(ellipse);
    }

    group.into()
}
