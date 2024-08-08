//! Loading and saving Rnote's `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`,
//! where X: semver major, Y: semver minor version.
//!
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod maj0min5patch8;
pub(crate) mod maj0min5patch9;
pub(crate) mod maj0min6;
pub(crate) mod maj0min9;

// Imports
use self::maj0min5patch8::RnoteFileMaj0Min5Patch8;
use self::maj0min5patch9::RnoteFileMaj0Min5Patch9;
use self::maj0min6::RnoteFileMaj0Min6;
use self::maj0min9::RnoteFileMaj0Min9;
use super::{FileFormatLoader, FileFormatSaver};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Decompress from gzip.
fn decompress_from_gzip(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    // Optimization for the gzip format, defined by RFC 1952
    // capacity of the vector defined by the size of the uncompressed data
    // given in little endian format, by the last 4 bytes of "compressed"
    //
    //   ISIZE (Input SIZE)
    //     This contains the size of the original (uncompressed) input data modulo 2^32.
    let mut bytes: Vec<u8> = {
        let mut decompressed_size: [u8; 4] = [0; 4];
        let idx_start = compressed
            .len()
            .checked_sub(4)
            // only happens if the file has less than 4 bytes
            .ok_or(
                anyhow::anyhow!("Not a valid gzip-compressed file")
                    .context("Failed to get the size of the decompressed data"),
            )?;
        decompressed_size.copy_from_slice(&compressed[idx_start..]);
        // u32 -> usize to avoid issues on 32-bit architectures
        // also more reasonable since the uncompressed size is given by 4 bytes
        Vec::with_capacity(u32::from_le_bytes(decompressed_size) as usize)
    };

    let mut decoder = flate2::read::MultiGzDecoder::new(compressed);
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Decompress bytes with zstd
pub fn decompress_from_zstd(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    // Optimization for the zstd format, less pretty than for gzip but this does shave off a bit of time
    // https://github.com/facebook/zstd/blob/dev/doc/zstd_compression_format.md#frame_header
    let mut bytes: Vec<u8> = {
        let frame_header_descriptor = compressed.get(4).ok_or(
            anyhow::anyhow!("Not a valid zstd-compressed file")
                .context("Failed to get the frame header descriptor of the file"),
        )?;

        let frame_content_size_flag = frame_header_descriptor >> 6;
        let single_segment_flag = (frame_header_descriptor >> 5) & 1;
        let did_field_size = {
            let dictionary_id_flag = frame_header_descriptor & 11;
            if dictionary_id_flag == 3 {
                4
            } else {
                dictionary_id_flag
            }
        };
        // frame header size start index
        let fcs_sidx = (6 + did_field_size - single_segment_flag) as usize;
        // magic number: 4 bytes + window descriptor: 1 byte if single segment flag is not set + frame header descriptor: 1 byte + dict. field size: 0-4 bytes
        // testing suggests that dicts. don't improve the compression ratio and worsen writing/reading speeds, therefore they won't be used
        // thus this part could be simplified, but wouldn't strictly adhere to zstd standards

        match frame_content_size_flag {
            // not worth it to potentially pre-allocate a maximum of 255 bytes
            0 => Vec::new(),
            1 => {
                let mut decompressed_size: [u8; 2] = [0; 2];
                decompressed_size.copy_from_slice(
                    compressed.get(fcs_sidx..fcs_sidx + 2).ok_or(
                        anyhow::anyhow!("Not a valid zstd-compressed file").context(
                            "Failed to get the uncompressed size of the data from two bytes",
                        ),
                    )?,
                );
                // 256 offset
                Vec::with_capacity(usize::from(256 + u16::from_le_bytes(decompressed_size)))
            }
            2 => {
                let mut decompressed_size: [u8; 4] = [0; 4];
                decompressed_size.copy_from_slice(
                    compressed.get(fcs_sidx..fcs_sidx + 4).ok_or(
                        anyhow::anyhow!("Not a valid zstd-compressed file").context(
                            "Failed to get the uncompressed size of the data from four bytes",
                        ),
                    )?,
                );
                Vec::with_capacity(
                    u32::from_le_bytes(decompressed_size)
                        .try_into()
                        .unwrap_or(usize::MAX),
                )
            }
            // in practice this should not happen, as a rnote file being larger than 4 GiB is very unlikely
            3 => {
                let mut decompressed_size: [u8; 8] = [0; 8];
                decompressed_size.copy_from_slice(compressed.get(fcs_sidx..fcs_sidx + 8).ok_or(
                    anyhow::anyhow!("Not a valid zstd-compressed file").context(
                        "Failed to get the uncompressed size of the data from eight bytes",
                    ),
                )?);
                Vec::with_capacity(
                    u64::from_le_bytes(decompressed_size)
                        .try_into()
                        .unwrap_or(usize::MAX),
                )
            }
            // unreachable since our u8 is formed by only 2 bits
            4.. => unreachable!(),
        }
    };
    let mut decoder = zstd::Decoder::new(compressed)?;
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Compress bytes with zstd
pub fn compress_to_zstd(to_compress: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut encoder = zstd::Encoder::new(Vec::<u8>::new(), 9)?;
    encoder.set_pledged_src_size(Some(to_compress.len() as u64))?;
    encoder.include_contentsize(true)?;
    if let Ok(num_workers) = std::thread::available_parallelism() {
        encoder.multithread(num_workers.get() as u32)?;
    }
    encoder.write_all(to_compress)?;
    Ok(encoder.finish()?)
}

/// The rnote file wrapper.
///
/// Used to extract and match the version up front, before deserializing the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rnotefile_wrapper")]
struct RnotefileWrapper {
    #[serde(rename = "version")]
    version: semver::Version,
    #[serde(rename = "data")]
    data: ijson::IValue,
}

/// The Rnote file in the newest format version.
///
/// This struct exists to allow for upgrading older versions before loading the file in.
pub type RnoteFile = RnoteFileMaj0Min9;

impl RnoteFile {
    pub const SEMVER: &'static str = crate::utils::crate_version();
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let wrapper = serde_json::from_slice::<RnotefileWrapper>(&{
            // zstd magic number
            if bytes.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
                decompress_from_zstd(bytes)?
            }
            // gzip ID1 and ID2
            else if bytes.starts_with(&[0x1f, 0x8b]) {
                decompress_from_gzip(bytes)?
            } else {
                Err(anyhow::anyhow!(
                    "Unknown compression format, expected zstd or gzip"
                ))?
            }
        })
        .context("deserializing RnotefileWrapper from bytes failed.")?;

        // Conversions for older file format versions happen here
        if semver::VersionReq::parse(">=0.9.0")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min9>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min9 failed.")
        } else if semver::VersionReq::parse(">=0.5.10")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min6>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min6 failed.")
                .and_then(RnoteFileMaj0Min9::try_from)
                .context("converting RnoteFileMaj0Min6 to newest file version failed.")
        } else if semver::VersionReq::parse(">=0.5.9")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min5Patch9>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min5Patch9 failed.")
                .and_then(RnoteFileMaj0Min6::try_from)
                .and_then(RnoteFileMaj0Min9::try_from)
                .context("converting RnoteFileMaj0Min5Patch9 to newest file version failed.")
        } else if semver::VersionReq::parse(">=0.5.0")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min5Patch8>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min5Patch8 failed")
                .and_then(RnoteFileMaj0Min5Patch9::try_from)
                .and_then(RnoteFileMaj0Min6::try_from)
                .and_then(RnoteFileMaj0Min9::try_from)
                .context("converting RnoteFileMaj0Min5Patch8 to newest file version failed.")
        } else {
            Err(anyhow::anyhow!(
                "failed to load rnote file from bytes, unsupported version: {}.",
                wrapper.version
            ))
        }
    }
}

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, _file_name: &str) -> anyhow::Result<Vec<u8>> {
        let wrapper = RnotefileWrapper {
            version: semver::Version::parse(Self::SEMVER).unwrap(),
            data: ijson::to_value(self).context("converting RnoteFile to JSON value failed.")?,
        };
        let compressed = compress_to_zstd(
            &serde_json::to_vec(&wrapper).context("Serializing RnoteFileWrapper failed.")?,
        )
        .context("compressing bytes failed.")?;

        Ok(compressed)
    }
}
