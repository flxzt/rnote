#![warn(missing_debug_implementations)]
//#![warn(missing_docs)]

//! The rnote-engine crate is the core of rnote. It holds the strokes store, the pens, has methods for importing / exporting, rendering, etc..
//! The main entry point is the RnoteEngine struct.

pub mod audioplayer;
pub mod camera;
pub mod document;
mod drawbehaviour;
pub mod engine;
pub mod pens;
pub mod render;
pub mod store;
pub mod strokes;
pub mod surfaceflags;
pub mod utils;

// Re-exports
pub use audioplayer::AudioPlayer;
pub use camera::Camera;
pub use document::Document;
pub use drawbehaviour::DrawBehaviour;
pub use drawbehaviour::DrawOnDocBehaviour;
pub use engine::RnoteEngine;
pub use pens::PenHolder;
pub use store::StrokeStore;
pub use surfaceflags::SurfaceFlags;

extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;
#[macro_use]
extern crate lazy_static;
