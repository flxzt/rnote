#![warn(missing_docs)]

//! The rough-rs crate.
//!
//! This is a port of the [Rough.js](https://roughjs.com/) javascript library to Rust.
//!
//! Rough.js is a small (<9kB gzipped) graphics library that lets you draw in a sketchy, hand-drawn-like, style.
//! The library defines primitives to draw lines, curves, arcs, polygons, circles, and ellipses. It also supports drawing SVG paths.
use std::{error::Error, io};

/// The generator module
pub mod generator;
/// The options module
pub mod options;
pub(crate) mod renderer;
/// The utils module
pub mod utils;

extern crate nalgebra as na;

/// Converting a svg::Node to a String
pub fn node_to_string<N>(node: &N) -> Result<String, Box<dyn Error>>
where
    N: svg::Node,
{
    let mut document_buffer = Vec::<u8>::new();
    svg::write(io::BufWriter::new(&mut document_buffer), node)?;
    Ok(String::from_utf8(document_buffer)?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
