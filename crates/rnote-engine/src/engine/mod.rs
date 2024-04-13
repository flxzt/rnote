// Modules
pub mod export;
pub mod import;
pub mod rendering;
pub mod snapshot;
pub mod strokecontent;
pub mod visual_debug;

// Re-exports
pub use export::ExportPrefs;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
pub use import::ImportPrefs;
pub use snapshot::EngineSnapshot;
pub use strokecontent::StrokeContent;

// Imports
use crate::document::Layout;
use crate::pens::{Pen, PenStyle};
use crate::pens::{PenMode, PensConfig};
use crate::store::render_comp::{self, RenderCompState};
use crate::store::StrokeKey;
use crate::strokes::content::GeneratedContentImages;
use crate::strokes::textstroke::{TextAttribute, TextStyle};
use crate::{render, AudioPlayer, CloneConfig, SelectionCollision, WidgetFlags};
use crate::{Camera, Document, PenHolder, StrokeStore};
use futures::channel::{mpsc, oneshot};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::eventresult::EventPropagation;
use rnote_compose::ext::AabbExt;
use rnote_compose::penevent::{PenEvent, ShortcutKey};
use rnote_compose::{Color, SplitOrder};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// An immutable view into the engine, excluding the penholder.
#[derive(Debug)]
pub struct EngineView<'a> {
    pub tasks_tx: EngineTaskSender,
    pub pens_config: &'a PensConfig,
    pub document: &'a Document,
    pub store: &'a StrokeStore,
    pub camera: &'a Camera,
    pub audioplayer: &'a Option<AudioPlayer>,
}

/// A mutable view into the engine, excluding the penholder.
#[derive(Debug)]
pub struct EngineViewMut<'a> {
    pub tasks_tx: EngineTaskSender,
    pub pens_config: &'a mut PensConfig,
    pub document: &'a mut Document,
    pub store: &'a mut StrokeStore,
    pub camera: &'a mut Camera,
    pub audioplayer: &'a mut Option<AudioPlayer>,
}

impl<'a> EngineViewMut<'a> {
    // Converts itself to the immutable view.
    pub(crate) fn as_im<'m>(&'m self) -> EngineView<'m> {
        EngineView::<'m> {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: self.pens_config,
            document: self.document,
            store: self.store,
            camera: self.camera,
            audioplayer: self.audioplayer,
        }
    }
}

#[derive(Debug, Clone)]
/// An engine task, usually coming from a spawned thread and to be processed with [Engine::handle_engine_task].
pub enum EngineTask {
    /// Replace the images for rendering of the given stroke.
    ///
    /// The state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component's state on resize/zooming, etc. .
    UpdateStrokeWithImages {
        /// The stroke key.
        key: StrokeKey,
        /// The generated images.
        images: GeneratedContentImages,
        /// The image scale-factor the render task was using while generating the images.
        image_scale: f64,
    },
    /// Appends the images to the rendering of the given stroke.
    ///
    /// The state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component's state on resize/zooming, etc. .
    AppendImagesToStroke {
        /// The stroke key
        key: StrokeKey,
        /// The generated images
        images: GeneratedContentImages,
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
    #[serde(rename = "optimize_epd")]
    optimize_epd: bool,
}

#[derive(Debug, Clone)]
pub struct EngineTaskSender(mpsc::UnboundedSender<EngineTask>);

impl EngineTaskSender {
    pub fn send(&self, task: EngineTask) {
        if let Err(e) = self.0.unbounded_send(task) {
            let err = format!("{e:?}");
            tracing::error!(
                "Failed to send engine task {:?}, Err: {err}",
                e.into_inner()
            );
        }
    }
}

#[derive(Debug)]
pub struct EngineTaskReceiver(mpsc::UnboundedReceiver<EngineTask>);

impl EngineTaskReceiver {
    pub fn recv(&mut self) -> futures::stream::Next<'_, UnboundedReceiver<EngineTask>> {
        self.0.next()
    }
}

