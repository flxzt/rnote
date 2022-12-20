pub mod background;
pub mod format;

// Re-exports
pub use background::Background;
pub use format::Format;
use rnote_compose::Color;

use crate::{Camera, StrokeStore};
use rnote_compose::helpers::AabbHelpers;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename = "layout")]
pub enum Layout {
    #[serde(rename = "fixed_size")]
    FixedSize,
    #[serde(rename = "continuous_vertical", alias = "endless_vertical")]
    ContinuousVertical,
    #[serde(rename = "infinite")]
    Infinite,
}

impl Default for Layout {
    fn default() -> Self {
        Self::Infinite
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "document")]
pub struct Document {
    #[serde(rename = "x")]
    pub x: f64,
    #[serde(rename = "y")]
    pub y: f64,
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "height")]
    pub height: f64,
    #[serde(rename = "format")]
    pub format: Format,
    #[serde(rename = "background")]
    pub background: Background,
    #[serde(rename = "layout", alias = "expand_mode")]
    layout: Layout,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: Format::default().width,
            height: Format::default().height,
            format: Format::default(),
            background: Background::default(),
            layout: Layout::default(),
        }
    }
}

impl Document {
    pub const SHADOW_WIDTH: f64 = 12.0;
    pub const SHADOW_OFFSET: na::Vector2<f64> = na::vector![4.0, 4.0];
    pub const SHADOW_COLOR: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.35,
    };

    pub(crate) fn layout(&self) -> Layout {
        self.layout
    }

    pub(crate) fn set_layout(&mut self, layout: Layout, store: &StrokeStore, camera: &Camera) {
        self.layout = layout;

        self.resize_to_fit_strokes(store, camera);
    }

    pub fn bounds(&self) -> Aabb {
        Aabb::new(
            na::point![self.x, self.y],
            na::point![self.x + self.width, self.y + self.height],
        )
    }

    // Generates bounds for each page for the doc bounds, extended to fit the format. May contain many empty pages (in infinite mode)
    pub fn pages_bounds(&self) -> Vec<Aabb> {
        let doc_bounds = self.bounds();

        if self.format.height > 0.0 && self.format.width > 0.0 {
            doc_bounds
                .split_extended_origin_aligned(na::vector![self.format.width, self.format.height])
        } else {
            vec![]
        }
    }

    pub fn calc_n_pages(&self) -> u32 {
        // Avoid div by 0
        if self.format.height > 0.0 && self.format.width > 0.0 {
            (self.width / self.format.width).round() as u32
                * (self.height / self.format.height).round() as u32
        } else {
            0
        }
    }

    pub(crate) fn resize_to_fit_strokes(&mut self, store: &StrokeStore, camera: &Camera) {
        match self.layout {
            Layout::FixedSize => {
                self.resize_doc_fixed_size_layout(store);
            }
            Layout::ContinuousVertical => {
                self.resize_doc_continuous_vertical_layout(store);
            }
            Layout::Infinite => {
                self.resize_doc_infinite_layout_to_fit_strokes(store);
                self.expand_doc_infinite_layout(camera.viewport());
            }
        }
    }

    pub(crate) fn resize_autoexpand(&mut self, store: &StrokeStore, camera: &Camera) {
        match self.layout {
            Layout::FixedSize => {
                // Does not resize in fixed size mode, if wanted use resize_doc_to_fit_strokes() for it.
            }
            Layout::ContinuousVertical => {
                self.resize_doc_continuous_vertical_layout(store);
            }
            Layout::Infinite => {
                self.resize_doc_infinite_layout_to_fit_strokes(store);
                self.expand_doc_infinite_layout(camera.viewport());
            }
        }
    }

    pub(crate) fn resize_doc_fixed_size_layout(&mut self, store: &StrokeStore) {
        let format_height = self.format.height;

        let new_width = self.format.width;
        // max(1.0) because then 'fraction'.ceil() is at least 1
        let new_height = ((store.calc_height().max(1.0)) / format_height).ceil() * format_height;

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_width;
        self.height = new_height;
    }

    pub(crate) fn resize_doc_continuous_vertical_layout(&mut self, store: &StrokeStore) {
        let padding_bottom = self.format.height;
        let new_height = store.calc_height() + padding_bottom;
        let new_width = self.format.width;

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_width;
        self.height = new_height;
    }

    pub(crate) fn expand_doc_infinite_layout(&mut self, viewport: Aabb) {
        let padding_horizontal = self.format.width * 2.0;
        let padding_vertical = self.format.height * 2.0;

        let new_bounds = self
            .bounds()
            .merged(&viewport.extend_by(na::vector![padding_horizontal, padding_vertical]));

        self.x = new_bounds.mins[0];
        self.y = new_bounds.mins[1];
        self.width = new_bounds.extents()[0];
        self.height = new_bounds.extents()[1];
    }

    pub(crate) fn resize_doc_infinite_layout_to_fit_strokes(&mut self, store: &StrokeStore) {
        let padding_horizontal = self.format.width * 2.0;
        let padding_vertical = self.format.height * 2.0;

        let keys = store.stroke_keys_as_rendered();

        let new_bounds = if let Some(new_bounds) = store.bounds_for_strokes(&keys) {
            new_bounds.extend_by(na::vector![padding_horizontal, padding_vertical])
        } else {
            // If doc is empty, resize to one page with the format size
            Aabb::new(
                na::point![0.0, 0.0],
                na::point![self.format.width, self.format.height],
            )
            .extend_by(na::vector![padding_horizontal, padding_vertical])
        };
        self.x = new_bounds.mins[0];
        self.y = new_bounds.mins[1];
        self.width = new_bounds.extents()[0];
        self.height = new_bounds.extents()[1];
    }
}
