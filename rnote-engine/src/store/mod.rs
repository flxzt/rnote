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

use std::collections::VecDeque;
use std::sync::Arc;

use crate::pens::penholder::PenStyle;
use crate::strokes::strokebehaviour::GeneratedStrokeImages;
use crate::strokes::Stroke;
use crate::surfaceflags::SurfaceFlags;
use crate::Camera;
use rnote_compose::shapes::ShapeBehaviour;
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};

/// StrokeStore implements a Entity - Component - System pattern.
/// The Entities are the StrokeKey's, which represent a stroke. There are different components for them:
///     * 'stroke_components': Hold geometric data. These components are special in that they are the primary map. A new stroke must have this component. (could also be called geometric components)
///     * 'trash_components': Hold state wether the strokes are trashed
///     * 'selection_components': Hold state wether the strokes are selected
///     * 'chrono_components': Hold state about the chronological ordering
///     * 'render_components': Hold state about the current rendering of the strokes.
///
/// The systems are implemented as methods on StrokesStore, loosely categorized to the different components (but often modify others as well).
/// Most systems take a key or a slice of keys, and iterate with them over the different components.
/// There also is a different category of methods which return filtered keys, (e.g. `.keys_sorted_chrono` returns the keys in chronological ordering,
///     `.stroke_keys_in_order_rendering` filters and returns keys in the order which they should be rendered)

#[derive(Debug, Clone)]
/// A store task, usually coming from a spawned thread and to be processed with `process_received_task()`.
pub enum StoreTask {
    /// Replace the images of the render_comp.
    /// Note that usually the state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component state on resize / zooming, etc.
    UpdateStrokeWithImages {
        key: StrokeKey,
        images: GeneratedStrokeImages,
    },
    /// Appends the images to the rendering of the stroke
    /// Note that usually the state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component state on resize / zooming, etc.
    AppendImagesToStroke {
        key: StrokeKey,
        images: GeneratedStrokeImages,
    },
    /// Inserts a new stroke to the store
    /// Note that usually the state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component state on resize / zooming, etc.
    InsertStroke { stroke: Stroke },
    /// indicates that the application is quitting. Usually handled to quit the async loop which receives the tasks
    Quit,
}

fn default_threadpool() -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::default()
        .build()
        .unwrap_or_else(|e| {
            log::error!("default_render_threadpool() failed with Err {}", e);
            panic!()
        })
}

slotmap::new_key_type! {
    pub struct StrokeKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    strokes: Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>,
    trash_components: Arc<SecondaryMap<StrokeKey, Arc<TrashComponent>>>,
    selection_components: Arc<SecondaryMap<StrokeKey, Arc<SelectionComponent>>>,
    chrono_components: Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>,
}

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

    // The history
    #[serde(skip)]
    history: VecDeque<Arc<HistoryEntry>>,
    #[serde(skip)]
    history_pos: Option<usize>,

    // A rtree backed by the slotmap, for faster spatial queries. Needs to be updated with update_with_key() when strokes changed their geometry or position!
    #[serde(skip)]
    key_tree: KeyTree,

    // Other state
    /// incrementing counter for chrono_components. value is equal chrono_component of the newest inserted or modified stroke.
    #[serde(rename = "chrono_counter")]
    chrono_counter: u32,

    #[serde(skip)]
    pub tasks_tx: futures::channel::mpsc::UnboundedSender<StoreTask>,
    /// To be taken out into a loop which processes the receiver stream. The received tasks should be processed with process_received_task()
    #[serde(skip)]
    pub tasks_rx: Option<futures::channel::mpsc::UnboundedReceiver<StoreTask>>,
    #[serde(skip, default = "default_threadpool")]
    threadpool: rayon::ThreadPool,
}

impl Default for StrokeStore {
    fn default() -> Self {
        let threadpool = default_threadpool();

        let (tasks_tx, tasks_rx) = futures::channel::mpsc::unbounded::<StoreTask>();

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

            tasks_tx,
            tasks_rx: Some(tasks_rx),
            threadpool,
        }
    }
}

impl StrokeStore {
    /// The max length of the history
    pub(crate) const HISTORY_MAX_LEN: usize = 100;

    pub fn new() -> Self {
        Self::default()
    }

