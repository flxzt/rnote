use std::collections::VecDeque;

use chrono::Utc;
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

/// Represents an input element
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(default, rename = "element")]
pub struct Element {
    #[serde(rename = "pos")]
    pub pos: na::Vector2<f64>,
    // Between 0.0 and 1.0
    #[serde(rename = "pressure")]
    pub pressure: f64,
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<Utc>,
}

impl Default for Element {
    fn default() -> Self {
        Self::new(na::vector![0.0, 0.0], Self::PRESSURE_DEFAULT)
    }
}

impl Element {
    pub const PRESSURE_DEFAULT: f64 = 0.5;

    pub fn new(pos: na::Vector2<f64>, pressure: f64) -> Self {
        Self {
            pos,
            pressure,
            timestamp: Utc::now(),
        }
    }

    pub fn update_timestamp(&mut self) {
        self.timestamp = Utc::now();
    }

    pub fn set_pressure_clamped(&mut self, pressure: f64) {
        self.pressure = pressure.clamp(0.0, 1.0);
    }

    /// Returns true if element pos is not inside the bounds
    pub fn filter_by_bounds(&self, filter_bounds: AABB) -> bool {
        !filter_bounds.contains_local_point(&na::Point2::from(self.pos))
    }

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
