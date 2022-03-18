use crate::pens::PenStyle;

/// Flags returned to the appwindow
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
        }
    }
}

impl SurfaceFlags {
    /// Merging with another SurfaceFlags struct, prioritizing self for conflicting values.
    pub fn merge_with_other(&mut self, other: Self) {
        self.quit |= other.quit;
        self.redraw |= other.resize;
        self.resize_to_fit_strokes |= other.resize_to_fit_strokes;
        self.change_to_pen = if self.change_to_pen.is_none() {
            other.change_to_pen
        } else {
            self.change_to_pen
        };

        self.pen_changed |= other.pen_changed;
        self.sheet_changed |= other.sheet_changed;
        self.selection_changed |= other.selection_changed;
    }
}
