pub mod export;
pub mod import;
pub mod rendering;
pub mod visual_debug;

// Re-Exports
pub use self::export::ExportPrefs;
pub use self::import::ImportPrefs;

// Imports
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use self::import::XoppImportPrefs;
use crate::document::{background, Layout};
use crate::pens::PenStyle;
use crate::pens::{PenMode, PensConfig};
use crate::store::render_comp::{self, RenderCompState};
use crate::store::{ChronoComponent, StrokeKey};
use crate::strokes::strokebehaviour::GeneratedStrokeImages;
use crate::strokes::Stroke;
use crate::{render, AudioPlayer, WidgetFlags};
use crate::{Camera, Document, PenHolder, StrokeStore};
use anyhow::Context;
use rnote_compose::helpers::AabbHelpers;
use rnote_compose::penevents::{PenEvent, ShortcutKey};

use futures::channel::{mpsc, oneshot};
use gtk4::gsk;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_fileformats::{rnoteformat, xoppformat, FileFormatLoader};
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};

/// A view into the rest of the engine, excluding the penholder
#[allow(missing_debug_implementations)]
pub struct EngineView<'a> {
    pub tasks_tx: EngineTaskSender,
    pub pens_config: &'a PensConfig,
    pub doc: &'a Document,
    pub store: &'a StrokeStore,
    pub camera: &'a Camera,
    pub audioplayer: &'a Option<AudioPlayer>,
}

/// A mutable view into the rest of the engine, excluding the penholder
#[allow(missing_debug_implementations)]
pub struct EngineViewMut<'a> {
    pub tasks_tx: EngineTaskSender,
    pub pens_config: &'a mut PensConfig,
    pub doc: &'a mut Document,
    pub store: &'a mut StrokeStore,
    pub camera: &'a mut Camera,
    pub audioplayer: &'a mut Option<AudioPlayer>,
}

impl<'a> EngineViewMut<'a> {
    // converts itself to the immutable view
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
/// A engine task, usually coming from a spawned thread and to be processed with `process_received_task()`.
pub enum EngineTask {
    /// Replace the images of the render_comp.
    /// Note that the state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component state on resize / zooming, etc.
    UpdateStrokeWithImages {
        /// The stroke key
        key: StrokeKey,
        /// The generated images
        images: GeneratedStrokeImages,
        /// The image scale-factor the render task was using while generating the images
        image_scale: f64,
        /// The stroke bounds at the time when the render task has launched
        stroke_bounds: Aabb,
    },
    /// Appends the images to the rendering of the stroke
    /// Note that usually the state of the render component should be set **before** spawning a thread, generating images and sending this task,
    /// to avoid spawning large amounts of already outdated rendering tasks when checking the render component state on resize / zooming, etc.
    AppendImagesToStroke {
        /// The stroke key
        key: StrokeKey,
        /// The generated images
        images: GeneratedStrokeImages,
    },
    /// indicates that the application is quitting. Usually handled to quit the async loop which receives the tasks
    Quit,
}

#[allow(missing_debug_implementations)]
#[derive(Serialize, Deserialize)]
#[serde(default, rename = "engine_config")]
pub struct EngineConfig {
    #[serde(rename = "document")]
    document: serde_json::Value,
    #[serde(rename = "pens_config")]
    pens_config: serde_json::Value,
    #[serde(rename = "penholder")]
    penholder: serde_json::Value,
    #[serde(rename = "import_prefs")]
    import_prefs: serde_json::Value,
    #[serde(rename = "export_prefs")]
    export_prefs: serde_json::Value,
    #[serde(rename = "pen_sounds")]
    pen_sounds: serde_json::Value,
}

impl Default for EngineConfig {
    fn default() -> Self {
        let engine = RnoteEngine::default();

        Self {
            document: serde_json::to_value(engine.document).unwrap(),
            pens_config: serde_json::to_value(&engine.pens_config).unwrap(),
            penholder: serde_json::to_value(&engine.penholder).unwrap(),

            import_prefs: serde_json::to_value(engine.import_prefs).unwrap(),
            export_prefs: serde_json::to_value(engine.export_prefs).unwrap(),
            pen_sounds: serde_json::to_value(engine.pen_sounds).unwrap(),
        }
    }
}

// the engine snapshot, used when saving and loading to and from a file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "engine_snapshot")]
pub struct EngineSnapshot {
    #[serde(rename = "document")]
    pub document: Document,
    #[serde(rename = "stroke_components")]
    pub stroke_components: Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>,
    #[serde(rename = "chrono_components")]
    pub chrono_components: Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>,
    #[serde(rename = "chrono_counter")]
    pub chrono_counter: u32,
}

impl Default for EngineSnapshot {
    fn default() -> Self {
        Self {
            document: Document::default(),
            stroke_components: Arc::new(HopSlotMap::with_key()),
            chrono_components: Arc::new(SecondaryMap::new()),
            chrono_counter: 0,
        }
    }
}

impl EngineSnapshot {
    /// loads a snapshot from the bytes of a .rnote file.
    ///
    /// To import this snapshot into the current engine, use `import_snapshot()`.
    pub async fn load_from_rnote_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let (snapshot_sender, snapshot_receiver) = oneshot::channel::<anyhow::Result<Self>>();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Self> {
                let rnote_file = rnoteformat::RnoteFile::load_from_bytes(&bytes)
                    .context("RnoteFile load_from_bytes() failed")?;

                serde_json::from_value(rnote_file.engine_snapshot)
                    .context("serde_json::from_value() for rnote_file.engine_snapshot failed")
            };

