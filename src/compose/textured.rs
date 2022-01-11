use crate::pens::brush::Brush;

use super::curves;

use rand::Rng;
use serde::{Deserialize, Serialize};
use svg::node::element::{self, Element};

/// The Configuration of how the textured shape should look

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct TexturedConfig {
    /// Amount dots per square dot area
    density: f64,
}

impl Default for TexturedConfig {
    fn default() -> Self {
        Self { density: 0.5 }
    }
}

pub fn compose_line(line: curves::Line, width: f64, brush: &Brush) -> Element {
    let mut rng = rand::thread_rng();

    let rect = line.line_w_width_to_rect(width);
    let area = 4.0 * rect.shape.half_extents[0] * rect.shape.half_extents[1];
    let range_x = -rect.shape.half_extents[0]..rect.shape.half_extents[0];
    let range_y = -rect.shape.half_extents[1]..rect.shape.half_extents[1];
    let range_rot = -std::f64::consts::FRAC_PI_4..std::f64::consts::FRAC_PI_4;

    let n_dots = (area * brush.textured_conf.density).round() as i32;
    let vec = line.end - line.start;
    let radii = (na::Rotation2::<f64>::new(
        rng.gen_range(range_rot) + vec.angle(&na::Vector2::<f64>::x_axis()),
    ) * na::vector![1.0, 0.1])
    .abs();

    let mut group = element::Group::new();

    for _ in 0..n_dots {
        let pos = rect.transform
            * na::point![
                rng.gen_range(range_x.clone()),
                rng.gen_range(range_y.clone())
            ];

        let ellipse = element::Ellipse::new()
            .set("cx", pos[0])
            .set("cy", pos[1])
            .set("rx", radii[0])
            .set("ry", radii[1])
            .set("fill", brush.color.to_css_color())
            .set("stroke", brush.color.to_css_color());

        group = group.add(ellipse);
    }

    group.into()
}
