// Imports
use crate::fileformats::xoppformat;
use anyhow::Context;
use futures::{AsyncReadExt, AsyncWriteExt};
use geo::line_string;
use p2d::bounding_volume::Aabb;
use rnote_compose::Color;
use std::ops::Range;

pub const fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn color_from_xopp(xopp_color: xoppformat::XoppColor) -> Color {
    Color {
        r: f64::from(xopp_color.red) / 255.0,
        g: f64::from(xopp_color.green) / 255.0,
        b: f64::from(xopp_color.blue) / 255.0,
        a: f64::from(xopp_color.alpha) / 255.0,
    }
}

pub fn xoppcolor_from_color(color: Color) -> xoppformat::XoppColor {
    xoppformat::XoppColor {
        red: (color.r * 255.0).floor() as u8,
        green: (color.g * 255.0).floor() as u8,
        blue: (color.b * 255.0).floor() as u8,
        alpha: (color.a * 255.0).floor() as u8,
    }
}

pub fn now_formatted_string() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string()
}

pub fn doc_pages_files_names(file_stem_name: String, i: usize) -> String {
    file_stem_name + &format!(" - Page {i:02}")
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

#[cfg(feature = "ui")]
pub fn transform_to_gsk(transform: &rnote_compose::Transform) -> gtk4::gsk::Transform {
    gtk4::gsk::Transform::new().matrix(&gtk4::graphene::Matrix::from_2d(
        transform.affine[(0, 0)],
        transform.affine[(1, 0)],
        transform.affine[(0, 1)],
        transform.affine[(1, 1)],
        transform.affine[(0, 2)],
        transform.affine[(1, 2)],
    ))
}

/// Convert an [Aabb] to [`geo::Polygon<f64>`]
pub fn p2d_aabb_to_geo_polygon(aabb: Aabb) -> geo::Polygon<f64> {
    let line_string = line_string![
        (x: aabb.mins[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.mins[1]),
    ];
    geo::Polygon::new(line_string, vec![])
}

pub fn positive_range<I>(first: I, second: I) -> Range<I>
where
    I: PartialOrd,
{
    if first < second {
        first..second
    } else {
        second..first
    }
}

/// (De)Serialize a [glib::Bytes] with base64 encoding
pub mod glib_bytes_base64 {
    use serde::{Deserializer, Serializer};

    /// Serialize a [`Vec<u8>`] as base64 encoded
    pub fn serialize<S: Serializer>(v: &glib::Bytes, s: S) -> Result<S::Ok, S::Error> {
        rnote_compose::serialize::sliceu8_base64::serialize(v, s)
    }

    /// Deserialize base64 encoded [glib::Bytes]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<glib::Bytes, D::Error> {
        rnote_compose::serialize::sliceu8_base64::deserialize(d).map(glib::Bytes::from_owned)
    }
}

pub async fn atomic_save_to_file<Q>(filepath: Q, bytes: &[u8]) -> anyhow::Result<()>
where
    Q: AsRef<std::path::Path>,
{
    let filepath = filepath.as_ref().to_owned();

    // checks that the extension is not already 'tmp'
    if filepath
        .extension()
        .ok_or_else(|| anyhow::anyhow!("Specified filepath does not have an extension"))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("The extension of the specified filepath is invalid"))?
        == "tmp"
    {
        Err(anyhow::anyhow!("The extension of the file cannot be 'tmp'"))?;
    }

    let tmp_filepath = filepath.with_extension("tmp");

    let file_write_operation = async {
        let mut write_file = async_fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_filepath)
            .await
            .with_context(|| {
                format!(
                    "Failed to create/open/truncate tmp file with path '{}'",
                    tmp_filepath.display()
                )
            })?;
        write_file.write_all(bytes).await.with_context(|| {
            format!(
                "Failed to write to tmp file with path '{}'",
                tmp_filepath.display()
            )
        })?;
        write_file.sync_all().await.with_context(|| {
            format!(
                "Failed to sync tmp file with path '{}'",
                tmp_filepath.display()
            )
        })?;

        Ok::<(), anyhow::Error>(())
    };
    file_write_operation.await?;

    let file_check_operation = async {
        let internal_checksum = crc32fast::hash(bytes);

        let mut read_file = async_fs::OpenOptions::new()
            .read(true)
            .open(&tmp_filepath)
            .await
            .with_context(|| {
                format!(
                    "Failed to open/read tmp file with path '{}'",
                    &tmp_filepath.display()
                )
            })?;
        let mut data: Vec<u8> = Vec::with_capacity(bytes.len());
        read_file.read_to_end(&mut data).await?;
        let external_checksum = crc32fast::hash(&data);

        if internal_checksum != external_checksum {
            return Err(anyhow::anyhow!(
                "Mismatch between the internal and external checksums, temporary file most likely corrupted"
            ));
        }

        Ok::<(), anyhow::Error>(())
    };
    file_check_operation.await?;

    let file_swap_operation = async {
        async_fs::rename(&tmp_filepath, &filepath)
            .await
            .context("Failed to rename the temporary file into the original one")?;

        Ok::<(), anyhow::Error>(())
    };
    file_swap_operation.await?;

    Ok(())
}
