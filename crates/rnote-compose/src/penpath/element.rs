// Imports
use crate::ext::DPose2Ext;
use crate::transform::Transformable;
use p2d::bounding_volume::Aabb;
use p2d::glamx::DAffine2;
use p2d::glamx::prelude::DPose2;
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

/// A pen input element.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename = "element")]
pub struct Element {
    #[serde(rename = "pos", with = "crate::serialize::glam_vector2_dp3")]
    /// The position of the element.
    pub pos: Vector2,
    #[serde(rename = "pressure", with = "crate::serialize::f64_dp3")]
    /// The pen pressure. The valid range is [0.0, 1.0].
    pub pressure: f64,
}

impl Default for Element {
    fn default() -> Self {
        Self::new(Vector2::ZERO, Self::PRESSURE_DEFAULT)
    }
}

impl Transformable for Element {
    fn translate(&mut self, offset: Vector2) {
        self.pos += offset;
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        let pose = DPose2::IDENTITY.append_rotation_wrt_center(angle, center);
        self.pos = pose.transform_point(self.pos);
    }

    fn scale(&mut self, scale: Vector2) {
        self.pos *= scale;
    }
}

impl Element {
    /// The default fallback pen pressure, when it could not be retrieved from the input.
    pub const PRESSURE_DEFAULT: f64 = 0.5;

    /// A new element from a position and pressure.
    pub fn new(pos: Vector2, pressure: f64) -> Self {
        Self {
            pos,
            pressure: pressure.clamp(0.0, 1.0),
        }
    }

    /// Sets the pressure, clamped to the range [0.0 - 1.0].
    pub fn set_pressure_clamped(&mut self, pressure: f64) {
        self.pressure = pressure.clamp(0.0, 1.0);
    }

    /// Indicates if a element is out of valid bounds and should be filtered out.
    ///
    /// Returns true if element pos is not inside the bounds.
    pub fn filter_by_bounds(&self, filter_bounds: Aabb) -> bool {
        !filter_bounds.contains_local_point(self.pos)
    }

    /// Transforms the element position by the given transform.
    pub fn transform_by(&mut self, transform: DAffine2) {
        self.pos = transform.transform_point2(self.pos);
    }
}
