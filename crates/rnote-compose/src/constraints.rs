// Imports
use p2d::math::Vector2;
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
    pub fn constrain(&self, pos: Vector2) -> Vector2 {
        if !self.enabled {
            return pos;
        }
        self.ratios
            .iter()
            .map(|ratio| ((ratio.constrain(pos) - pos).length(), ratio.constrain(pos)))
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
    pub fn constrain(&self, pos: Vector2) -> Vector2 {
        let x = pos.x;
        let y = pos.y;

        match self {
            ConstraintRatio::Horizontal => Vector2::new(x, 0.),
            ConstraintRatio::Vertical => Vector2::new(0., y),
            ConstraintRatio::OneToOne => {
                if x.abs() > y.abs() {
                    Vector2::new(x, x.abs() * y.signum())
                } else {
                    Vector2::new(y.abs() * x.signum(), y)
                }
            }
            ConstraintRatio::ThreeToTwo => {
                if x.abs() > y.abs() {
                    Vector2::new(x, (x / 1.5).abs() * y.signum())
                } else {
                    Vector2::new((y / 1.5).abs() * x.signum(), y)
                }
            }
            ConstraintRatio::Golden => {
                if x.abs() > y.abs() {
                    Vector2::new(x, (x / Self::GOLDEN_RATIO).abs() * y.signum())
                } else {
                    Vector2::new((y / Self::GOLDEN_RATIO).abs() * x.signum(), y)
                }
            }
        }
    }
}