            if let Err(_data) = snapshot_sender.send(result()) {
                log::error!("sending result to receiver in open_from_rnote_bytes() failed. Receiver already dropped");
            }
        });

        snapshot_receiver.await?
    }
    /// Loads from the bytes of a Xournal++ .xopp file.
    ///
    /// To import this snapshot into the current engine, use `import_snapshot()`.
    pub async fn load_from_xopp_bytes(
        bytes: Vec<u8>,
        xopp_import_prefs: XoppImportPrefs,
    ) -> anyhow::Result<Self> {
        let (snapshot_sender, snapshot_receiver) = oneshot::channel::<anyhow::Result<Self>>();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Self> {
                let xopp_file = xoppformat::XoppFile::load_from_bytes(&bytes)?;

                // Extract the largest width of all pages, add together all heights
                let (doc_width, doc_height) = xopp_file
                    .xopp_root
                    .pages
                    .iter()
                    .map(|page| (page.width, page.height))
                    .fold((0_f64, 0_f64), |prev, next| {
                        // Max of width, sum heights
                        (prev.0.max(next.0), prev.1 + next.1)
                    });
                let no_pages = xopp_file.xopp_root.pages.len() as u32;

                let mut engine = RnoteEngine::default();

                // We convert all values from the hardcoded 72 DPI of Xopp files to the preferred dpi
                engine.document.format.dpi = xopp_import_prefs.dpi;

                engine.document.x = 0.0;
                engine.document.y = 0.0;
                engine.document.width = crate::utils::convert_value_dpi(
                    doc_width,
                    xoppformat::XoppFile::DPI,
                    xopp_import_prefs.dpi,
                );
                engine.document.height = crate::utils::convert_value_dpi(
                    doc_height,
                    xoppformat::XoppFile::DPI,
                    xopp_import_prefs.dpi,
                );

                engine.document.format.width = crate::utils::convert_value_dpi(
                    doc_width,
                    xoppformat::XoppFile::DPI,
                    xopp_import_prefs.dpi,
                );
                engine.document.format.height = crate::utils::convert_value_dpi(
                    doc_height / (no_pages as f64),
                    xoppformat::XoppFile::DPI,
                    xopp_import_prefs.dpi,
                );

                if let Some(first_page) = xopp_file.xopp_root.pages.get(0) {
                    if let xoppformat::XoppBackgroundType::Solid {
                        color: _color,
                        style: _style,
                    } = &first_page.background.bg_type
                    {
                        // Xopp background styles are not compatible with Rnotes, so everything is plain for now
                        engine.document.background.pattern = background::PatternStyle::None;
                    }
                }

                // Offsetting as rnote has one global coordinate space
                let mut offset = na::Vector2::<f64>::zeros();

                for (_page_i, page) in xopp_file.xopp_root.pages.into_iter().enumerate() {
                    for layers in page.layers.into_iter() {
                        // import strokes
                        for new_xoppstroke in layers.strokes.into_iter() {
                            match Stroke::from_xoppstroke(
                                new_xoppstroke,
                                offset,
                                xopp_import_prefs.dpi,
                            ) {
                                Ok((new_stroke, layer)) => {
                                    engine.store.insert_stroke(new_stroke, Some(layer));
                                }
                                Err(e) => {
                                    log::error!(
                                        "from_xoppstroke() failed in open_from_xopp_bytes() with Err {:?}",
                                        e
                                    );
                                }
                            }
                        }

                        // import images
                        for new_xoppimage in layers.images.into_iter() {
                            match Stroke::from_xoppimage(
                                new_xoppimage,
                                offset,
                                xopp_import_prefs.dpi,
                            ) {
                                Ok(new_image) => {
                                    engine.store.insert_stroke(new_image, None);
                                }
                                Err(e) => {
                                    log::error!(
                                        "from_xoppimage() failed in open_from_xopp_bytes() with Err {:?}",
                                        e
                                    );
                                }
                            }
                        }
                    }

                    // Only add to y offset, results in vertical pages
                    offset[1] += crate::utils::convert_value_dpi(
                        page.height,
                        xoppformat::XoppFile::DPI,
                        xopp_import_prefs.dpi,
                    );
                }

                Ok(engine.take_snapshot())
            };

            if let Err(_data) = snapshot_sender.send(result()) {
                log::error!("sending result to receiver in open_from_xopp_bytes() failed. Receiver already dropped");
            }
        });

        snapshot_receiver.await?
    }
}

