// Imports
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Constraints.
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
    /// Constrain the coordinates of a vector by the current stored constraint ratios
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

/// A constraint ratio.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "constraint_ratio")]
pub enum ConstraintRatio {
    #[serde(rename = "horizontal")]
    /// Horizontal axis.
    Horizontal,
    #[serde(rename = "vertical")]
    /// Vertical axis.
    Vertical,
    #[serde(rename = "one_to_one")]
    /// 1:1 (enables drawing circles, squares, etc.).
    OneToOne,
    #[serde(rename = "three_to_two")]
    /// 3:2.
    ThreeToTwo,
    #[serde(rename = "golden")]
    /// Golden ratio.
    Golden,
}

impl ConstraintRatio {
    /// Golden ratio.
    pub const GOLDEN_RATIO: f64 = 1.618;

    /// Constrain the coordinates of a vector by the constraint ratio.
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
