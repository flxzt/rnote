/// Flags returned to the UI surface
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFlags {
    pub quit: bool,
    pub redraw: bool,
    pub resize: bool,
    pub activate_selector: bool,
    pub resize_to_fit_strokes: bool,
}

impl Default for SurfaceFlags {
    fn default() -> Self {
        Self {
            quit: false,
            redraw: false,
            resize: false,
            activate_selector: false,
            resize_to_fit_strokes: false,
        }
    }
}
