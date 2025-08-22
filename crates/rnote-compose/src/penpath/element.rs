// Imports
use crate::{point_utils, transform::Transformable};
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

/// A pen input element.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename = "element")]
pub struct Element {
    #[serde(rename = "pos", with = "crate::serialize::na_vector2_f64_dp3")]
    /// The position of the element.
    pub pos: na::Vector2<f64>,
    #[serde(rename = "pressure", with = "crate::serialize::f64_dp3")]
    /// The pen pressure. The valid range is [0.0, 1.0].
    pub pressure: f64,
}

impl Default for Element {
    fn default() -> Self {
        Self::new(na::vector![0.0, 0.0], Self::PRESSURE_DEFAULT)
    }
}

impl Transformable for Element {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.pos += offset;
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);
        self.pos = isometry.transform_point(&self.pos.into()).coords;
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.pos = self.pos.component_mul(&scale);
    }
}

impl Element {
    /// The default fallback pen pressure, when it could not be retrieved from the input.
    pub const PRESSURE_DEFAULT: f64 = 0.5;

    /// A new element from a position and pressure.
    pub fn new(pos: na::Vector2<f64>, pressure: f64) -> Self {
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
        !filter_bounds.contains_local_point(&self.pos.into())
    }

    /// Transforms the element position by the given transform.
    pub fn transform_by(&mut self, transform: na::Affine2<f64>) {
        self.pos = transform.transform_point(&self.pos.into()).coords;
    }

    /// Mirrors position of element around line 'x = centerline_x'
    pub fn mirror_x(&mut self, centerline_x: f64) {
        point_utils::mirror_point_x(&mut self.pos, centerline_x);
    }

    /// Mirrors position of element around line 'y = centerline_y'
    pub fn mirror_y(&mut self, centerline_y: f64) {
        point_utils::mirror_point_y(&mut self.pos, centerline_y);
    }
}
