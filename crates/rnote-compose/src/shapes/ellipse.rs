// Imports
use super::Line;
use crate::Transformable;
use crate::ext::{DAffine2Ext, Vector2Ext};
use crate::shapes::Shapeable;
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use p2d::glamx::DAffine2;
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "ellipse")]
/// An Ellipse.
pub struct Ellipse {
    /// The radii of the ellipse.
    #[serde(rename = "radii", with = "crate::serialize::glam_vector2_dp3")]
    pub radii: Vector2,
    /// The affine transform of the center of the ellipse.
    #[serde(
        rename = "affine",
        alias = "transform",
        with = "crate::serialize::glam_daffine2_f64_dp3"
    )]
    pub affine: DAffine2,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            radii: Vector2::ZERO,
            affine: DAffine2::IDENTITY,
        }
    }
}

impl Transformable for Ellipse {
    fn translate(&mut self, offset: Vector2) {
        self.affine.append_translation_mut(offset);
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        self.affine.append_rotation_wrt_center_mut(angle, center)
    }

    fn scale(&mut self, scale: Vector2) {
        self.affine.append_scale_mut(scale);
    }
}

impl Shapeable for Ellipse {
    fn bounds(&self) -> Aabb {
        self.affine
            .transform_aabb(Aabb::from_half_extents(Vector2::ZERO, self.radii))
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        self.approx_with_lines()
            .into_iter()
            .map(|line| line.bounds())
            .collect()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        (self.affine.to_kurbo()
            * kurbo::Ellipse::new(kurbo::Point::ZERO, self.radii.to_kurbo_vec(), 0.0))
        .to_path(0.25)
    }
}

impl Ellipse {
    /// from foci and point
    pub fn from_foci_and_point(foci: [Vector2; 2], point: Vector2) -> Self {
        let sum = (point - foci[0]).length() + (point - foci[1]).length();
        let d = (foci[0] - foci[1]).length() * 0.5;
        let semimajor = sum * 0.5;
        let semiminor = (semimajor.powi(2) - d.powi(2)).sqrt();
        let vec = foci[1] - foci[0];
        let center = (foci[0] + foci[1]) * 0.5;
        let angle = Vector2::X.angle_to(vec);
        let semimajor = if semimajor == 0.0 { 1.0 } else { semimajor };
        let semiminor = if semiminor == 0.0 { 1.0 } else { semiminor };
        let radii = Vector2::new(semimajor, semiminor);
        let affine = DAffine2::from_angle_translation(angle, center);

        Self { radii, affine }
    }

    /// Approximate with lines.
    pub fn approx_with_lines(&self) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut prev = kurbo::Point::ZERO;

        kurbo::flatten(self.outline_path(), 0.25, |el| match el {
            kurbo::PathEl::MoveTo(point) => prev = point,
            kurbo::PathEl::LineTo(next) => {
                lines.push(Line {
                    start: Vector2::new(prev.x, prev.y),
                    end: Vector2::new(next.x, next.y),
                });
                prev = next
            }
            _ => {}
        });

        lines
    }
}
