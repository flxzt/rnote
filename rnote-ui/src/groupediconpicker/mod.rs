mod group;

// Re-exports
pub(crate) use group::GroupedIconPickerGroup;
pub(crate) use group::GroupedIconPickerGroupData;

use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Widget};
use gtk4::{Label, StringList, StringObject};
use once_cell::sync::Lazy;

mod imp {
    use std::cell::RefCell;

    use gtk4::ListBox;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/groupediconpicker/groupediconpicker.ui")]
    pub(crate) struct GroupedIconPicker {
        pub(crate) picked: RefCell<Option<String>>,

        #[template_child]
        pub(crate) listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) selection_label: TemplateChild<Label>,
    }

    impl Default for GroupedIconPicker {
        fn default() -> Self {
            Self {
                picked: RefCell::new(None),

                listbox: TemplateChild::<ListBox>::default(),
                selection_label: TemplateChild::<Label>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupedIconPicker {
        const NAME: &'static str = "GroupedIconPicker";
        type Type = super::GroupedIconPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GroupedIconPicker {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                // Since this is nullable we can use it to represent Option<String>
                vec![glib::ParamSpecString::new(
                    "picked",
                    "picked",
                    "picked",
                    None,
                    glib::ParamFlags::READWRITE,
                )]
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

    impl WidgetImpl for GroupedIconPicker {}
}

glib::wrapper! {
    pub(crate) struct GroupedIconPicker(ObjectSubclass<imp::GroupedIconPicker>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for GroupedIconPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupedIconPicker {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
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

        self.imp().listbox.get().bind_model(Some(&model), clone!(@weak self as iconpicker => @default-panic, move |object| {
            let group_name = object.downcast_ref::<StringObject>().expect(
                "IconPickerListFactory bind() failed, item has to be of type `StringObject`",
            ).string();

            let icon_names = &groups.iter().find(|x| x.name.as_str() == group_name.as_str()).unwrap().icons;

            let group = GroupedIconPickerGroup::new(&group_name.to_string(), icon_names, &iconpicker, generate_display_name);
            group.upcast::<Widget>()
        }));
    }
}
