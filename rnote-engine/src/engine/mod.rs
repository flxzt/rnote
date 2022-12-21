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

use self::export::{SelectionExportFormat, SelectionExportPrefs};
use self::import::XoppImportPrefs;
use crate::document::{background, Layout};
use crate::pens::PenStyle;
use crate::pens::{PenMode, PensConfig};
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
    /// indicates that the application is quitting. Usually handled to quit the async loop which receives the tasks
    Quit,
}

#[allow(missing_debug_implementations)]
#[derive(Serialize, Deserialize)]
#[serde(default, rename = "engine_config")]
struct EngineConfig {
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
            document: serde_json::to_value(&engine.document).unwrap(),
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
                    .context("RnoteFile load_from_bytes() failed.")?;

                serde_json::from_value(rnote_file.engine_snapshot)
                    .context("serde_json::from_value() for rnote_file.engine_snapshot failed")
            };

            if let Err(_data) = snapshot_sender.send(result()) {
                log::error!("sending result to receiver in open_from_rnote_bytes() failed. Receiver already dropped.");
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
                log::error!("sending result to receiver in open_from_xopp_bytes() failed. Receiver already dropped.");
            }
        });

        snapshot_receiver.await?
    }
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
    #[serde(skip)]
    pub tasks_tx: EngineTaskSender,
    /// To be taken out into a loop which processes the receiver stream. The received tasks should be processed with process_received_task()
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
    /// If pen sound should be enabled, the rnote data dir must be provided.
    pub fn set_pen_sounds(&mut self, pen_sounds: bool, data_dir: Option<PathBuf>) {
        self.pen_sounds = pen_sounds;

        if pen_sounds {
            if let Some(data_dir) = data_dir {
                // Only create and init a new audioplayer if it does not already exists
                if self.audioplayer.is_none() {
                    self.audioplayer = match AudioPlayer::new_init(data_dir) {
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
            document: self.document.clone(),
            stroke_components: Arc::clone(&store_history_entry.stroke_components),
            chrono_components: Arc::clone(&store_history_entry.chrono_components),
            chrono_counter: store_history_entry.chrono_counter,
        }
    }

    /// imports a engine snapshot. A save file should always be loaded with this method.
    /// the store then needs to update its rendering
    pub fn load_snapshot(&mut self, snapshot: EngineSnapshot) -> WidgetFlags {
        self.document = snapshot.document.clone();
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

    // Clears the store
    pub fn clear(&mut self) -> WidgetFlags {
        self.store.clear();

        self.update_state_current_pen()
    }

    /// processes the received task from tasks_rx.
    /// Returns widget flags to indicate what needs to be updated in the UI.
    /// An example how to use it:
    /// ```rust, ignore
    /// let main_cx = glib::MainContext::default();

    /// main_cx.spawn_local(clone!(@strong canvas, @strong appwindow => async move {
    ///            let mut task_rx = canvas.engine().borrow_mut().store.tasks_rx.take().unwrap();

    ///           loop {
    ///              if let Some(task) = task_rx.next().await {
    ///                    let widget_flags = canvas.engine().borrow_mut().process_received_task(task);
    ///                    if appwindow.handle_widget_flags(widget_flags) {
    ///                         break;
    ///                    }
    ///                }
    ///            }
    ///        }));
    /// ```
    /// Processes a received store task. Usually called from a receiver loop which polls tasks_rx.
    pub fn process_received_task(&mut self, task: EngineTask) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        match task {
            EngineTask::UpdateStrokeWithImages { key, images } => {
                self.store.replace_rendering_with_images(key, images);

                widget_flags.redraw = true;
            }
            EngineTask::AppendImagesToStroke { key, images } => {
                self.store.append_rendering_images(key, images);

                widget_flags.redraw = true;
            }
            EngineTask::Quit => {
                widget_flags.quit = true;
            }
        }

        widget_flags
    }

    /// handle an pen event
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
    pub fn handle_pen_pressed_shortcut_key(
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

    // Generates bounds for each page on the document which contains content
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
                // Filter the pages out that doesn't intersect with any stroke
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

    /// the current document layout
    pub fn doc_layout(&self) -> Layout {
        self.document.layout()
    }

    pub fn set_doc_layout(&mut self, layout: Layout) {
        self.document.set_layout(layout, &self.store, &self.camera);
    }

    /// resizes the doc to the format and to fit all strokes
    /// Document background rendering then needs to be updated.
    pub fn resize_to_fit_strokes(&mut self) {
        self.document
            .resize_to_fit_strokes(&self.store, &self.camera);
    }

    /// resize the doc when in autoexpanding layouts. called e.g. when finishing a new stroke
    /// Document background rendering then needs to be updated.
    pub fn resize_autoexpand(&mut self) {
        self.document.resize_autoexpand(&self.store, &self.camera);
    }

    /// Updates the camera and expands doc dimensions with offset
    /// Document background rendering then needs to be updated.
    pub fn update_camera_offset(&mut self, new_offset: na::Vector2<f64>) {
        self.camera.offset = new_offset;

        match self.document.layout() {
            Layout::FixedSize => {
                // Does not resize in fixed size mode, use resize_doc_to_fit_strokes() for it.
            }
            Layout::ContinuousVertical => {
                self.document
                    .resize_doc_continuous_vertical_layout(&self.store);
            }
            Layout::Infinite => {
                // only expand, don't resize to fit strokes
                self.document
                    .expand_doc_infinite_layout(self.camera.viewport());
            }
        }
    }

    /// Updates the current pen with the current engine state.
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

    /// clipboard content from current state.
    /// Returns (the content, mime_type)
    #[allow(clippy::type_complexity)]
    pub fn fetch_clipboard_content(
        &self,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        let export_bytes = self.export_selection(Some(SelectionExportPrefs {
            with_background: true,
            export_format: SelectionExportFormat::Svg,
            ..Default::default()
        }));

        // First try exporting the selection as svg
        if let Some(selection_bytes) = futures::executor::block_on(async { export_bytes.await? })? {
            return Ok((
                Some((selection_bytes, String::from("image/svg+xml"))),
                WidgetFlags::default(),
            ));
        }

        // else fetch from pen
        self.penholder.fetch_clipboard_content(&EngineView {
            tasks_tx: self.tasks_tx(),
            pens_config: &self.pens_config,
            doc: &self.document,
            store: &self.store,
            camera: &self.camera,
            audioplayer: &self.audioplayer,
        })
    }

    /// Cuts clipboard content from current state.
    /// Returns (the content, mime_type)
    #[allow(clippy::type_complexity)]
    pub fn cut_clipboard_content(
        &mut self,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        /*
        // FIXME: Until svg import is broken, we don't want users being able to cut the selection without the possibility to insert it again.

                let export_bytes = self.export_selection(Some(SelectionExportPrefs {
                    with_background: true,
                    export_format: SelectionExportFormat::Svg,
                    ..Default::default()
                }));

                // First try exporting the selection as svg
                if let Some(selection_bytes) = futures::executor::block_on(async { export_bytes.await? })? {
                    return Ok(Some((selection_bytes, String::from("image/svg+xml"))));
                }
         */

        // else fetch from pen
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
