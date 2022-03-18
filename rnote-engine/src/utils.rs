use std::collections::VecDeque;
use std::io::prelude::*;

use flate2::read::MultiGzDecoder;
use flate2::{Compression, GzBuilder};
use gtk4::glib;
use p2d::bounding_volume::AABB;
use rand::{Rng, SeedableRng};

use crate::strokes::inputdata::InputData;

pub const INPUT_OVERSHOOT: f64 = 30.0;

pub fn now_formatted_string() -> String {
    match glib::DateTime::now_local() {
        Ok(datetime) => match datetime.format("%F_%H-%M-%S") {
            Ok(s) => s.to_string(),
            Err(_) => String::from("1970-01-01_12-00-00"),
        },
        Err(_) => String::from("1970-01-01_12-00-00"),
    }
}

/// returns a new seed by generating a random value seeded from the old seed
pub fn seed_advance(seed: u64) -> u64 {
    let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
    rng.gen()
}

pub fn convert_value_dpi(value: f64, current_dpi: f64, target_dpi: f64) -> f64 {
    (value / current_dpi) * target_dpi
}

pub fn convert_coord_dpi(
    coord: na::Vector2<f64>,
    current_dpi: f64,
    target_dpi: f64,
) -> na::Vector2<f64> {
    (coord / current_dpi) * target_dpi
}

pub fn compress_to_gzip(to_compress: &[u8], file_name: &str) -> Result<Vec<u8>, anyhow::Error> {
    let compressed_bytes = Vec::<u8>::new();

    let mut encoder = GzBuilder::new()
        .filename(file_name)
        .comment("test")
        .write(compressed_bytes, Compression::default());

    encoder.write_all(to_compress)?;

    Ok(encoder.finish()?)
}

pub fn decompress_from_gzip(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut decoder = MultiGzDecoder::new(compressed);
    let mut bytes: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut bytes)?;

    Ok(bytes)
}

/// Filter inputdata
pub fn filter_mapped_inputdata(filter_bounds: AABB, data_entries: &mut VecDeque<InputData>) {
    data_entries.retain(|data| filter_bounds.contains_local_point(&na::Point2::from(data.pos())));
}
