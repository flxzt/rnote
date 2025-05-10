// Modules
mod actions;
mod appsettings;
mod imp;

// Imports
use crate::{
    FileType, RnApp, RnCanvas, RnCanvasWrapper, RnMainHeader, RnOverlays, RnSidebar, config,
    dialogs, env,
};
use adw::{prelude::*, subclass::prelude::*};
use core::cell::{Ref, RefMut};
use gettextrs::gettext;
use gtk4::{Application, IconTheme, gdk, gio, glib};
use rnote_compose::Color;
use rnote_engine::document::DocumentConfig;
use rnote_engine::engine::{EngineConfig, EngineConfigShared};
use rnote_engine::ext::GdkRGBAExt;
use rnote_engine::pens::PenStyle;
use rnote_engine::pens::pensconfig::brushconfig::BrushStyle;
use rnote_engine::pens::pensconfig::shaperconfig::ShaperStyle;
use rnote_engine::{WidgetFlags, engine::EngineTask};
use std::path::Path;
use tracing::{debug, error};

glib::wrapper! {
    pub(crate) struct RnAppWindow(ObjectSubclass<imp::RnAppWindow>)
        @extends gtk4::Widget, gtk4::Window, adw::Window, gtk4::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable,
                    gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl RnAppWindow {
    const AUTOSAVE_INTERVAL_DEFAULT: u32 = 30;
    const PERIODIC_CONFIGSAVE_INTERVAL: u32 = 10;

    pub(crate) fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub(crate) fn engine_config(&self) -> &EngineConfigShared {
        &self.imp().engine_config
    }

    pub(crate) fn document_config_preset_ref(&self) -> Ref<DocumentConfig> {
        self.imp().document_config_preset.borrow()
    }

    pub(crate) fn document_config_preset_mut(&self) -> RefMut<DocumentConfig> {
        self.imp().document_config_preset.borrow_mut()
    }

    #[allow(unused)]
    pub(crate) fn pen_sounds(&self) -> bool {
        self.property::<bool>("pen-sounds")
    }

    #[allow(unused)]
    pub(crate) fn set_pen_sounds(&self, pen_sounds: bool) {
        self.set_property("pen-sounds", pen_sounds.to_value());
    }

    #[allow(unused)]
    pub(crate) fn snap_positions(&self) -> bool {
        self.property::<bool>("snap-positions")
    }

    #[allow(unused)]
    pub(crate) fn set_snap_positions(&self, snap_positions: bool) {
        self.set_property("snap-positions", snap_positions.to_value());
    }

    #[allow(unused)]
    pub(crate) fn pen_style(&self) -> PenStyle {
        PenStyle::from_variant(&self.property::<glib::Variant>("pen-style")).unwrap()
    }

    #[allow(unused)]
    pub(crate) fn set_pen_style(&self, pen_style: PenStyle) {
        self.set_property("pen-style", pen_style.to_variant().to_value());
    }

    #[allow(unused)]
    pub(crate) fn autosave(&self) -> bool {
        self.property::<bool>("autosave")
    }

    #[allow(unused)]
    pub(crate) fn set_autosave(&self, autosave: bool) {
        self.set_property("autosave", autosave.to_value());
    }

    #[allow(unused)]
    pub(crate) fn autosave_interval_secs(&self) -> u32 {
        self.property::<u32>("autosave-interval-secs")
    }

    #[allow(unused)]
    pub(crate) fn set_autosave_interval_secs(&self, autosave_interval_secs: u32) {
        self.set_property("autosave-interval-secs", autosave_interval_secs.to_value());
    }

    #[allow(unused)]
    pub(crate) fn righthanded(&self) -> bool {
        self.property::<bool>("righthanded")
    }

    #[allow(unused)]
    pub(crate) fn set_righthanded(&self, righthanded: bool) {
        self.set_property("righthanded", righthanded.to_value());
    }

    #[allow(unused)]
    pub(crate) fn block_pinch_zoom(&self) -> bool {
        self.property::<bool>("block-pinch-zoom")
    }

    #[allow(unused)]
    pub(crate) fn set_block_pinch_zoom(&self, block_pinch_zoom: bool) {
        self.set_property("block-pinch-zoom", block_pinch_zoom.to_value());
    }

    #[allow(unused)]
    pub(crate) fn respect_borders(&self) -> bool {
        self.property::<bool>("respect-borders")
    }

    #[allow(unused)]
    pub(crate) fn set_respect_borders(&self, respect_borders: bool) {
        self.set_property("respect-borders", respect_borders.to_value());
    }

    #[allow(unused)]
    pub(crate) fn touch_drawing(&self) -> bool {
        self.property::<bool>("touch-drawing")
    }

    #[allow(unused)]
    pub(crate) fn set_touch_drawing(&self, touch_drawing: bool) {
        self.set_property("touch-drawing", touch_drawing.to_value());
    }

    #[allow(unused)]
    pub(crate) fn focus_mode(&self) -> bool {
        self.property::<bool>("focus-mode")
    }

    #[allow(unused)]
    pub(crate) fn set_focus_mode(&self, focus_mode: bool) {
        self.set_property("focus-mode", focus_mode.to_value());
    }

    #[allow(unused)]
    pub(crate) fn devel_mode(&self) -> bool {
        self.property::<bool>("devel-mode")
    }

    #[allow(unused)]
    pub(crate) fn set_devel_mode(&self, devel_mode: bool) {
        self.set_property("devel-mode", devel_mode.to_value());
    }

    #[allow(unused)]
    pub(crate) fn visual_debug(&self) -> bool {
        self.property::<bool>("visual-debug")
    }

    #[allow(unused)]
    pub(crate) fn set_visual_debug(&self, visual_debug: bool) {
        self.set_property("visual-debug", visual_debug.to_value());
    }

    #[allow(unused)]
    pub(crate) fn save_in_progress(&self) -> bool {
        self.property::<bool>("save-in-progress")
    }

    #[allow(unused)]
    pub(crate) fn set_save_in_progress(&self, save_in_progress: bool) {
        self.set_property("save-in-progress", save_in_progress.to_value());
    }

    pub(crate) fn app(&self) -> RnApp {
        self.application().unwrap().downcast::<RnApp>().unwrap()
    }

    pub(crate) fn overview(&self) -> adw::TabOverview {
        self.imp().overview.get()
    }

    pub(crate) fn main_header(&self) -> RnMainHeader {
        self.imp().main_header.get()
    }

    pub(crate) fn split_view(&self) -> adw::OverlaySplitView {
        self.imp().split_view.get()
    }

    pub(crate) fn sidebar(&self) -> RnSidebar {
        self.imp().sidebar.get()
    }

    pub(crate) fn overlays(&self) -> RnOverlays {
        self.imp().overlays.get()
    }

    /// Must be called after application is associated with the window else the init will panic
    pub(crate) fn init(&self) {
        let imp = self.imp();

        imp.overlays.get().init(self);
        imp.sidebar.get().init(self);
        imp.main_header.get().init(self);

        // actions and settings AFTER widget inits
        self.setup_icon_theme();
        self.setup_actions();
        self.setup_action_accels();

        if !self.app().settings_schema_found() {
            // Display an error toast if settings schema could not be found
            self.overlays().dispatch_toast_error(&gettext(
                "Settings schema is not installed. App settings could not be loaded and won't be saved.",
            ));
        } else {
            if let Err(e) = self.setup_settings_binds() {
                error!("Failed to setup settings binds, Err: {e:?}");
            }
            if let Err(e) = self.setup_periodic_save() {
                error!("Failed to setup periodic save, Err: {e:?}");
            }
            if let Err(e) = self.load_settings() {
                error!("Failed to load initial settings, Err: {e:?}");
            }
        }

        // An initial tab (canvas).
        self.add_initial_tab();

        // Anything that needs to be done right before showing the appwindow

        self.refresh_ui();
    }

    fn setup_icon_theme(&self) {
        // add icon theme resource path because automatic lookup does not work in the devel build.
        let app_icon_theme =
            IconTheme::for_display(&<Self as gtk4::prelude::WidgetExt>::display(self));
        app_icon_theme.add_resource_path((String::from(config::APP_IDPATH) + "icons").as_str());
    }

    /// Called to close the window
    pub(crate) fn close_force(&self) {
        if self.app().settings_schema_found() {
            // Saving all state
            if let Err(e) = self.save_to_settings() {
                error!("Failed to save appwindow to settings, Err: {e:?}");
            }
        }

        // Closing the state tasks channel receiver for all tabs
        for tab in self
            .tabs_snapshot()
            .into_iter()
            .map(|p| p.child().downcast::<RnCanvasWrapper>().unwrap())
        {
            let _ = tab.canvas().engine_mut().set_active(false);
            tab.canvas()
                .engine_ref()
                .engine_tasks_tx()
                .send(EngineTask::Quit);
        }

        self.destroy();
    }

    // Returns true if the flags indicate that any loop that handles the flags should be quit. (usually an async event loop)
    pub(crate) fn handle_widget_flags(&self, widget_flags: WidgetFlags, canvas: &RnCanvas) {
        //debug!("handling widget flags: '{widget_flags:?}'");

        if widget_flags.redraw {
            canvas.queue_draw();
        }
        if widget_flags.resize {
            canvas.queue_resize();
        }
        if widget_flags.refresh_ui {
            self.refresh_ui();
        }
        if widget_flags.store_modified {
            canvas.set_unsaved_changes(true);
            canvas.set_empty(false);
        }
        if widget_flags.view_modified {
            let widget_size = canvas.widget_size();
            let offset_mins_maxs = canvas.engine_ref().camera_offset_mins_maxs();
            let offset = canvas.engine_ref().camera.offset();
            // Keep the adjustments configuration in sync
            canvas.configure_adjustments(widget_size, offset_mins_maxs, offset);
            canvas.queue_resize();
        }
        if widget_flags.zoomed_temporarily {
            let total_zoom = canvas.engine_ref().camera.total_zoom();

            self.main_header()
                .canvasmenu()
                .refresh_zoom_reset_label(total_zoom);
            canvas.queue_resize();
        }
        if widget_flags.zoomed {
            let total_zoom = canvas.engine_ref().camera.total_zoom();
            let viewport = canvas.engine_ref().camera.viewport();

            canvas.canvas_layout_manager().update_old_viewport(viewport);
            self.main_header()
                .canvasmenu()
                .refresh_zoom_reset_label(total_zoom);
            canvas.queue_resize();
        }
        if widget_flags.deselect_color_setters {
            self.overlays().colorpicker().deselect_setters();
        }
        if let Some(hide_undo) = widget_flags.hide_undo {
            self.overlays()
                .penpicker()
                .undo_button()
                .set_sensitive(!hide_undo);
        }
        if let Some(hide_redo) = widget_flags.hide_redo {
            self.overlays()
                .penpicker()
                .redo_button()
                .set_sensitive(!hide_redo);
        }
        if let Some(enable_text_preprocessing) = widget_flags.enable_text_preprocessing {
            canvas.set_text_preprocessing(enable_text_preprocessing);
        }
    }

    /// Get the active (selected) tab page.
    pub(crate) fn active_tab_page(&self) -> Option<adw::TabPage> {
        self.imp().overlays.tabview().selected_page()
    }

    pub(crate) fn n_tabs_open(&self) -> usize {
        self.imp().overlays.tabview().pages().n_items() as usize
    }

    /// Returns a vector of all tabs of the current windows
    pub(crate) fn get_all_tabs(&self) -> Vec<RnCanvasWrapper> {
        let n_tabs = self.n_tabs_open();
        let mut tabs = Vec::with_capacity(n_tabs);

        for i in 0..n_tabs {
            let wrapper = self
                .imp()
                .overlays
                .tabview()
                .pages()
                .item(i as u32)
                .unwrap()
                .downcast::<adw::TabPage>()
                .unwrap()
                .child()
                .downcast::<crate::RnCanvasWrapper>()
                .unwrap();
            tabs.push(wrapper);
        }
        tabs
    }

    /// Get the active (selected) tab page child.
    pub(crate) fn active_tab_wrapper(&self) -> Option<RnCanvasWrapper> {
        self.active_tab_page()
            .map(|c| c.child().downcast::<RnCanvasWrapper>().unwrap())
    }

    /// Get the active (selected) tab page canvas.
    pub(crate) fn active_tab_canvas(&self) -> Option<RnCanvas> {
        self.active_tab_wrapper().map(|w| w.canvas())
    }

    /// adds the initial tab to the tabview
    fn add_initial_tab(&self) -> adw::TabPage {
        let wrapper = self.new_canvas_wrapper();
        self.append_wrapper_new_tab(&wrapper)
    }

    /// Creates a new canvas wrapper without attaching it as a tab.
    pub(crate) fn new_canvas_wrapper(&self) -> RnCanvasWrapper {
        let wrapper = RnCanvasWrapper::new();
        let widget_flags = wrapper
            .canvas()
            .engine_mut()
            .install_config(self.engine_config(), crate::env::pkg_data_dir().ok());
        wrapper.canvas().engine_mut().document.config = self.document_config_preset_ref().clone();
        self.handle_widget_flags(widget_flags, &wrapper.canvas());
        wrapper
    }

    /// Append the wrapper as a new tab and set it selected.
    pub(crate) fn append_wrapper_new_tab(&self, wrapper: &RnCanvasWrapper) -> adw::TabPage {
        // The tab page connections are handled in page_attached,
        // which is emitted when the page is added to the tabview.
        let page = self.overlays().tabview().append(wrapper);
        self.overlays().tabview().set_selected_page(&page);
        page
    }

    pub(crate) fn tabs_snapshot(&self) -> Vec<adw::TabPage> {
        self.overlays()
            .tabview()
            .pages()
            .snapshot()
            .into_iter()
            .map(|o| o.downcast::<adw::TabPage>().unwrap())
            .collect()
    }

    pub(crate) fn tabs_any_unsaved_changes(&self) -> bool {
        self.overlays()
            .tabview()
            .pages()
            .snapshot()
            .iter()
            .map(|o| {
                o.downcast_ref::<adw::TabPage>()
                    .unwrap()
                    .child()
                    .downcast_ref::<RnCanvasWrapper>()
                    .unwrap()
                    .canvas()
            })
            .any(|c| c.unsaved_changes())
    }

    pub(crate) fn tabs_any_saves_in_progress(&self) -> bool {
        self.overlays()
            .tabview()
            .pages()
            .snapshot()
            .iter()
            .map(|o| {
                o.downcast_ref::<adw::TabPage>()
                    .unwrap()
                    .child()
                    .downcast_ref::<RnCanvasWrapper>()
                    .unwrap()
                    .canvas()
            })
            .any(|c| c.save_in_progress())
    }

    pub(crate) fn tabs_query_file_opened(
        &self,
        input_file_path: impl AsRef<Path>,
    ) -> Option<adw::TabPage> {
        self.overlays()
            .tabview()
            .pages()
            .snapshot()
            .into_iter()
            .filter_map(|o| {
                let tab_page = o.downcast::<adw::TabPage>().unwrap();
                Some((
                    tab_page.clone(),
                    tab_page
                        .child()
                        .downcast_ref::<RnCanvasWrapper>()
                        .unwrap()
                        .canvas()
                        .output_file()?
                        .path()?,
                ))
            })
            .find(|(_, output_file_path)| {
                crate::utils::paths_abs_eq(output_file_path, input_file_path.as_ref())
                    .unwrap_or(false)
            })
            .map(|(found, _)| found)
    }

    /// Set all unselected tabs inactive.
    ///
    /// This clears the rendering and deinits the current pen of the engine in the tabs.
    ///
    /// To set a tab active again and reinit all necessary state, use `canvas.engine_mut().set_active(true)`.
    pub(crate) fn tabs_set_unselected_inactive(&self) {
        for inactive_page in self
            .overlays()
            .tabview()
            .pages()
            .snapshot()
            .into_iter()
            .map(|o| o.downcast::<adw::TabPage>().unwrap())
            .filter(|p| !p.is_selected())
        {
            let canvas = inactive_page
                .child()
                .downcast::<RnCanvasWrapper>()
                .unwrap()
                .canvas();
            // no need to handle the widget flags, since the tabs become inactive
            let _ = canvas.engine_mut().set_active(false);
        }
    }

    /// Request to close the given tab.
    ///
    /// This must then be followed up by close_tab_finish() with confirm = true to close the tab,
    /// or confirm = false to revert.
    pub(crate) fn close_tab_request(&self, tab_page: &adw::TabPage) {
        self.overlays().tabview().close_page(tab_page);
    }

    /// Complete a close_tab_request.
    ///
    /// Closes the given tab when confirm is true, else reverts so that close_tab_request() can be called again.
    pub(crate) fn close_tab_finish(&self, tab_page: &adw::TabPage, confirm: bool) {
        self.overlays()
            .tabview()
            .close_page_finish(tab_page, confirm);
    }

    pub(crate) fn refresh_titles(&self, canvas: &RnCanvas) {
        // Titles
        let title = canvas.doc_title_display();
        let subtitle = canvas.doc_folderpath_display();

        self.set_title(Some(
            &(title.clone() + " - " + config::APP_NAME_CAPITALIZED),
        ));

        self.main_header()
            .main_title_unsaved_indicator()
            .set_visible(canvas.unsaved_changes());
        if canvas.unsaved_changes() {
            self.main_header()
                .main_title()
                .add_css_class("unsaved_changes");
        } else {
            self.main_header()
                .main_title()
                .remove_css_class("unsaved_changes");
        }

        self.main_header().main_title().set_title(&title);
        self.main_header().main_title().set_subtitle(&subtitle);
    }

    /// Open the file, with import dialogs when appropriate.
    ///
    /// When the file is a rnote save file, `rnote_file_new_tab` determines if a new tab is opened,
    /// or if it loads and overwrites the content of the current active one.
    pub(crate) async fn open_file_w_dialogs(
        &self,
        input_file: gio::File,
        target_pos: Option<na::Vector2<f64>>,
        rnote_file_new_tab: bool,
    ) {
        self.overlays().progressbar_start_pulsing();
        match self
            .try_open_file(input_file, target_pos, rnote_file_new_tab)
            .await
        {
            Ok(true) => {
                self.overlays().progressbar_finish();
            }
            Ok(false) => {
                self.overlays().progressbar_abort();
            }
            Err(e) => {
                error!("Opening file with dialogs failed, Err: {e:?}");

                self.overlays()
                    .dispatch_toast_error(&gettext("Opening file failed"));
                self.overlays().progressbar_abort();
            }
        }
    }

    /// Internal method for opening/importing content from a file with a supported content type.
    ///
    /// Returns Ok(true) if file was imported, Ok(false) if not, Err(_) if the import failed.
    async fn try_open_file(
        &self,
        input_file: gio::File,
        target_pos: Option<na::Vector2<f64>>,
        rnote_file_new_tab: bool,
    ) -> anyhow::Result<bool> {
        let file_imported = match FileType::lookup_file_type(&input_file) {
            FileType::RnoteFile => {
                let input_file_path = input_file.path().ok_or_else(|| {
                    anyhow::anyhow!("Could not open file '{input_file:?}', file path is None.")
                })?;

                // If the file is already opened in a tab, simply switch to it
                if let Some(page) = self.tabs_query_file_opened(input_file_path) {
                    self.overlays().tabview().set_selected_page(&page);
                    false
                } else {
                    let (rnote_file_new_tab, wrapper) =
                        match (rnote_file_new_tab, self.active_tab_wrapper()) {
                            (true, None) => (true, self.new_canvas_wrapper()),
                            // Create a new tab when the existing is already used
                            (true, Some(active_wrapper))
                                if !active_wrapper.canvas().empty()
                                    || active_wrapper.canvas().output_file().is_some() =>
                            {
                                (true, self.new_canvas_wrapper())
                            }
                            // Re-use the existing empty tab otherwise
                            (true, Some(active_wrapper)) => (false, active_wrapper),
                            (false, None) => (true, self.new_canvas_wrapper()),
                            (false, Some(active_wrapper)) => (false, active_wrapper),
                        };

                    let (bytes, _) = input_file.load_bytes_future().await?;
                    let widget_flags = wrapper
                        .canvas()
                        .load_in_rnote_bytes(bytes.to_vec(), input_file.path())
                        .await?;
                    if rnote_file_new_tab {
                        self.append_wrapper_new_tab(&wrapper);
                    }
                    self.handle_widget_flags(widget_flags, &wrapper.canvas());
                    self.present();
                    true
                }
            }
            FileType::VectorImageFile => {
                let canvas = self
                    .active_tab_wrapper()
                    .ok_or_else(|| anyhow::anyhow!("No active tab to import into"))?
                    .canvas();
                let (bytes, _) = input_file.load_bytes_future().await?;
                canvas
                    .load_in_vectorimage_bytes(bytes.to_vec(), target_pos, self.respect_borders())
                    .await?;
                true
            }
            FileType::BitmapImageFile => {
                let canvas = self
                    .active_tab_wrapper()
                    .ok_or_else(|| anyhow::anyhow!("No active tab to import into"))?
                    .canvas();
                let (bytes, _) = input_file.load_bytes_future().await?;
                canvas
                    .load_in_bitmapimage_bytes(bytes.to_vec(), target_pos, self.respect_borders())
                    .await?;
                true
            }
            FileType::XoppFile => {
                // a new tab for xopp file import
                let wrapper = self.new_canvas_wrapper();
                let canvas = wrapper.canvas();
                let file_imported =
                    dialogs::import::dialog_import_xopp_w_prefs(self, &canvas, input_file).await?;
                if file_imported {
                    self.append_wrapper_new_tab(&wrapper);
                }
                file_imported
            }
            FileType::PdfFile => {
                let canvas = self
                    .active_tab_wrapper()
                    .ok_or_else(|| anyhow::anyhow!("No active tab to import into"))?
                    .canvas();
                dialogs::import::dialog_import_pdf_w_prefs(self, &canvas, input_file, target_pos)
                    .await?
            }
            FileType::PlaintextFile => {
                let canvas = self
                    .active_tab_wrapper()
                    .ok_or_else(|| anyhow::anyhow!("No active tab to import into"))?
                    .canvas();
                let (bytes, _) = input_file.load_bytes_future().await?;
                canvas.load_in_text(String::from_utf8(bytes.to_vec())?, target_pos)?;
                true
            }
            FileType::Folder => {
                if let Some(dir) = input_file.path() {
                    self.sidebar()
                        .workspacebrowser()
                        .workspacesbar()
                        .set_selected_workspace_dir(dir);
                }
                false
            }
            FileType::Unsupported => {
                return Err(anyhow::anyhow!("Tried to open unsupported file type"));
            }
        };

        Ok(file_imported)
    }

    /// Refresh the UI from the global state and from the current active tab page.
    pub(crate) fn refresh_ui(&self) {
        let canvas = self.active_tab_canvas();

        self.overlays().penssidebar().brush_page().refresh_ui(self);
        self.overlays().penssidebar().shaper_page().refresh_ui(self);
        self.overlays()
            .penssidebar()
            .typewriter_page()
            .refresh_ui(self);
        self.overlays().penssidebar().eraser_page().refresh_ui(self);
        self.overlays()
            .penssidebar()
            .selector_page()
            .refresh_ui(self);
        self.overlays().penssidebar().tools_page().refresh_ui(self);
        self.sidebar().settings_panel().refresh_ui(self);

        if let Some(canvas) = canvas {
            self.refresh_titles(&canvas);

            // Avoids already borrowed
            let pen_style = canvas.engine_ref().current_pen_style_w_override();
            let pen_sounds = canvas.engine_ref().pen_sounds();
            let snap_positions = self.engine_config().read().snap_positions;
            let total_zoom = canvas.engine_ref().camera.total_zoom();
            let can_undo = canvas.engine_ref().can_undo();
            let can_redo = canvas.engine_ref().can_redo();
            let visual_debug = self.engine_config().read().visual_debug;

            self.overlays()
                .penpicker()
                .undo_button()
                .set_sensitive(can_undo);
            self.overlays()
                .penpicker()
                .redo_button()
                .set_sensitive(can_redo);
            self.main_header()
                .canvasmenu()
                .refresh_zoom_reset_label(total_zoom);
            self.set_pen_style(pen_style);
            self.set_pen_sounds(pen_sounds);
            self.set_snap_positions(snap_positions);
            self.set_visual_debug(visual_debug);

            // Current pen
            match pen_style {
                PenStyle::Brush => {
                    self.overlays().penpicker().brush_toggle().set_active(true);
                    self.overlays()
                        .penssidebar()
                        .sidebar_stack()
                        .set_visible_child_name("brush_page");

                    let style = self.engine_config().read().pens_config.brush_config.style;
                    match style {
                        BrushStyle::Marker => {
                            let stroke_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .brush_config
                                .marker_options
                                .stroke_color
                                .unwrap_or(Color::TRANSPARENT);
                            let fill_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .brush_config
                                .marker_options
                                .fill_color
                                .unwrap_or(Color::TRANSPARENT);
                            self.overlays()
                                .colorpicker()
                                .set_stroke_color(gdk::RGBA::from_compose_color(stroke_color));
                            self.overlays()
                                .colorpicker()
                                .set_fill_color(gdk::RGBA::from_compose_color(fill_color));
                        }
                        BrushStyle::Solid => {
                            let stroke_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .brush_config
                                .solid_options
                                .stroke_color
                                .unwrap_or(Color::TRANSPARENT);
                            let fill_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .brush_config
                                .solid_options
                                .fill_color
                                .unwrap_or(Color::TRANSPARENT);
                            self.overlays()
                                .colorpicker()
                                .set_stroke_color(gdk::RGBA::from_compose_color(stroke_color));
                            self.overlays()
                                .colorpicker()
                                .set_fill_color(gdk::RGBA::from_compose_color(fill_color));
                        }
                        BrushStyle::Textured => {
                            let stroke_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .brush_config
                                .textured_options
                                .stroke_color
                                .unwrap_or(Color::TRANSPARENT);
                            self.overlays()
                                .colorpicker()
                                .set_stroke_color(gdk::RGBA::from_compose_color(stroke_color));
                        }
                    }
                }
                PenStyle::Shaper => {
                    self.overlays().penpicker().shaper_toggle().set_active(true);
                    self.overlays()
                        .penssidebar()
                        .sidebar_stack()
                        .set_visible_child_name("shaper_page");

                    let style = self.engine_config().read().pens_config.shaper_config.style;
                    match style {
                        ShaperStyle::Smooth => {
                            let stroke_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .shaper_config
                                .smooth_options
                                .stroke_color
                                .unwrap_or(Color::TRANSPARENT);
                            let fill_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .shaper_config
                                .smooth_options
                                .fill_color
                                .unwrap_or(Color::TRANSPARENT);
                            self.overlays()
                                .colorpicker()
                                .set_stroke_color(gdk::RGBA::from_compose_color(stroke_color));
                            self.overlays()
                                .colorpicker()
                                .set_fill_color(gdk::RGBA::from_compose_color(fill_color));
                        }
                        ShaperStyle::Rough => {
                            let stroke_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .shaper_config
                                .rough_options
                                .stroke_color
                                .unwrap_or(Color::TRANSPARENT);
                            let fill_color = self
                                .engine_config()
                                .read()
                                .pens_config
                                .shaper_config
                                .rough_options
                                .fill_color
                                .unwrap_or(Color::TRANSPARENT);
                            self.overlays()
                                .colorpicker()
                                .set_stroke_color(gdk::RGBA::from_compose_color(stroke_color));
                            self.overlays()
                                .colorpicker()
                                .set_fill_color(gdk::RGBA::from_compose_color(fill_color));
                        }
                    }
                }
                PenStyle::Typewriter => {
                    self.overlays()
                        .penpicker()
                        .typewriter_toggle()
                        .set_active(true);
                    self.overlays()
                        .penssidebar()
                        .sidebar_stack()
                        .set_visible_child_name("typewriter_page");

                    let text_color = self
                        .engine_config()
                        .read()
                        .pens_config
                        .typewriter_config
                        .text_style
                        .color;
                    self.overlays()
                        .colorpicker()
                        .set_stroke_color(gdk::RGBA::from_compose_color(text_color));
                }
                PenStyle::Eraser => {
                    self.overlays().penpicker().eraser_toggle().set_active(true);
                    self.overlays()
                        .penssidebar()
                        .sidebar_stack()
                        .set_visible_child_name("eraser_page");
                }
                PenStyle::Selector => {
                    self.overlays()
                        .penpicker()
                        .selector_toggle()
                        .set_active(true);
                    self.overlays()
                        .penssidebar()
                        .sidebar_stack()
                        .set_visible_child_name("selector_page");
                }
                PenStyle::Tools => {
                    self.overlays().penpicker().tools_toggle().set_active(true);
                    self.overlays()
                        .penssidebar()
                        .sidebar_stack()
                        .set_visible_child_name("tools_page");
                }
            }
        }
    }

    pub(crate) fn load_global_config_from_settings(
        &self,
        settings: &gio::Settings,
    ) -> anyhow::Result<()> {
        {
            // load engine config
            let engine_config_str = settings.string("engine-config");

            if engine_config_str.is_empty() {
                // On first app startup the engine config is empty, so we don't log an error
                debug!("Did not load `engine-config` from settings, was empty");
            } else {
                let engine_config = serde_json::from_str::<EngineConfig>(&engine_config_str)?;
                self.engine_config().load_values(engine_config);
            }
        }

        {
            // load document config preset
            let document_config_preset_str = settings.string("document-config-preset");

            if document_config_preset_str.is_empty() {
                // On first app startup the document config preset is empty, so we don't log an error
                debug!("Did not load `document-config-preset` from settings, was empty");
            } else {
                let document_config_preset =
                    serde_json::from_str::<DocumentConfig>(&document_config_preset_str)?;
                self.imp()
                    .document_config_preset
                    .replace(document_config_preset);
            }
        }

        if let Some(canvas) = self.active_tab_canvas() {
            let widget_flags = canvas
                .engine_mut()
                .install_config(self.engine_config(), env::pkg_data_dir().ok());
            self.handle_widget_flags(widget_flags, &canvas);
        }
        Ok(())
    }

    pub(crate) fn save_global_config_to_settings(
        &self,
        settings: &gio::Settings,
    ) -> anyhow::Result<()> {
        let engine_config_str = serde_json::to_string(self.engine_config())?;
        settings.set_string("engine-config", engine_config_str.as_str())?;
        let document_config_preset_str =
            serde_json::to_string(&*self.document_config_preset_ref())?;
        settings.set_string(
            "document-config-preset",
            document_config_preset_str.as_str(),
        )?;
        Ok(())
    }

    /// exports and writes the engine config as json into the file.
    /// Only for debugging!
    pub(crate) async fn export_engine_config(&self, file: &gio::File) -> anyhow::Result<()> {
        let config_serialized = serde_json::to_string_pretty(self.engine_config())?;

        crate::utils::create_replace_file_future(config_serialized.into_bytes(), file).await?;

        if let Some(canvas) = self.active_tab_canvas() {
            canvas.set_last_export_dir(file.parent());
        }

        Ok(())
    }
}
