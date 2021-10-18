mod imp {
    use std::{
        cell::{Cell, RefCell},
        path,
        rc::Rc,
    };

    use adw::subclass::prelude::AdwApplicationImpl;
    use gtk4::{gio, glib, prelude::*, subclass::prelude::*, IconTheme};
    use once_cell::sync::Lazy;

    use crate::{
        config,
        sheet::format::MeasureUnit,
        sheet::format::PredefinedFormat,
        sheet::Sheet,
        ui::{
            appmenu::AppMenu, appwindow::RnoteAppWindow, canvas::Canvas, canvasmenu::CanvasMenu,
            colorpicker::colorsetter::ColorSetter, colorpicker::ColorPicker,
            develactions::DevelActions, mainheader::MainHeader, penssidebar::brushpage::BrushPage,
            penssidebar::eraserpage::EraserPage, penssidebar::markerpage::MarkerPage,
            penssidebar::selectorpage::SelectorPage,
            penssidebar::shaperpage::ShaperPage, penssidebar::PensSideBar,
            selectionmodifier::modifiernode::ModifierNode, selectionmodifier::SelectionModifier,
            settingspanel::SettingsPanel, templatechooser::TemplateChooser,
            workspacebrowser::WorkspaceBrowser,
        },
        utils,
    };

    #[derive(Debug, Default)]
    pub struct RnoteApp {
        pub input_file: Rc<RefCell<Option<gio::File>>>,
        pub output_file: Rc<RefCell<Option<gio::File>>>,
        pub unsaved_changes: Cell<bool>,
        pub rng: Rc<RefCell<rand::rngs::ThreadRng>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteApp {
        const NAME: &'static str = "RnoteApp";
        type Type = super::RnoteApp;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for RnoteApp {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_boolean(
                    "unsaved-changes",
                    "unsaved-changes",
                    "unsaved-changes",
                    false,
                    glib::ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "unsaved-changes" => {
                    let unsaved_changes: bool =
                        value.get().expect("The value needs to be of type `bool`.");
                    self.unsaved_changes.replace(unsaved_changes);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl ApplicationImpl for RnoteApp {
        fn activate(&self, application: &Self::Type) {
            // Custom buildable Widgets must be initalized
            RnoteAppWindow::static_type();
            DevelActions::static_type();
            Sheet::static_type();
            Canvas::static_type();
            ColorPicker::static_type();
            ColorSetter::static_type();
            SelectionModifier::static_type();
            ModifierNode::static_type();
            CanvasMenu::static_type();
            SettingsPanel::static_type();
            AppMenu::static_type();
            MainHeader::static_type();
            TemplateChooser::static_type();
            PensSideBar::static_type();
            MarkerPage::static_type();
            BrushPage::static_type();
            ShaperPage::static_type();
            EraserPage::static_type();
            SelectorPage::static_type();
            WorkspaceBrowser::static_type();
            PredefinedFormat::static_type();
            MeasureUnit::static_type();

            application.set_resource_base_path(Some(config::APP_IDPATH));
            let res = gio::Resource::load(path::Path::new(config::RESOURCES_FILE))
                .expect("Could not load gresource file");
            gio::resources_register(&res);

            let appwindow = RnoteAppWindow::new(application.upcast_ref::<gtk4::Application>());
            appwindow.init();

            // add icon theme resource path because automatic lookup does not work in Devel build.
            let app_icon_theme = IconTheme::for_display(&appwindow.display())
                .expect("failed to get icon theme for appwindow");
            app_icon_theme.add_resource_path((String::from(config::APP_IDPATH) + "icons").as_str());

            appwindow.show();
        }

        fn open(&self, application: &Self::Type, files: &[gio::File], _hint: &str) {
            for file in files {
                match utils::FileType::lookup_file_type(file) {
                    utils::FileType::Unknown => {
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

use std::{cell::RefCell, rc::Rc};

use gtk4::{gio, glib, prelude::*, subclass::prelude::*};

use crate::config;

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

    pub fn input_file(&self) -> Rc<RefCell<Option<gio::File>>> {
        let priv_ = imp::RnoteApp::from_instance(self);
        priv_.input_file.clone()
    }

    pub fn output_file(&self) -> Rc<RefCell<Option<gio::File>>> {
        let priv_ = imp::RnoteApp::from_instance(self);
        priv_.output_file.clone()
    }

    pub fn rng(&self) -> Rc<RefCell<rand::rngs::ThreadRng>> {
        let priv_ = imp::RnoteApp::from_instance(self);
        priv_.rng.clone()
    }

    pub fn unsaved_changes(&self) -> bool {
        self.property("unsaved-changes")
            .unwrap()
            .get::<bool>()
            .unwrap()
    }

    pub fn set_unsaved_changes(&self, unsaved_changes: bool) {
        match self.set_property("unsaved-changes", unsaved_changes.to_value()) {
            Ok(_) => {}
            Err(e) => {
                log::error!("failed to set property `unsaved-changes` of `App`, {}", e)
            }
        }
    }
}
