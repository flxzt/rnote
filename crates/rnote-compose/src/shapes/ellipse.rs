// Imports
use super::Line;
use crate::Transform;
use crate::ext::{Affine2Ext, Vector2Ext};
use crate::shapes::Shapeable;
use crate::transform::Transformable;
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "ellipse")]
/// An Ellipse.
pub struct Ellipse {
    /// The radii of the ellipse.
    #[serde(rename = "radii", with = "crate::serialize::na_vector2_f64_dp3")]
    pub radii: na::Vector2<f64>,
    /// The transform of the center of the ellipse.
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

impl Transformable for Ellipse {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.transform.append_translation_mut(offset);
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.transform.append_rotation_wrt_point_mut(angle, center)
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.transform.append_scale_mut(scale);
    }
}

impl Shapeable for Ellipse {
    fn bounds(&self) -> Aabb {
        let center = self.transform.affine * na::point![0.0, 0.0];
        // using a vector to ignore the translation
        let half_extents = na::Vector2::from_homogeneous(
            self.transform.affine.into_inner().abs() * self.radii.to_homogeneous(),
        )
        .unwrap()
        .abs();

        Aabb::from_half_extents(center, half_extents)
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        self.approx_with_lines()
            .into_iter()
            .map(|line| line.bounds())
            .collect()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        (self.transform.affine.to_kurbo()
            * kurbo::Ellipse::new(kurbo::Point::ZERO, self.radii.to_kurbo_vec(), 0.0))
        .to_path(0.25)
    }
}

impl Ellipse {
    /// from foci and point
    pub fn from_foci_and_point(foci: [na::Vector2<f64>; 2], point: na::Vector2<f64>) -> Self {
        let sum = (point - foci[0]).magnitude() + (point - foci[1]).magnitude();

        let d = (foci[0] - foci[1]).magnitude() * 0.5;
        let semimajor = sum * 0.5;
        let semiminor = (semimajor.powi(2) - d.powi(2)).sqrt();
        let v = foci[1] - foci[0];

        let center = (foci[0] + foci[1]) * 0.5;
        let angle = na::Vector2::x().angle_ahead(&v);

        let semimajor = if semimajor == 0.0 { 1.0 } else { semimajor };
        let semiminor = if semiminor == 0.0 { 1.0 } else { semiminor };

        let radii = na::vector![semimajor, semiminor];

        let transform = Transform::new_w_isometry(na::Isometry2::new(center, angle));

        Self { radii, transform }
    }

    /// Approximate with lines.
    pub fn approx_with_lines(&self) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut prev = kurbo::Point::new(0.0, 0.0);

        kurbo::flatten(self.outline_path(), 0.25, |el| match el {
            kurbo::PathEl::MoveTo(point) => prev = point,
            kurbo::PathEl::LineTo(next) => {
                lines.push(Line {
                    start: na::vector![prev.x, prev.y],
                    end: na::vector![next.x, next.y],
                });
                prev = next
            }
            _ => {}
        });

        lines
    }
}
