// Modules
pub mod chrono_comp;
pub mod keytree;
pub mod render_comp;
pub mod selection_comp;
pub mod stroke_comp;
pub mod trash_comp;

// Re-exports
pub use chrono_comp::ChronoComponent;
use keytree::KeyTree;
pub use render_comp::RenderComponent;
pub use selection_comp::SelectionComponent;
pub use trash_comp::TrashComponent;

// Imports
use self::chrono_comp::StrokeLayer;
use crate::engine::EngineSnapshot;
use crate::strokes::Stroke;
use crate::WidgetFlags;
use rnote_compose::shapes::ShapeBehaviour;
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

slotmap::new_key_type! {
    pub struct StrokeKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "history_entry")]
pub struct HistoryEntry {
    #[serde(rename = "stroke_components")]
    pub stroke_components: Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>,
    #[serde(rename = "trash_components")]
    pub trash_components: Arc<SecondaryMap<StrokeKey, Arc<TrashComponent>>>,
    #[serde(rename = "selection_components")]
    pub selection_components: Arc<SecondaryMap<StrokeKey, Arc<SelectionComponent>>>,
    #[serde(rename = "chrono_components")]
    pub chrono_components: Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>,

    #[serde(rename = "chrono_counter")]
    pub chrono_counter: u32,
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            stroke_components: Arc::new(HopSlotMap::with_key()),
            trash_components: Arc::new(SecondaryMap::new()),
            selection_components: Arc::new(SecondaryMap::new()),
            chrono_components: Arc::new(SecondaryMap::new()),

            chrono_counter: 0,
        }
    }
}

/// StrokeStore implements a Entity - Component - System pattern.
/// The Entities are the StrokeKey's, which represent a stroke. There are different components for them:
///     * 'stroke_components': Holds state about geometric properties. These components are special in the way that they are the primary map.
///         A new stroke must have this component. (another name for them could be 'geometric_components')
///     * 'trash_components': Holds state whether the strokes are trashed
///     * 'selection_components': Holds state whether the strokes are selected
///     * 'chrono_components': Holds state about the chronological ordering
///     * 'render_components': Holds state about the rendering.
///
/// The systems are implemented as methods on StrokesStore, loosely categorized to the different components (but often modify others as well).
/// Most systems take a key or a slice of keys, and iterate with them over the different components.
/// There also is a different category of methods which return filtered keys.
/// For example: [StrokeStore::keys_sorted_chrono] returns the keys in chronological ordering,
///     [StrokeStore::selection_keys_as_rendered] filters and returns only the selection keys in the order which should be drawn/rendered).

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "stroke_store")]
pub struct StrokeStore {
    // Components
    #[serde(rename = "stroke_components")]
    stroke_components: Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>,
    #[serde(rename = "trash_components")]
    trash_components: Arc<SecondaryMap<StrokeKey, Arc<TrashComponent>>>,
    #[serde(rename = "selection_components")]
    selection_components: Arc<SecondaryMap<StrokeKey, Arc<SelectionComponent>>>,
    #[serde(rename = "chrono_components")]
    chrono_components: Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>,
    #[serde(skip)]
    render_components: SecondaryMap<StrokeKey, RenderComponent>,

    #[serde(skip)]
    history: VecDeque<Arc<HistoryEntry>>,
    #[serde(skip)]
    history_pos: Option<usize>,

    /// An rtree backed by the slotmap store, for faster spatial queries.
    /// Needs to be updated with update_with_key() when strokes changed their geometry or position!
    #[serde(skip)]
    key_tree: KeyTree,

    /// Incrementing counter for chrono_components.
    ///
    /// Value must be equal to the [ChronoComponent] of the newest inserted or modified stroke.
    #[serde(rename = "chrono_counter")]
    chrono_counter: u32,
}

impl Default for StrokeStore {
    fn default() -> Self {
        Self {
            stroke_components: Arc::new(HopSlotMap::with_key()),
            trash_components: Arc::new(SecondaryMap::new()),
            selection_components: Arc::new(SecondaryMap::new()),
            chrono_components: Arc::new(SecondaryMap::new()),
            render_components: SecondaryMap::new(),

            history: VecDeque::new(),
            history_pos: None,

            key_tree: KeyTree::default(),

            chrono_counter: 0,
        }
    }
}

impl StrokeStore {
    /// Max length of the history.
    pub(crate) const HISTORY_MAX_LEN: usize = 100;

