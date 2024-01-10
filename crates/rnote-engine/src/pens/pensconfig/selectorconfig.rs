// Imports
use serde::{Deserialize, Serialize};

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "selector_style")]
pub enum SelectorStyle {
    #[serde(rename = "polygon")]
    Polygon = 0,
    #[serde(rename = "rectangle")]
    Rectangle,
    #[serde(rename = "single", alias = "apiece")]
    Single,
    #[serde(rename = "intersectingpath")]
    IntersectingPath,
}

impl Default for SelectorStyle {
    fn default() -> Self {
        Self::Rectangle
    }
}

impl TryFrom<u32> for SelectorStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("SelectorStyle try_from::<u32>() for value {} failed", value)
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "selector_config")]
pub struct SelectorConfig {
    #[serde(rename = "style")]
    pub style: SelectorStyle,
    #[serde(rename = "resize_lock_aspectratio")]
    pub resize_lock_aspectratio: bool,
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self {
            style: SelectorStyle::default(),
            resize_lock_aspectratio: false,
        }
    }
}
