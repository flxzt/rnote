use std::{error::Error, io};

pub mod generator;
pub mod math;
pub mod options;
pub mod renderer;
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
