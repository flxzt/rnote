/// ellipse builder
pub mod ellipsebuilder;
/// foci and point ellipse builder
pub mod fociellipsebuilder;
/// line builder
pub mod linebuilder;
/// The pen path builder.
pub mod penpathbuilder;
/// Rectangle builder
pub mod rectanglebuilder;
mod shapebuilderbehaviour;

// Re-exports
pub use ellipsebuilder::EllipseBuilder;
pub use fociellipsebuilder::FociEllipseBuilder;
pub use linebuilder::LineBuilder;
pub use penpathbuilder::PenPathBuilder;
pub use rectanglebuilder::RectangleBuilder;
pub use shapebuilderbehaviour::ShapeBuilderBehaviour;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "shape_type")]
/// A choice of a shape builder type
pub enum ShapeBuilderType {
    #[serde(rename = "line")]
    /// A line shape
    Line,
    #[serde(rename = "rectangle")]
    /// A rectangle shape
    Rectangle,
    #[serde(rename = "ellipse")]
    /// An ellipse shape
    Ellipse,
    #[serde(rename = "foci_ellipse")]
    /// An ellipse shape
    FociEllipse,
}

impl Default for ShapeBuilderType {
    fn default() -> Self {
        Self::Line
    }
}
