// Modules
mod arrow;
/// cubic bezier curves
pub mod cubbez;
mod ellipse;
mod line;
/// quadratic bezier curves
pub mod quadbez;
mod rectangle;
mod shape;
mod shapebehaviour;

// Re-exports
pub use arrow::Arrow;
pub use cubbez::CubicBezier;
pub use ellipse::Ellipse;
pub use line::Line;
pub use quadbez::QuadraticBezier;
pub use rectangle::Rectangle;
pub use shape::Shape;
pub use shapebehaviour::ShapeBehaviour;

/// Calculate the number hitbox elems for the given length ( e.g. length of a line, curve, etc.).
fn hitbox_elems_for_shape_len(len: f64) -> i32 {
    const MAX_HITBOX_DIAGONAL: f64 = 15.0;

    ((len / MAX_HITBOX_DIAGONAL).ceil() as i32).max(1)
}
