mod appactions;

use adw::subclass::prelude::AdwApplicationImpl;
use gtk4::{gio, glib, prelude::*, subclass::prelude::*};
use rnote_engine::document::format::MeasureUnit;
use rnote_engine::pens::penholder::PenStyle;
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
        fn activate(&self) {
            let inst = self.instance();

            // Load in the needed gresources
            self.load_gresources();

            // setup the app
            inst.setup_actions();
            inst.setup_action_accels();

            // and init and show the first window
            self.new_window_show(None);
        }

        fn open(&self, files: &[gio::File], _hint: &str) {
            self.load_gresources();

            self.new_window_show(files.first().cloned());
        }
    }

    impl GtkApplicationImpl for RnoteApp {}
    impl AdwApplicationImpl for RnoteApp {}

    impl RnoteApp {
        /// Initializes and shows a new app window
        fn new_window_show(&self, input_file: Option<gio::File>) {
            let appwindow = RnoteAppWindow::new(self.instance().upcast_ref::<gtk4::Application>());
            appwindow.init(input_file);

            appwindow.show();
        }

        fn load_gresources(&self) {
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