pub const RNOTE_STROKE_CONTENT_MIME_TYPE: &str = "application/rnote-stroke-content";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "stroke_content")]
pub struct StrokeContent {
    #[serde(rename = "strokes")]
    pub strokes: Vec<Arc<Stroke>>,
}

pub type EngineTaskSender = mpsc::UnboundedSender<EngineTask>;
pub type EngineTaskReceiver = mpsc::UnboundedReceiver<EngineTask>;

/// The engine.
#[allow(missing_debug_implementations)]
#[derive(Serialize, Deserialize)]
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
    /// the task sender. Must not be modified, only cloned. To install a new engine task handler, regenerate the channel through `regenerate_channel()`
    #[serde(skip)]
    pub tasks_tx: EngineTaskSender,
    // Background rendering
    #[serde(skip)]
    pub background_tile_image: Option<render::Image>,
    #[serde(skip)]
    background_rendernodes: Vec<gsk::RenderNode>,
}

impl Default for RnoteEngine {
    fn default() -> Self {
        let (tasks_tx, _tasks_rx) = futures::channel::mpsc::unbounded::<EngineTask>();

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
            background_tile_image: None,
            background_rendernodes: Vec::default(),
        }
    }
}

impl RnoteEngine {
    pub fn tasks_tx(&self) -> EngineTaskSender {
        self.tasks_tx.clone()
    }

    /// Regenerates the tasks channel, saves the sender in the struct and returns the receiver which can be awaited in a engine tasks handler through `handle_engine_tasks()`
    pub fn regenerate_channel(&mut self) -> EngineTaskReceiver {
        let (tasks_tx, tasks_rx) = futures::channel::mpsc::unbounded::<EngineTask>();

        self.tasks_tx = tasks_tx;

        tasks_rx
    }

    /// Gets the EngineView
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

    /// Gets the EngineViewMut
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

    /// whether pen sounds are enabled
    pub fn pen_sounds(&self) -> bool {
        self.pen_sounds
    }

    /// enables / disables the pen sounds.
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

    /// Takes a snapshot of the current state
    pub fn take_snapshot(&self) -> EngineSnapshot {
        let mut store_history_entry = self.store.history_entry_from_current_state();

        // Remove all trashed strokes
        let trashed_keys = store_history_entry
            .trash_components
            .iter()
            .filter_map(|(key, trash_comp)| if trash_comp.trashed { Some(key) } else { None })
            .collect::<Vec<StrokeKey>>();

        for key in trashed_keys {
            Arc::make_mut(&mut Arc::make_mut(&mut store_history_entry).stroke_components)
                .remove(key);
        }

        EngineSnapshot {
            document: self.document,
            stroke_components: Arc::clone(&store_history_entry.stroke_components),
            chrono_components: Arc::clone(&store_history_entry.chrono_components),
            chrono_counter: store_history_entry.chrono_counter,
        }
    }

    /// imports a engine snapshot. A save file should always be loaded with this method.
    /// the store then needs to update its rendering
    pub fn load_snapshot(&mut self, snapshot: EngineSnapshot) -> WidgetFlags {
        self.document = snapshot.document;
        self.store.import_from_snapshot(&snapshot);

        self.update_state_current_pen()
    }

