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
    // We wrap the bytes into a cursor to make handling a whole lot easier and more terse.
    let mut cursor = BCursor::new(bytes);

    // A quick check to see if the file starts with the magic number associated with Gzip, to handle legacy Rnote files.
    let prelude = if cursor.try_seek(2)? != [0x1f, 0x8b] {
        // The first main step is to try deciphering the file's prelude, which specifies the file's version,
        // the version of Rnote it was created with, and the size of the header we'll have to parse next.
        Prelude::try_from_bytes(&mut cursor).with_context(|| "Failed to load the prelude")?
    } else {
        // Since we have a legacy Rnote file, we have to manually create a specific prelude so it can be handled later.
        Prelude::new(0, semver::Version::new(0, 13, 0), 0)
    };

    if semver::VersionReq::parse(">=0.14.0")
        .unwrap()
        .matches(&prelude.rnote_version)
    {
        RnoteFileInterface::bytes_to_engine_snapshot(cursor, prelude.header_size)

        // Example on how to upgrade in the future
        /*
        RnoteFileInterface::bytes_to_compat(cursor, prelude.header_size)
            .map(CompatV1For::<0, 14, 0>::from)
            .and_then(CompatV1For::<0, 15, 0>::try_from)
            .and_then(RnoteFileInterface::compat_to_engine_snapshot)
        */
    } else {
        let compat = CompatV1::try_from(LegacyRnoteFile::load_from_bytes(bytes)?)?;
        RnoteFileInterface::compat_to_engine_snapshot(compat)
    }
}

pub fn save_engine_snapshot_to_bytes(
    engine_snapshot: EngineSnapshot,
    compression_method: CompressionMethod,
) -> anyhow::Result<Vec<u8>> {
    RnoteFileInterface::engine_snapshot_to_bytes(engine_snapshot, compression_method)
}
