mod composer;
/// Draw helpers
pub mod drawhelpers;
/// The rough module for rough styles
pub mod rough;
/// The smooth module for smooth styles
pub mod smooth;
/// The textured module for textured styles
pub mod textured;

// Re exports
use self::rough::RoughOptions;
use self::smooth::SmoothOptions;
use self::textured::TexturedOptions;
pub use composer::Composer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A style choice holding the style options inside its variants
pub enum Style {
    /// A smooth style
    Smooth(SmoothOptions),
    /// A rough style
    Rough(RoughOptions),
    /// A textured style
    Textured(TexturedOptions),
}

impl Default for Style {
    fn default() -> Self {
        Self::Smooth(SmoothOptions::default())
    }
}
