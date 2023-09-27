// Modules
pub mod background;
pub mod format;

// Re-exports
pub use background::Background;
pub use format::Format;

// Imports
use crate::{Camera, StrokeStore, WidgetFlags};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::ext::AabbExt;
use rnote_compose::{Color, SplitOrder};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "layout")]
pub enum Layout {
    #[serde(rename = "fixed_size")]
    FixedSize,
    #[serde(rename = "continuous_vertical", alias = "endless_vertical")]
    ContinuousVertical,
    #[serde(rename = "semi_infinite")]
    SemiInfinite,
    #[serde(rename = "infinite")]
    Infinite,
}

impl Default for Layout {
    fn default() -> Self {
        Self::Infinite
    }
}

impl TryFrom<u32> for Layout {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .ok_or_else(|| anyhow::anyhow!("Layout try_from::<u32>() for value {} failed", value))
    }
}

impl std::str::FromStr for Layout {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fixed-size" => Ok(Self::FixedSize),
            "continuous-vertical" => Ok(Self::ContinuousVertical),
            "semi-infinite" => Ok(Self::SemiInfinite),
            "infinite" => Ok(Self::Infinite),
            s => Err(anyhow::anyhow!(
                "Layout from_string failed, invalid name: {s}"
            )),
        }
    }
}