    /// imports a store. A loaded strokes store should always be imported with this method, to not replace the threadpool, channel handlers..
    /// stroke then needs to update its rendering
    pub fn import_store(&mut self, store: Self) {
        self.clear();
        self.stroke_components = store.stroke_components;
        self.trash_components = store.trash_components;
        self.selection_components = store.selection_components;
        self.chrono_components = store.chrono_components;

        self.chrono_counter = store.chrono_counter;

        self.reload_tree();
        self.reload_render_components_slotmap();
    }

    /// Reloads the rtree with the current bounds of the strokes.
    pub fn reload_tree(&mut self) {
        let tree_objects = self
            .stroke_components
            .iter()
            .map(|(key, stroke)| (key, stroke.bounds()))
            .collect();
        self.key_tree.reload_with_vec(tree_objects);
    }

    /// Processes a received store task. Usually called from a receiver loop which polls tasks_rx.
    pub(crate) fn process_received_task(
        &mut self,
        task: StoreTask,
        camera: &Camera,
    ) -> SurfaceFlags {
        let viewport_expanded = camera.viewport();
        let image_scale = camera.image_scale();
        let mut surface_flags = SurfaceFlags::default();

        match task {
            StoreTask::UpdateStrokeWithImages { key, images } => {
                if let Err(e) = self.replace_rendering_with_images(key, images) {
                    log::error!("replace_rendering_with_images() in process_received_task() failed with Err {}", e);
                }

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;
            }
            StoreTask::AppendImagesToStroke { key, images } => {
                if let Err(e) = self.append_rendering_images(key, images) {
                    log::error!(
                        "append_rendering_images() in process_received_task() failed with Err {}",
                        e
                    );
                }

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;
            }
            StoreTask::InsertStroke { stroke } => {
                self.record();

                match stroke {
                    Stroke::BrushStroke(brushstroke) => {
                        let _inserted = self.insert_stroke(Stroke::BrushStroke(brushstroke));

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                    }
                    Stroke::ShapeStroke(shapestroke) => {
                        let _inserted = self.insert_stroke(Stroke::ShapeStroke(shapestroke));

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                    }
                    Stroke::VectorImage(vectorimage) => {
                        let inserted = self.insert_stroke(Stroke::VectorImage(vectorimage));
                        self.set_selected(inserted, true);

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.resize_to_fit_strokes = true;
                        surface_flags.change_to_pen = Some(PenStyle::Selector);
                        surface_flags.sheet_changed = true;
                        surface_flags.update_selector = true;
                    }
                    Stroke::BitmapImage(bitmapimage) => {
                        let inserted = self.insert_stroke(Stroke::BitmapImage(bitmapimage));
                        self.set_selected(inserted, true);

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.resize_to_fit_strokes = true;
                        surface_flags.change_to_pen = Some(PenStyle::Selector);
                        surface_flags.sheet_changed = true;
                        surface_flags.update_selector = true;
                    }
                }

                self.regenerate_rendering_in_viewport_threaded(
                    false,
                    viewport_expanded,
                    image_scale,
                );
            }
            StoreTask::Quit => {
                surface_flags.quit = true;
            }
        }

        surface_flags
    }

    /// Returns true if the current state is pointer equal to the given history entry
    fn ptr_eq_history(&self, history_entry: &Arc<HistoryEntry>) -> bool {
        Arc::ptr_eq(&self.stroke_components, &history_entry.strokes)
            && Arc::ptr_eq(&self.trash_components, &history_entry.trash_components)
            && Arc::ptr_eq(
                &self.selection_components,
                &history_entry.selection_components,
            )
            && Arc::ptr_eq(&self.chrono_components, &history_entry.chrono_components)
    }

    /// Returns a history entry created from the current state
    fn history_entry_from_current_state(&self) -> Arc<HistoryEntry> {
        Arc::new(HistoryEntry {
            strokes: Arc::clone(&self.stroke_components),
            trash_components: Arc::clone(&self.trash_components),
            selection_components: Arc::clone(&self.selection_components),
            chrono_components: Arc::clone(&self.chrono_components),
        })
    }

