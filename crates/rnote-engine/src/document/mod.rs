// Modules
pub mod background;
pub mod config;
pub mod format;
pub mod layout;

// Re-exports
pub use background::Background;
pub use config::DocumentConfig;
pub use format::Format;
pub use layout::Layout;
use rstar::Envelope;

// Imports
use crate::engine::EngineConfig;
use crate::engine::snapshot::Snapshotable;
use crate::{Camera, StrokeStore, WidgetFlags};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::ext::{AabbExt, Vector2Ext};
use rnote_compose::{Color, SplitOrder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "document")]
pub struct Document {
    #[serde(rename = "config")]
    pub config: DocumentConfig,
    #[serde(rename = "x", with = "rnote_compose::serialize::f64_dp3")]
    pub x: f64,
    #[serde(rename = "y", with = "rnote_compose::serialize::f64_dp3")]
    pub y: f64,
    #[serde(rename = "width", with = "rnote_compose::serialize::f64_dp3")]
    pub width: f64,
    #[serde(rename = "height", with = "rnote_compose::serialize::f64_dp3")]
    pub height: f64,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            config: DocumentConfig::default(),
            x: 0.0,
            y: 0.0,
            width: Format::default().width(),
            height: Format::default().height(),
        }
    }
}

impl Snapshotable for Document {
    fn extract_snapshot_data(&self) -> Self {
        self.clone()
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

        if self.config.format.height() > 0.0 && self.config.format.width() > 0.0 {
            doc_bounds.split_extended_origin_aligned(
                na::vector![self.config.format.width(), self.config.format.height()],
                split_order,
            )
        } else {
            vec![]
        }
    }

