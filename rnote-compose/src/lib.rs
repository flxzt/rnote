#![warn(missing_debug_implementations)]

pub mod builders;
mod color;
pub mod helpers;
pub mod penevent;
pub mod penpath;
pub mod shapes;
pub mod style;
pub mod transform;
pub mod utils;

// Re-exports
pub use color::Color;
pub use penevent::PenEvent;
pub use penpath::PenPath;
pub use shapes::Shape;
pub use style::Style;
pub use transform::Transform;

extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;
