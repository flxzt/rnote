//! Loading and saving Rnote's `.rnote` file format

// Modules
pub(crate) mod bcursor;
pub(crate) mod compression;
pub(crate) mod legacy;
pub(crate) mod prelude;
pub(crate) mod v1;

use anyhow::Context;

// Imports
use crate::fileformats::{
    FileFormatLoader, FileFormatSaver,
    rnoteformat::{bcursor::BCursor, prelude::Prelude, v1::RnoteFileV1},
};

pub type RnoteFile = RnoteFileV1;

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
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
            RnoteFileV1::load(cursor, prelude.header_size)
        } else {
            RnoteFileV1::try_from(legacy::LegacyRnoteFile::load_from_bytes(bytes)?)
        }
    }
}

impl FileFormatSaver for RnoteFile {
    #[allow(unused_variables)]
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        self.save()
    }
}
