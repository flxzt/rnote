pub mod composer;
pub mod rough;
pub mod smooth;
pub mod textured;

// Re exports
pub use composer::Composer;

use self::rough::RoughOptions;
use self::smooth::SmoothOptions;
use self::textured::TexturedOptions;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Style {
    Smooth(SmoothOptions),
    Rough(RoughOptions),
    Textured(TexturedOptions),
}
