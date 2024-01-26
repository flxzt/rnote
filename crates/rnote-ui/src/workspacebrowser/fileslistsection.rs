// Imports
use gtk4::{gio, glib, prelude::*, subclass::prelude::*, SortListModel};
use once_cell::sync::Lazy;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct RnFilesListSection {
        name: RefCell<String>,
        model: RefCell<SortListModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnFilesListSection {
        const NAME: &'static str = "RnFilesListSection";
        type Type = super::RnFilesListSection;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for RnFilesListSection {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("name").build(),
                    glib::ParamSpecObject::builder::<SortListModel>("model").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "name" => {
                    let name = value
                        .get::<String>()
                        .expect("The value needs to be of type `String`");
                    self.name.replace(name);
                }
                "model" => {
                    let model = value
                        .get::<SortListModel>()
                        .expect("The value needs to be of type `SortListModel");
                    self.model.replace(model);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => self.name.borrow().to_value(),
                "model" => self.model.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for RnFilesListSection {
        fn item_type(&self) -> glib::Type {
            self.model.borrow().item_type()
        }

        fn n_items(&self) -> u32 {
            self.model.borrow().n_items()
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.model.borrow().item(position)
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnFilesListSection(ObjectSubclass<imp::RnFilesListSection>)
        @implements gio::ListModel;
}

impl Default for RnFilesListSection {
    fn default() -> Self {
        Self::new()
    }
}

impl RnFilesListSection {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn from_name_and_model(name: &str, model: SortListModel) -> Self {
        let obj = Self::new();
        obj.set_name(name);
        obj.set_model(model);
        obj
    }

    #[allow(unused)]
    pub(crate) fn name(&self) -> String {
        self.property::<String>("name")
    }

    #[allow(unused)]
    pub(crate) fn set_name(&self, name: &str) {
        self.set_property("name", name);
    }

    #[allow(unused)]
    pub(crate) fn model(&self) -> SortListModel {
        self.property::<SortListModel>("model")
    }

    #[allow(unused)]
    pub(crate) fn set_model(&self, model: SortListModel) {
        self.set_property("model", model);
    }
}