    /// Imports a given history entry and replaces the current state with it.
    fn import_history_entry(&mut self, history_entry: &Arc<HistoryEntry>) {
        self.stroke_components = Arc::clone(&history_entry.strokes);
        self.trash_components = Arc::clone(&history_entry.trash_components);
        self.selection_components = Arc::clone(&history_entry.selection_components);
        self.chrono_components = Arc::clone(&history_entry.chrono_components);

        // Since we don't store the tree in the history, we need to reload it.
        self.reload_tree();
        // render_components are also not stored in the history, but for the duration of the running app we don't ever remove from render_components,
        // so we can actually skip rebuilding it when importing a history entry. This avoids visual glitches where we have already rebuilt the components
        // and can't display anything until the asynchronous rendering is finished
        //self.reload_render_components_slotmap();
    }

    /// Saves the current state in the history
    pub fn record(&mut self) {
        /*
               log::debug!(
                   "before record - history len: {}, pos: {:?}",
                   self.history.len(),
                   self.history_pos
               );
        */
        self.simple_style_record();
        /*
               log::debug!(
                   "after record - history len: {}, pos: {:?}",
                   self.history.len(),
                   self.history_pos
               );
        */
    }

    /// Undos the latest changes
    pub fn undo(&mut self) {
        /*
               log::debug!(
                   "before undo - history len: {}, pos: {:?}",
                   self.history.len(),
                   self.history_pos
               );
        */
        self.simple_style_undo();
        /*
               log::debug!(
                   "after undo - history len: {}, pos: {:?}",
                   self.history.len(),
                   self.history_pos
               );
        */
    }

    /// Redos the latest changes. The actual behaviour might differ depending on the history mode (simple style, emacs style, ..)
    pub fn redo(&mut self) {
        /*
               log::debug!(
                   "before redo - history len: {}, pos: {:?}",
                   self.history.len(),
                   self.history_pos
               );
        */
        self.simple_style_redo();
        /*
               log::debug!(
                   "after redo - history len: {}, pos: {:?}",
                   self.history.len(),
                   self.history_pos
               );
        */
    }

    fn simple_style_record(&mut self) {
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
            .map(|last| !self.ptr_eq_history(last))
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
    }

    fn simple_style_undo(&mut self) {
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
        } else {
            log::debug!("no history, can't undo");
        }
    }

    fn simple_style_redo(&mut self) {
        let index = self.history_pos.unwrap_or(self.history.len() - 1);

        if index < self.history.len() - 1 {
            let next = Arc::clone(&self.history[index + 1]);
            self.import_history_entry(&next);

            self.history_pos = Some(index + 1);
        } else {
            log::debug!("no future history entries, can't redo");
        }
    }

    /// Saves the current state in the history.
    /// Only to be used in combination with emacs_style_break_undo_chain() and emacs_style_undo()
    #[allow(unused)]
    fn emacs_style_record(&mut self) {
        self.history_pos = None;

        if self
            .history
            .back()
            .map(|last| !self.ptr_eq_history(last))
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
    }

    /// emacs style undo, where the undo operation is pushed to the history as well.
    /// after that, regenerate rendering for the current viewport.
    /// Only to be used in combination with emacs_style_break_undo_chain() and emacs_style_record()
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
    /// Only to be used in combination with emacs_style_undo() and emacs_style_record()
    #[allow(unused)]
    fn emacs_style_break_undo_chain(&mut self) {
        // move the position to the end, so we can do "undo the undos", emacs-style
        self.history_pos = None;
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_pos = None;
    }

    /// inserts a new stroke into the store
    /// stroke then needs to update its rendering
    pub fn insert_stroke(&mut self, stroke: Stroke) -> StrokeKey {
        let bounds = stroke.bounds();

        let key = Arc::make_mut(&mut self.stroke_components).insert(Arc::new(stroke));
        self.key_tree.insert_with_key(key, bounds);
        self.chrono_counter += 1;

        Arc::make_mut(&mut self.trash_components).insert(key, Arc::new(TrashComponent::default()));
        Arc::make_mut(&mut self.selection_components)
            .insert(key, Arc::new(SelectionComponent::default()));
        Arc::make_mut(&mut self.chrono_components)
            .insert(key, Arc::new(ChronoComponent::new(self.chrono_counter)));
        self.render_components
            .insert(key, RenderComponent::default());

        key
    }

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

    /// Clears the entire store
    pub fn clear(&mut self) {
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
