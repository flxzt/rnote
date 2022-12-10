/// 2D coordinate system builder
pub mod coordsystem2dbuilder;
/// 3D coordinate system builder
pub mod coordsystem3dbuilder;
/// cubic bezier builder
pub mod cubbezbuilder;
/// ellipse builder
pub mod ellipsebuilder;
/// foci and point ellipse builder
pub mod fociellipsebuilder;
/// grid builder
pub mod gridbuilder;
/// line builder
pub mod linebuilder;
/// pen path builder behaviour
pub mod penpathbuilderbehaviour;
/// the regular pen path builder, using bezier curves to interpolate between input elements.
pub mod penpathcurvedbuilder;
/// modeled pen path builder, uses ink-stroke-modeler for smooth paths with advanced algorithms and its predictor to reduce input latency
pub mod penpathmodeledbuilder;
/// simple pen path builder, only produces line segments
pub mod penpathsimplebuilder;
/// quadratic bezier builder
pub mod quadbezbuilder;
/// 2D single quadrant coordinate system builder
pub mod quadrantcoordsystem2dbuilder;
/// rectangle builder
pub mod rectanglebuilder;
/// shape builder behaviour
pub mod shapebuilderbehaviour;

// Re-exports
pub use coordsystem2dbuilder::CoordSystem2DBuilder;
pub use coordsystem3dbuilder::CoordSystem3DBuilder;
pub use cubbezbuilder::CubBezBuilder;
pub use ellipsebuilder::EllipseBuilder;
pub use fociellipsebuilder::FociEllipseBuilder;
pub use gridbuilder::GridBuilder;
pub use linebuilder::LineBuilder;
pub use penpathbuilderbehaviour::PenPathBuilderBehaviour;
pub use penpathbuilderbehaviour::PenPathBuilderCreator;
pub use penpathbuilderbehaviour::PenPathBuilderProgress;
pub use penpathcurvedbuilder::PenPathCurvedBuilder;
pub use penpathmodeledbuilder::PenPathModeledBuilder;
pub use penpathsimplebuilder::PenPathSimpleBuilder;
pub use quadbezbuilder::QuadBezBuilder;
pub use quadrantcoordsystem2dbuilder::QuadrantCoordSystem2DBuilder;
pub use rectanglebuilder::RectangleBuilder;
pub use shapebuilderbehaviour::ShapeBuilderBehaviour;
pub use shapebuilderbehaviour::ShapeBuilderCreator;
pub use shapebuilderbehaviour::ShapeBuilderProgress;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "shapebuilder_type")]
/// A choice for a shape builder type
pub enum ShapeBuilderType {
    /// A line builder
    #[serde(rename = "line")]
    Line = 0,
    /// A rectangle builder
    #[serde(rename = "rectangle")]
    Rectangle,
    /// A grid
    #[serde(rename = "grid")]
    Grid,
    /// A 2D coordinate system builder
    #[serde(rename = "coord_system_2d")]
    CoordSystem2D,
    /// A 3D coordinate system builder
    #[serde(rename = "coord_system_3d")]
    CoordSystem3D,
    /// A 2D single quadrant coordinate system builder
    #[serde(rename = "quadrant_coord_system_2d")]
    QuadrantCoordSystem2D,
    /// An ellipse builder
    #[serde(rename = "ellipse")]
    Ellipse,
    /// A foci ellipse builder
    #[serde(rename = "foci_ellipse")]
    FociEllipse,
    /// An quadbez builder
    #[serde(rename = "quadbez")]
    QuadBez,
    /// An cubic bezier builder
    #[serde(rename = "cubbez")]
    CubBez,
}

impl Default for ShapeBuilderType {
    fn default() -> Self {
        Self::Line
    }
}

impl TryFrom<u32> for ShapeBuilderType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("ShapeBuilderType try_from::<u32>() for value {value} failed"))
    }
}

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "penpathbuilder_type")]
/// A choice for a pen path builder type
pub enum PenPathBuilderType {
    #[serde(rename = "simple")]
    /// the simple pen path builder
    Simple = 0,
    #[serde(rename = "curved")]
    /// the curved pen path builder
    Curved,
    #[serde(rename = "modeled")]
    /// the modeled pen path builder
    Modeled,
}

impl Default for PenPathBuilderType {
    fn default() -> Self {
        Self::Modeled
    }
}

impl TryFrom<u32> for PenPathBuilderType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).with_context(|| {
            format!("PenPathBuilderType try_from::<u32>() for value {value} failed",)
        })
    }
}

/// constraints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "constraints")]
pub struct Constraints {
    /// Whether constraints are enabled
    #[serde(rename = "enabled")]
    pub enabled: bool,
    /// stores the constraint ratios
    #[serde(rename = "ratios")]
    pub ratios: HashSet<ConstraintRatio>,
}

impl Constraints {
    /// constrain the coordinates of a vector by the current stored constraint ratios
    pub fn constrain(&self, pos: na::Vector2<f64>) -> na::Vector2<f64> {
        if !self.enabled {
            return pos;
        }
        self.ratios
            .iter()
            .map(|ratio| ((ratio.constrain(pos) - pos).norm(), ratio.constrain(pos)))
            .reduce(|(acc_dist, acc_posi), (dist, posi)| {
                if dist <= acc_dist {
                    (dist, posi)
                } else {
                    (acc_dist, acc_posi)
                }
            })
            .map(|(_d, p)| p)
            .unwrap_or(pos)
    }
}

/// the constraint ratio
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "constraint_ratio")]
pub enum ConstraintRatio {
    #[serde(rename = "horizontal")]
    /// Horizontal axis
    Horizontal,
    #[serde(rename = "vertical")]
    /// Vertical axis
    Vertical,
    #[serde(rename = "one_to_one")]
    /// 1:1 (enables drawing circles, squares, etc.)
    OneToOne,
    #[serde(rename = "three_to_two")]
    /// 3:2
    ThreeToTwo,
    #[serde(rename = "golden")]
    /// Golden ratio
    Golden,
}

impl ConstraintRatio {
    /// the golden ratio
    pub const GOLDEN_RATIO: f64 = 1.618;

    /// Constrain the coordinates of a vector by the constraint ratio
    pub fn constrain(&self, pos: na::Vector2<f64>) -> na::Vector2<f64> {
        let dx = pos[0];
        let dy = pos[1];

        match self {
            ConstraintRatio::Horizontal => na::vector![dx, 0.0],
            ConstraintRatio::Vertical => na::vector![0.0, dy],
            ConstraintRatio::OneToOne => {
                if dx.abs() > dy.abs() {
                    na::vector![dx, dx.abs() * dy.signum()]
                } else {
                    na::vector![dy.abs() * dx.signum(), dy]
                }
            }
            ConstraintRatio::ThreeToTwo => {
                if dx.abs() > dy.abs() {
                    na::vector![dx, (dx / 1.5).abs() * dy.signum()]
                } else {
                    na::vector![(dy / 1.5).abs() * dx.signum(), dy]
                }
            }
            ConstraintRatio::Golden => {
                if dx.abs() > dy.abs() {
                    na::vector![dx, (dx / Self::GOLDEN_RATIO).abs() * dy.signum()]
                } else {
                    na::vector![(dy / Self::GOLDEN_RATIO).abs() * dx.signum(), dy]
                }
            }
        }
    }
}
