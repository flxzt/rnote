// Modules
pub mod export;
pub mod import;
pub mod rendering;
pub mod snapshot;
pub mod strokecontent;
pub mod visual_debug;

// Re-exports
pub use export::ExportPrefs;
pub use import::ImportPrefs;
pub use snapshot::EngineSnapshot;
pub use strokecontent::StrokeContent;

// Imports
use crate::document::Layout;
use crate::pens::{Pen, PenStyle};
use crate::pens::{PenMode, PensConfig};
use crate::store::render_comp::{self, RenderCompState};
use crate::store::StrokeKey;
use crate::strokes::strokebehaviour::GeneratedStrokeImages;
use crate::{render, AudioPlayer, WidgetFlags};
use crate::{Camera, Document, PenHolder, StrokeStore};
use futures::channel::{mpsc, oneshot};
use gtk4::gsk;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::helpers::{AabbHelpers, SplitOrder};
use rnote_compose::penevents::{PenEvent, ShortcutKey};
use rnote_compose::shapes::ShapeBehaviour;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// An immutable view into the engine, excluding the penholder.
#[derive(Debug)]
pub struct EngineView<'a> {
    pub tasks_tx: EngineTaskSender,
    pub pens_config: &'a PensConfig,
    pub doc: &'a Document,
    pub store: &'a StrokeStore,
    pub camera: &'a Camera,
    pub audioplayer: &'a Option<AudioPlayer>,
}

/// A mutable view into the engine, excluding the penholder.
#[derive(Debug)]
pub struct EngineViewMut<'a> {
    pub tasks_tx: EngineTaskSender,
    pub pens_config: &'a mut PensConfig,
    pub doc: &'a mut Document,
    pub store: &'a mut StrokeStore,
    pub camera: &'a mut Camera,
    pub audioplayer: &'a mut Option<AudioPlayer>,
}

impl<'a> EngineViewMut<'a> {
    // Converts itself to the immutable view.
    pub fn as_im<'m>(&'m self) -> EngineView<'m> {
        EngineView::<'m> {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: self.pens_config,
            doc: self.doc,
            store: self.store,
            camera: self.camera,
            audioplayer: self.audioplayer,
        }
    }
}

#[derive(Debug, Clone)]
/// An engine task, usually coming from a spawned thread and to be processed with [RnoteEngine::handle_engine_task].
pub enum EngineTask {
    /// Replace the images for rendering of the given stroke.
    ///
    /// The state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component's state on resize/zooming, etc. .
    UpdateStrokeWithImages {
        /// The stroke key.
        key: StrokeKey,
        /// The generated images.
        images: GeneratedStrokeImages,
        /// The image scale-factor the render task was using while generating the images.
        image_scale: f64,
        /// The stroke bounds at the time when the render task has launched.
        stroke_bounds: Aabb,
    },
    /// Appends the images to the rendering of the given stroke.
    ///
    /// The state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component's state on resize/zooming, etc. .
    AppendImagesToStroke {
        /// The stroke key
        key: StrokeKey,
        /// The generated images
        images: GeneratedStrokeImages,
    },
    /// Requests that the typewriter cursor should be blinked/toggled
    BlinkTypewriterCursor,
    /// Change the permanent zoom to the given value
    Zoom(f64),
    /// Indicates that the application is quitting. Sent to quit the handler which receives the tasks.
    Quit,
}

/// The engine configuration. Used when loading/saving the current configuration from/into persistent application settings.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename = "engine_config")]
pub struct EngineConfig {
    #[serde(rename = "document")]
    document: Document,
    #[serde(rename = "pens_config")]
    pens_config: PensConfig,
    #[serde(rename = "penholder")]
    penholder: PenHolder,
    #[serde(rename = "import_prefs")]
    import_prefs: ImportPrefs,
    #[serde(rename = "export_prefs")]
    export_prefs: ExportPrefs,
    #[serde(rename = "pen_sounds")]
    pen_sounds: bool,
}

pub type EngineTaskSender = mpsc::UnboundedSender<EngineTask>;
pub type EngineTaskReceiver = mpsc::UnboundedReceiver<EngineTask>;

/// The engine.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "engine")]
pub struct RnoteEngine {
    #[serde(rename = "document")]
    pub document: Document,
    #[serde(rename = "store")]
    pub store: StrokeStore,
    #[serde(rename = "pens_config")]
    pub pens_config: PensConfig,
    #[serde(rename = "camera")]
    pub camera: Camera,
    #[serde(rename = "penholder")]
    pub penholder: PenHolder,

