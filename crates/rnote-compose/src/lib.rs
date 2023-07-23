#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::single_match)]

//! the rnote-compose crate provides rnote with building blocks for creating, styling, composing, drawing, transforming shapes and paths.

// Modules
/// module for shape builders
pub mod builders;
/// colors
pub mod color;
/// constraints
pub mod constraints;
/// module for helper traits that extend foreign types
pub mod helpers;
/// module for pen helpers
pub mod penevents;
/// module for pen paths
pub mod penpath;
/// utilities for serializing / deserializing
pub mod serialize;
/// module for shapes
pub mod shapes;
/// module for styles, that can be applied onto shapes
pub mod style;
/// module for transformation
pub mod transform;
/// other misc utilities
pub mod utils;

// Re-exports
pub use color::Color;
pub use constraints::Constraints;
pub use penpath::PenPath;
pub use shapes::Shape;
pub use style::Style;
pub use transform::Transform;

// Renames
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;
