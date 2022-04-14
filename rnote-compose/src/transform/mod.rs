mod transformbehaviour;

// Re-exports
pub use transformbehaviour::TransformBehaviour;

use serde::{Deserialize, Serialize};

/// To be used as state in a stroke to help implement the StrokeBehaviour trait
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "transform")]
pub struct Transform {
    #[serde(rename = "affine")]
    /// The affine transform matrix
    pub affine: na::Affine2<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            affine: na::Affine2::identity(),
        }
    }
}

impl From<Transform> for kurbo::Affine {
    fn from(transform: Transform) -> Self {
        let matrix = transform.affine.to_homogeneous();

        kurbo::Affine::new([
            matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5],
        ])
    }
}

impl Transform {
    /// A new transform given the affine
    pub fn new(transform: na::Affine2<f64>) -> Self {
        Self { affine: transform }
    }

    /// A new transform given the isometry
    pub fn new_w_isometry(isometry: na::Isometry2<f64>) -> Self {
        Self {
            affine: na::convert(isometry),
        }
    }

    /// Returns the translation part of the transform
    pub fn translation_part(&self) -> na::Vector2<f64> {
        (self.affine * na::point![0.0, 0.0]).coords
    }

    /// transforms a point by the transform
    pub fn transform_point(&self, point: na::Point2<f64>) -> na::Point2<f64> {
        self.affine * point
    }

    /// transform a vec ( translation will be ignored! )
    pub fn transform_vec(&self, vec: na::Vector2<f64>) -> na::Vector2<f64> {
        self.affine * vec
    }

    /// appends a translation to the transform
    pub fn append_translation_mut(&mut self, offset: na::Vector2<f64>) {
        self.affine = na::Translation2::from(offset) * self.affine;
    }

    /// appends a rotation around a point to the transform
    pub fn append_rotation_wrt_point_mut(&mut self, angle: f64, center: na::Point2<f64>) {
        self.affine = na::Translation2::from(-center.coords) * self.affine;
        self.affine = na::Rotation2::new(angle) * self.affine;
        self.affine = na::Translation2::from(center.coords) * self.affine;
    }

    /// appends a scale to the transform
    pub fn append_scale_mut(&mut self, scale: na::Vector2<f64>) {
        let translation = self.translation_part();

        self.affine = na::Translation2::from(-translation) * self.affine;

        self.affine = na::try_convert(
            na::Scale2::<f64>::from(scale).to_homogeneous() * self.affine.to_homogeneous(),
        )
        .unwrap();

        self.affine = na::Translation2::from(translation) * self.affine;
    }

    /// converts the transform to a svg attribute string, insertable into svg elements
    pub fn to_svg_transform_attr_str(&self) -> String {
        let matrix = self.affine;

        format!(
            "matrix({:.3} {:.3} {:.3} {:.3} {:.3} {:.3})",
            matrix[(0, 0)],
            matrix[(1, 0)],
            matrix[(0, 1)],
            matrix[(1, 1)],
            matrix[(0, 2)],
            matrix[(1, 2)],
        )
    }
}
