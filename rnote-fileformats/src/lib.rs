#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::single_match)]

//! The rnote-fileformats crate is a helper crate for loading/saving from and to various file formats
//! used by note taking and drawing applications.
//!
//! Crates used for loading and writing:  
//! Xml: loading: [roxmltree], writing: [xmlwriter]  
//! Json: loading and writing [serde], [serde_json]  
//!
//! The following formats are currently included:
//!
//! - Rnote - `.rnote`
//! - Xournal++ - `.xopp`

// Modules
/// The Rnote `.rnote` file format.
pub mod rnoteformat;
/// The Xournal++ `.xopp` file format.
pub mod xoppformat;

// Renames
extern crate nalgebra as na;

// Imports
use roxmltree::Node;

/// The file format loader trait, implemented by `<Format>File` types.
pub trait FileFormatLoader {
    /// load from bytes.
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized;
}

/// The file format saver trait, implemented by `<Format>File` types.
pub trait FileFormatSaver {
    /// Save as bytes.
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>>;
}

/// Implemented on types that are loadable from a Xml. Using roxmltree as parser.
pub trait XmlLoadable {
    /// load from an Xml node.
    fn load_from_xml(&mut self, node: Node) -> anyhow::Result<()>;
}

/// Implemented on types that can write to a Xml. Using xmlwriter as writer.
pub trait XmlWritable {
    /// Write to the xml writer.
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter);
}

/// Implemented on types that can be represented as a Xml attribute value.
pub trait AsXmlAttributeValue {
    /// Type as Xml attribute value.
    fn as_xml_attr_value(&self) -> String;
}

/// Implemented on types that can be loaded from a Xml attribute value.
pub trait FromXmlAttributeValue {
    /// load from a Xml attribute value string.
    fn from_xml_attr_value(s: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}