    /// Import from a engine snapshot. A loaded strokes store should always be imported with this method.
    ///
    /// The store then needs to update its rendering.
    pub fn import_from_snapshot(&mut self, snapshot: &EngineSnapshot) {
        self.clear();
        self.stroke_components = Arc::clone(&snapshot.stroke_components);
        self.chrono_components = Arc::clone(&snapshot.chrono_components);

        self.chrono_counter = snapshot.chrono_counter;

        self.update_geometry_for_strokes(&self.keys_unordered());

        self.rebuild_selection_components_slotmap();
        self.rebuild_trash_components_slotmap();
        self.rebuild_render_components_slotmap();
        self.reload_rtree();
    }

    /// Reload the rtree with the current bounds of the strokes.
    pub fn reload_rtree(&mut self) {
        let tree_objects = self
            .stroke_components
            .iter()
            .map(|(key, stroke)| (key, stroke.bounds()))
            .collect();
        self.key_tree.reload_with_vec(tree_objects);
    }

    /// Checks the pointer equality of current state to the given history entry.
    fn ptr_eq_w_history_entry(&self, history_entry: &Arc<HistoryEntry>) -> bool {
        Arc::ptr_eq(&self.stroke_components, &history_entry.stroke_components)
            && Arc::ptr_eq(&self.trash_components, &history_entry.trash_components)
            && Arc::ptr_eq(
                &self.selection_components,
                &history_entry.selection_components,
            )
            && Arc::ptr_eq(&self.chrono_components, &history_entry.chrono_components)
    }

    /// Create a history entry from the current state.
    pub fn history_entry_from_current_state(&self) -> Arc<HistoryEntry> {
        Arc::new(HistoryEntry {
            stroke_components: Arc::clone(&self.stroke_components),
            trash_components: Arc::clone(&self.trash_components),
            selection_components: Arc::clone(&self.selection_components),
            chrono_components: Arc::clone(&self.chrono_components),
            chrono_counter: self.chrono_counter,
        })
    }

    /// Import the given history entry and replaces the current state with it.
    fn import_history_entry(&mut self, history_entry: &Arc<HistoryEntry>) {
        self.stroke_components = Arc::clone(&history_entry.stroke_components);
        self.trash_components = Arc::clone(&history_entry.trash_components);
        self.selection_components = Arc::clone(&history_entry.selection_components);
        self.chrono_components = Arc::clone(&history_entry.chrono_components);
        self.chrono_counter = history_entry.chrono_counter;

        // Since we don't store the rtree in the history, we need to reload it.
        self.reload_rtree();
        // render components are also not stored in the history, but for the duration of the running app we don't ever remove them,
        // so we can actually skip rebuilding them when importing a history entry.
        // This avoids flickering when we have already rebuilt the components
        // and wouldn't be able to display anything until the rendering is finished.
        //self.reload_render_components_slotmap();

        let all_strokes = self.stroke_keys_unordered();
        self.set_rendering_dirty_for_strokes(&all_strokes);
    }

    /// Record the current state and saves it in the history.
    pub fn record(&mut self, _now: Instant) -> WidgetFlags {
        self.simple_style_record()
    }

    /// Undo the latest changes.
    ///
    /// Should only be called inside the engine undo wrapper function.
    pub(super) fn undo(&mut self, _now: Instant) -> WidgetFlags {
        self.simple_style_undo()
    }

    /// Redo the latest changes.
    ///
    /// Should only be called inside the engine redo wrapper function.
    pub(super) fn redo(&mut self, _now: Instant) -> WidgetFlags {
        self.simple_style_redo()
    }

    pub(super) fn can_undo(&self) -> bool {
        let index = self.history_pos.unwrap_or(self.history.len());

        index > 0
    }

    pub(super) fn can_redo(&self) -> bool {
        let history_len = self.history.len();
        let index = self.history_pos.unwrap_or(history_len);

        index + 1 < history_len
    }

    fn simple_style_record(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        // as soon as the current state is recorded, remove the future
        self.history.truncate(
            self.history_pos
                .map(|pos| pos + 1)
                .unwrap_or(self.history.len()),
        );
        self.history_pos = None;

        if self
            .history
            .back()
            .map(|last| !self.ptr_eq_w_history_entry(last))
            .unwrap_or(true)
        {
            self.history
                .push_back(self.history_entry_from_current_state());

            if self.history.len() > Self::HISTORY_MAX_LEN {
                self.history.pop_front();
            }
        } else {
            log::trace!("state has not changed, no need to record");
        }

        widget_flags.hide_redo = Some(true);
        widget_flags.hide_undo = Some(false);

        widget_flags
    }

    fn simple_style_undo(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        let index = match self.history_pos {
            Some(index) => index,
            None => {
                // If we are in the present, we push the current state to the history
                let current = self.history_entry_from_current_state();
                self.history.push_back(current);

                self.history.len() - 1
            }
        };

        if index > 0 {
            let prev = Arc::clone(&self.history[index - 1]);
            self.import_history_entry(&prev);

            self.history_pos = Some(index - 1);
            widget_flags.hide_redo = Some(false);
        } else {
            log::debug!("no history, can't undo");
        }

        widget_flags.hide_undo = Some(!self.can_undo());

        widget_flags
    }

