use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// configuration for the vertical tool
pub struct VerticalToolConfig {
    /// horizontal limit
    pub limit_movement_horizontal_borders: bool,
    /// vertical limit
    pub limit_movement_vertical_borders: bool,
}
