// Modules
mod appactions;

// Imports
use crate::{
    RnAppMenu, RnAppWindow, RnCanvas, RnCanvasMenu, RnCanvasWrapper, RnColorPicker, RnIconPicker,
    RnMainHeader, RnOverlays, RnPenPicker, RnPensSideBar, RnSettingsPanel, RnSidebar,
    RnStrokeContentPreview, RnStrokeWidthPicker, RnUnitEntry, RnWorkspaceBrowser,
    colorpicker::RnColorPad, colorpicker::RnColorSetter, config, penssidebar::RnBrushPage,
    penssidebar::RnEraserPage, penssidebar::RnSelectorPage, penssidebar::RnShaperPage,
    penssidebar::RnToolsPage, penssidebar::RnTypewriterPage, settingspanel::RnPenShortcutRow,
    strokewidthpicker::RnStrokeWidthPreview, strokewidthpicker::RnStrokeWidthSetter,
    strokewidthpicker::StrokeWidthPreviewStyle, workspacebrowser::RnFileRow,
    workspacebrowser::RnWorkspacesBar, workspacebrowser::workspacesbar::RnWorkspaceRow,
};
use adw::subclass::prelude::AdwApplicationImpl;
use adw::prelude::*;
use gtk4::{WindowGroup, gio, glib, glib::clone, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnApp {
        pub(crate) app_settings: Option<gio::Settings>,
    }

    impl Default for RnApp {
        fn default() -> Self {
            let app_settings = gio::SettingsSchemaSource::default().and_then(|schema_source| {
                Some(gio::Settings::new_full(
                    &schema_source.lookup(config::APP_ID, true)?,
                    None::<&gio::SettingsBackend>,
                    None,
                ))
            });

            Self { app_settings }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnApp {
        const NAME: &'static str = "RnApp";
        type Type = super::RnApp;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for RnApp {}

    impl ApplicationImpl for RnApp {
        fn startup(&self) {
            let obj = self.obj();
            self.parent_startup();

            self.setup_buildables();
            obj.setup_actions();
            obj.setup_action_accels();
        }

        fn activate(&self) {
            self.parent_activate();

            // init and show a new window
            self.new_appwindow_init_show(None);
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);

            let input_file = files.first().cloned();
            if let Some(appwindow) = self
                .obj()
                .active_window()
                .map(|w| w.downcast::<RnAppWindow>().unwrap())
            {
                if let Some(input_file) = input_file {
                    glib::spawn_future_local(clone!(
                        #[weak]
                        appwindow,
                        async move {
                            appwindow.open_file_w_dialogs(input_file, None, true).await;
                        }
                    ));
                }
            } else {
                self.new_appwindow_init_show(input_file);
            }
        }
    }

    impl GtkApplicationImpl for RnApp {}

    impl AdwApplicationImpl for RnApp {}

    impl RnApp {
        /// Custom buildable Widgets need to register
        fn setup_buildables(&self) {
            RnAppWindow::static_type();
            RnOverlays::static_type();
            RnCanvasWrapper::static_type();
            RnCanvas::static_type();
            RnColorPicker::static_type();
            RnColorSetter::static_type();
            RnColorPad::static_type();
            RnCanvasMenu::static_type();
            RnSettingsPanel::static_type();
            RnAppMenu::static_type();
            RnMainHeader::static_type();
            RnPensSideBar::static_type();
            RnBrushPage::static_type();
            RnShaperPage::static_type();
            RnEraserPage::static_type();
            RnSelectorPage::static_type();
            RnTypewriterPage::static_type();
            RnToolsPage::static_type();
            RnWorkspaceBrowser::static_type();
            RnWorkspacesBar::static_type();
            RnFileRow::static_type();
            RnWorkspaceRow::static_type();
            RnUnitEntry::static_type();
            RnIconPicker::static_type();
            RnPenShortcutRow::static_type();
            RnStrokeWidthPicker::static_type();
            RnStrokeWidthSetter::static_type();
            RnStrokeWidthPreview::static_type();
            StrokeWidthPreviewStyle::static_type();
            RnStrokeContentPreview::static_type();
            RnSidebar::static_type();
            RnPenPicker::static_type();
        }

        /// Initializes and shows a new app window
        pub(crate) fn new_appwindow_init_show(&self, input_file: Option<gio::File>) {
            let appwindow = RnAppWindow::new(self.obj().upcast_ref::<gtk4::Application>());
            appwindow.init(true);
            // create a window group for each app window
            // to make modals only impact the current app
            // window.
            // See issue: https://github.com/flxzt/rnote/issues/1461
            let window_group = WindowGroup::new();
            window_group.add_window(&appwindow);

            appwindow.present();

            // --- AUTOSAVE RECOVERY CHECK ---
            let input_file_clone = input_file.clone();
            glib::spawn_future_local(clone!(#[weak] appwindow, async move {
                let cache_dir = glib::user_cache_dir();
                let backup_dir = cache_dir.join("rnote").join("autosaves");

                let mut found_backups = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&backup_dir) {
                    for entry in entries.flatten() {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.ends_with(".rnote") {
                                found_backups.push(entry.path());
                            }
                        }
                    }
                }

                if !found_backups.is_empty() && input_file_clone.is_none() {
                    let dialog = adw::AlertDialog::builder()
                        .heading(&gettextrs::gettext("Recover Unsaved Drafts"))
                        .body(&gettextrs::gettext("Found unsaved drafts from a previous session. What would you like to do?"))
                        .build();

                    dialog.add_response("ignore", &gettextrs::gettext("Ignore"));
                    dialog.add_response("delete", &gettextrs::gettext("Delete"));
                    dialog.add_response("open", &gettextrs::gettext("Open Drafts"));
                    dialog.set_default_response(Some("open"));
                    dialog.set_close_response("ignore");
                    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                    dialog.set_response_appearance("open", adw::ResponseAppearance::Suggested);

                    // adw::AlertDialog is shown on a transient parent window using .choose_future()
                    let response = dialog.choose_future(Some(&appwindow)).await;
                    
                    match response.as_str() {
                        "open" => {
                            for backup_path in found_backups {
                                let file = gio::File::for_path(&backup_path);
                                
                                // Open the autosave. 
                                appwindow.open_file_w_dialogs(file, None, true).await;
                                
                                // Find the tab we just opened and "detach" it from the cache file
                                for tab in appwindow.tabs_snapshot() {
                                    if let Ok(wrapper) = tab.child().downcast::<RnCanvasWrapper>() {
                                        let canvas = wrapper.canvas();
                                        
                                        if let Some(out_file) = canvas.output_file() {
                                            if out_file.path() == Some(backup_path.clone()) {
                                                // Trick Rnote into thinking this is a brand new draft!
                                                canvas.set_output_file(None);
                                                canvas.set_unsaved_changes(true);
                                                canvas.set_draft_needs_backup(true);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "delete" => {
                            for path in found_backups {
                                let _ = std::fs::remove_file(path);
                            }
                        }
                        _ => { /* Ignore */ }
                    }
                }
            }));

            // Loading in input file in the first tab, if Some
            if let Some(input_file) = input_file {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        appwindow.open_file_w_dialogs(input_file, None, false).await;
                    }
                ));
            }
        }

        pub(crate) fn new_appwindow_init_return_tab(&self) -> adw::TabView {
            let appwindow = RnAppWindow::new(self.obj().upcast_ref::<gtk4::Application>());
            appwindow.init(false);

            // create a window group for each app window
            // to make modals only impact the current app
            // window.
            // See issue: https://github.com/flxzt/rnote/issues/1461
            let window_group = WindowGroup::new();
            window_group.add_window(&appwindow);
            appwindow.present();

            appwindow.overlays().tabview()
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnApp(ObjectSubclass<imp::RnApp>)
        @extends gio::Application, gtk4::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for RnApp {
    fn default() -> Self {
        Self::new()
    }
}

impl RnApp {
    pub(crate) fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", config::APP_IDPATH)
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .property("register-session", true)
            .build()
    }

    /// Returns the app settings, if the schema is found in the compiled gschema. If not, returns None.
    ///
    /// Callers that query the settings should implement good default fallback accordingly
    pub(crate) fn app_settings(&self) -> Option<gio::Settings> {
        self.imp().app_settings.clone()
    }

    pub(crate) fn settings_schema_found(&self) -> bool {
        self.app_settings().is_some()
    }

    pub(crate) fn new_appwindow_init_show(&self) {
        self.imp().new_appwindow_init_show(None);
    }

    pub(crate) fn new_appwindow_init_return_tab(&self) -> adw::TabView {
        self.imp().new_appwindow_init_return_tab()
    }
}
