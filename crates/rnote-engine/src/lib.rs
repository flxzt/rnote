#![warn(missing_debug_implementations)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::single_match)]
#![allow(clippy::derivable_impls)]
//#![warn(missing_docs)]

//! The rnote-engine crate is the core of Rnote. It holds the strokes store, the pens, has methods for importing / exporting, rendering, etc.. .
//!
//! The main entry point is the [Engine] struct.

// Modules
pub mod audioplayer;
pub mod camera;
pub mod document;
pub mod drawable;
pub mod engine;
pub mod ext;
pub mod fileformats;
pub mod image;
pub mod pens;
pub mod selectioncollision;
pub mod snap;
pub mod store;
pub mod strokes;
pub mod svg;
pub mod tasks;
#[cfg(feature = "ui")]
pub mod typst;
pub mod utils;
pub mod widgetflags;

// Re-exports
pub use audioplayer::AudioPlayer;
pub use camera::Camera;
pub use document::Document;
pub use drawable::Drawable;
pub use drawable::DrawableOnDoc;
pub use engine::Engine;
pub use image::Image;
pub use pens::PenHolder;
pub use selectioncollision::SelectionCollision;
pub use store::StrokeStore;
pub use svg::Svg;
pub use widgetflags::WidgetFlags;

// Renames
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;
