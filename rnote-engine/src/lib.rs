#![warn(missing_debug_implementations)]
//#![warn(missing_docs)]

pub mod compose;
pub mod drawbehaviour;
pub mod pens;
pub mod render;
pub mod sheet;
pub mod strokes;
pub mod strokesstate;
pub mod utils;
pub mod surfaceflags;

extern crate gstreamer as gst;
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
