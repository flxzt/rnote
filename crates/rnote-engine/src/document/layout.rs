// Imports
use core::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "layout")]
pub enum Layout {
    #[serde(rename = "fixed_size")]
    FixedSize,
    #[serde(rename = "continuous_vertical", alias = "endless_vertical")]
    ContinuousVertical,
    #[serde(rename = "semi_infinite")]
    SemiInfinite,
    #[serde(rename = "infinite")]
    Infinite,
}

impl Default for Layout {
    fn default() -> Self {
        Self::Infinite
    }
}

impl TryFrom<u32> for Layout {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .ok_or_else(|| anyhow::anyhow!("Layout try_from::<u32>() for value {} failed", value))
    }
}

impl std::str::FromStr for Layout {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fixed-size" => Ok(Self::FixedSize),
            "continuous-vertical" => Ok(Self::ContinuousVertical),
            "semi-infinite" => Ok(Self::SemiInfinite),
            "infinite" => Ok(Self::Infinite),
            s => Err(anyhow::anyhow!(
                "Layout from_string failed, invalid name: {s}"
            )),
        }
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::FixedSize => write!(f, "fixed-size"),
            Layout::ContinuousVertical => write!(f, "continuous-vertical"),
            Layout::SemiInfinite => write!(f, "semi-infinite"),
            Layout::Infinite => write!(f, "infinite"),
        }
    }
}

impl Layout {
    /// checks if the layout is constrained in the horizontal direction
    pub fn is_fixed_width(&self) -> bool {
        matches!(self, Layout::FixedSize | Layout::ContinuousVertical)
    }
}