    #[serde(rename = "import_prefs")]
    pub import_prefs: ImportPrefs,
    #[serde(rename = "export_prefs")]
    pub export_prefs: ExportPrefs,
    #[serde(rename = "pen_sounds")]
    pen_sounds: bool,

    #[serde(skip)]
    pub audioplayer: Option<AudioPlayer>,
    #[serde(skip)]
    pub visual_debug: bool,
    // the task sender. Must not be modified, only cloned.
    #[serde(skip)]
    pub tasks_tx: EngineTaskSender,
    #[serde(skip)]
    pub tasks_rx: Option<EngineTaskReceiver>,
    // Background rendering
    #[serde(skip)]
    pub background_tile_image: Option<render::Image>,
    #[serde(skip)]
    background_rendernodes: Vec<gsk::RenderNode>,
}

impl Default for RnoteEngine {
    fn default() -> Self {
        let (tasks_tx, tasks_rx) = futures::channel::mpsc::unbounded::<EngineTask>();

        Self {
            document: Document::default(),
            store: StrokeStore::default(),
            pens_config: PensConfig::default(),
            camera: Camera::default(),
            penholder: PenHolder::default(),

            import_prefs: ImportPrefs::default(),
            export_prefs: ExportPrefs::default(),
            pen_sounds: false,

            audioplayer: None,
            visual_debug: false,
            tasks_tx,
            tasks_rx: Some(tasks_rx),
            background_tile_image: None,
            background_rendernodes: Vec::default(),
        }
    }
}

impl RnoteEngine {
    pub fn tasks_tx(&self) -> EngineTaskSender {
        self.tasks_tx.clone()
    }

    pub fn view(&self) -> EngineView {
        EngineView {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: &self.pens_config,
            doc: &self.document,
            store: &self.store,
            camera: &self.camera,
            audioplayer: &self.audioplayer,
        }
    }

    pub fn view_mut(&mut self) -> EngineViewMut {
        EngineViewMut {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: &mut self.pens_config,
            doc: &mut self.document,
            store: &mut self.store,
            camera: &mut self.camera,
            audioplayer: &mut self.audioplayer,
        }
    }

    /// Whether pen sounds are enabled.
    pub fn pen_sounds(&self) -> bool {
        self.pen_sounds
    }

    /// Enables/disables the pen sounds.
    ///
    /// If pen sound should be enabled, the pkg data dir must be provided.
    pub fn set_pen_sounds(&mut self, pen_sounds: bool, pkg_data_dir: Option<PathBuf>) {
        self.pen_sounds = pen_sounds;

        if pen_sounds {
            if let Some(pkg_data_dir) = pkg_data_dir {
                // Only create and init a new audioplayer if it does not already exists
                if self.audioplayer.is_none() {
                    self.audioplayer = match AudioPlayer::new_init(pkg_data_dir) {
                        Ok(audioplayer) => Some(audioplayer),
                        Err(e) => {
                            log::error!("creating a new audioplayer failed, Err: {e:?}");
                            None
                        }
                    }
                }
            }
        } else {
            self.audioplayer.take();
        }
    }

    /// Takes a snapshot of the current state.
    pub fn take_snapshot(&self) -> EngineSnapshot {
        let mut store_history_entry = self.store.create_history_entry();

        // Remove all trashed strokes
        let trashed_keys = store_history_entry
            .trash_components
            .iter()
            .filter_map(|(key, trash_comp)| if trash_comp.trashed { Some(key) } else { None })
            .collect::<Vec<StrokeKey>>();

        for key in trashed_keys {
            Arc::make_mut(&mut store_history_entry.stroke_components).remove(key);
        }

        EngineSnapshot {
            document: self.document,
            stroke_components: Arc::clone(&store_history_entry.stroke_components),
            chrono_components: Arc::clone(&store_history_entry.chrono_components),
            chrono_counter: store_history_entry.chrono_counter,
        }
    }

    /// Imports an engine snapshot. A save file should always be loaded with this method.
    ///
    /// The store then needs to update its rendering.
    pub fn load_snapshot(&mut self, snapshot: EngineSnapshot) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        self.document = snapshot.document;
        widget_flags.merge(self.store.import_from_snapshot(&snapshot));
        widget_flags.merge(self.current_pen_update_state());