/// The engine.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "engine")]
pub struct Engine {
    #[serde(rename = "document")]
    pub document: Document,
    #[serde(rename = "store")]
    pub store: StrokeStore,
    #[serde(rename = "camera")]
    pub camera: Camera,
    #[serde(rename = "pens_config")]
    pub pens_config: PensConfig,
    #[serde(rename = "penholder")]
    pub penholder: PenHolder,
    #[serde(rename = "import_prefs")]
    pub import_prefs: ImportPrefs,
    #[serde(rename = "export_prefs")]
    pub export_prefs: ExportPrefs,
    #[serde(rename = "pen_sounds")]
    pen_sounds: bool,
    #[serde(rename = "optimize_epd")]
    optimize_epd: bool,

    #[serde(skip)]
    audioplayer: Option<AudioPlayer>,
    #[serde(skip)]
    visual_debug: bool,
    // the task sender. Must not be modified, only cloned.
    #[serde(skip)]
    tasks_tx: EngineTaskSender,
    #[serde(skip)]
    tasks_rx: Option<EngineTaskReceiver>,
    // Background rendering
    #[serde(skip)]
    background_tile_image: Option<render::Image>,
    #[cfg(feature = "ui")]
    #[serde(skip)]
    background_rendernodes: Vec<gtk4::gsk::RenderNode>,
    // Origin indicator rendering
    #[serde(skip)]
    origin_indicator_image: Option<render::Image>,
    #[cfg(feature = "ui")]
    #[serde(skip)]
    origin_indicator_rendernode: Option<gtk4::gsk::RenderNode>,
}

impl Default for Engine {
    fn default() -> Self {
        let (tasks_tx, tasks_rx) = futures::channel::mpsc::unbounded::<EngineTask>();

        Self {
            document: Document::default(),
            store: StrokeStore::default(),
            camera: Camera::default(),
            pens_config: PensConfig::default(),
            penholder: PenHolder::default(),
            import_prefs: ImportPrefs::default(),
            export_prefs: ExportPrefs::default(),
            pen_sounds: false,
            optimize_epd: false,

            audioplayer: None,
            visual_debug: false,
            tasks_tx: EngineTaskSender(tasks_tx),
            tasks_rx: Some(EngineTaskReceiver(tasks_rx)),
            background_tile_image: None,
            #[cfg(feature = "ui")]
            background_rendernodes: Vec::default(),
            origin_indicator_image: None,
            #[cfg(feature = "ui")]
            origin_indicator_rendernode: None,
        }
    }
}

impl Engine {
    pub(crate) const STROKE_BOUNDS_INTERSECTION_TOLERANCE: f64 = 1e-3;

    pub fn engine_tasks_tx(&self) -> EngineTaskSender {
        self.tasks_tx.clone()
    }

    pub fn take_engine_tasks_rx(&mut self) -> Option<EngineTaskReceiver> {
        self.tasks_rx.take()
    }

    #[allow(unused)]
    pub(crate) fn view(&self) -> EngineView {
        EngineView {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: &self.pens_config,
            document: &self.document,
            store: &self.store,
            camera: &self.camera,
            audioplayer: &self.audioplayer,
        }
    }