impl std::string::ToString for Layout {
    fn to_string(&self) -> String {
        match self {
            Layout::FixedSize => String::from("fixed-size"),
            Layout::ContinuousVertical => String::from("continuous-vertical"),
            Layout::SemiInfinite => String::from("semi-infinite"),
            Layout::Infinite => String::from("infinite"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "document")]
pub struct Document {
    #[serde(rename = "x", with = "rnote_compose::serialize::f64_dp3")]
    pub x: f64,
    #[serde(rename = "y", with = "rnote_compose::serialize::f64_dp3")]
    pub y: f64,
    #[serde(rename = "width", with = "rnote_compose::serialize::f64_dp3")]
    pub width: f64,
    #[serde(rename = "height", with = "rnote_compose::serialize::f64_dp3")]
    pub height: f64,
    #[serde(rename = "format")]
    pub format: Format,
    #[serde(rename = "background")]
    pub background: Background,
    #[serde(rename = "layout", alias = "expand_mode")]
    pub layout: Layout,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: Format::default().width(),
            height: Format::default().height(),
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

    pub fn clone_config(&self) -> Self {
        self.clone()
    }

    pub(crate) fn bounds(&self) -> Aabb {
        Aabb::new(
            na::point![self.x, self.y],
            na::point![self.x + self.width, self.y + self.height],
        )
    }

    /// Generate bounds for each page for the doc bounds, extended to fit the format.
    ///
    /// May contain many empty pages (in infinite mode)
    #[allow(unused)]
    pub(crate) fn pages_bounds(&self, split_order: SplitOrder) -> Vec<Aabb> {
        let doc_bounds = self.bounds();

        if self.format.height() > 0.0 && self.format.width() > 0.0 {
            doc_bounds.split_extended_origin_aligned(
                na::vector![self.format.width(), self.format.height()],
                split_order,
            )
        } else {
            vec![]
        }
    }

    #[allow(unused)]
    pub(crate) fn calc_n_pages(&self) -> u32 {
        // Avoid div by 0
        if self.format.height() > 0.0 && self.format.width() > 0.0 {
            (self.width / self.format.width()).ceil() as u32
                * (self.height / self.format.height()).ceil() as u32
        } else {
            0
        }
    }

    pub(crate) fn resize_to_fit_content(
        &mut self,
        store: &StrokeStore,
        camera: &Camera,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        match self.layout {
            Layout::FixedSize => {
                self.resize_doc_fixed_size_layout(store);
            }
            Layout::ContinuousVertical => {
                self.resize_doc_continuous_vertical_layout(store);
            }
            Layout::SemiInfinite => {
                self.resize_doc_semi_infinite_layout_to_fit_content(store);
                self.expand_doc_semi_infinite_layout(camera.viewport());
            }
            Layout::Infinite => {
                self.resize_doc_infinite_layout_to_fit_content(store);
                self.expand_doc_infinite_layout(camera.viewport());
            }
        }
        widget_flags.resize = true;
        widget_flags
    }

    pub(crate) fn resize_autoexpand(
        &mut self,
        store: &StrokeStore,
        camera: &Camera,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        match self.layout {
            Layout::FixedSize => {
                // do not resize in fixed size mode, if wanted use resize_to_fit_content() for it.
            }
            Layout::ContinuousVertical => {
                self.resize_doc_continuous_vertical_layout(store);
                widget_flags.resize = true;
            }
            Layout::SemiInfinite => {
                self.resize_doc_semi_infinite_layout_to_fit_content(store);
                self.expand_doc_semi_infinite_layout(camera.viewport());
                widget_flags.resize = true;
            }
            Layout::Infinite => {
                self.resize_doc_infinite_layout_to_fit_content(store);
                self.expand_doc_infinite_layout(camera.viewport());
                widget_flags.resize = true;
            }
        }
        widget_flags
    }

    pub(crate) fn expand_autoexpand(&mut self, camera: &Camera) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        match self.layout {
            Layout::FixedSize | Layout::ContinuousVertical => {
                // not resizing in these modes, the size is not dependent on the camera
            }
            Layout::SemiInfinite => {
                // only expand, don't resize to fit content
                self.expand_doc_semi_infinite_layout(camera.viewport());
                widget_flags.resize = true;
            }
            Layout::Infinite => {
                // only expand, don't resize to fit content
                self.expand_doc_infinite_layout(camera.viewport());
                widget_flags.resize = true;
            }
        }
        widget_flags
    }

    /// Adds a page when in fixed-size layout.
    ///
    /// Returns false when not in fixed-size layout.
    pub(crate) fn add_page_fixed_size(&mut self) -> bool {
        if self.layout != Layout::FixedSize {
            return false;
        }
        let format_height = self.format.height();
        let new_doc_height = self.height + format_height;
        self.height = new_doc_height;
        true
    }

    /// Removes a page when in fixed-size layout and the size is not the last page.
    ///
    /// Returns false when not in fixed-size layout.
    pub(crate) fn remove_page_fixed_size(&mut self) -> bool {
        if self.layout != Layout::FixedSize || self.height <= self.format.height() {
            return false;
        }
        self.height -= self.format.height();
        true
    }

    fn resize_doc_fixed_size_layout(&mut self, store: &StrokeStore) {
        let format_height = self.format.height();

        let new_width = self.format.width();
        // max(1.0) because then 'fraction'.ceil() is at least 1
        let new_height = ((store.calc_height().max(1.0)) / format_height).ceil() * format_height;

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_width;
        self.height = new_height;
    }

    fn resize_doc_continuous_vertical_layout(&mut self, store: &StrokeStore) {
        let padding_bottom = self.format.height();
        let new_height = store.calc_height() + padding_bottom;
        let new_width = self.format.width();

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_width;
        self.height = new_height;
    }

    fn expand_doc_semi_infinite_layout(&mut self, viewport: Aabb) {
        let padding_horizontal = self.format.width() * 2.0;
        let padding_vertical = self.format.height() * 2.0;

        let new_bounds = self.bounds().merged(
            &viewport.extend_right_and_bottom_by(na::vector![padding_horizontal, padding_vertical]),
        );

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_bounds.maxs[0];
        self.height = new_bounds.maxs[1];
    }

    fn expand_doc_infinite_layout(&mut self, viewport: Aabb) {
        let padding_horizontal = self.format.width() * 2.0;
        let padding_vertical = self.format.height() * 2.0;

        let new_bounds = self
            .bounds()
            .merged(&viewport.extend_by(na::vector![padding_horizontal, padding_vertical]));

        self.x = new_bounds.mins[0];
        self.y = new_bounds.mins[1];
        self.width = new_bounds.extents()[0];
        self.height = new_bounds.extents()[1];
    }

    fn resize_doc_semi_infinite_layout_to_fit_content(&mut self, store: &StrokeStore) {
        let padding_horizontal = self.format.width() * 2.0;
        let padding_vertical = self.format.height() * 2.0;

        let keys = store.stroke_keys_as_rendered();

        let new_bounds = if let Some(new_bounds) = store.bounds_for_strokes(&keys) {
            new_bounds.extend_right_and_bottom_by(na::vector![padding_horizontal, padding_vertical])
        } else {
            // If doc is empty, resize to one page with the format size
            Aabb::new(na::point![0.0, 0.0], self.format.size().into())
                .extend_right_and_bottom_by(na::vector![padding_horizontal, padding_vertical])
        };
        self.x = 0.0;
        self.y = 0.0;
        self.width = new_bounds.extents()[0];
        self.height = new_bounds.extents()[1];
    }

    fn resize_doc_infinite_layout_to_fit_content(&mut self, store: &StrokeStore) {
        let padding_horizontal = self.format.width() * 2.0;
        let padding_vertical = self.format.height() * 2.0;

        let keys = store.stroke_keys_as_rendered();

        let new_bounds = if let Some(new_bounds) = store.bounds_for_strokes(&keys) {
            new_bounds.extend_by(na::vector![padding_horizontal, padding_vertical])
        } else {
            // If doc is empty, resize to one page with the format size
            Aabb::new(na::point![0.0, 0.0], self.format.size().into())
                .extend_by(na::vector![padding_horizontal, padding_vertical])
        };
        self.x = new_bounds.mins[0];
        self.y = new_bounds.mins[1];
        self.width = new_bounds.extents()[0];
        self.height = new_bounds.extents()[1];
    }
}
