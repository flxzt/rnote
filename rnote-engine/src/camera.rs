use gtk4::{graphene, gsk};
use p2d::bounding_volume::Aabb;
use rnote_compose::helpers::AabbHelpers;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "camera")]
pub struct Camera {
    /// The offset in surface coords.
    #[serde(rename = "offset")]
    pub offset: na::Vector2<f64>,
    /// The dimensions in surface coords
    #[serde(rename = "size")]
    pub size: na::Vector2<f64>,
    /// The camera zoom, origin at (0.0, 0.0)
    #[serde(rename = "zoom")]
    zoom: f64,
    /// the temporary zoom. Is used to overlay the "permanent" zoom
    #[serde(rename = "temporary_zoom")]
    temporary_zoom: f64,

    /// The scale factor of the surface, usually 1.0 or 2.0 for high-dpi screens. (Could become a non-integer value in the future, so it is stored as float.)
    #[serde(rename = "scale_factor")]
    pub scale_factor: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            offset: na::vector![0.0, 0.0],
            size: na::vector![800.0, 600.0],
            zoom: 1.0,
            temporary_zoom: 1.0,
            scale_factor: 1.0,
        }
    }
}

impl Camera {
    pub const ZOOM_MIN: f64 = 0.2;
    pub const ZOOM_MAX: f64 = 6.0;
    pub const ZOOM_DEFAULT: f64 = 1.0;

    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.set_zoom(zoom);
        self
    }

    pub fn with_offset(mut self, offset: na::Vector2<f64>) -> Self {
        self.offset = offset;
        self
    }
    pub fn with_size(mut self, size: na::Vector2<f64>) -> Self {
        self.size = size;
        self
    }

    /// the permanent zoom
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// sets the zoom
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(Self::ZOOM_MIN, Self::ZOOM_MAX)
    }

    /// The temporary zoom, supposed to be overlaid on the surface when zooming with a timeout
    pub fn temporary_zoom(&self) -> f64 {
        self.temporary_zoom
    }

    /// sets the temporary zoom
    pub fn set_temporary_zoom(&mut self, temporary_zoom: f64) {
        self.temporary_zoom =
            temporary_zoom.clamp(Camera::ZOOM_MIN / self.zoom, Camera::ZOOM_MAX / self.zoom)
    }

    /// The total zoom of the camera, including the temporary zoom
    pub fn total_zoom(&self) -> f64 {
        self.zoom * self.temporary_zoom
    }

    /// The scaling factor for generating pixel images with the current zoom. also takes the scale factor in account
    pub fn image_scale(&self) -> f64 {
        self.zoom * self.temporary_zoom * self.scale_factor
    }

    /// the viewport in document coordinate space
    pub fn viewport(&self) -> Aabb {
        let inv_zoom = 1.0 / self.total_zoom();

        Aabb::new_positive(
            na::Point2::from(self.offset * inv_zoom),
            na::Point2::from((self.offset + self.size) * inv_zoom),
        )
    }

    /// from document coords -> surface coords
    pub fn transform_bounds(&self, bounds: Aabb) -> Aabb {
        bounds.scale(self.total_zoom()).translate(-self.offset)
    }

    /// from surface coords -> document coords
    pub fn transform_inv_bounds(&self, bounds: Aabb) -> Aabb {
        bounds.translate(self.offset).scale(1.0 / self.total_zoom())
    }

    /// The transform from document coords -> surface coords
    /// To have the inverse, call .inverse()
    pub fn transform(&self) -> na::Affine2<f64> {
        let total_zoom = self.total_zoom();

        na::try_convert(
            // LHS is applied onto RHS, so the order is scaling by zoom -> Translation by offset
            na::Translation2::from(-self.offset).to_homogeneous()
                * na::Scale2::from(na::Vector2::from_element(total_zoom)).to_homogeneous(),
        )
        .unwrap()
    }

    // The gsk transform for the GTK snapshot func
    // GTKs transformations are applied on its coordinate system, so we need to reverse the order (translate, then scale)
    // To have the inverse, call .invert()
    pub fn transform_for_gtk_snapshot(&self) -> gsk::Transform {
        let total_zoom = self.total_zoom();

        gsk::Transform::new()
            .translate(&graphene::Point::new(
                -self.offset[0] as f32,
                -self.offset[1] as f32,
            ))
            .scale(total_zoom as f32, total_zoom as f32)
    }
}

#[cfg(test)]
mod tests {
    use crate::Camera;
    use approx::assert_relative_eq;

    #[test]
    fn transform_vec() {
        let offset = na::vector![4.0, 2.0];
        let zoom = 1.5;
        let camera = Camera::default().with_zoom(zoom).with_offset(offset);

        // Point in document coordinates
        let p0 = na::point![10.0, 2.0];

        // First zoom, then scale
        assert_relative_eq!(
            (camera.transform() * p0).coords,
            (p0.coords * zoom) - offset
        );
    }

    #[test]
    fn viewport() {
        let zoom = 2.0;
        let offset = na::vector![10.0, 10.0];
        let size = na::vector![20.0, 30.0];
        let camera = Camera::default()
            .with_zoom(zoom)
            .with_offset(offset)
            .with_size(size);

        let mins = na::Point2::from(offset / zoom);
        let maxs = na::Point2::from((offset + size) / zoom);

        let viewport = camera.viewport();

        assert_relative_eq!(viewport.mins, mins);
        assert_relative_eq!(viewport.maxs, maxs);
    }
}
