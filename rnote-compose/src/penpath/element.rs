use std::collections::VecDeque;

use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

use crate::transform::TransformBehaviour;

/// A pen input element
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename = "element")]
pub struct Element {
    #[serde(rename = "pos")]
    /// The position of the element
    pub pos: na::Vector2<f64>,
    #[serde(rename = "pressure")]
    /// The pen pressure. The valid range is [0.0, 1.0]
    pub pressure: f64,
}

impl Default for Element {
    fn default() -> Self {
        Self::new(na::vector![0.0, 0.0], Self::PRESSURE_DEFAULT)
    }
}

impl TransformBehaviour for Element {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.pos += offset;
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.pos = (isometry * na::Point2::from(self.pos)).coords;
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.pos = self.pos.component_mul(&scale);
    }
}

impl Element {
    /// The default fallback pen pressure, when it could not be retrieved from the input
    pub const PRESSURE_DEFAULT: f64 = 0.5;

    /// A new element from a position and pressure
    pub fn new(pos: na::Vector2<f64>, pressure: f64) -> Self {
        Self {
            pos,
            pressure: pressure.clamp(0.0, 1.0),
        }
    }

    /// Sets the pressure, clamped to the range [0.0 - 1.0]
    pub fn set_pressure_clamped(&mut self, pressure: f64) {
        self.pressure = pressure.clamp(0.0, 1.0);
    }

    /// indicates if a element is out of valid bounds and should be filtered out. Returns true if element pos is not inside the bounds
    pub fn filter_by_bounds(&self, filter_bounds: Aabb) -> bool {
        !filter_bounds.contains_local_point(&na::Point2::from(self.pos))
    }

    /// Transforms the element position by the transform
    pub fn transform_by(&mut self, transform: na::Affine2<f64>) {
        self.pos = (transform * na::Point2::from(self.pos)).coords;
    }

    /// transform pen input data entries
    pub fn transform_elements(data_entries: &mut VecDeque<Self>, transform: na::Affine2<f64>) {
        data_entries.iter_mut().for_each(|element| {
            element.transform_by(transform);
        });
    }
}
