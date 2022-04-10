use crate::pens::penholder::PenStyle;

/// Flags returned to the surface drawing the engine
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFlags {
    pub quit: bool,
    pub redraw: bool,
    pub resize: bool,
    pub resize_to_fit_strokes: bool,
    pub change_to_pen: Option<PenStyle>,
    pub pen_changed: bool,
    pub sheet_changed: bool,
    pub selection_changed: bool,
    /// Is Some when scrollbar visibility should be changed. Is None if should not be changed
    pub hide_scrollbars: Option<bool>,
    pub new_camera_offset: bool,
}

impl Default for SurfaceFlags {
    fn default() -> Self {
        Self {
            quit: false,
            redraw: false,
            resize: false,
            resize_to_fit_strokes: false,
            change_to_pen: None,
            pen_changed: false,
            sheet_changed: false,
            selection_changed: false,
            hide_scrollbars: None,
            new_camera_offset: false,
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

        self.pen_changed |= other.pen_changed;
        self.sheet_changed |= other.sheet_changed;
        self.selection_changed |= other.selection_changed;
        self.hide_scrollbars = if other.hide_scrollbars.is_some() {
            other.hide_scrollbars
        } else {
            self.hide_scrollbars
        };
        self.new_camera_offset |= other.new_camera_offset;

        self
    }

    pub fn merge_with_other(&mut self, other: Self) {
        *self = self.merged_with_other(other);
    }
}
