use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// configuration for the vertical tool
pub struct VerticalToolConfig {
    /// horizontal limit
    pub horizontal_border: bool,
    /// vertical limit
    pub vertical_border: bool,
}
