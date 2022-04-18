pub mod background;
pub mod format;

// Re-exports
pub use background::Background;
pub use format::Format;
use rnote_compose::Color;

use crate::utils::{GdkRGBAHelpers, GrapheneRectHelpers};
use crate::StrokeStore;
use rnote_compose::helpers::AABBHelpers;

use gtk4::{gdk, graphene, gsk, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "sheet")]
pub struct Sheet {
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
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: Format::default().width,
            height: Format::default().height,
            format: Format::default(),
            background: Background::default(),
        }
    }
}

impl Sheet {
    pub const SHADOW_WIDTH: f64 = 30.0;
    pub const SHADOW_COLOR: Color = Color {
        r: 0.1,
        g: 0.1,
        b: 0.1,
        a: 0.3,
    };

    pub fn bounds(&self) -> AABB {
        AABB::new(
            na::point![self.x, self.y],
            na::point![self.x + self.width, self.y + self.height],
        )
    }

    // Generates bounds for each page for the sheet size, extended to fit the sheet format. May contain many empty pages (in infinite mode)
    pub fn pages_bounds(&self) -> Vec<AABB> {
        let sheet_bounds = self.bounds();

        if self.format.height > 0.0 && self.format.width > 0.0 {
            sheet_bounds
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

    pub(crate) fn resize_sheet_mode_fixed_size(&mut self, store: &StrokeStore) {
        let format_height = self.format.height;

        let new_width = self.format.width;
        // +1.0 because then 'fraction'.ceil() is at least 1
        let new_height = (f64::from(store.calc_height() + 1.0) / f64::from(format_height)).ceil()
            * format_height;

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_width;
        self.height = new_height;
    }

    pub(crate) fn resize_sheet_mode_endless_vertical(&mut self, store: &StrokeStore) {
        let padding_bottom = self.format.height;
        let new_height = store.calc_height() + padding_bottom;
        let new_width = self.format.width;

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_width;
        self.height = new_height;
    }

    pub(crate) fn expand_sheet_mode_infinite(&mut self, viewport: AABB) {
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

    pub(crate) fn resize_sheet_mode_infinite_to_fit_strokes(&mut self, store: &StrokeStore) {
        let padding_horizontal = self.format.width * 2.0;
        let padding_vertical = self.format.height * 2.0;

        let mut keys = store.stroke_keys_as_rendered();
        keys.append(&mut store.selection_keys_as_rendered());

        let new_bounds = if let Some(new_bounds) = store.gen_bounds(&keys) {
            new_bounds.extend_by(na::vector![padding_horizontal, padding_vertical])
        } else {
            // If sheet is empty, resize to one page with the format size
            AABB::new(
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

    pub fn draw_shadow(&self, snapshot: &Snapshot) {
        let shadow_width = Self::SHADOW_WIDTH;
        let bounds = self.bounds();

        let corner_radius =
            graphene::Size::new(shadow_width as f32 / 4.0, shadow_width as f32 / 4.0);

        let rounded_rect = gsk::RoundedRect::new(
            graphene::Rect::from_p2d_aabb(bounds),
            corner_radius.clone(),
            corner_radius.clone(),
            corner_radius.clone(),
            corner_radius,
        );

        snapshot.append_outset_shadow(
            &rounded_rect,
            &gdk::RGBA::from_compose_color(Self::SHADOW_COLOR),
            0.0,
            0.0,
            (shadow_width / 2.0) as f32,
            (shadow_width / 2.0) as f32,
        );
    }
}