    #[allow(unused)]
    pub(crate) fn view_mut(&mut self) -> EngineViewMut {
        EngineViewMut {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: &mut self.pens_config,
            document: &mut self.document,
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
                            tracing::error!("Creating a new audioplayer failed while enabling pen sounds, Err: {e:?}");
                            None
                        }
                    }
                }
            }
        } else {
            self.audioplayer.take();
        }
    }

    pub fn optimize_epd(&self) -> bool {
        self.optimize_epd
    }

    pub fn set_optimize_epd(&mut self, optimize_epd: bool) {
        self.optimize_epd = optimize_epd
    }

    pub fn visual_debug(&self) -> bool {
        self.visual_debug
    }

    pub fn set_visual_debug(&mut self, visual_debug: bool) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.visual_debug = visual_debug;
        widget_flags.redraw = true;
        widget_flags
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
            document: self.document.clone_config(),
            camera: self.camera.clone_config(),
            stroke_components: Arc::clone(&store_history_entry.stroke_components),
            chrono_components: Arc::clone(&store_history_entry.chrono_components),
            chrono_counter: store_history_entry.chrono_counter,
        }
    }

    /// Imports an engine snapshot. A save file should always be loaded with this method.
    pub fn load_snapshot(&mut self, snapshot: EngineSnapshot) -> WidgetFlags {
        self.document = snapshot.document.clone_config();
        self.camera = snapshot.camera.clone_config();
        let mut widget_flags = self.store.import_from_snapshot(&snapshot)
            | self.doc_resize_autoexpand()
            | self.current_pen_update_state()
            | self.background_rendering_regenerate()
            | self.update_content_rendering_current_viewport();
        widget_flags.refresh_ui = true;
        widget_flags.view_modified = true;
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
        self.store.undo(now)
            | self.doc_resize_autoexpand()
            | self.current_pen_update_state()
            | self.update_rendering_current_viewport()
    }

    /// Redo the latest changes.
    pub fn redo(&mut self, now: Instant) -> WidgetFlags {
        self.store.redo(now)
            | self.doc_resize_autoexpand()
            | self.current_pen_update_state()
            | self.update_rendering_current_viewport()
    }

    pub fn can_undo(&self) -> bool {
        self.store.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.store.can_redo()
    }

    // Clears the entire engine.
    pub fn clear(&mut self) -> WidgetFlags {
        self.store.clear() | self.current_pen_update_state() | self.return_to_origin(None)
    }

    /// Handle a received task from tasks_rx.
    /// Returns [WidgetFlags] to indicate what needs to be updated in the UI.
    ///
    /// An example how to use it:
    /// ```rust, ignore
    ///
    /// glib::spawn_future_local(clone!(@weak canvas, @weak appwindow => async move {
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
            } => {
                if let Some(state) = self.store.render_comp_state(key) {
                    match state {
                        RenderCompState::Complete | RenderCompState::ForViewport(_) => {
                            // The rendering was already regenerated in the meantime,
                            // so we just discard the the render task result
                        }
                        RenderCompState::BusyRenderingInTask => {
                            if (self.camera.image_scale()
                                - render_comp::RENDER_IMAGE_SCALE_TOLERANCE
                                ..self.camera.image_scale()
                                    + render_comp::RENDER_IMAGE_SCALE_TOLERANCE)
                                .contains(&image_scale)
                            {
                                // Only when the image scale is roughly the same to when the render task was started,
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
                widget_flags |= self.camera.zoom_temporarily_to(1.0) | self.camera.zoom_to(zoom);

                let all_strokes = self.store.stroke_keys_unordered();
                self.store.set_rendering_dirty_for_strokes(&all_strokes);
                widget_flags |= self.doc_resize_autoexpand()
                    | self.background_rendering_regenerate()
                    | self.update_rendering_current_viewport();
            }
            EngineTask::Quit => {
                widget_flags |= self.set_active(false);
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
    ) -> (EventPropagation, WidgetFlags) {
        self.penholder.handle_pen_event(
            event,
            pen_mode,
            now,
            &mut EngineViewMut {
                tasks_tx: self.engine_tasks_tx(),
                pens_config: &mut self.pens_config,
                document: &mut self.document,
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
    ) -> (EventPropagation, WidgetFlags) {
        self.penholder.handle_pressed_shortcut_key(
            shortcut_key,
            now,
            &mut EngineViewMut {
                tasks_tx: self.engine_tasks_tx(),
                pens_config: &mut self.pens_config,
                document: &mut self.document,
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
                tasks_tx: self.engine_tasks_tx(),
                pens_config: &mut self.pens_config,
                document: &mut self.document,
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
                tasks_tx: self.engine_tasks_tx(),
                pens_config: &mut self.pens_config,
                document: &mut self.document,
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
                tasks_tx: self.engine_tasks_tx(),
                pens_config: &mut self.pens_config,
                document: &mut self.document,
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
                tasks_tx: self.engine_tasks_tx(),
                pens_config: &mut self.pens_config,
                document: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            })
    }

    /// Set the engine active or inactive.
    pub fn set_active(&mut self, active: bool) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if active {
            widget_flags |= self.reinstall_pen_current_style()
                | self.background_rendering_regenerate()
                | self.update_content_rendering_current_viewport();
            widget_flags.view_modified = true;
        } else {
            widget_flags |= self.clear_rendering() | self.penholder.deinit_current_pen();
        }
        widget_flags
    }

    /// Generate bounds for each page on the document which contains content.
    pub fn pages_bounds_w_content(&self, split_order: SplitOrder) -> Vec<Aabb> {
        let doc_bounds = self.document.bounds();
        let keys = self.store.stroke_keys_as_rendered();

        let strokes_bounds = self.store.strokes_bounds(&keys);

        let pages_bounds = doc_bounds
            .split_extended_origin_aligned(self.document.format.size(), split_order)
            .into_iter()
            .filter(|page_bounds| {
                // Filter the pages out that don't intersect with any stroke
                strokes_bounds.iter().any(|stroke_bounds| {
                    stroke_bounds.intersects_w_tolerance(
                        page_bounds,
                        Self::STROKE_BOUNDS_INTERSECTION_TOLERANCE,
                    )
                })
            })
            .collect::<Vec<Aabb>>();

        if pages_bounds.is_empty() {
            // If no page has content, return the origin page
            vec![Aabb::new(
                na::point![0.0, 0.0],
                self.document.format.size().into(),
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

    pub fn set_scale_factor(&mut self, scale_factor: f64) -> WidgetFlags {
        self.store
            .set_rendering_dirty_for_strokes(&self.store.stroke_keys_as_rendered());
        self.camera.set_scale_factor(scale_factor)
            | self.background_rendering_regenerate()
            | self.update_content_rendering_current_viewport()
    }

    /// Resizes the doc to the format and to fit all strokes.
    ///
    /// Background rendering then needs to be updated.
    pub fn doc_resize_to_fit_content(&mut self) -> WidgetFlags {
        self.document
            .resize_to_fit_content(&self.store, &self.camera)
            | self.update_rendering_current_viewport()
    }

    pub fn return_to_origin(&mut self, parent_width: Option<f64>) -> WidgetFlags {
        let zoom = self.camera.zoom();
        let new_offset = if let Some(parent_width) = parent_width {
            if self.document.format.width() * zoom <= parent_width {
                na::vector![
                    (self.document.format.width() * 0.5 * zoom) - parent_width * 0.5,
                    -Document::SHADOW_WIDTH * zoom
                ]
            } else {
                // If the zoomed format width is larger than the displayed surface, we zoom to a fixed origin
                na::vector![
                    -Document::SHADOW_WIDTH * zoom,
                    -Document::SHADOW_WIDTH * zoom
                ]
            }
        } else {
            na::vector![
                -Document::SHADOW_WIDTH * zoom,
                -Document::SHADOW_WIDTH * zoom
            ]
        };
        self.camera_set_offset_expand(new_offset)
    }

    /// Resize the doc when in autoexpanding layouts. called e.g. when finishing a new stroke.
    ///
    /// Background rendering then needs to be updated.
    pub fn doc_resize_autoexpand(&mut self) -> WidgetFlags {
        self.document.resize_autoexpand(&self.store, &self.camera)
            | self.update_rendering_current_viewport()
    }

    /// Expand the doc to the camera when in autoexpanding layouts. called e.g. when dragging with touch.
    ///
    /// Background and content rendering then needs to be updated.
    pub fn doc_expand_autoexpand(&mut self) -> WidgetFlags {
        self.document.expand_autoexpand(&self.camera, &self.store)
    }

    /// Add a page to the document when in fixed size layout.
    ///
    /// Document layout must be set to fixed-size.
    pub fn doc_add_page_fixed_size(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if self.document.add_page_fixed_size() {
            widget_flags |= self.update_rendering_current_viewport();
            widget_flags.resize = true;
        }
        widget_flags
    }

    /// Remove a page from the document when in fixed size layout.
    ///
    /// Document layout must be set to fixed-size.
    pub fn doc_remove_page_fixed_size(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if self.document.remove_page_fixed_size() {
            self.store.set_trashed_keys(
                &self
                    .store
                    .keys_below_y(self.document.y + self.document.height),
                true,
            );
            widget_flags |= self.record(Instant::now()) | self.update_rendering_current_viewport();
            widget_flags.resize = true;
        }
        widget_flags
    }

    /// Update the viewport offset of the camera, clamped to mins and maxs values depending on the document layout.
    ///
    /// Background and content rendering then need to be updated.
    pub fn camera_set_offset(&mut self, offset: na::Vector2<f64>) -> WidgetFlags {
        self.camera.set_offset(offset, &self.document)
    }

    /// Update the viewport offset of the camera, clamped to mins and maxs values depending on the document layout.
    ///
    /// Expands the document when in autoexpanding layouts.
    ///
    /// Background and content rendering then need to be updated.
    pub fn camera_set_offset_expand(&mut self, offset: na::Vector2<f64>) -> WidgetFlags {
        self.camera.set_offset(offset, &self.document) | self.doc_expand_autoexpand()
    }

    /// Update the viewport size of the camera.
    ///
    /// Background and content rendering then need to be updated.
    pub fn camera_set_size(&mut self, size: na::Vector2<f64>) -> WidgetFlags {
        self.camera.set_size(size)
    }

    /// Update the viewport size of the camera.
    ///
    /// Background and content rendering then need to be updated.
    pub fn camera_offset_mins_maxs(&self) -> (na::Vector2<f64>, na::Vector2<f64>) {
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
            document: &mut self.document,
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
            tasks_tx: self.engine_tasks_tx(),
            pens_config: &self.pens_config,
            document: &self.document,
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
            tasks_tx: self.engine_tasks_tx(),
            pens_config: &mut self.pens_config,
            document: &mut self.document,
            store: &mut self.store,
            camera: &mut self.camera,
            audioplayer: &mut self.audioplayer,
        })
    }

    pub fn set_doc_layout(&mut self, layout: Layout) -> WidgetFlags {
        if self.document.layout != layout {
            self.document.layout = layout;
            self.doc_resize_to_fit_content()
        } else {
            self.doc_resize_autoexpand()
        }
    }

    pub fn select_all_strokes(&mut self) -> WidgetFlags {
        let widget_flags = self.change_pen_style(PenStyle::Selector);
        self.store
            .set_selected_keys(&self.store.stroke_keys_as_rendered(), true);
        widget_flags
            | self.current_pen_update_state()
            | self.doc_resize_autoexpand()
            | self.record(Instant::now())
            | self.update_rendering_current_viewport()
    }

    pub fn deselect_all_strokes(&mut self) -> WidgetFlags {
        let widget_flags = self.change_pen_style(PenStyle::Selector);
        self.store
            .set_selected_keys(&self.store.selection_keys_as_rendered(), false);
        widget_flags
            | self.current_pen_update_state()
            | self.doc_resize_autoexpand()
            | self.record(Instant::now())
            | self.update_rendering_current_viewport()
    }

    pub fn select_with_bounds(
        &mut self,
        bounds: Aabb,
        collision: SelectionCollision,
    ) -> WidgetFlags {
        let select = match collision {
            SelectionCollision::Contains => self.store.stroke_keys_as_rendered_in_bounds(bounds),
            SelectionCollision::Intersects => self
                .store
                .stroke_keys_as_rendered_intersecting_bounds(bounds),
        };
        self.store.set_selected_keys(&select, true);
        self.doc_resize_autoexpand()
            | self.record(Instant::now())
            | self.update_rendering_current_viewport()
    }

    pub fn duplicate_selection(&mut self) -> WidgetFlags {
        let new_selected = self.store.duplicate_selection();
        self.store.update_geometry_for_strokes(&new_selected);
        self.current_pen_update_state()
            | self.doc_resize_autoexpand()
            | self.record(Instant::now())
            | self.update_rendering_current_viewport()
    }

    pub fn trash_selection(&mut self) -> WidgetFlags {
        let selection_keys = self.store.selection_keys_as_rendered();
        self.store.set_trashed_keys(&selection_keys, true);
        self.current_pen_update_state()
            | self.doc_resize_autoexpand()
            | self.record(Instant::now())
            | self.update_rendering_current_viewport()
    }

    pub fn nothing_selected(&self) -> bool {
        self.store.selection_keys_unordered().is_empty()
    }

    pub fn change_selection_stroke_colors(&mut self, stroke_color: Color) -> WidgetFlags {
        self.store
            .change_stroke_colors(&self.store.selection_keys_as_rendered(), stroke_color)
            | self.record(Instant::now())
            | self.update_content_rendering_current_viewport()
    }

    pub fn change_selection_fill_colors(&mut self, fill_color: Color) -> WidgetFlags {
        self.store
            .change_fill_colors(&self.store.selection_keys_as_rendered(), fill_color)
            | self.record(Instant::now())
            | self.update_content_rendering_current_viewport()
    }

    pub fn invert_selection_colors(&mut self) -> WidgetFlags {
        self.store
            .invert_color_brightness(&self.store.selection_keys_as_rendered())
            | self.record(Instant::now())
            | self.update_content_rendering_current_viewport()
    }

    pub fn text_selection_change_style<F>(&mut self, modify_func: F) -> WidgetFlags
    where
        F: FnOnce(&mut TextStyle),
    {
        let mut widget_flags = WidgetFlags::default();
        if let Pen::Typewriter(typewriter) = self.penholder.current_pen_mut() {
            widget_flags |= typewriter.change_text_style_in_modifying_stroke(
                modify_func,
                &mut EngineViewMut {
                    tasks_tx: self.tasks_tx.clone(),
                    pens_config: &mut self.pens_config,
                    document: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                },
            )
        }
        widget_flags
    }

    pub fn text_selection_remove_attributes(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if let Pen::Typewriter(typewriter) = self.penholder.current_pen_mut() {
            widget_flags |=
                typewriter.remove_text_attributes_current_selection(&mut EngineViewMut {
                    tasks_tx: self.tasks_tx.clone(),
                    pens_config: &mut self.pens_config,
                    document: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                })
        }
        widget_flags
    }

    pub fn text_selection_toggle_attribute(
        &mut self,
        text_attribute: TextAttribute,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if let Pen::Typewriter(typewriter) = self.penholder.current_pen_mut() {
            widget_flags |= typewriter.toggle_text_attribute_current_selection(
                text_attribute,
                &mut EngineViewMut {
                    tasks_tx: self.tasks_tx.clone(),
                    pens_config: &mut self.pens_config,
                    document: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                },
            )
        }
        widget_flags
    }

    pub fn text_selection_add_attribute(&mut self, text_attribute: TextAttribute) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if let Pen::Typewriter(typewriter) = self.penholder.current_pen_mut() {
            widget_flags |= typewriter.add_text_attribute_current_selection(
                text_attribute,
                &mut EngineViewMut {
                    tasks_tx: self.tasks_tx.clone(),
                    pens_config: &mut self.pens_config,
                    document: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                },
            )
        }
        widget_flags
    }
}
