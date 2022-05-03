/// Flags returned to the surface drawing the engine
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFlags {
    /// application should be quit
    pub quit: bool,
    /// needs surface redrawing
    pub redraw: bool,
    /// needs surface resizing
    pub resize: bool,
    /// Penholder state has has changed
    pub penholder_changed: bool,
    /// wether the store has changed, i.e. new strokes inserted, modified, etc.
    pub store_changed: bool,
    /// camera has changed
    pub camera_changed: bool,
    /// Is Some when scrollbar visibility should be changed. Is None if should not be changed
    pub hide_scrollbars: Option<bool>,
}

impl Default for SurfaceFlags {
    fn default() -> Self {
        Self {
            quit: false,
            redraw: false,
            resize: false,
            penholder_changed: false,
            store_changed: false,
            camera_changed: false,
            hide_scrollbars: None,
        }
    }
}

impl SurfaceFlags {
    /// Merging with another SurfaceFlags struct, prioritizing other for conflicting values.
    pub fn merged_with_other(mut self, other: Self) -> Self {
        self.quit |= other.quit;
        self.redraw |= other.redraw;
        self.resize |= other.resize;
        self.penholder_changed |= other.penholder_changed;
        self.store_changed |= other.store_changed;
        self.camera_changed |= other.camera_changed;
        self.hide_scrollbars = if other.hide_scrollbars.is_some() {
            other.hide_scrollbars
        } else {
            self.hide_scrollbars
        };

        self
    }

    /// Merging with another SurfaceFlags struct in place, prioritizing other for conflicting values.
    pub fn merge_with_other(&mut self, other: Self) {
        *self = self.merged_with_other(other);
    }
}
