// Modules
/// Arrow
pub mod arrow;
/// Cubic-bezier curve
pub mod cubbez;
/// Ellipse
pub mod ellipse;
/// Line
pub mod line;
/// Polygon
pub mod polygon;
/// Polyline
pub mod polyline;
/// Polyline
pub mod quadbez;
/// Rectangle
pub mod rectangle;
/// Shape
pub mod shape;
/// Shapeable
pub mod shapeable;

// Re-exports
pub use arrow::Arrow;
pub use cubbez::CubicBezier;
pub use ellipse::Ellipse;
pub use line::Line;
pub use polygon::Polygon;
pub use polyline::Polyline;
pub use quadbez::QuadraticBezier;
pub use rectangle::Rectangle;
pub use shape::Shape;
pub use shapeable::Shapeable;

/// Calculate the number hitbox elems for the given length ( e.g. length of a line, curve, etc.).
fn hitbox_elems_for_shape_len(len: f64) -> u32 {
    const MAX_HITBOX_DIAGONAL: f64 = 15.0;

    (len / MAX_HITBOX_DIAGONAL).round().max(0.) as u32
}
