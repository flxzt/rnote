use crate::utils;

use super::curves;

use serde::{Deserialize, Serialize};
use svg::node::element::{self, Element};

/// The Configuration of how the textured shape should look

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct TexturedConfig {
    /// An optional seed to generate reproducable strokes
    pub seed: Option<u64>,
    /// Amount dots per square dot area
    pub density: f64,
    /// The color of the dots
    pub color: utils::Color,
    /// the radii of the dots
    pub radii: na::Vector2<f64>,
}

impl Default for TexturedConfig {
    fn default() -> Self {
        Self {
            seed: None,
            density: 0.7,
            color: utils::Color::black(),
            radii: na::vector![1.0, 0.3],
        }
    }
}

pub fn compose_line(line: curves::Line, width: f64, config: &mut TexturedConfig) -> Element {
    let rect = line.line_w_width_to_rect(width);
    let area = 4.0 * rect.shape.half_extents[0] * rect.shape.half_extents[1];

    // Ranges for randomization
    let range_x = -rect.shape.half_extents[0]..rect.shape.half_extents[0];
    let range_y = -rect.shape.half_extents[1]..rect.shape.half_extents[1];
    let range_dots_rot = -std::f64::consts::FRAC_PI_8..std::f64::consts::FRAC_PI_8;
    let range_dots_len_x = config.radii[0] * 0.8..config.radii[0] * 1.2;
    let range_dots_len_y = config.radii[1] * 0.8..config.radii[1] * 1.2;

    let n_dots = (area * config.density).round() as i32;
    let vec = line.end - line.start;

    let mut group = element::Group::new();

    for _ in 0..n_dots {
        let pos = rect.transform
            * na::point![
                utils::rand_range_advance(&mut config.seed, range_x.clone()),
                utils::rand_range_advance(&mut config.seed, range_y.clone())
            ];
        let rotation_angle = na::Rotation2::rotation_between(&na::Vector2::x(), &vec).angle()
            + utils::rand_range_advance(&mut config.seed, range_dots_rot.clone());
        let radii = na::vector![
            utils::rand_range_advance(&mut config.seed, range_dots_len_x.clone()),
            utils::rand_range_advance(&mut config.seed, range_dots_len_y.clone())
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
