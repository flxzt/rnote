/// Cubic bezier curves
pub mod cubbez;
mod ellipse;
mod line;
/// Quadratic bezier curves
pub mod quadbez;
mod rectangle;
mod shape;
mod shapebehaviour;

// Re-exports
pub use cubbez::CubicBezier;
pub use ellipse::Ellipse;
pub use line::Line;
pub use quadbez::QuadraticBezier;
pub use rectangle::Rectangle;
pub use shape::Shape;
pub use shapebehaviour::ShapeBehaviour;

/// Calculatese the hitbox steps for the given length
pub(super) fn hitbox_elems_for_len(len: f64) -> i32 {
    if len < 5.0 {
        1
    } else if len < 10.0 {
        4
    } else if len < 20.0 {
        8
    } else {
        // capping the no of elements for bigger len's, avoiding huge amounts of hitboxes for huge strokes that are drawn when zoomed out
        10
    }
}
