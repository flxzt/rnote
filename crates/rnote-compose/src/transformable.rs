// Imports
use p2d::math::Vector2;

/// Trait for types that can be (geometrically) transformed.
pub trait Transformable {
    /// Translate (as in moves) by the given offset.
    fn translate(&mut self, offset: Vector2);
    /// Rotate by the given angle (in radians).
    fn rotate(&mut self, angle: f64, center: Vector2);
    /// Scale by the given scale-factor.
    fn scale(&mut self, scale: Vector2);
}