        widget_flags
    }

    /// Records the current store state and saves it as a history entry.
    pub fn record(&mut self, now: Instant) -> WidgetFlags {
        self.store.record(now)
    }

    /// Update the state of the latest history entry with the current document state.
    pub fn update_latest_history_entry(&mut self, now: Instant) -> WidgetFlags {
        self.store.update_latest_history_entry(now)
    }

    /// Undo the latest changes.
    pub fn undo(&mut self, now: Instant) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        widget_flags.merge(self.store.undo(now));
        widget_flags.merge(self.doc_resize_autoexpand());
        widget_flags.merge(self.current_pen_update_state());
        self.update_rendering_current_viewport();
        widget_flags.redraw = true;

        widget_flags
    }

    /// Redo the latest changes.
    pub fn redo(&mut self, now: Instant) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        widget_flags.merge(self.store.redo(now));
        widget_flags.merge(self.doc_resize_autoexpand());
        widget_flags.merge(self.current_pen_update_state());
        self.update_rendering_current_viewport();
        widget_flags.redraw = true;

        widget_flags
    }

    pub fn can_undo(&self) -> bool {
        self.store.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.store.can_redo()
    }

    // Clears the entire store.
    pub fn clear(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        widget_flags.merge(self.store.clear());
        widget_flags.merge(self.current_pen_update_state());

        widget_flags
    }

    /// Handle a received task from tasks_rx.
    /// Returns [WidgetFlags] to indicate what needs to be updated in the UI.
    ///
    /// An example how to use it:
    /// ```rust, ignore
    ///
    /// glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
    ///    let mut task_rx = canvas.engine_mut().regenerate_channel();
    ///
    ///    loop {
    ///        if let Some(task) = task_rx.next().await {
    ///            let (widget_flags, quit) = canvas.engine_mut().handle_engine_task(task);
    ///            canvas.emit_handle_widget_flags(widget_flags);

    ///            if quit {
    ///                break;
    ///            }
    ///        }
    ///    }
    /// }));
    /// ```
    pub fn handle_engine_task(&mut self, task: EngineTask) -> (WidgetFlags, bool) {
        let mut widget_flags = WidgetFlags::default();
        let mut quit = false;

        match task {
            EngineTask::UpdateStrokeWithImages {
                key,
                images,
                image_scale,
                stroke_bounds,
            } => {
                if let Some(state) = self.store.render_comp_state(key) {
                    match state {
                        RenderCompState::Complete | RenderCompState::ForViewport(_) => {
                            // The rendering was already regenerated in the meantime,
                            // so we just discard the the render task result
                        }
                        RenderCompState::BusyRenderingInTask => {
                            if (self.camera.image_scale()
                                - render_comp::RENDER_IMAGE_SCALE_EQUALITY_TOLERANCE
                                ..self.camera.image_scale()
                                    + render_comp::RENDER_IMAGE_SCALE_EQUALITY_TOLERANCE)
                                .contains(&image_scale)
                                && self
                                    .store
                                    .get_stroke_ref(key)
                                    .map(|s| s.bounds() == stroke_bounds)
                                    .unwrap_or(true)
                            {
                                // Only when the image scale and stroke bounds are the same
                                // to when the render task was started,
                                // the new images are considered valid and can replace the old.
                                self.store.replace_rendering_with_images(key, images);
                            }
                            widget_flags.redraw = true;
                        }
                        RenderCompState::Dirty => {
                            // If the state was flagged dirty in the meantime,
                            // it is expected that retriggering rendering will be handled elsewhere
                        }
                    }
                }
            }
            EngineTask::AppendImagesToStroke { key, images } => {
                self.store.append_rendering_images(key, images);
                widget_flags.redraw = true;
            }
            EngineTask::BlinkTypewriterCursor => {
                if let Pen::Typewriter(typewriter) = self.penholder.current_pen_mut() {
                    typewriter.toggle_cursor_visibility();
                    widget_flags.redraw = true;
                }
            }
            EngineTask::Zoom(zoom) => {
                widget_flags.merge(self.camera.zoom_temporarily_to(1.0));
                widget_flags.merge(self.camera.zoom_to(zoom));

                let all_strokes = self.store.stroke_keys_unordered();
                self.store.set_rendering_dirty_for_strokes(&all_strokes);
                widget_flags.merge(self.doc_resize_autoexpand());

                self.background_regenerate_pattern();
                self.update_rendering_current_viewport();
            }
            EngineTask::Quit => {
                widget_flags.merge(self.set_active(false));
                quit = true;
            }
        }

        (widget_flags, quit)
    }

    /// Handle a pen event.
    pub fn handle_pen_event(
        &mut self,
        event: PenEvent,
        pen_mode: Option<PenMode>,
        now: Instant,
    ) -> WidgetFlags {
        self.penholder.handle_pen_event(
            event,
            pen_mode,
            now,
            &mut EngineViewMut {
                tasks_tx: self.tasks_tx(),
                pens_config: &mut self.pens_config,
                doc: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            },
        )
    }

    /// Handle a pressed shortcut key.
    pub fn handle_pressed_shortcut_key(
        &mut self,
        shortcut_key: ShortcutKey,
        now: Instant,
    ) -> WidgetFlags {
        self.penholder.handle_pressed_shortcut_key(
            shortcut_key,
            now,
            &mut EngineViewMut {
                tasks_tx: self.tasks_tx(),
                pens_config: &mut self.pens_config,
                doc: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            },
        )
    }

    /// Change the pen style.
    pub fn change_pen_style(&mut self, new_style: PenStyle) -> WidgetFlags {
        self.penholder.change_style(
            new_style,
            &mut EngineViewMut {
                tasks_tx: self.tasks_tx(),
                pens_config: &mut self.pens_config,
                doc: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            },
        )
    }

    /// Change the pen style (temporary) override.
    pub fn change_pen_style_override(
        &mut self,
        new_style_override: Option<PenStyle>,
    ) -> WidgetFlags {
        self.penholder.change_style_override(
            new_style_override,
            &mut EngineViewMut {
                tasks_tx: self.tasks_tx(),
                pens_config: &mut self.pens_config,
                doc: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            },
        )
    }

    /// Change the pen mode. Relevant for stylus input.
    pub fn change_pen_mode(&mut self, pen_mode: PenMode) -> WidgetFlags {
        self.penholder.change_pen_mode(
            pen_mode,
            &mut EngineViewMut {
                tasks_tx: self.tasks_tx(),
                pens_config: &mut self.pens_config,
                doc: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            },
        )
    }

    /// Reinstall the pen in the current style.
    pub fn reinstall_pen_current_style(&mut self) -> WidgetFlags {
        self.penholder
            .reinstall_pen_current_style(&mut EngineViewMut {
                tasks_tx: self.tasks_tx(),
                pens_config: &mut self.pens_config,
                doc: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            })
    }

    /// Set the engine active or inactive.
    pub fn set_active(&mut self, active: bool) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if active {
            widget_flags.merge(self.reinstall_pen_current_style());
            self.background_regenerate_pattern();
            self.update_content_rendering_current_viewport();
        } else {
            self.clear_rendering();
            widget_flags.merge(self.penholder.deinit_current_pen());
        }
        widget_flags
    }

    /// Generate bounds for each page on the document which contains content.
    pub fn pages_bounds_w_content(&self, split_order: SplitOrder) -> Vec<Aabb> {
        let doc_bounds = self.document.bounds();
        let keys = self.store.stroke_keys_as_rendered();

        let strokes_bounds = self.store.strokes_bounds(&keys);

        let pages_bounds = doc_bounds
            .split_extended_origin_aligned(
                na::vector![self.document.format.width, self.document.format.height],
                split_order,
            )
            .into_iter()
            .filter(|page_bounds| {
                // Filter the pages out that don't intersect with any stroke
                strokes_bounds
                    .iter()
                    .any(|stroke_bounds| stroke_bounds.intersects(page_bounds))
            })
            .collect::<Vec<Aabb>>();

        if pages_bounds.is_empty() {
            // If no page has content, return the origin page
            vec![Aabb::new(
                na::point![0.0, 0.0],
                na::point![self.document.format.width, self.document.format.height],
            )]
        } else {
            pages_bounds
        }
    }

    /// Generates bounds which contain all pages on the doc with content, extended to fit the current format.
    pub fn bounds_w_content_extended(&self) -> Option<Aabb> {
        let pages_bounds = self.pages_bounds_w_content(SplitOrder::default());

        if pages_bounds.is_empty() {
            return None;
        }

        Some(
            pages_bounds
                .into_iter()
                .fold(Aabb::new_invalid(), |prev, next| prev.merged(&next)),
        )
    }

    /// First zoom temporarily and then permanently after a timeout.
    ///
    /// Repeated calls to this function reset the timeout.
    pub fn zoom_w_timeout(&mut self, zoom: f64) -> WidgetFlags {
        self.camera.zoom_w_timeout(zoom, self.tasks_tx.clone())
    }

    /// Resizes the doc to the format and to fit all strokes.
    ///
    /// Background rendering then needs to be updated.
    pub fn doc_resize_to_fit_strokes(&mut self) -> WidgetFlags {
        self.document
            .resize_to_fit_strokes(&self.store, &self.camera)
    }

    /// Resize the doc when in autoexpanding layouts. called e.g. when finishing a new stroke.
    ///
    /// Background rendering then needs to be updated.
    pub fn doc_resize_autoexpand(&mut self) -> WidgetFlags {
        self.document.resize_autoexpand(&self.store, &self.camera)
    }

    /// Expand the doc to the camera when in autoexpanding layouts. called e.g. when dragging with touch.
    ///
    /// Background rendering then needs to be updated.
    pub fn doc_expand_autoexpand(&mut self) -> WidgetFlags {
        self.document.expand_autoexpand(&self.camera)
    }

    /// Add a page to the document when in fixed size layout.
    ///
    /// Returns true when document is in fixed size layout and a pages was added,
    /// else false.
    ///
    /// Background and strokes rendering then need to be updated.
    pub fn doc_add_page_fixed_size(&mut self) -> bool {
        if self.document.layout != Layout::FixedSize {
            return false;
        }

        let format_height = self.document.format.height;
        let new_doc_height = self.document.height + format_height;
        self.document.height = new_doc_height;

        true
    }

    /// Remove a page from the document when in fixed size layout.
    ///
    /// Returns true when document is in fixed size layout and a page was removed,
    /// else false.
    ///
    /// Background and strokes rendering then need to be updated.
    pub fn doc_remove_page_fixed_size(&mut self) -> bool {
        if self.document.layout != Layout::FixedSize {
            return false;
        }
        let format_height = self.document.format.height;
        let doc_y = self.document.y;
        let doc_height = self.document.height;
        let new_doc_height = doc_height - format_height;

        if doc_height > format_height {
            let remove_area_keys = self.store.keys_below_y(doc_y + new_doc_height);
            self.store.set_trashed_keys(&remove_area_keys, true);
            self.document.height = new_doc_height;
        }

        true
    }

    /// Update the viewport offset of the camera, clamped to mins and maxs values depending on the document layout.
    ///
    /// Background and strokes rendering then need to be updated.
    pub fn camera_set_offset(&mut self, offset: na::Vector2<f64>) -> WidgetFlags {
        self.camera.set_offset(offset, &self.document)
    }

    /// Update the viewport size of the camera.
    ///
    /// Background and strokes rendering then need to be updated.
    pub fn camera_set_size(&mut self, size: na::Vector2<f64>) -> WidgetFlags {
        self.camera.set_size(size)
    }

    /// Update the viewport size of the camera.
    ///
    /// Background and strokes rendering then need to be updated.
    pub fn camera_offset_mins_maxs(&mut self) -> (na::Vector2<f64>, na::Vector2<f64>) {
        self.camera.offset_lower_upper(&self.document)
    }

    /// Update the current pen with the current engine state.
    ///
    /// Needs to be called when the engine state was changed outside of pen events.
    /// ( e.g. trash all strokes, set strokes selected, etc. )
    pub fn current_pen_update_state(&mut self) -> WidgetFlags {
        self.penholder.current_pen_update_state(&mut EngineViewMut {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: &mut self.pens_config,
            doc: &mut self.document,
            store: &mut self.store,
            camera: &mut self.camera,
            audioplayer: &mut self.audioplayer,
        })
    }

    /// Fetch clipboard content from the current pen.
    #[allow(clippy::type_complexity)]
    pub fn fetch_clipboard_content(
        &self,
    ) -> oneshot::Receiver<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>> {
        self.penholder.fetch_clipboard_content(&EngineView {
            tasks_tx: self.tasks_tx(),
            pens_config: &self.pens_config,
            doc: &self.document,
            store: &self.store,
            camera: &self.camera,
            audioplayer: &self.audioplayer,
        })
    }

    /// Cut clipboard content from the current pen.
    #[allow(clippy::type_complexity)]
    pub fn cut_clipboard_content(
        &mut self,
    ) -> oneshot::Receiver<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>> {
        self.penholder.cut_clipboard_content(&mut EngineViewMut {
            tasks_tx: self.tasks_tx(),
            pens_config: &mut self.pens_config,
            doc: &mut self.document,
            store: &mut self.store,
            camera: &mut self.camera,
            audioplayer: &mut self.audioplayer,
        })
    }

    pub fn extract_document_content(&self) -> StrokeContent {
        StrokeContent::default()
            .with_strokes(
                self.store
                    .get_strokes_arc(&self.store.stroke_keys_as_rendered()),
            )
            .with_bounds(self.bounds_w_content_extended())
            .with_background(Some(self.document.background))
    }
}
