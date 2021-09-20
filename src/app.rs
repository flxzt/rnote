mod imp {
    use std::{cell::RefCell, path, rc::Rc};

    use gtk4::{gio, glib, prelude::*, subclass::prelude::*};

    use crate::{config, sheet::Sheet, ui::{
            appmenu::AppMenu, appwindow::RnoteAppWindow, canvas::Canvas, canvasmenu::CanvasMenu,
            colorpicker::colorsetter::ColorSetter, colorpicker::ColorPicker,
            mainheader::MainHeader, penssidebar::PensSideBar,
            selectionmodifier::modifiernode::ModifierNode, selectionmodifier::SelectionModifier,
            templatechooser::TemplateChooser, workspacebrowser::WorkspaceBrowser,
        }, utils};

    #[derive(Debug, Default)]
    pub struct RnoteApp {
        pub input_file: Rc<RefCell<Option<gio::File>>>,
        pub output_file: Rc<RefCell<Option<gio::File>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteApp {
        const NAME: &'static str = "RnoteApp";
        type Type = super::RnoteApp;
        type ParentType = gtk4::Application;
    }

    impl ObjectImpl for RnoteApp {}

    impl ApplicationImpl for RnoteApp {
        fn activate(&self, application: &Self::Type) {
            // Custom buildable Widgets must be initalized
            RnoteAppWindow::static_type();
            Sheet::static_type();
            Canvas::static_type();
            ColorPicker::static_type();
            ColorSetter::static_type();
            SelectionModifier::static_type();
            ModifierNode::static_type();
            CanvasMenu::static_type();
            AppMenu::static_type();
            MainHeader::static_type();
            TemplateChooser::static_type();
            PensSideBar::static_type();
            WorkspaceBrowser::static_type();

            application.set_resource_base_path(Some(config::APP_IDPATH));
            let res = gio::Resource::load(path::Path::new(config::RESOURCES_FILE))
                .expect("Could not load gresource file");
            gio::resources_register(&res);

            let appwindow = RnoteAppWindow::new(application.upcast_ref::<gtk4::Application>());
            appwindow.init();

            appwindow.show();
        }

        fn open(&self, application: &Self::Type, files: &[gio::File], _hint: &str) {
            for file in files {
                match utils::FileType::lookup_file_type(&file) {
                    utils::FileType::Unknown => {
                        log::warn!("tried to open unsupported file type");
                    },
                    _ => {
                        *self.input_file.borrow_mut() = Some(file.clone());

                    }
                };
            }
            application.activate();
        }
    }

    impl GtkApplicationImpl for RnoteApp {}
}

use std::{cell::RefCell, rc::Rc};

use gtk4::{gio, glib, subclass::prelude::*};

use crate::config;

glib::wrapper! {
    pub struct RnoteApp(ObjectSubclass<imp::RnoteApp>)
        @extends gio::Application, gtk4::Application,
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

    pub fn input_file(&self) -> Rc<RefCell<Option<gio::File>>> {
        let priv_ = imp::RnoteApp::from_instance(self);
        priv_.input_file.clone()
    }

    pub fn output_file(&self) -> Rc<RefCell<Option<gio::File>>> {
        let priv_ = imp::RnoteApp::from_instance(self);
        priv_.output_file.clone()
    }
}
