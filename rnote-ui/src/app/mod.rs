mod appactions;

use std::{cell::RefCell, path};

use adw::subclass::prelude::AdwApplicationImpl;
use gtk4::{gio, glib, prelude::*, subclass::prelude::*};
use rnote_engine::pens::penholder::PenStyle;
use rnote_engine::{
    sheet::format::MeasureUnit,
    sheet::{background::PatternStyle, format::PredefinedFormat},
};

use crate::{
    colorpicker::ColorSetter, config, penssidebar::BrushPage, penssidebar::EraserPage,
    penssidebar::SelectorPage, penssidebar::ShaperPage, penssidebar::ToolsPage,
    settingspanel::PenShortcutRow, utils, workspacebrowser::FileRow, AppMenu, CanvasMenu,
    ColorPicker, MainHeader, PensSideBar, RnoteAppWindow, RnoteCanvas,
    SettingsPanel, UnitEntry, WorkspaceBrowser,
};

mod imp {
    use super::*;
    #[allow(missing_debug_implementations)]
    pub struct RnoteApp {
        pub input_file: RefCell<Option<gio::File>>,
    }

    impl Default for RnoteApp {
        fn default() -> Self {
            Self {
                input_file: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteApp {
        const NAME: &'static str = "RnoteApp";
        type Type = super::RnoteApp;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for RnoteApp {}

    impl ApplicationImpl for RnoteApp {
        fn activate(&self, app: &Self::Type) {
            // Custom buildable Widgets need to register
            RnoteAppWindow::static_type();
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
            ToolsPage::static_type();
            PenStyle::static_type();
            WorkspaceBrowser::static_type();
            FileRow::static_type();
            PredefinedFormat::static_type();
            MeasureUnit::static_type();
            PatternStyle::static_type();
            UnitEntry::static_type();
            PenShortcutRow::static_type();

            // Load the resources
            app.set_resource_base_path(Some(config::APP_IDPATH));
            let resource = gio::Resource::load(path::Path::new(config::RESOURCES_FILE))
                .expect("Could not load gresource file");
            gio::resources_register(&resource);

            let appwindow = RnoteAppWindow::new(app.upcast_ref::<gtk4::Application>());
            appwindow.init();

            // setup the app
            app.setup_actions();
            app.setup_action_accels();

            // Everything else before showing
            app.init_misc(&appwindow);

            appwindow.show();
        }

        fn open(&self, application: &Self::Type, files: &[gio::File], _hint: &str) {
            for file in files {
                match utils::FileType::lookup_file_type(file) {
                    utils::FileType::Unsupported => {
                        log::warn!("tried to open unsupported file type");
                    }
                    _ => {
                        *self.input_file.borrow_mut() = Some(file.clone());
                    }
                };
            }
            application.activate();
        }
    }

    impl GtkApplicationImpl for RnoteApp {}
    impl AdwApplicationImpl for RnoteApp {}
}

glib::wrapper! {
    pub struct RnoteApp(ObjectSubclass<imp::RnoteApp>)
        @extends gio::Application, gtk4::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for RnoteApp {
    fn default() -> Self {
        Self::new()
    }
}

impl RnoteApp {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &config::APP_ID),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
        ])
        .expect("failed to create RnoteApp")
    }

    pub fn input_file(&self) -> Option<gio::File> {
        self.imp().input_file.borrow().clone()
    }

    pub fn set_input_file(&self, input_file: Option<gio::File>) {
        *self.imp().input_file.borrow_mut() = input_file;
    }

    // Anything that needs to be done right before showing the appwindow
    pub fn init_misc(&self, appwindow: &RnoteAppWindow) {
        appwindow.canvas().return_to_origin_page();
        appwindow.canvas().regenerate_background(false);
        appwindow.canvas().regenerate_content(true, true);
    }
}
