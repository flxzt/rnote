/// Flags returned to the widget holding the engine
#[must_use]
#[derive(Debug, Clone)]
pub struct WidgetFlags {
    /// application should be quit
    pub quit: bool,
    /// needs surface redrawing
    pub redraw: bool,
    /// needs surface resizing
    pub resize: bool,
    /// refresh the UI with the engine state
    pub refresh_ui: bool,
    /// whether the store has changed, i.e. new strokes inserted, modified, etc.
    pub indicate_changed_store: bool,
    /// update the current view offsets and size
    pub update_view: bool,
    /// Is Some when undo button visibility should be changed. Is None if should not be changed
    pub hide_undo: Option<bool>,
    /// Is Some when undo button visibility should be changed. Is None if should not be changed
    pub hide_redo: Option<bool>,
    /// Changes whether text preprocessing should be enabled. Meaning, instead of key events text events are then emitted
    /// for regular unicode text. Used when writing text with the typewriter
    pub enable_text_preprocessing: Option<bool>,
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
            hide_undo: None,
            hide_redo: None,
            enable_text_preprocessing: None,
        }
    }
}

impl WidgetFlags {
    /// Merging with another SurfaceFlags struct, prioritizing other for conflicting values.
    pub fn merge(&mut self, other: Self) {
        self.quit |= other.quit;
        self.redraw |= other.redraw;
        self.resize |= other.resize;
        self.refresh_ui |= other.refresh_ui;
        self.indicate_changed_store |= other.indicate_changed_store;
        self.update_view |= other.update_view;
        if other.hide_undo.is_some() {
            self.hide_undo = other.hide_undo
        }
        if other.hide_redo.is_some() {
            self.hide_redo = other.hide_redo;
        }
        if other.enable_text_preprocessing.is_some() {
            self.enable_text_preprocessing = other.enable_text_preprocessing;
        }
    }
}
