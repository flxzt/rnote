use crate::pens::PenStyle;

/// Flags returned to the UI surface
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFlags {
    pub quit: bool,
    pub redraw: bool,
    pub resize: bool,
    pub resize_to_fit_strokes: bool,
    pub pen_change: Option<PenStyle>,
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
            pen_change: None,
            sheet_changed: false,
            selection_changed: false,
        }
    }
}
