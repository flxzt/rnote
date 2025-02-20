// Modules
mod group;

// Re-exports
pub(crate) use group::GroupedIconPickerGroupData;
pub(crate) use group::RnGroupedIconPickerGroup;

// Imports
use gtk4::{
    CompositeTemplate, Label, ListBox, StringList, StringObject, Widget, glib, glib::clone,
    prelude::*, subclass::prelude::*,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/groupediconpicker/groupediconpicker.ui")]
    pub(crate) struct RnGroupedIconPicker {
        pub(crate) picked: RefCell<Option<String>>,

        #[template_child]
        pub(crate) listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) selection_label: TemplateChild<Label>,
    }

    impl Default for RnGroupedIconPicker {
        fn default() -> Self {
            Self {
                picked: RefCell::new(None),

                listbox: TemplateChild::<ListBox>::default(),
                selection_label: TemplateChild::<Label>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnGroupedIconPicker {
        const NAME: &'static str = "RnGroupedIconPicker";
        type Type = super::RnGroupedIconPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnGroupedIconPicker {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                // we can use it to represent Option<String>
                vec![
                    glib::ParamSpecString::builder("picked")
                        .default_value(None)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "picked" => self.picked.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "picked" => {
                    let picked = value
                        .get::<Option<String>>()
                        .expect("The value needs to be of type `Option<String>`");

                    self.picked.replace(picked);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnGroupedIconPicker {}
}

glib::wrapper! {
    pub(crate) struct RnGroupedIconPicker(ObjectSubclass<imp::RnGroupedIconPicker>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnGroupedIconPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl RnGroupedIconPicker {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn picked(&self) -> Option<String> {
        self.property::<Option<String>>("picked")
    }

    #[allow(unused)]
    pub(crate) fn set_picked(&self, picked: Option<String>) {
        self.set_property("picked", picked.to_value());
    }

    pub(crate) fn set_selection_label_text(&self, text: String) {
        self.imp().selection_label.get().set_text(text.as_str());
    }

    pub(crate) fn set_groups(
        &self,
        groups: Vec<GroupedIconPickerGroupData>,
        generate_display_name: fn(&str) -> String,
    ) {
        let model = StringList::from_iter(groups.iter().map(|x| x.name.clone()));

        self.imp().listbox.get().bind_model(
            Some(&model),
            clone!(
                #[weak(rename_to=iconpicker)]
                self,
                #[upgrade_or_panic]
                move |obj| {
                    let group_name = obj.downcast_ref::<StringObject>().expect(
                "Binding IconPickerListFactory model failed, item has to be of type `StringObject`",
            ).string();
                    let icon_names = &groups
                        .iter()
                        .find(|x| x.name.as_str() == group_name.as_str())
                        .unwrap()
                        .icons;
                    let group = RnGroupedIconPickerGroup::new(
                        &group_name.to_string(),
                        icon_names,
                        &iconpicker,
                        generate_display_name,
                    );
                    group.upcast::<Widget>()
                }
            ),
        );
    }
}
