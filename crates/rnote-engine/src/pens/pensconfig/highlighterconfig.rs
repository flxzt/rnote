use crate::pens::pensconfig::brushconfig::MarkerOptions;
use rnote_compose::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "highlighter_config")]
pub struct HighlighterConfig {
    #[serde(rename = "marker_options")]
    pub marker_options: MarkerOptions,
}

impl Default for HighlighterConfig {
    fn default() -> Self {
        let mut marker_options = MarkerOptions::default();
        marker_options.stroke_width = 30.0;
        marker_options.stroke_color = Some(Color::new(1.0, 1.0, 0.0, 0.7));
        marker_options.fill_color = Some(Color::TRANSPARENT);

        Self { marker_options }
    }
}
