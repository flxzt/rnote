use crate::utils;

use super::curves;

use serde::{Deserialize, Serialize};
use svg::node::element::{self, Element};

/// The Configuration of how the textured shape should look

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct TexturedConfig {
    pub seed: Option<u64>,
    /// Amount dots per square dot area
    pub density: f64,
    pub color: utils::Color,
}

impl Default for TexturedConfig {
    fn default() -> Self {
        Self {
            seed: None,
            density: 0.5,
            color: utils::Color::black(),
        }
    }
}

pub fn compose_line(line: curves::Line, width: f64, config: &mut TexturedConfig) -> Element {
    let rect = line.line_w_width_to_rect(width);
    let area = 4.0 * rect.shape.half_extents[0] * rect.shape.half_extents[1];
    let range_x = -rect.shape.half_extents[0]..rect.shape.half_extents[0];
    let range_y = -rect.shape.half_extents[1]..rect.shape.half_extents[1];
    let range_rot = -std::f64::consts::FRAC_PI_8..std::f64::consts::FRAC_PI_8;

    let n_dots = (area * config.density).round() as i32;
    let vec = line.end - line.start;

    let radii_angle = na::Vector2::x().angle(&vec)
        + utils::rand_range_advance(&mut config.seed, range_rot);
    let radii = (na::Rotation2::new(radii_angle) * na::vector![0.5, 0.07]).abs();

    let mut group = element::Group::new();

    for _ in 0..n_dots {
        let pos = rect.transform
            * na::point![
                utils::rand_range_advance(&mut config.seed, range_x.clone()),
                utils::rand_range_advance(&mut config.seed, range_y.clone())
            ];

        let ellipse = element::Ellipse::new()
            .set("cx", pos[0])
            .set("cy", pos[1])
            .set("rx", radii[0])
            .set("ry", radii[1])
            .set("fill", config.color.to_css_color());

        group = group.add(ellipse);
    }

    group.into()
}
