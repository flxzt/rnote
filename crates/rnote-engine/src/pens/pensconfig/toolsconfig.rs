// Imports
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "tool_style")]
pub enum ToolStyle {
    #[serde(rename = "verticalspace")]
    VerticalSpace,
    #[serde(rename = "offsetcamera")]
    OffsetCamera,
    #[serde(rename = "zoom")]
    Zoom,
}

impl Default for ToolStyle {
    fn default() -> Self {
        Self::VerticalSpace
    }
}

impl TryFrom<u32> for ToolStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("ToolStyle try_from::<u32>() for value {} failed", value)
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename = "verticalspace_tool_config")]
pub struct VerticalSpaceToolConfig {
    /// horizontal limit
    pub limit_movement_horizontal_borders: bool,
    /// vertical limit
    pub limit_movement_vertical_borders: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename = "tools_config")]
pub struct ToolsConfig {
    #[serde(rename = "style")]
    pub style: ToolStyle,
    pub verticalspace_tool_config: VerticalSpaceToolConfig,
}
