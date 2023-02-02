mod appactions;

use adw::subclass::prelude::AdwApplicationImpl;
use gtk4::{gio, glib, prelude::*, subclass::prelude::*};
use rnote_engine::document::format::MeasureUnit;
use rnote_engine::pens::PenStyle;
use std::path::Path;

use crate::{
    colorpicker::RnColorPad, colorpicker::RnColorSetter, config, penssidebar::RnBrushPage,
    penssidebar::RnEraserPage, penssidebar::RnSelectorPage, penssidebar::RnShaperPage,
    penssidebar::RnToolsPage, penssidebar::RnTypewriterPage, settingspanel::RnPenShortcutRow,
    strokewidthpicker::RnStrokeWidthPreview, strokewidthpicker::RnStrokeWidthSetter,
    workspacebrowser::workspacesbar::RnWorkspaceRow, workspacebrowser::RnFileRow,
    workspacebrowser::RnWorkspacesBar, RnAppMenu, RnAppWindow, RnCanvas, RnCanvasMenu,
    RnCanvasWrapper, RnColorPicker, RnIconPicker, RnMainHeader, RnOverlays, RnPensSideBar,
    RnSettingsPanel, RnStrokeWidthPicker, RnUnitEntry, RnWorkspaceBrowser,
};

mod imp {
    use super::*;

    #[allow(missing_debug_implementations)]
    #[derive(Default)]
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

            // init and show the first window
            self.new_appwindow_init_show(None);
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);

            // open another appwindow and load the file
            self.new_appwindow_init_show(files.first().cloned());
        }
    }

    impl GtkApplicationImpl for RnApp {}

    impl AdwApplicationImpl for RnApp {}

    impl RnApp {
        fn init(&self) {
            let inst = self.instance();

            self.setup_logging();
            self.setup_i18n();
            self.setup_gresources();
            inst.setup_actions();
            inst.setup_action_accels();
        }

        /// Initializes and shows a new app window
        fn new_appwindow_init_show(&self, input_file: Option<gio::File>) {
            let appwindow = RnAppWindow::new(self.instance().upcast_ref::<gtk4::Application>());
            appwindow.init();
            appwindow.show();

            // Loading in input file in the first tab, if Some
            if let Some(input_file) = input_file {
                appwindow.open_file_w_dialogs(input_file, None, false);
            }
        }

        fn setup_logging(&self) {
            if let Err(e) = pretty_env_logger::try_init_timed() {
                eprintln!("initializing logging failed, Err: {e:?}");
            } else {
                log::debug!("... env_logger initialized");
            }
        }

        fn setup_i18n(&self) {
            gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
            gettextrs::bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR)
                .expect("Unable to bind the text domain");
            gettextrs::textdomain(config::GETTEXT_PACKAGE)
                .expect("Unable to switch to the text domain");
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
            PenStyle::static_type();
            RnWorkspaceBrowser::static_type();
            RnWorkspacesBar::static_type();
            RnFileRow::static_type();
            RnWorkspaceRow::static_type();
            MeasureUnit::static_type();
            RnUnitEntry::static_type();
            RnIconPicker::static_type();
            RnPenShortcutRow::static_type();
            RnStrokeWidthPicker::static_type();
            RnStrokeWidthSetter::static_type();
            RnStrokeWidthPreview::static_type();

            self.instance()
                .set_resource_base_path(Some(config::APP_IDPATH));
            let resource = gio::Resource::load(Path::new(config::RESOURCES_FILE))
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
        glib::Object::new(&[
            ("application-id", &config::APP_ID),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
        ])
    }
}
