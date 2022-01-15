use p2d::bounding_volume::AABB;
use p2d::utils::IsometryOps;
use serde::{Deserialize, Serialize};

use crate::strokes::strokebehaviour::{self, StrokeBehaviour};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "rectangle")]
pub struct Rectangle {
    #[serde(rename = "cuboid")]
    pub cuboid: p2d::shape::Cuboid,
    #[serde(rename = "transform")]
    pub transform: strokebehaviour::StrokeTransform,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            cuboid: p2d::shape::Cuboid::new(na::Vector2::zeros()),
            transform: strokebehaviour::StrokeTransform::default(),
        }
    }
}

impl Rectangle {
    pub fn global_aabb(&self) -> p2d::bounding_volume::AABB {
        let center = na::Point2::from(self.transform.isometry.translation.vector);
        let ws_half_extents = self.transform.shear
            * self
                .transform
                .isometry
                .absolute_transform_vector(&self.cuboid.half_extents);

        AABB::from_half_extents(center, ws_half_extents)
    }
}

impl StrokeBehaviour for Rectangle {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        let translation = na::Translation2::<f64>::from(offset);

        self.transform.isometry.append_translation_mut(&translation);
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.transform
            .isometry
            .append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center)
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.cuboid.half_extents =
            na::Vector2::from(self.cuboid.half_extents.component_mul(&scale));
    }

    fn shear(&mut self, shear: nalgebra::Vector2<f64>) {
        let mut shear_matrix = na::Matrix3::<f64>::identity();
        shear_matrix[(0, 1)] = shear[0].tan();
        shear_matrix[(1, 0)] = shear[1].tan();

        // Unwrapping because we know its an Affine2
        self.transform.shear =
            na::try_convert(shear_matrix * self.transform.shear.to_homogeneous()).unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "ellipse")]
pub struct Ellipse {
    /// The radii of the ellipse
    #[serde(rename = "radii")]
    pub radii: na::Vector2<f64>,
    /// The transform
    #[serde(rename = "transform")]
    pub transform: strokebehaviour::StrokeTransform,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            radii: na::Vector2::zeros(),
            transform: strokebehaviour::StrokeTransform::default(),
        }
    }
}

impl StrokeBehaviour for Ellipse {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        let translation = na::Translation2::<f64>::from(offset);

        self.transform.isometry.append_translation_mut(&translation);
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.transform
            .isometry
            .append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center)
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.radii = na::Vector2::from(self.radii.component_mul(&scale));
    }

    fn shear(&mut self, shear: nalgebra::Vector2<f64>) {
        let mut shear_matrix = na::Matrix3::<f64>::identity();
        shear_matrix[(0, 1)] = shear[0].tan();
        shear_matrix[(1, 0)] = shear[1].tan();

        // Unwrapping because we know its an Affine2
        self.transform.shear =
            na::try_convert(shear_matrix * self.transform.shear.to_homogeneous()).unwrap();
    }
}

impl Ellipse {
    pub fn global_aabb(&self) -> p2d::bounding_volume::AABB {
        let center = na::Point2::from(self.transform.isometry.translation.vector);
        let ws_half_extents = self.transform.shear
            * self
                .transform
                .isometry
                .absolute_transform_vector(&self.radii);

        AABB::from_half_extents(center, ws_half_extents)
    }
}
