use crate::utils;

use super::curves;

use serde::{Deserialize, Serialize};
use svg::node::element::{self, Element};

/// The Configuration of how the textured shape should look

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct TexturedConfig {
    /// An optional seed to generate reproducable strokes
    seed: Option<u64>,
    /// The color of the dots
    color: utils::Color,
    /// Amount dots per 10x10 area
    density: f64,
    /// the radii of the dots
    radii: na::Vector2<f64>,
    /// uniformity. 1.0 to spread the dots equally out up to the width, 0.0 to have all dots on the center
    uniformity: f64,
}

impl Default for TexturedConfig {
    fn default() -> Self {
        Self {
            seed: None,
            density: Self::DENSITY_DEFAULT,
            color: Self::COLOR_DEFAULT,
            radii: Self::RADII_DEFAULT,
            uniformity: Self::UNIFORMITY_DEFAULT,
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
    pub const DENSITY_DEFAULT: f64 = 1.0;
    /// Radii min
    pub const RADII_MIN: na::Vector2<f64> = na::vector![0.0, 0.0];
    /// Radii max
    pub const RADII_MAX: na::Vector2<f64> = na::vector![50.0, 50.0];
    /// Radii default
    pub const RADII_DEFAULT: na::Vector2<f64> = na::vector![1.7, 0.3];
    /// Uniformity min
    pub const UNIFORMITY_MIN: f64 = 0.0;
    /// Uniformity max
    pub const UNIFORMITY_MAX: f64 = 1.0;
    /// Uniformity default
    pub const UNIFORMITY_DEFAULT: f64 = 0.9;

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

    pub fn uniformity(&self) -> f64 {
        self.uniformity
    }

    pub fn set_uniformity(&mut self, uniformity: f64) {
        self.uniformity = uniformity.clamp(Self::UNIFORMITY_MIN, Self::UNIFORMITY_MAX);
    }
}

pub fn compose_line(line: curves::Line, width: f64, config: &mut TexturedConfig) -> Element {
    let rect = line.line_w_width_to_rect(width);
    let area = 4.0 * rect.shape.half_extents[0] * rect.shape.half_extents[1];

    // Ranges for randomization
    let range_x = -rect.shape.half_extents[0]..rect.shape.half_extents[0];
    let range_y = -rect.shape.half_extents[1]..rect.shape.half_extents[1];
    let range_dots_rot = -std::f64::consts::FRAC_PI_8..std::f64::consts::FRAC_PI_8;
    let range_dots_rx = config.radii[0] * 0.5..config.radii[0] * 2.0;
    let range_dots_ry = config.radii[1] * 0.5..config.radii[1] * 2.0;

    let n_dots = (area * 0.1 * config.density).round() as i32;
    let vec = line.end - line.start;

    fn y_spread_calc(y_coord: f64, config: &mut TexturedConfig) -> f64 {
        y_coord * (utils::rand_range_advance(&mut config.seed, 0.0..1.0) - config.uniformity).abs()
    }

    let mut group = element::Group::new();

    for _ in 0..n_dots {
        let x_pos = utils::rand_range_advance(&mut config.seed, range_x.clone());
        let y_pos = utils::rand_range_advance(&mut config.seed, range_y.clone());
        let y_pos = y_spread_calc(y_pos, config);

        let pos = rect.transform * na::point![x_pos, y_pos];

        let rotation_angle = na::Rotation2::rotation_between(&na::Vector2::x(), &vec).angle()
            + utils::rand_range_advance(&mut config.seed, range_dots_rot.clone());
        let radii = na::vector![
            utils::rand_range_advance(&mut config.seed, range_dots_rx.clone()),
            utils::rand_range_advance(&mut config.seed, range_dots_ry.clone())
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