    /// records the current store state and saves it as a history entry.
    pub fn record(&mut self, now: Instant) -> WidgetFlags {
        self.store.record(now)
    }

    /// Undo the latest changes
    pub fn undo(&mut self, now: Instant) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        widget_flags.merge(
            self.penholder
                .reinstall_pen_current_style(&mut EngineViewMut {
                    tasks_tx: self.tasks_tx(),
                    pens_config: &mut self.pens_config,
                    doc: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                }),
        );

        widget_flags.merge(self.store.undo(now));

        widget_flags.merge(self.update_state_current_pen());

        self.resize_autoexpand();
        if let Err(e) = self.update_rendering_current_viewport() {
            log::error!("failed to update rendering for current viewport while undo, Err: {e:?}");
        }

        widget_flags.redraw = true;

        widget_flags
    }

    /// redo the latest changes
    pub fn redo(&mut self, now: Instant) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        widget_flags.merge(
            self.penholder
                .reinstall_pen_current_style(&mut EngineViewMut {
                    tasks_tx: self.tasks_tx(),
                    pens_config: &mut self.pens_config,
                    doc: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                }),
        );

        widget_flags.merge(self.store.redo(now));

        widget_flags.merge(self.update_state_current_pen());

        self.resize_autoexpand();
        if let Err(e) = self.update_rendering_current_viewport() {
            log::error!("failed to update rendering for current viewport while redo, Err: {e:?}");
        }

        widget_flags.redraw = true;

        widget_flags
    }

    pub fn can_undo(&self) -> bool {
        self.store.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.store.can_redo()
    }

    // Clears the store
    pub fn clear(&mut self) -> WidgetFlags {
        self.store.clear();

        self.update_state_current_pen()
    }

    /// processes the received task from tasks_rx.
    /// Returns widget flags to indicate what needs to be updated in the UI.
    /// An example how to use it:
    /// ```rust, ignore
    ///
    /// glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
    ///    let mut task_rx = canvas.engine().borrow_mut().regenerate_channel();
    ///
    ///    loop {
    ///        if let Some(task) = task_rx.next().await {
    ///            let (widget_flags, quit) = canvas.engine().borrow_mut().handle_engine_task(task);
    ///            canvas.emit_handle_widget_flags(widget_flags);

    ///            if quit {
    ///                break;
    ///            }
    ///        }
    ///    }
    /// }));
    /// ```
    /// Processes a received store task. Usually called from a receiver loop which awaits tasks_rx.
    ///
    /// Returns the widget flags, and whether the handler should quit
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
                    //log::debug!("key: {key:?} - render state: {state:?}");

                    match state {
                        RenderCompState::Complete | RenderCompState::ForViewport(_) => {
                            // The rendering was already regenerated in the meantime,
                            // so we just discard the the render task results
                        }
                        RenderCompState::BusyRenderingInTask => {
                            if (self.camera.image_scale()
                                - render_comp::RENDER_IMAGE_SCALE_TOLERANCE
                                ..self.camera.image_scale()
                                    + render_comp::RENDER_IMAGE_SCALE_TOLERANCE)
                                .contains(&image_scale)
                                && self
                                    .store
                                    .get_stroke_ref(key)
                                    .map(|s| s.bounds() == stroke_bounds)
                                    .unwrap_or(true)
                            {
                                // Only when the image scale and stroke bounds are the same
                                // as when the render task was started, the new images are considered valid
                                // and can replace the old
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
            EngineTask::Quit => {
                quit = true;
            }
        }

        (widget_flags, quit)
    }

    /// handle a pen event
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

    /// Handle a pressed shortcut key
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

    /// change the pen style
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

    /// change the pen style override
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

    /// change the pen mode. Relevant for stylus input
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

    /// Reinstalls the pen in the current style
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

    /// Generates bounds for each page on the document which contains content
    pub fn pages_bounds_w_content(&self) -> Vec<Aabb> {
        let doc_bounds = self.document.bounds();
        let keys = self.store.stroke_keys_as_rendered();

        let strokes_bounds = self.store.strokes_bounds(&keys);

        let pages_bounds = doc_bounds
            .split_extended_origin_aligned(na::vector![
                self.document.format.width,
                self.document.format.height
            ])
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

    /// Generates bounds which contain all pages on the doc with content extended to fit the format.
    pub fn bounds_w_content_extended(&self) -> Option<Aabb> {
        let pages_bounds = self.pages_bounds_w_content();

        if pages_bounds.is_empty() {
            return None;
        }

        Some(
            pages_bounds
                .into_iter()
                .fold(Aabb::new_invalid(), |prev, next| prev.merged(&next)),
        )
    }

    /// Resizes the doc to the format and to fit all strokes.
    ///
    /// Document background rendering then needs to be updated.
    pub fn resize_to_fit_strokes(&mut self) {
        self.document
            .resize_to_fit_strokes(&self.store, &self.camera);
    }

    /// Resize the doc when in autoexpanding layouts. called e.g. when finishing a new stroke.
    ///
    /// Document background rendering then needs to be updated.
    pub fn resize_autoexpand(&mut self) {
        self.document.resize_autoexpand(&self.store, &self.camera);
    }

    /// Expands the doc when in autoexpanding layouts. e.g. when dragging with touch
    pub fn expand_doc_autoexpand(&mut self) {
        match self.document.layout {
            Layout::FixedSize | Layout::ContinuousVertical => {
                // not resizing in these modes, the size is not dependent on the camera
            }
            Layout::SemiInfinite => {
                // only expand, don't resize to fit strokes
                self.document
                    .expand_doc_semi_infinite_layout(self.camera.viewport());
            }
            Layout::Infinite => {
                // only expand, don't resize to fit strokes
                self.document
                    .expand_doc_infinite_layout(self.camera.viewport());
            }
        }
    }

    /// Adds a page to the document when in fixed size layout.
    ///
    /// Returns true when document is in fixed size layout and a pages was added,
    /// else false.
    ///
    /// background and strokes rendering then need to be updated.
    pub fn add_page_doc_fixed_size(&mut self) -> bool {
        if self.document.layout != Layout::FixedSize {
            return false;
        }

        let format_height = self.document.format.height;
        let new_doc_height = self.document.height + format_height;
        self.document.height = new_doc_height;

        true
    }

    /// Removes a page from the document when in fixed size layout.
    ///
    /// Returns true when document is in fixed size layout and a pages was removed,
    /// else false.
    ///
    /// background and strokes rendering then need to be updated.
    pub fn remove_page_doc_fixed_size(&mut self) -> bool {
        if self.document.layout != Layout::FixedSize {
            return false;
        }
        let format_height = self.document.format.height;
        let doc_y = self.document.y;
        let doc_height = self.document.height;
        let new_doc_height = doc_height - format_height;

        if doc_height > format_height {
            let remove_area_keys = self.store.keys_below_y_pos(doc_y + new_doc_height);
            self.store.set_trashed_keys(&remove_area_keys, true);
            self.document.height = new_doc_height;
        }

        true
    }

    /// Updates the camera and updates doc dimensions with the new offset and size.
    ///
    /// background and strokes rendering then need to be updated.
    pub fn update_camera_offset_size(
        &mut self,
        new_offset: na::Vector2<f64>,
        new_size: na::Vector2<f64>,
    ) {
        self.camera.offset = new_offset;
        self.camera.size = new_size;
    }

    /// Updates the current pen with the current engine state.
    ///
    /// needs to be called when the engine state was changed outside of pen events. ( e.g. trash all strokes, set strokes selected, etc. )
    pub fn update_state_current_pen(&mut self) -> WidgetFlags {
        self.penholder.update_state_current_pen(&mut EngineViewMut {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: &mut self.pens_config,
            doc: &mut self.document,
            store: &mut self.store,
            camera: &mut self.camera,
            audioplayer: &mut self.audioplayer,
        })
    }

    /// fetches clipboard content from the current pen.
    ///
    /// Returns (the content, mime_type)
    #[allow(clippy::type_complexity)]
    pub fn fetch_clipboard_content(
        &self,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        self.penholder.fetch_clipboard_content(&EngineView {
            tasks_tx: self.tasks_tx(),
            pens_config: &self.pens_config,
            doc: &self.document,
            store: &self.store,
            camera: &self.camera,
            audioplayer: &self.audioplayer,
        })
    }

    /// Cuts clipboard content from the current pen.
    ///
    /// Returns (the content, mime_type)
    #[allow(clippy::type_complexity)]
    pub fn cut_clipboard_content(
        &mut self,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        self.penholder.cut_clipboard_content(&mut EngineViewMut {
            tasks_tx: self.tasks_tx(),
            pens_config: &mut self.pens_config,
            doc: &mut self.document,
            store: &mut self.store,
            camera: &mut self.camera,
            audioplayer: &mut self.audioplayer,
        })
    }
}
