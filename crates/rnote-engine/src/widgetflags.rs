/// Flags returned to the UI widget that holds the engine.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetFlags {
    /// Needs surface redrawing.
    pub redraw: bool,
    /// Needs surface resizing.
    pub resize: bool,
    /// Refresh the UI with the engine state.
    pub refresh_ui: bool,
    /// Whether the store was modified, i.e. new strokes inserted, modified, etc. .
    pub store_modified: bool,
    /// Update the current view offsets and size.
    pub update_view: bool,
    /// Indicates that the camera has changed it's temporary zoom.
    pub zoomed_temporarily: bool,
    /// Indicates that the camera has changed it's permanent zoom.
    pub zoomed: bool,
    /// Deselect the elements of the global color picker.
    pub deselect_color_setters: bool,
    /// Is Some when undo button visibility should be changed. Is None if should not be changed.
    pub hide_undo: Option<bool>,
    /// Is Some when redo button visibility should be changed. Is None if should not be changed.
    pub hide_redo: Option<bool>,
    /// Changes whether text preprocessing in the UI toolkit should be enabled.
    /// Meaning, when enabled instead of key events, text events are then emitted
    /// for regular unicode text. Used when writing text with the typewriter.
    pub enable_text_preprocessing: Option<bool>,
}

impl Default for WidgetFlags {
    fn default() -> Self {
        Self {
            redraw: false,
            resize: false,
            refresh_ui: false,
            store_modified: false,
            update_view: false,
            zoomed_temporarily: false,
            zoomed: false,
            deselect_color_setters: false,
            hide_undo: None,
            hide_redo: None,
            enable_text_preprocessing: None,
        }
    }
}

impl WidgetFlags {
    /// Merge with another WidgetFlags struct, prioritizing other for conflicting values.
    pub fn merge(&mut self, other: Self) {
        self.redraw |= other.redraw;
        self.resize |= other.resize;
        self.refresh_ui |= other.refresh_ui;
        self.store_modified |= other.store_modified;
        self.update_view |= other.update_view;
        self.zoomed_temporarily |= other.zoomed_temporarily;
        self.zoomed |= other.zoomed;
        self.deselect_color_setters |= other.deselect_color_setters;
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
