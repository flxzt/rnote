use crate::pens::penholder::PenStyle;

/// Flags returned to the surface drawing the engine
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFlags {
    /// application should be quit
    pub quit: bool,
    /// needs redrawing
    pub redraw: bool,
    /// needs resizing
    pub resize: bool,
    /// Sheet should be resized to fit the strokes
    pub resize_to_fit_strokes: bool,
    /// Should change to the pen style
    pub change_to_pen: Option<PenStyle>,
    /// Penholder state has has changed
    pub penholder_changed: bool,
    /// wether the sheet has changed, i.e. new strokes inserted, modified, etc.
    pub sheet_changed: bool,
    /// Selection has changed
    pub update_selector: bool,
    /// Is Some when scrollbar visibility should be changed. Is None if should not be changed
    pub hide_scrollbars: Option<bool>,
    /// camera offset changed
    pub camera_offset_changed: bool,
}

impl Default for SurfaceFlags {
    fn default() -> Self {
        Self {
            quit: false,
            redraw: false,
            resize: false,
            resize_to_fit_strokes: false,
            change_to_pen: None,
            penholder_changed: false,
            sheet_changed: false,
            update_selector: false,
            hide_scrollbars: None,
            camera_offset_changed: false,
        }
    }
}

impl SurfaceFlags {
    /// Merging with another SurfaceFlags struct, prioritizing other for conflicting values.
    pub fn merged_with_other(mut self, other: Self) -> Self {
        self.quit |= other.quit;
        self.redraw |= other.redraw;
        self.resize |= other.resize;
        self.resize_to_fit_strokes |= other.resize_to_fit_strokes;
        self.change_to_pen = if other.change_to_pen.is_some() {
            other.change_to_pen
        } else {
            self.change_to_pen
        };

        self.penholder_changed |= other.penholder_changed;
        self.sheet_changed |= other.sheet_changed;
        self.update_selector |= other.update_selector;
        self.hide_scrollbars = if other.hide_scrollbars.is_some() {
            other.hide_scrollbars
        } else {
            self.hide_scrollbars
        };
        self.camera_offset_changed |= other.camera_offset_changed;

        self
    }

    /// Merging with another SurfaceFlags struct in place, prioritizing other for conflicting values.
    pub fn merge_with_other(&mut self, other: Self) {
        *self = self.merged_with_other(other);
    }
}
