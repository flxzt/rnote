use serde::{Deserialize, Serialize};

use super::geometry;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Rectangle {
    pub shape: p2d::shape::Cuboid,
    pub transform: na::Isometry2<f64>,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            shape: geometry::default_cuboid(),
            transform: geometry::default_isometry(),
        }
    }
}

impl Rectangle {
    fn top_left(&self) -> na::Vector2<f64> {
        let half_extents = self.shape.half_extents;

        self.transform * -half_extents
    }

    fn top_right(&self) -> na::Vector2<f64> {
        let half_extents = self.shape.half_extents;
        self.transform * na::vector![half_extents[0], -half_extents[1]]
    }

    fn bottom_left(&self) -> na::Vector2<f64> {
        let half_extents = self.shape.half_extents;
        self.transform * na::vector![-half_extents[0], half_extents[1]]
    }

    fn bottom_right(&self) -> na::Vector2<f64> {
        let half_extents = self.shape.half_extents;
        self.transform * half_extents
    }

    fn vertices(&self) -> Vec<na::Vector2<f64>> {
        vec![
            self.top_left(),
            self.top_right(),
            self.bottom_right(),
            self.bottom_left(),
        ]
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ellipse {
    /// The center
    center: na::Vector2<f64>,
    /// The radii of the ellipse
    radii: na::Vector2<f64>,
    /// The rotation angle, in radians
    rot_angle: f64,
}
