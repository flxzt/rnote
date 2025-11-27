//! Loading and saving Rnote's `.rnote` file format

// Modules
pub(crate) mod bcursor;
pub(crate) mod compression;
pub(crate) mod legacy;
pub(crate) mod prelude;
pub(crate) mod v1;

// Re-exports
pub use compression::{
    CompressionMethod, DEFAULT_ZSTD_COMPRESSION_INTEGER, ZstdCompressionInteger,
};
pub use legacy::LegacyRnoteFile;

// Imports
use crate::{
    engine::EngineSnapshot,
    fileformats::{
        FileFormatLoader,
        rnoteformat::{
            bcursor::BCursor,
            prelude::Prelude,
            v1::{CompatV1, RnoteFileInterfaceV1},
        },
    },
};
use anyhow::Context;

type RnoteFileInterface = RnoteFileInterfaceV1;

/// This function attempts to load an `EngineSnapshot` from bytes.
pub fn load_engine_snapshot_from_bytes(bytes: &[u8]) -> anyhow::Result<EngineSnapshot> {
    // We wrap the bytes in a cursor to make keeping track of what we have processed much easier.
    let mut cursor = BCursor::new(bytes);

    // We check that the file isn't of the legacy type, which is indicated by the presence of the magic number of Gzip at the start of the file.
    let prelude = if cursor.try_seek(2)? != [0x1f, 0x8b] {
        // Not a legacy file, so we try loading the prelude.
        Prelude::try_from_bytes(&mut cursor).with_context(|| "Failed to load the prelude")?
    } else {
        // Legacy file, so we manually create a specific prelude so it can be handled later.
        Prelude::new(0, semver::Version::new(0, 0, 0), 0)
    };

    match prelude.file_version {
        1 => {
            RnoteFileInterface::bytes_to_engine_snapshot(cursor, prelude.header_size)

            // Template for a future version change that requires an upgrade.
            /*
            if semver::VersionReq::parse(">=0.15.0")
                .unwrap()
                .matches(&prelude.rnote_version)
            {
                RnoteFileInterface::bytes_to_engine_snapshot(cursor, prelude.header_size)
            } else {
                RnoteFileInterface::bytes_to_compat(cursor, prelude.header_size)
                    .map(CompatV1For::<0, 14, 0>::from)
                    .and_then(CompatV1For::<0, 15, 0>::try_from)
                    .and_then(RnoteFileInterface::compat_to_engine_snapshot)
            }
            */
        }
        0 => RnoteFileInterface::compat_to_engine_snapshot(CompatV1::try_from(
            LegacyRnoteFile::load_from_bytes(bytes)?,
        )?),
        _ => unreachable!(),
    }
}

pub fn save_engine_snapshot_to_bytes(
    engine_snapshot: EngineSnapshot,
    compression_method: CompressionMethod,
) -> anyhow::Result<Vec<u8>> {
    RnoteFileInterface::engine_snapshot_to_bytes(engine_snapshot, compression_method)
}
