// Modules
mod transformable;

// Re-exports
pub use transformable::Transformable;

// Imports
use crate::ext::{AabbExt, DAffine2Ext};
use p2d::bounding_volume::Aabb;
use p2d::glamx::DAffine2;
use p2d::glamx::prelude::DPose2;
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

/// An affine transformation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "transform")]
pub struct Transform {
    #[serde(rename = "affine", with = "crate::serialize::glam_daffine2_f64_dp3")]
    /// The affine transform matrix
    pub affine: DAffine2,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl From<Transform> for kurbo::Affine {
    fn from(transform: Transform) -> Self {
        let array = transform.affine.to_cols_array_2d();
        kurbo::Affine::new([
            array[0][0],
            array[0][1],
            array[1][0],
            array[1][1],
            array[2][0],
            array[2][1],
        ])
    }
}

impl Transformable for Transform {
    fn translate(&mut self, offset: Vector2) {
        self.append_translation_mut(offset)
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        self.append_rotation_wrt_center_mut(angle, center);
    }

    fn scale(&mut self, scale: Vector2) {
        self.append_scale_mut(scale);
    }
}

impl Transform {
    /// The identity transform.
    pub const IDENTITY: Self = Self {
        affine: DAffine2::IDENTITY,
    };

    /// Construct a new transform given the [`DAffine2`].
    pub fn new(affine: DAffine2) -> Self {
        Self { affine }
    }

    /// Construct a new transform given the [`DPose2`].
    pub fn new_w_pose(pose: DPose2) -> Self {
        Self {
            affine: DAffine2::from_angle_translation(pose.rotation.angle(), pose.translation),
        }
    }

    /// The translation part of the transform.
    pub fn translation_part(&self) -> Vector2 {
        self.affine.translation
    }

    /// Transform a point by the transform.
    pub fn transform_point(&self, point: Vector2) -> Vector2 {
        self.affine.transform_point2(point)
    }

    /// Transform a [`Vector2`].
    ///
    /// The translational part will be ignored!
    pub fn transform_vec(&self, vec: Vector2) -> Vector2 {
        self.affine.transform_vector2(vec)
    }

    /// Transforms the Aabb vertices and calculates a new that contains them.
    pub fn transform_aabb(&self, aabb: Aabb) -> Aabb {
        let p0 = self
            .affine
            .transform_point2(Vector2::new(aabb.mins.x, aabb.mins.y));
        let p1 = self
            .affine
            .transform_point2(Vector2::new(aabb.mins.x, aabb.maxs.y));
        let p2 = self
            .affine
            .transform_point2(Vector2::new(aabb.maxs.x, aabb.maxs.y));
        let p3 = self
            .affine
            .transform_point2(Vector2::new(aabb.maxs.x, aabb.mins.y));
        let min_x = p0.x.min(p1.x).min(p2.x).min(p3.x);
        let min_y = p0.y.min(p1.y).min(p2.y).min(p3.y);
        let max_x = p0.x.max(p1.x).max(p2.x).max(p3.x);
        let max_y = p0.y.max(p1.y).max(p2.y).max(p3.y);
        Aabb::new_positive(Vector2::new(min_x, min_y), Vector2::new(max_x, max_y))
    }

    /// Append a translation to the transform.
    pub fn append_translation_mut(&mut self, offset: Vector2) {
        self.affine = DAffine2::from_translation(offset) * self.affine;
    }

    /// Append a rotation around a center to the transform.
    pub fn append_rotation_wrt_center_mut(&mut self, angle: f64, center: Vector2) {
        self.affine = DAffine2::from_translation(-center) * self.affine;
        self.affine = DAffine2::from_angle(angle) * self.affine;
        self.affine = DAffine2::from_translation(center) * self.affine;
    }

    /// Append a scale to the transform.
    pub fn append_scale_mut(&mut self, scale: Vector2) {
        self.affine = DAffine2::from_scale(scale) * self.affine;
    }

    /// Convert the transform to a Svg attribute string, insertable into svg elements.
    pub fn to_svg_transform_attr_str(&self) -> String {
        let array = self.affine.to_cols_array_2d();
        format!(
            "matrix({:.3} {:.3} {:.3} {:.3} {:.3} {:.3})",
            array[0][0], array[0][1], array[1][0], array[1][1], array[2][0], array[2][1],
        )
    }

    /// Convert to [kurbo::Affine]
    pub fn to_kurbo(&self) -> kurbo::Affine {
        self.affine.to_kurbo()
    }
}
