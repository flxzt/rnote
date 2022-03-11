#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

//! The rnote-fileformats crate is a helper crate for loading / saving from and to various file formats used by note taking and drawing applications.
//!
//! Crates used for loading and writing:  
//! XML: loading: `roxmltree`, writing: `xmlwriter`  
//! Json: loading and writing `serde`, `serde-json`  
//!
//! it includes the following formats:
//!
//! | Format | file ending | XML | JSON | info |
//! | --- | --- | --- | --- | --- |
//! | Rnote | .rnote | - | native | see <https://github.com/flxzt/rnote> |
//! | Xournal++ | .xopp | native | x | see <https://github.com/xournalpp/xournalpp> |

/// The Rnote `.rnote` file format
pub mod rnoteformat;
/// The Xournal++ `.xopp` file format
pub mod xoppformat;

extern crate nalgebra as na;

use std::io::{Read, Write};

use flate2::read::MultiGzDecoder;
use flate2::{Compression, GzBuilder};
use roxmltree::Node;

/// The file format loader trait, implemented by <Format>File types
pub trait FileFormatLoader {
    /// load type from bytes
    fn load_from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

/// The file format saver trait, implemented by <Format>File types
pub trait FileFormatSaver {
    /// Save type as bytes
    fn save_as_bytes(&self, file_name: &str) -> Result<Vec<u8>, anyhow::Error>;
}

/// Implemented on types that are loadable from a XML. Using roxmltree as parser
pub trait XmlLoadable {
    /// load from an XML node
    fn load_from_xml(&mut self, node: Node) -> Result<(), anyhow::Error>;
}

/// Implemented on types that can write to a XML. Using xmlwriter as writer
pub trait XmlWritable {
    /// Write to the xml writer
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter);
}

/// Implemented on types that are represented as a XML attribute value
pub trait AsXmlAttributeValue {
    /// Type as XML attribute value
    fn as_xml_attr_value(&self) -> String;
}

/// Implemented on types that can be loaded from a XML attribute value
pub trait FromXmlAttributeValue {
    /// loading from a XML attribute value str
    fn from_xml_attr_value(s: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

/// Compress bytes with gzip
pub fn compress_to_gzip(to_compress: &[u8], file_name: &str) -> Result<Vec<u8>, anyhow::Error> {
    let compressed_bytes = Vec::<u8>::new();

    let mut encoder = GzBuilder::new()
        .filename(file_name)
        .comment("test")
        .write(compressed_bytes, Compression::default());

    encoder.write_all(to_compress)?;

    Ok(encoder.finish()?)
}

/// Decompress from gzip
pub fn decompress_from_gzip(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut decoder = MultiGzDecoder::new(compressed);
    let mut bytes: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut bytes)?;

    Ok(bytes)
}
