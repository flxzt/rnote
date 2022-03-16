use crate::pens::PenStyle;

/// Flags returned to the UI surface
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFlags {
    pub quit: bool,
    pub redraw: bool,
    pub resize: bool,
    pub pen_change: Option<PenStyle>,
    pub resize_to_fit_strokes: bool,
}

impl Default for SurfaceFlags {
    fn default() -> Self {
        Self {
            quit: false,
            redraw: false,
            resize: false,
            pen_change: None,
            resize_to_fit_strokes: false,
        }
    }
}
