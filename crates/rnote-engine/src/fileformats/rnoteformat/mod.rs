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
            v1::{CompatBridgeV1, RnoteFileInterfaceV1},
        },
    },
};
use anyhow::Context;

type RnoteFileInterface = RnoteFileInterfaceV1;

pub fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<EngineSnapshot> {
    let mut cursor = BCursor::new(bytes);

    let prelude = if cursor.try_seek(2)? != [0x1f, 0x8b] {
        Prelude::try_from_bytes(&mut cursor).with_context(|| "Failed to load the prelude")?
    } else {
        // We create a "phony" prelude if the file is found to be entirely compressed with gzip (meaning it's a legacy Rnote file)
        Prelude::new(0, semver::Version::new(0, 13, 0), 0)
    };

    if semver::VersionReq::parse(">=0.14.0")
        .unwrap()
        .matches(&prelude.rnote_version)
    {
        RnoteFileInterface::bytes_to_engine_snapshot(cursor, prelude.header_size)

        // Example on how to upgrade in the future
        /*
        RnoteFileInterface::bytes_to_sc_bridge(cursor, prelude.header_size)
            .map(CompatBridgeV1Ver::<0, 14, 0>::from)
            .and_then(CompatBridgeV1Ver::<0, 15, 0>::try_from)
            .map(CompatBridgeV1::from)
            .and_then(RnoteFileInterface::bridge_to_engine_snapshot)
        */
    } else {
        let bridge = CompatBridgeV1::try_from(LegacyRnoteFile::load_from_bytes(bytes)?)?;
        RnoteFileInterface::bridge_to_engine_snapshot(bridge)
    }
}

pub fn save_to_bytes(
    engine_snapshot: EngineSnapshot,
    compression_method: CompressionMethod,
) -> anyhow::Result<Vec<u8>> {
    RnoteFileInterface::engine_snapshot_to_bytes(engine_snapshot, compression_method)
}
