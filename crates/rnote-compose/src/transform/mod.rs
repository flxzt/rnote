// Modules
mod transformable;

// Re-exports
pub use transformable::Transformable;

// Imports
use crate::ext::{AabbExt, Affine2Ext};
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

/// An affine transformation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "transform")]
pub struct Transform {
    #[serde(rename = "affine", with = "crate::serialize::na_affine2_f64_dp3")]
    /// The affine transform matrix
    pub affine: na::Affine2<f64>,
}

/// Selection of which direction a mirror should use
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MirrorOrientation {
    /// Mirror is applied accross the line 'x = centerline'
    Horizontal,
    /// Mirror is applied accross the line 'y = centerline'
    Vertical,
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
            matrix[(0, 0)],
            matrix[(1, 0)],
            matrix[(0, 1)],
            matrix[(1, 1)],
            matrix[(0, 2)],
            matrix[(1, 2)],
        ])
    }
}

impl Transformable for Transform {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.append_translation_mut(offset)
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.append_rotation_wrt_point_mut(angle, center);
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.append_scale_mut(scale);
    }

    fn mirror(&mut self, centerline: f64, orientation: MirrorOrientation) {
        self.append_mirror_mut(centerline, orientation);
    }
}

impl Transform {
    /// Construct a new transform given the [`na::Affine2<f64>`].
    pub fn new(transform: na::Affine2<f64>) -> Self {
        Self { affine: transform }
    }

    /// Construct a new transform given the [`na::Isometry2<f64>`].
    pub fn new_w_isometry(isometry: na::Isometry2<f64>) -> Self {
        Self {
            affine: na::convert(isometry),
        }
    }

    /// The translation part of the transform.
    pub fn translation_part(&self) -> na::Vector2<f64> {
        (self.affine * na::point![0.0, 0.0]).coords
    }

    /// Transform a point by the transform.
    pub fn transform_point(&self, point: na::Point2<f64>) -> na::Point2<f64> {
        self.affine * point
    }

    /// Transform a [`na::Vector2<f64>`].
    ///
    /// The translational part will be ignored!
    pub fn transform_vec(&self, vec: na::Vector2<f64>) -> na::Vector2<f64> {
        self.affine * vec
    }

    /// Transforms the Aabb vertices and calculates a new that contains them.
    pub fn transform_aabb(&self, aabb: Aabb) -> Aabb {
        let p0 = self.affine * na::point![aabb.mins[0], aabb.mins[1]];
        let p1 = self.affine * na::point![aabb.mins[0], aabb.maxs[1]];
        let p2 = self.affine * na::point![aabb.maxs[0], aabb.maxs[1]];
        let p3 = self.affine * na::point![aabb.maxs[0], aabb.mins[1]];

        let min_x = p0[0].min(p1[0]).min(p2[0]).min(p3[0]);
        let min_y = p0[1].min(p1[1]).min(p2[1]).min(p3[1]);
        let max_x = p0[0].max(p1[0]).max(p2[0]).max(p3[0]);
        let max_y = p0[1].max(p1[1]).max(p2[1]).max(p3[1]);

        Aabb::new_positive(na::point![min_x, min_y], na::point![max_x, max_y])
    }

    /// Append a translation to the transform.
    pub fn append_translation_mut(&mut self, offset: na::Vector2<f64>) {
        self.affine = na::Translation2::from(offset) * self.affine;
    }

    /// Append a rotation around a point to the transform.
    pub fn append_rotation_wrt_point_mut(&mut self, angle: f64, center: na::Point2<f64>) {
        self.affine = na::Translation2::from(-center.coords) * self.affine;
        self.affine = na::Rotation2::new(angle) * self.affine;
        self.affine = na::Translation2::from(center.coords) * self.affine;
    }

    /// Append a scale to the transform.
    pub fn append_scale_mut(&mut self, scale: na::Vector2<f64>) {
        self.affine = na::try_convert(
            na::Scale2::<f64>::from(scale).to_homogeneous() * self.affine.to_homogeneous(),
        )
        .unwrap();
    }

    /// Apply a reflection across either Horizontal: 'x = centerline' or Vertical: 'y = centerline' to the
    /// affine matrix based on the orientation
    pub fn append_mirror_mut(&mut self, centerline: f64, orientation: MirrorOrientation) {
        let mirror_transformation = match orientation {
            MirrorOrientation::Horizontal => {
                na::matrix![
                    -1.0, 0.0, 2.0 * centerline;
                    0.0, 1.0, 0.0;
                    0.0, 0.0, 1.0;
                ]
            }
            MirrorOrientation::Vertical => {
                na::matrix![
                    1.0, 0.0, 0.0;
                    0.0, -1.0, 2.0 * centerline;
                    0.0, 0.0, 1.0;
                ]
            }
        };

        let transformed_affine = mirror_transformation * self.affine.matrix();

        self.affine = na::Affine2::from_matrix_unchecked(transformed_affine);
    }

    /// Convert the transform to a Svg attribute string, insertable into svg elements.
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

    /// Convert to [kurbo::Affine]
    pub fn to_kurbo(&self) -> kurbo::Affine {
        self.affine.to_kurbo()
    }
}
