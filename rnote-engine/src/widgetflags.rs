/// Flags returned to the widget holding the engine
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct WidgetFlags {
    /// application should be quit
    pub quit: bool,
    /// needs surface redrawing
    pub redraw: bool,
    /// needs surface resizing
    pub resize: bool,
    /// refresh the UI with the engine state
    pub refresh_ui: bool,
    /// wether the store has changed, i.e. new strokes inserted, modified, etc.
    pub indicate_changed_store: bool,
    /// update the current view offsets and size
    pub update_view: bool,
    /// Is Some when scrollbar visibility should be changed. Is None if should not be changed
    pub hide_scrollbars: Option<bool>,
    /// Is Some when undo button visibility should be changed. Is None if should not be changed
    pub hide_undo: Option<bool>,
    /// Is Some when undo button visibility should be changed. Is None if should not be changed
    pub hide_redo: Option<bool>,
}

impl Default for WidgetFlags {
    fn default() -> Self {
        Self {
            quit: false,
            redraw: false,
            resize: false,
            refresh_ui: false,
            indicate_changed_store: false,
            update_view: false,
            hide_scrollbars: None,
            hide_undo: None,
            hide_redo: None,
        }
    }
}

impl WidgetFlags {
    /// Merging with another SurfaceFlags struct, prioritizing other for conflicting values.
    pub fn merged_with_other(mut self, other: Self) -> Self {
        self.quit |= other.quit;
        self.redraw |= other.redraw;
        self.resize |= other.resize;
        self.refresh_ui |= other.refresh_ui;
        self.indicate_changed_store |= other.indicate_changed_store;
        self.update_view |= other.update_view;
        self.hide_scrollbars = if other.hide_scrollbars.is_some() {
            other.hide_scrollbars
        } else {
            self.hide_scrollbars
        };
        self.hide_undo = if other.hide_undo.is_some() {
            other.hide_undo
        } else {
            self.hide_undo
        };
        self.hide_redo = if other.hide_redo.is_some() {
            other.hide_redo
        } else {
            self.hide_redo
        };

        self
    }

    /// Merging with another SurfaceFlags struct in place, prioritizing other for conflicting values.
    pub fn merge_with_other(&mut self, other: Self) {
        *self = self.merged_with_other(other);
    }
}
