use serde::{Deserialize, Serialize};

/// Specifies that a type behaves as a stroke
pub trait StrokeBehaviour {
    /// translates (as in moves) the stroke with offset
    fn translate(&mut self, offset: na::Vector2<f64>);
    /// rotates the stroke in angle (rad)
    fn rotate(&mut self, angle: f64, center: na::Point2<f64>);
    /// scales the stroke by the desired scale
    fn scale(&mut self, scale: na::Vector2<f64>);
    /// shears the stroke by the x_angle (rad) and y_angle (rad)
    fn shear(&mut self, shear: na::Vector2<f64>);
}

/// To be used as state in a stroke to help implement the StrokeBehaviour trait
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "stroke_transform")]
pub struct StrokeTransform {
    #[serde(rename = "isometry")]
    pub isometry: na::Isometry2<f64>,
    #[serde(rename = "scale")]
    pub shear: na::Affine2<f64>,
}

impl Default for StrokeTransform {
    fn default() -> Self {
        Self {
            isometry: na::Isometry2::identity(),
            shear: na::Affine2::identity(),
        }
    }
}

impl StrokeTransform {
    pub fn new(isometry: na::Isometry2<f64>, shear: na::Affine2<f64>) -> Self {
        Self { isometry, shear }
    }

    pub fn new_w_isometry(isometry: na::Isometry2<f64>) -> Self {
        Self {
            isometry,
            ..Self::default()
        }
    }

    pub fn matrix(&self) -> na::Affine2<f64> {
        self.shear * self.isometry
    }

    pub fn matrix_as_svg_transform_attr(&self) -> String {
        let transform_matrix = self.matrix();

        format!(
            "matrix({:.3} {:.3} {:.3} {:.3} {:.3} {:.3})",
            transform_matrix[(0, 0)],
            transform_matrix[(1, 0)],
            transform_matrix[(0, 1)],
            transform_matrix[(1, 1)],
            transform_matrix[(0, 2)],
            transform_matrix[(1, 2)],
        )
    }
}
