/// cubic bezier builder
pub mod cubbezbuilder;
/// ellipse builder
pub mod ellipsebuilder;
/// foci and point ellipse builder
pub mod fociellipsebuilder;
/// line builder
pub mod linebuilder;
/// pen path builder
pub mod penpathbuilder;
/// quadratic bezier builder
pub mod quadbezbuilder;
/// rectangle builder
pub mod rectanglebuilder;
/// shape builder behaviour
pub mod shapebuilderbehaviour;

use std::collections::HashSet;

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

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "shape_type")]
/// A choice for a shape builder type
pub enum ShapeBuilderType {
    #[serde(rename = "line")]
    /// A line builder
    Line = 0,
    #[serde(rename = "rectangle")]
    /// A rectangle builder
    Rectangle,
    #[serde(rename = "ellipse")]
    /// An ellipse builder
    Ellipse,
    #[serde(rename = "foci_ellipse")]
    /// A foci ellipse builder
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

impl TryFrom<u32> for ShapeBuilderType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "ShapeBuilderType try_from::<u32>() for value {} failed",
                value
            )
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
    /// constrain the coordinates of a vector by the current stored contraint ratios
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