    fn simple_style_redo(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        let index = self.history_pos.unwrap_or(self.history.len() - 1);

        if index < self.history.len() - 1 {
            let next = Arc::clone(&self.history[index + 1]);
            self.import_history_entry(&next);

            self.history_pos = Some(index + 1);
            widget_flags.hide_undo = Some(false);
        } else {
            log::debug!("no future history entries, can't redo");
        }

        widget_flags.hide_redo = Some(!self.can_redo());

        widget_flags
    }

    /// Save the current state in the history.
    ///
    /// Only to be used in combination with emacs_style_break_undo_chain() and emacs_style_undo()
    #[allow(unused)]
    fn emacs_style_record(&mut self) {
        self.history_pos = None;

        if self
            .history
            .back()
            .map(|last| !self.ptr_eq_w_history_entry(last))
            .unwrap_or(true)
        {
            self.history
                .push_back(self.history_entry_from_current_state());

            if self.history.len() > Self::HISTORY_MAX_LEN {
                self.history.pop_front();
            }
        } else {
            log::debug!("state has not changed, skipped record");
        }
    }

    /// Emacs style undo, where the undo operation is pushed to the history as well.
    ///
    /// Only to be used in combination with emacs_style_break_undo_chain() and emacs_style_record()
    ///
    /// The store then needs to update its rendering.
    #[allow(unused)]
    fn emacs_style_undo(&mut self) {
        let index = self.history_pos.unwrap_or(self.history.len());

        if index > 0 {
            let current = self.history_entry_from_current_state();
            let prev = Arc::clone(&self.history[index - 1]);

            self.history.push_back(current);

            self.import_history_entry(&prev);
            self.history_pos = Some(index - 1);
        } else {
            log::debug!("no history, can't undo");
        }
    }

    /// Breaks the undo chain, to enable redo by undoing the undos, emacs-style.
    ///
    /// Only to be used in combination with emacs_style_undo() and emacs_style_record()
    #[allow(unused)]
    fn emacs_style_break_undo_chain(&mut self) {
        // move the position to the end, so we can do "undo the undos", emacs-style
        self.history_pos = None;
    }

    /// Clear the entire history.
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_pos = None;
    }

    /// Insert a new stroke into the store.
    ///
    /// Optionally a desired layer can be specified, or the default stroke layer is used.
    ///
    /// The stroke then needs to update its rendering.
    pub fn insert_stroke(&mut self, stroke: Stroke, layer: Option<StrokeLayer>) -> StrokeKey {
        let bounds = stroke.bounds();
        let layer = layer.unwrap_or_else(|| stroke.extract_default_layer());

        let key = Arc::make_mut(&mut self.stroke_components).insert(Arc::new(stroke));
        self.key_tree.insert_with_key(key, bounds);
        self.chrono_counter += 1;

        Arc::make_mut(&mut self.trash_components).insert(key, Arc::new(TrashComponent::default()));
        Arc::make_mut(&mut self.selection_components)
            .insert(key, Arc::new(SelectionComponent::default()));
        Arc::make_mut(&mut self.chrono_components).insert(
            key,
            Arc::new(ChronoComponent::new(self.chrono_counter, layer)),
        );
        self.render_components
            .insert(key, RenderComponent::default());

        key
    }

    /// Permanently remove future history.
    pub fn remove_future(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        self.history
            .truncate(self.history_pos.unwrap_or(self.history.len()));
        self.history_pos = None;

        widget_flags.hide_redo = Some(true);
        widget_flags.hide_undo = Some(!self.can_undo());

        widget_flags
    }

    /// Permanently removes a stroke with the given key from the store.
    pub fn remove_stroke(&mut self, key: StrokeKey) -> Option<Stroke> {
        Arc::make_mut(&mut self.trash_components).remove(key);
        Arc::make_mut(&mut self.selection_components).remove(key);
        Arc::make_mut(&mut self.chrono_components).remove(key);
        self.render_components.remove(key);

        self.key_tree.remove_with_key(key);
        Arc::make_mut(&mut self.stroke_components)
            .remove(key)
            .map(|stroke| (*stroke).clone())
    }

    /// Clears the entire store.
    pub(super) fn clear(&mut self) {
        Arc::make_mut(&mut self.stroke_components).clear();
        Arc::make_mut(&mut self.trash_components).clear();
        Arc::make_mut(&mut self.selection_components).clear();
        Arc::make_mut(&mut self.chrono_components).clear();

        self.chrono_counter = 0;
        self.clear_history();

        self.render_components.clear();
        self.key_tree.clear();
    }
}
