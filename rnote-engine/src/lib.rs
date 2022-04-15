#![warn(missing_debug_implementations)]
//#![warn(missing_docs)]

//! The rnote-engine crate is the core of rnote. It holds the strokes store, the pens, has methods for importing / exporting, rendering, etc..
//! The main entry point is the RnoteEngine struct.

pub mod camera;
mod drawbehaviour;
pub mod engine;
pub mod pens;
pub mod render;
pub mod sheet;
pub mod store;
pub mod strokes;
pub mod surfaceflags;
pub mod utils;

// Re-exports
pub use camera::Camera;
pub use drawbehaviour::DrawBehaviour;
pub use drawbehaviour::DrawOnSheetBehaviour;
pub use engine::RnoteEngine;
pub use pens::PenHolder;
pub use sheet::Sheet;
pub use store::StrokeStore;
pub use surfaceflags::SurfaceFlags;

extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
