use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use crate::helpers::{Affine2Helpers, Vector2Helpers};
use crate::shapes::ShapeBehaviour;
use crate::transform::TransformBehaviour;
use crate::Transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "ellipse")]
/// A Ellipse
pub struct Ellipse {
    /// The radii of the ellipse
    #[serde(rename = "radii")]
    pub radii: na::Vector2<f64>,
    /// The transform
    #[serde(rename = "transform")]
    pub transform: Transform,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            radii: na::Vector2::zeros(),
            transform: Transform::default(),
        }
    }
}

impl TransformBehaviour for Ellipse {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.transform.append_translation_mut(offset);
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.transform.append_rotation_wrt_point_mut(angle, center)
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.transform.append_scale_mut(scale);
    }
}

impl ShapeBehaviour for Ellipse {
    fn bounds(&self) -> AABB {
        let center = self.transform.affine * na::point![0.0, 0.0];
        // using a vector to ignore the translation
        let half_extents = na::Vector2::from_homogeneous(
            self.transform.affine.into_inner().abs() * self.radii.to_homogeneous(),
        )
        .unwrap()
        .abs();

        AABB::from_half_extents(center, half_extents)
    }

    fn hitboxes(&self) -> Vec<AABB> {
        vec![self.bounds()]
    }
}

impl Ellipse {
    /// from foci and point
    pub fn from_foci_and_point(foci: [na::Vector2<f64>; 2], point: na::Vector2<f64>) -> Self {
        let sum = (point - foci[0]).magnitude() + (point - foci[1]).magnitude();

        let d = (foci[0] - foci[1]).magnitude() / 2.0;
        let semimajor = sum / 2.0;
        let semiminor = (semimajor.powi(2) - d.powi(2)).sqrt();
        let v = foci[1] - foci[0];

        let center = (foci[0] + foci[1]) / 2.0;
        let angle = na::Vector2::x().angle_ahead(&v);

        let semimajor = if semimajor == 0.0 { 1.0 } else { semimajor };
        let semiminor = if semiminor == 0.0 { 1.0 } else { semiminor };

        let radii = na::vector![semimajor, semiminor];

        let transform = Transform::new_w_isometry(na::Isometry2::new(center, angle));

        Self { radii, transform }
    }

    /// to kurbo
    pub fn to_kurbo(&self) -> kurbo::Ellipse {
        self.transform.affine.to_kurbo()
            * kurbo::Ellipse::new(kurbo::Point::ZERO, self.radii.to_kurbo_vec(), 0.0)
    }
}
