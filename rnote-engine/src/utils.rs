use std::io::prelude::*;
use flate2::read::MultiGzDecoder;
use flate2::{Compression, GzBuilder};
use gtk4::glib;

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

pub mod base64 {
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = base64::encode(v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        base64::decode(base64.as_bytes()).map_err(|e| serde::de::Error::custom(e))
    }
}
