use serde::{Deserialize, Serialize};

use crate::strokes::textstroke::TextStyle;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "typewriter_config")]
pub struct TypewriterConfig {
    #[serde(rename = "text_style")]
    pub text_style: TextStyle,
    #[serde(rename = "max_width_enabled")]
    pub max_width_enabled: bool,
    #[serde(rename = "text_width")]
    pub text_width: f64,
}

impl Default for TypewriterConfig {
    fn default() -> Self {
        Self {
            text_style: TextStyle::default(),
            max_width_enabled: true,
            text_width: 600.0,
        }
    }
}
