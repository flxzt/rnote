// Modules
mod appactions;

// Imports
use crate::{
    colorpicker::RnColorPad, colorpicker::RnColorSetter, config, globals, penssidebar::RnBrushPage,
    penssidebar::RnEraserPage, penssidebar::RnSelectorPage, penssidebar::RnShaperPage,
    penssidebar::RnToolsPage, penssidebar::RnTypewriterPage, settingspanel::RnPenShortcutRow,
    strokewidthpicker::RnStrokeWidthPreview, strokewidthpicker::RnStrokeWidthSetter,
    strokewidthpicker::StrokeWidthPreviewStyle, workspacebrowser::workspacesbar::RnWorkspaceRow,
    workspacebrowser::RnFileRow, workspacebrowser::RnWorkspacesBar, RnAppMenu, RnAppWindow,
    RnCanvas, RnCanvasMenu, RnCanvasWrapper, RnColorPicker, RnIconPicker, RnMainHeader, RnOverlays,
    RnPensSideBar, RnSettingsPanel, RnSidebar, RnStrokeContentPreview, RnStrokeWidthPicker,
    RnUnitEntry, RnWorkspaceBrowser,
};
use adw::subclass::prelude::AdwApplicationImpl;
use gtk4::{gio, glib, glib::clone, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct RnApp {}

    #[glib::object_subclass]
    impl ObjectSubclass for RnApp {
        const NAME: &'static str = "RnApp";
        type Type = super::RnApp;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for RnApp {}

    impl ApplicationImpl for RnApp {
        fn startup(&self) {
            self.parent_startup();

            self.init();
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
                    glib::MainContext::default().spawn_local(
                        clone!(@weak appwindow => async move {
                            appwindow.open_file_w_dialogs(input_file, None, true).await;
                        }),
                    );
                }
            } else {
                self.new_appwindow_init_show(input_file);
            }
        }
    }

    impl GtkApplicationImpl for RnApp {}

    impl AdwApplicationImpl for RnApp {}

    impl RnApp {
        fn init(&self) {
            let obj = self.obj();

            self.setup_gresources();
            obj.setup_actions();
            obj.setup_action_accels();
        }

        /// Initializes and shows a new app window
        fn new_appwindow_init_show(&self, input_file: Option<gio::File>) {
            let appwindow = RnAppWindow::new(self.obj().upcast_ref::<gtk4::Application>());
            appwindow.init();
            appwindow.present();

            // Loading in input file in the first tab, if Some
            if let Some(input_file) = input_file {
                glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                    appwindow.open_file_w_dialogs(input_file, None, false).await;
                }));
            }
        }

        fn setup_gresources(&self) {
            // Custom buildable Widgets need to register
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

            self.obj().set_resource_base_path(Some(config::APP_IDPATH));
            let resource = gio::Resource::load(
                crate::env::pkg_data_dir()
                    .expect("Could not retrieve pkg data dir")
                    .join(globals::GRESOURCES_FILENAME),
            )
            .expect("Could not load gresource file");
            gio::resources_register(&resource);
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
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .property("register-session", true)
            .build()
    }
}