    #[allow(unused)]
    pub(crate) fn calc_n_pages(&self) -> u32 {
        // Avoid div by 0
        if self.config.format.height() > 0.0 && self.config.format.width() > 0.0 {
            (self.width / self.config.format.width()).ceil() as u32
                * (self.height / self.config.format.height()).ceil() as u32
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
        match self.config.layout {
            Layout::FixedSize => {
                widget_flags.resize |= self.resize_doc_fixed_size_layout(store);
            }
            Layout::ContinuousVertical => {
                widget_flags.resize |= self.resize_doc_continuous_vertical_layout(store);
            }
            Layout::SemiInfinite => {
                widget_flags.resize |=
                    self.resize_doc_semi_infinite_layout(camera.viewport(), store, true);
            }
            Layout::Infinite => {
                widget_flags.resize |=
                    self.resize_doc_infinite_layout(camera.viewport(), store, true);
            }
        }
        widget_flags
    }

    pub(crate) fn resize_autoexpand(
        &mut self,
        store: &StrokeStore,
        camera: &Camera,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        match self.config.layout {
            Layout::FixedSize => {
                // do not resize in fixed size mode, if wanted use resize_to_fit_content() for it.
            }
            Layout::ContinuousVertical => {
                widget_flags.resize |= self.resize_doc_continuous_vertical_layout(store);
            }
            Layout::SemiInfinite => {
                widget_flags.resize |=
                    self.resize_doc_semi_infinite_layout(camera.viewport(), store, true);
            }
            Layout::Infinite => {
                widget_flags.resize |=
                    self.resize_doc_infinite_layout(camera.viewport(), store, true);
            }
        }
        widget_flags
    }

    pub(crate) fn expand_autoexpand(
        &mut self,
        camera: &Camera,
        store: &StrokeStore,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        match self.config.layout {
            Layout::FixedSize | Layout::ContinuousVertical => {
                // not resizing in these modes, the size is not dependent on the camera
            }
            Layout::SemiInfinite => {
                // only expand, don't resize to fit content
                widget_flags.resize |=
                    self.resize_doc_semi_infinite_layout(camera.viewport(), store, false);
            }
            Layout::Infinite => {
                // only expand, don't resize to fit content
                widget_flags.resize |=
                    self.resize_doc_infinite_layout(camera.viewport(), store, false);
            }
        }
        widget_flags
    }

    /// Adds a page when in fixed-size layout.
    ///
    /// Returns false when not in fixed-size layout.
    pub(crate) fn add_page_fixed_size(&mut self) -> bool {
        if self.config.layout != Layout::FixedSize {
            return false;
        }
        let format_height = self.config.format.height();
        let new_doc_height = self.height + format_height;
        self.height = new_doc_height;
        true
    }

    /// Removes a page when in fixed-size layout and the size is not the last page.
    ///
    /// Returns false when not in fixed-size layout.
    pub(crate) fn remove_page_fixed_size(&mut self) -> bool {
        if self.config.layout != Layout::FixedSize || self.height <= self.config.format.height() {
            return false;
        }
        self.height -= self.config.format.height();
        true
    }

    /// Returns true if a resize happened.
    #[must_use = "Determines if the resize flag should be set"]
    fn resize_doc_fixed_size_layout(&mut self, store: &StrokeStore) -> bool {
        let format_height = self.config.format.height();

        let new_width = self.config.format.width();
        // max(1.0) because then 'fraction'.ceil() is at least 1
        let new_height = ((store.calc_height().max(1.0)) / format_height).ceil() * format_height;

        set_dimensions_checked(
            &mut self.x,
            &mut self.y,
            &mut self.width,
            &mut self.height,
            0.,
            0.,
            new_width,
            new_height,
        )
    }

    /// Returns true if a resize happened.
    #[must_use = "Determines if the resize flag should be set"]
    fn resize_doc_continuous_vertical_layout(&mut self, store: &StrokeStore) -> bool {
        let padding_bottom = self.config.format.height();
        let new_height = store.calc_height() + padding_bottom;
        let new_width = self.config.format.width();

        set_dimensions_checked(
            &mut self.x,
            &mut self.y,
            &mut self.width,
            &mut self.height,
            0.,
            0.,
            new_width,
            new_height,
        )
    }

    /// Resizes the document to include the viewport for the semi-infinite layout mode.
    ///
    /// if `include_content` is set, this also expands to included the content.
    /// The computation will then get more expensive, though.
    ///
    /// Returns true if a resize happened.
    #[must_use = "Determines if the resize flag should be set"]
    fn resize_doc_semi_infinite_layout(
        &mut self,
        viewport: Aabb,
        store: &StrokeStore,
        include_content: bool,
    ) -> bool {
        let padding_horizontal = self.config.format.width() * 2.0;
        let padding_vertical = self.config.format.height() * 2.0;

        let mut new_bounds = self.bounds().merged(
            &viewport.extend_right_and_bottom_by(na::vector![padding_horizontal, padding_vertical]),
        );

        if include_content {
            let rendered_bounds = store.get_bounds();

            let content_bounds = if rendered_bounds.area() > 0.0 {
                Aabb::new(
                    na::point![rendered_bounds.lower()[0], rendered_bounds.lower()[1]],
                    na::point![rendered_bounds.upper()[0], rendered_bounds.upper()[1]],
                )
                .extend_right_and_bottom_by(na::vector![padding_horizontal, padding_vertical])
            } else {
                // If doc is empty, resize to one page with the format size
                Aabb::new(na::point![0.0, 0.0], self.config.format.size().into())
                    .extend_right_and_bottom_by(na::vector![padding_horizontal, padding_vertical])
            };
            new_bounds.merge(&content_bounds);
        }

        set_dimensions_checked(
            &mut self.x,
            &mut self.y,
            &mut self.width,
            &mut self.height,
            0.,
            0.,
            new_bounds.maxs[0],
            new_bounds.maxs[1],
        )
    }

    /// Resizes the document to include the viewport for the infinite layout mode.
    ///
    /// if `include_content` is set, this also expands to included the content.
    /// The computation will then get more expensive, though.
    ///
    /// Returns true if a resize happened.
    #[must_use = "Determines if the resize flag should be set"]
    fn resize_doc_infinite_layout(
        &mut self,
        viewport: Aabb,
        store: &StrokeStore,
        include_content: bool,
    ) -> bool {
        let padding_horizontal = self.config.format.width() * 2.0;
        let padding_vertical = self.config.format.height() * 2.0;

        let mut new_bounds = self
            .bounds()
            .merged(&viewport.extend_by(na::vector![padding_horizontal, padding_vertical]));

        if include_content {
            let rendered_bounds = store.get_bounds();

            let content_bounds = if rendered_bounds.area() > 0.0 {
                Aabb::new(
                    na::point![rendered_bounds.lower()[0], rendered_bounds.lower()[1]],
                    na::point![rendered_bounds.upper()[0], rendered_bounds.upper()[1]],
                )
                .extend_right_and_bottom_by(na::vector![padding_horizontal, padding_vertical])
            } else {
                // If doc is empty, resize to one page with the format size
                Aabb::new(na::point![0.0, 0.0], self.config.format.size().into())
                    .extend_by(na::vector![padding_horizontal, padding_vertical])
            };
            new_bounds.merge(&content_bounds);
        }

        set_dimensions_checked(
            &mut self.x,
            &mut self.y,
            &mut self.width,
            &mut self.height,
            new_bounds.mins[0],
            new_bounds.mins[1],
            new_bounds.extents()[0],
            new_bounds.extents()[1],
        )
    }

    /// Snap the position to the document and pattern grid when `snap_positions` is enabled.
    ///
    /// If not, the original coordinates are returned.
    pub(crate) fn snap_position(
        &self,
        pos: na::Vector2<f64>,
        config: &EngineConfig,
    ) -> na::Vector2<f64> {
        const DOCUMENT_SNAP_DIST: f64 = 10.;
        let doc_format_size = self.config.format.size();
        let pattern_size = self.config.background.pattern_size;

        if !config.snap_positions {
            return pos;
        }

        let snap_to_grid = |pos: na::Vector2<f64>, grid_size: na::Vector2<f64>| {
            let grid_pos = pos.component_div(&grid_size);
            grid_size.component_mul(&grid_pos.round())
        };

        let pos_snapped_pattern = snap_to_grid(pos, pattern_size);
        let pos_snapped_document = snap_to_grid(pos, doc_format_size);

        let mut pos_snapped = pos_snapped_pattern;

        // If the position is close to the document edges, then it is instead snapped to them.
        if (pos_snapped_document - pos)[0].abs() < DOCUMENT_SNAP_DIST {
            pos_snapped[0] = pos_snapped_document[0];
        }
        if (pos_snapped_document - pos)[1].abs() < DOCUMENT_SNAP_DIST {
            pos_snapped[1] = pos_snapped_document[1];
        }

        pos_snapped
    }
}

#[must_use = "Determines if the resize flag should be set"]
#[allow(clippy::too_many_arguments)]
fn set_dimensions_checked(
    x: &mut f64,
    y: &mut f64,
    width: &mut f64,
    height: &mut f64,
    new_x: f64,
    new_y: f64,
    new_width: f64,
    new_height: f64,
) -> bool {
    let mut check = false;
    if approx::relative_ne!(*x, new_x) {
        *x = new_x;
        check = true;
    }
    if approx::relative_ne!(*y, new_y) {
        *y = new_y;
        check = true
    }
    if approx::relative_ne!(*width, new_width) {
        *width = new_width;
        check = true
    }
    if approx::relative_ne!(*height, new_height) {
        *height = new_height;
        check = true;
    }
    check
}
