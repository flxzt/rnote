use crate::transform::MirrorOrientation;

/// Trait for types that can be (geometrically) transformed.
pub trait Transformable {
    /// Translate (as in moves) by the given offset.
    fn translate(&mut self, offset: na::Vector2<f64>);
    /// Rotate by the given angle (in radians).
    fn rotate(&mut self, angle: f64, center: na::Point2<f64>);
    /// Scale by the given scale-factor.
    fn scale(&mut self, scale: na::Vector2<f64>);
    /// Mirror around either Horizontal: 'x = centerline' or Vertical: 'y = centerline'
    fn mirror(&mut self, centerline: f64, orientation: MirrorOrientation);
}
