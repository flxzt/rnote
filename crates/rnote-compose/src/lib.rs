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
/// Event result.
pub mod eventresult;
/// module for extension traits for foreign types
pub mod ext;
/// module for pen events
pub mod penevent;
/// module for pen paths
pub mod penpath;
/// utilities for serializing / deserializing
pub mod serialize;
/// module for shapes
pub mod shapes;
/// module for splitorder
pub mod splitorder;
/// module for styles, that can be applied onto shapes
pub mod style;
/// module for transformation
pub mod transform;
/// other misc utilities
pub mod utils;
/// vertical tool options
pub mod verticaltoolconfig;

// Re-exports
pub use color::Color;
pub use constraints::Constraints;
pub use eventresult::EventResult;
pub use penevent::PenEvent;
pub use penpath::PenPath;
pub use shapes::Shape;
pub use splitorder::SplitOrder;
pub use style::Style;
pub use transform::Transform;
pub use verticaltoolconfig::VerticalToolConfig;

// Renames
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;
