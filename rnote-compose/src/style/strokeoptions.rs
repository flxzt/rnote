use crate::Color;
use serde::{Deserialize, Serialize};

/// The different width presets
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename = "brush_preset_size")]
pub enum StrokeWidthPreset {
    /// The smallest one
    #[serde(rename = "small")]
    Small,
    /// The one in the middle
    #[serde(rename = "medium")]
    Medium,
    /// The biggest one
    #[serde(rename = "large")]
    Large,
}

/// Common part for stroke width and stroke color
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(default, rename = "stroke_options")]
pub struct StrokeOptions {
    /// The small stroke width preset
    #[serde(rename = "stroke_width_small")]
    pub stroke_width_small: f64,
    /// The medium stroke width preset
    #[serde(rename = "stroke_width_medium")]
    pub stroke_width_medium: f64,
    /// The large stroke width preset
    #[serde(rename = "stroke_width_large")]
    pub stroke_width_large: f64,
    /// The selected width preset
    #[serde(rename = "stroke_width_preset")]
    pub stroke_width_preset: StrokeWidthPreset,
    /// The color of the stroke
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
}

impl StrokeOptions {
    /// Returns the selected stroke width
    pub fn get_stroke_width(&self) -> f64 {
        match self.stroke_width_preset {
            StrokeWidthPreset::Small => self.stroke_width_small,
            StrokeWidthPreset::Medium => self.stroke_width_medium,
            StrokeWidthPreset::Large => self.stroke_width_large,
        }
    }

    /// Sets a new stroke width to the selected preset
    pub fn set_stroke_width(mut self, value: f64) {
        match self.stroke_width_preset {
            StrokeWidthPreset::Small => self.stroke_width_small = value,
            StrokeWidthPreset::Medium => self.stroke_width_medium = value,
            StrokeWidthPreset::Large => self.stroke_width_large = value,
        }
    }
}

impl Default for StrokeOptions {
    fn default() -> Self {
        Self {
            stroke_width_small: 2.0,
            stroke_width_medium: 4.0,
            stroke_width_large: 12.0,
            stroke_width_preset: StrokeWidthPreset::Small,
            stroke_color: Some(Color::BLACK),
        }
    }
}
