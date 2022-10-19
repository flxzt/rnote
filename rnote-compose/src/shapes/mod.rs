/// Cubic bezier curves
pub mod cubbez;
mod ellipse;
mod line;
mod arrow;
/// Quadratic bezier curves
pub mod quadbez;
mod rectangle;
mod shape;
mod shapebehaviour;

// Re-exports
pub use cubbez::CubicBezier;
pub use ellipse::Ellipse;
pub use line::Line;
pub use arrow::Arrow;
pub use quadbez::QuadraticBezier;
pub use rectangle::Rectangle;
pub use shape::Shape;
pub use shapebehaviour::ShapeBehaviour;

/// Calculates the number hitbox elems for the given length ( e.g. length of a line, curve, etc.)
fn hitbox_elems_for_shape_len(len: f64) -> i32 {
    // Maximum hitbox diagonal length
    const MAX_HITBOX_DIAGONAL: f64 = 15.0;

    ((len / MAX_HITBOX_DIAGONAL).ceil() as i32).max(1)
}
