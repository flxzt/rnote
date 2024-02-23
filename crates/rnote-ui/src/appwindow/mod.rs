// Modules
mod actions;
mod appsettings;
mod imp;

// Imports
use crate::{
    config, dialogs, FileType, RnApp, RnCanvas, RnCanvasWrapper, RnMainHeader, RnOverlays,
    RnSidebar,
};
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk4::{gdk, gio, glib, Application, IconTheme};
use rnote_compose::Color;
use rnote_engine::ext::GdkRGBAExt;
use rnote_engine::pens::pensconfig::brushconfig::BrushStyle;
use rnote_engine::pens::pensconfig::shaperconfig::ShaperStyle;
use rnote_engine::pens::PenStyle;
use rnote_engine::{engine::EngineTask, WidgetFlags};
use std::path::Path;

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

    pub(crate) fn app(&self) -> RnApp {
        self.application().unwrap().downcast::<RnApp>().unwrap()
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

        // An initial tab. Must! come before setting up the settings binds and import
        self.add_initial_tab();

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
                tracing::error!("Failed to setup settings binds, Err: {e:}");
            }
            if let Err(e) = self.setup_periodic_save() {
                tracing::error!("Failed to setup periodic save, Err: {e:}");
            }
            if let Err(e) = self.load_settings() {
                tracing::error!("Failed to load initial settings, Err: {e:}");
            }
        }

        // Anything that needs to be done right before showing the appwindow

        // Set undo / redo as not sensitive as default - setting it in .ui file did not work for some reason
        self.overlays()
            .penpicker()
            .undo_button()
            .set_sensitive(false);
        self.overlays()
            .penpicker()
            .redo_button()
            .set_sensitive(false);
        self.refresh_ui_from_engine(&self.active_tab_wrapper());
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
                tracing::error!("Failed to save appwindow to settings, Err: {e:?}");
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
        if widget_flags.redraw {
            canvas.queue_draw();
        }
        if widget_flags.resize {
            canvas.queue_resize();
        }
        if widget_flags.refresh_ui {
            self.refresh_ui_from_engine(&self.active_tab_wrapper());
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
    ///
    /// Panics if there is none, but this should never be the case,
    /// since a first one is added initially and the UI hides closing the last tab.
    pub(crate) fn active_tab_page(&self) -> adw::TabPage {
        self.imp()
            .overlays
            .tabview()
            .selected_page()
            .expect("there must always be one active tab")
    }

    pub(crate) fn get_number_of_tabs(&self) -> u32 {
        self.imp().overlays.tabview().pages().n_items()
    }

    /// returns a vector of all tabs of the current windows
    pub(crate) fn get_all_tabs(&self) -> Result<Vec<RnCanvas>, anyhow::Error> {
        let n_tabs = self.get_number_of_tabs();
        let mut output = Vec::<RnCanvas>::new();

        for i in 0..n_tabs {
            let glib_obj = self.imp().overlays.tabview().pages().item(i);
            if glib_obj.is_none() {
                return Err(anyhow::anyhow!("could not obtain the tab {:?}", i));
            }
            let tab_glib = glib_obj
                .expect("downcast failure")
                .downcast::<adw::TabPage>();
            let wrapper = tab_glib
                .unwrap()
                .child()
                .downcast::<crate::RnCanvasWrapper>();
            let canvas = wrapper.unwrap().canvas();
            output.push(canvas);
        }
        Ok(output)
    }

    /// Get the active (selected) tab page child.
    pub(crate) fn active_tab_wrapper(&self) -> RnCanvasWrapper {
        self.active_tab_page()
            .child()
            .downcast::<RnCanvasWrapper>()
            .unwrap()
    }

    /// adds the initial tab to the tabview
    fn add_initial_tab(&self) -> adw::TabPage {
        let wrapper = RnCanvasWrapper::new();
        if let Some(app_settings) = self.app().app_settings() {
            if let Err(e) = wrapper
                .canvas()
                .load_engine_config_from_settings(&app_settings)
            {
                tracing::error!("Failed to load engine config for initial tab, Err: {e:?}");
            }
        } else {
            tracing::warn!("Could not load settings for initial tab. Settings schema not found.");
        }
        self.append_wrapper_new_tab(&wrapper)
    }

    /// Creates a new canvas wrapper without attaching it as a tab.
    pub(crate) fn new_canvas_wrapper(&self) -> RnCanvasWrapper {
        let engine_config = self
            .active_tab_wrapper()
            .canvas()
            .engine_ref()
            .extract_engine_config();
        let wrapper = RnCanvasWrapper::new();
        let widget_flags = wrapper
            .canvas()
            .engine_mut()
            .load_engine_config(engine_config, crate::env::pkg_data_dir().ok());
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
                same_file::is_same_file(output_file_path, input_file_path.as_ref()).unwrap_or(false)
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

    pub(crate) fn refresh_titles(&self, active_tab: &RnCanvasWrapper) {
        let canvas = active_tab.canvas();

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
                tracing::error!("Opening file with dialogs failed, Err: {e:?}");

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
                    let rnote_file_new_tab = if self.active_tab_wrapper().canvas().empty()
                        && self.active_tab_wrapper().canvas().output_file().is_none()
                    {
                        false
                    } else {
                        rnote_file_new_tab
                    };

                    let wrapper = if rnote_file_new_tab {
                        // a new tab for rnote files
                        self.new_canvas_wrapper()
                    } else {
                        self.active_tab_wrapper()
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
                    true
                }
            }
            FileType::VectorImageFile => {
                let canvas = self.active_tab_wrapper().canvas();
                let (bytes, _) = input_file.load_bytes_future().await?;
                canvas
                    .load_in_vectorimage_bytes(bytes.to_vec(), target_pos)
                    .await?;
                true
            }
            FileType::BitmapImageFile => {
                let canvas = self.active_tab_wrapper().canvas();
                let (bytes, _) = input_file.load_bytes_future().await?;
                canvas
                    .load_in_bitmapimage_bytes(bytes.to_vec(), target_pos)
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
                let canvas = self.active_tab_wrapper().canvas();
                dialogs::import::dialog_import_pdf_w_prefs(self, &canvas, input_file, target_pos)
                    .await?
            }
            FileType::PlaintextFile => {
                let canvas = self.active_tab_wrapper().canvas();
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

    /// Refresh the UI from the engine state from the given tab page.
    pub(crate) fn refresh_ui_from_engine(&self, active_tab: &RnCanvasWrapper) {
        let canvas = active_tab.canvas();

        // Avoids already borrowed
        let pen_style = canvas.engine_ref().penholder.current_pen_style_w_override();
        let pen_sounds = canvas.engine_ref().pen_sounds();
        let doc_format = canvas.engine_ref().document.format;
        let total_zoom = canvas.engine_ref().camera.total_zoom();
        let snap_positions = canvas.engine_ref().document.snap_positions;
        let can_undo = canvas.engine_ref().can_undo();
        let can_redo = canvas.engine_ref().can_redo();

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

        // we change the state through the actions, because they themselves hold state.
        // (for example needed to display ticks in menus for boolean actions)
        adw::prelude::ActionGroupExt::change_action_state(
            self,
            "pen-style",
            &pen_style.to_string().to_variant(),
        );
        adw::prelude::ActionGroupExt::change_action_state(
            self,
            "pen-sounds",
            &pen_sounds.to_variant(),
        );
        adw::prelude::ActionGroupExt::change_action_state(
            self,
            "snap-positions",
            &snap_positions.to_variant(),
        );
        adw::prelude::ActionGroupExt::change_action_state(
            self,
            "show-format-borders",
            &doc_format.show_borders.to_variant(),
        );
        adw::prelude::ActionGroupExt::change_action_state(
            self,
            "show-origin-indicator",
            &doc_format.show_origin_indicator.to_variant(),
        );

        // Current pen
        match pen_style {
            PenStyle::Brush => {
                self.overlays().penpicker().brush_toggle().set_active(true);
                self.overlays()
                    .penssidebar()
                    .sidebar_stack()
                    .set_visible_child_name("brush_page");

                let style = canvas.engine_ref().pens_config.brush_config.style;
                match style {
                    BrushStyle::Marker => {
                        let stroke_color = canvas
                            .engine_ref()
                            .pens_config
                            .brush_config
                            .marker_options
                            .stroke_color
                            .unwrap_or(Color::TRANSPARENT);
                        let fill_color = canvas
                            .engine_ref()
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
                        let stroke_color = canvas
                            .engine_ref()
                            .pens_config
                            .brush_config
                            .solid_options
                            .stroke_color
                            .unwrap_or(Color::TRANSPARENT);
                        let fill_color = canvas
                            .engine_ref()
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
                        let stroke_color = canvas
                            .engine_ref()
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

                let style = canvas.engine_ref().pens_config.shaper_config.style;
                match style {
                    ShaperStyle::Smooth => {
                        let stroke_color = canvas
                            .engine_ref()
                            .pens_config
                            .shaper_config
                            .smooth_options
                            .stroke_color
                            .unwrap_or(Color::TRANSPARENT);
                        let fill_color = canvas
                            .engine_ref()
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
                        let stroke_color = canvas
                            .engine_ref()
                            .pens_config
                            .shaper_config
                            .rough_options
                            .stroke_color
                            .unwrap_or(Color::TRANSPARENT);
                        let fill_color = canvas
                            .engine_ref()
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

                let text_color = canvas
                    .engine_ref()
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

        self.overlays()
            .penssidebar()
            .brush_page()
            .refresh_ui(active_tab);
        self.overlays()
            .penssidebar()
            .shaper_page()
            .refresh_ui(active_tab);
        self.overlays()
            .penssidebar()
            .typewriter_page()
            .refresh_ui(active_tab);
        self.overlays()
            .penssidebar()
            .eraser_page()
            .refresh_ui(active_tab);
        self.overlays()
            .penssidebar()
            .selector_page()
            .refresh_ui(active_tab);
        self.overlays()
            .penssidebar()
            .tools_page()
            .refresh_ui(active_tab);
        self.sidebar().settings_panel().refresh_ui(active_tab);
        self.refresh_titles(active_tab);
    }

    /// Sync the state from the previous active tab and the current one. Used when the selected tab changes.
    pub(crate) fn sync_state_between_tabs(
        &self,
        prev_tab: &adw::TabPage,
        active_tab: &adw::TabPage,
    ) {
        if *prev_tab == *active_tab {
            return;
        }
        let prev_canvas_wrapper = prev_tab.child().downcast::<RnCanvasWrapper>().unwrap();
        let prev_canvas = prev_canvas_wrapper.canvas();
        let active_canvas_wrapper = active_tab.child().downcast::<RnCanvasWrapper>().unwrap();
        let active_canvas = active_canvas_wrapper.canvas();

        let mut widget_flags = active_canvas.engine_mut().load_engine_config_sync_tab(
            prev_canvas.engine_ref().extract_engine_config(),
            crate::env::pkg_data_dir().ok(),
        );
        // The visual-debug field is not saved in the config, but we want to sync its value between tabs.
        widget_flags |= active_canvas
            .engine_mut()
            .set_visual_debug(prev_canvas.engine_mut().visual_debug());

        self.handle_widget_flags(widget_flags, &active_canvas);
    }
}
