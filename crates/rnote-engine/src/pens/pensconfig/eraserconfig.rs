// Imports
use p2d::bounding_volume::Aabb;
use rnote_compose::penpath::Element;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "eraser_style")]
pub enum EraserStyle {
    #[serde(rename = "trash_colliding_strokes")]
    TrashCollidingStrokes,
    #[serde(rename = "split_colliding_strokes")]
    SplitCollidingStrokes,
}

impl Default for EraserStyle {
    fn default() -> Self {
        Self::TrashCollidingStrokes
    }
}

impl TryFrom<u32> for EraserStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("EraserStyle try_from::<u32>() for value {} failed", value)
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "eraser_config")]
pub struct EraserConfig {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "style")]
    pub style: EraserStyle,
    #[serde(rename = "speed_scale")]
    pub speed_scaling: bool,
}

impl Default for EraserConfig {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            style: EraserStyle::default(),
            speed_scaling: true,
        }
    }
}

impl EraserConfig {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 12.0;
    pub const SPEED_SCALING: f64 = 0.001;

    fn width_from_speed(&self, speed: f64) -> f64 {
        if !self.speed_scaling {
            return self.width;
        }
        self.width + Self::SPEED_SCALING * speed
    }

    pub(crate) fn eraser_bounds(&self, element: Element, speed: f64) -> Aabb {
        Aabb::from_half_extents(
            element.pos.into(),
            na::Vector2::repeat(self.width_from_speed(speed) * 0.5),
        )
    }
}
