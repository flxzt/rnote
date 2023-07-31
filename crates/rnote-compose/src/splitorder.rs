// Imports
use serde::{Deserialize, Serialize};

/// Page split direction.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    Default,
)]
#[serde(rename = "split_order")]
pub enum SplitOrder {
    #[default]
    /// Split in row-major order.
    #[serde(rename = "row_major")]
    RowMajor,
    /// Split in column-major order.
    #[serde(rename = "column_major")]
    ColumnMajor,
}

impl TryFrom<u32> for SplitOrder {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("SplitOrder try_from::<u32>() for value {} failed", value)
        })
    }
}
