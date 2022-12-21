mod appactions;

use adw::subclass::prelude::AdwApplicationImpl;
use gtk4::{gio, glib, prelude::*, subclass::prelude::*};
use rnote_engine::document::format::MeasureUnit;
use rnote_engine::pens::PenStyle;
use std::path::Path;

use crate::{
    colorpicker::ColorSetter, config, penssidebar::BrushPage, penssidebar::EraserPage,
    penssidebar::SelectorPage, penssidebar::ShaperPage, penssidebar::ToolsPage,
    penssidebar::TypewriterPage, settingspanel::PenShortcutRow, workspacebrowser::FileRow,
    workspacebrowser::WorkspaceRow, AppMenu, CanvasMenu, ColorPicker, IconPicker, MainHeader,
    PensSideBar, RnoteAppWindow, RnoteCanvas, RnoteCanvasWrapper, SettingsPanel, UnitEntry,
    WorkspaceBrowser,
};

mod imp {

    use super::*;
    #[allow(missing_debug_implementations)]
    #[derive(Default)]
    pub(crate) struct RnoteApp {}

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteApp {
        const NAME: &'static str = "RnoteApp";
        type Type = super::RnoteApp;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for RnoteApp {}

    impl ApplicationImpl for RnoteApp {
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

    impl GtkApplicationImpl for RnoteApp {}

    impl AdwApplicationImpl for RnoteApp {}

    impl RnoteApp {
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
            let appwindow = RnoteAppWindow::new(self.instance().upcast_ref::<gtk4::Application>());
            appwindow.init(input_file);

            appwindow.show();
        }

        fn setup_logging(&self) {
            pretty_env_logger::init();
            log::debug!("... env_logger initialized");
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
            RnoteAppWindow::static_type();
            RnoteCanvasWrapper::static_type();
            RnoteCanvas::static_type();
            ColorPicker::static_type();
            ColorSetter::static_type();
            CanvasMenu::static_type();
            SettingsPanel::static_type();
            AppMenu::static_type();
            MainHeader::static_type();
            PensSideBar::static_type();
            BrushPage::static_type();
            ShaperPage::static_type();
            EraserPage::static_type();
            SelectorPage::static_type();
            TypewriterPage::static_type();
            ToolsPage::static_type();
            PenStyle::static_type();
            WorkspaceBrowser::static_type();
            FileRow::static_type();
            WorkspaceRow::static_type();
            MeasureUnit::static_type();
            UnitEntry::static_type();
            IconPicker::static_type();
            PenShortcutRow::static_type();

            self.instance()
                .set_resource_base_path(Some(config::APP_IDPATH));
            let resource = gio::Resource::load(Path::new(config::RESOURCES_FILE))
                .expect("Could not load gresource file");
            gio::resources_register(&resource);
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnoteApp(ObjectSubclass<imp::RnoteApp>)
        @extends gio::Application, gtk4::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for RnoteApp {
    fn default() -> Self {
        Self::new()
    }
}

impl RnoteApp {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &config::APP_ID),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
        ])
    }
}
