/// Cubi bezier builder
pub mod cubbezbuilder;
/// ellipse builder
pub mod ellipsebuilder;
/// foci and point ellipse builder
pub mod fociellipsebuilder;
/// line builder
pub mod linebuilder;
/// The pen path builder.
pub mod penpathbuilder;
/// Quadratic bezier builder
pub mod quadbezbuilder;
/// Rectangle builder
pub mod rectanglebuilder;
/// shape builder behaviour
pub mod shapebuilderbehaviour;

// Re-exports
pub use cubbezbuilder::CubBezBuilder;
pub use ellipsebuilder::EllipseBuilder;
pub use fociellipsebuilder::FociEllipseBuilder;
pub use linebuilder::LineBuilder;
pub use penpathbuilder::PenPathBuilder;
pub use quadbezbuilder::QuadBezBuilder;
pub use rectanglebuilder::RectangleBuilder;
pub use shapebuilderbehaviour::ShapeBuilderBehaviour;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "shape_type")]
/// A choice of a shape builder type
pub enum ShapeBuilderType {
    #[serde(rename = "line")]
    /// A line builder
    Line,
    #[serde(rename = "rectangle")]
    /// A rectangle builder
    Rectangle,
    #[serde(rename = "ellipse")]
    /// An ellipse builder
    Ellipse,
    #[serde(rename = "foci_ellipse")]
    /// An foci ellipse builder
    FociEllipse,
    #[serde(rename = "quadbez")]
    /// An quadbez builder
    QuadBez,
    #[serde(rename = "cubbez")]
    /// An cubic bezier builder
    CubBez,
}

impl Default for ShapeBuilderType {
    fn default() -> Self {
        Self::Line
    }
}

#[derive(Debug, Clone)]
pub enum ConstraintRatio {
    Disabled,
    OneToOne,
    ThreeToTwo,
    Golden,
}

impl ConstraintRatio {
    fn constrain(&self, pos: na::Vector2<f64>) -> na::Vector2<f64> {
        let max = pos.max();
        let dx = *pos.index((0, 0));
        let dy = *pos.index((1, 0));
        match self {
            ConstraintRatio::Disabled => pos,
            ConstraintRatio::OneToOne => na::vector![max, max],
            ConstraintRatio::ThreeToTwo => {
                if dx > dy {
                    na::vector![dx, dx / 1.5]
                } else {
                    na::vector![dy / 1.5, dy]
                }
            }
            ConstraintRatio::Golden => {
                if dx > dy {
                    na::vector![dx, dx / 1.618]
                } else {
                    na::vector![dy / 1.618, dy]
                }
            }
        }
    }
}
