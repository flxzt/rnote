use cairo::glib::clone;
use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use gtk4::{subclass::prelude::ObjectSubclass, ListBoxRow};
use gtk4::{
    Align, Box, FlowBox, FlowBoxChild, IconSize, Image, Label, StringList, StringObject,
    TemplateChild, Widget,
};

use crate::GroupedIconPicker;

pub(crate) struct GroupedIconPickerGroupData {
    pub(crate) name: String,
    pub(crate) icons: StringList,
}

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/groupediconpickergroup.ui")]
    pub struct GroupedIconPickerGroup {
        pub name: RefCell<String>,
        pub icons: RefCell<StringList>,

        #[template_child]
        pub flowbox: TemplateChild<FlowBox>,
        #[template_child]
        pub name_label: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupedIconPickerGroup {
        const NAME: &'static str = "GroupedIconPickerGroup";
        type Type = super::GroupedIconPickerGroup;
        type ParentType = ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GroupedIconPickerGroup {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("name").build(),
                    glib::ParamSpecObject::builder::<StringList>("icons").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "name" => {
                    let name = value.get().unwrap();
                    self.name.replace(name);
                }
                "icons" => {
                    let icons = value.get().unwrap();
                    self.icons.replace(icons);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => self.name.borrow().to_value(),
                "icons" => self.icons.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for GroupedIconPickerGroup {}
    impl BoxImpl for GroupedIconPickerGroup {}
    impl ListBoxRowImpl for GroupedIconPickerGroup {}
}

glib::wrapper! {
    pub struct GroupedIconPickerGroup(ObjectSubclass<imp::GroupedIconPickerGroup>)
        @extends Widget, Box, ListBoxRow;
}

#[gtk4::template_callbacks]
impl GroupedIconPickerGroup {
    pub(crate) fn new(
        name: &String,
        icons: &StringList,
        grouped_icon_picker: &GroupedIconPicker,
        generate_display_name: fn(&str) -> String,
    ) -> Self {
        let widget = glib::Object::new::<Self>(&[("name", name), ("icons", icons)]);
        widget.init(grouped_icon_picker, generate_display_name);
        widget
    }

    #[allow(unused)]
    pub fn icon_list(&self) -> StringList {
        self.property::<StringList>("icons")
    }

    #[allow(unused)]
    pub fn name(&self) -> String {
        self.property::<String>("name")
    }

    fn init(
        &self,
        grouped_icon_picker: &GroupedIconPicker,
        generate_display_name: fn(&str) -> String,
    ) {
        let imp = self.imp();
        let model = self.icon_list();

        imp.name_label.set_text(self.name().as_str());

        imp.flowbox.bind_model(Some(&model), move |object| {
            let icon_name = object
                .downcast_ref::<StringObject>()
                .expect(
                    "GroupIconPickerFlowBox bind() failed, item has to be of type `StringObject`",
                )
                .string();

            let icon_image = Image::builder()
                .halign(Align::Center)
                .valign(Align::Center)
                .icon_size(IconSize::Large)
                .icon_name(icon_name.as_str())
                .tooltip_text(generate_display_name(icon_name.as_str()).as_str())
                .margin_top(3)
                .margin_bottom(3)
                .margin_start(3)
                .margin_end(3)
                .build();

            icon_image.upcast::<Widget>()
        });

        imp.flowbox.connect_child_activated(
            clone!(@weak grouped_icon_picker => move |_flowbox: &FlowBox, flowbox_child: &FlowBoxChild| {
                let child = flowbox_child.child().expect("GroupIconPickerFlowBox child_activated() failed, child has to exist");
                let icon = child.downcast_ref::<Image>().expect("GroupIconPickerFlowBox child_activated() failed, child has to be of type `Image`");
                let icon_name = icon.icon_name().expect("GroupIconPickerFlowBox child_activated() failed, child `Image` has to have an icon name");

                grouped_icon_picker.set_picked(Some(icon_name.to_string()));
            }),
        );

        // Icon has been picked, update selection and label text.
        grouped_icon_picker.connect_notify_local(
            Some("picked"),
            clone!(@weak self as group => move |grouped_icon_picker, _| {
                let flowbox = group.imp().flowbox.get();

                if let Some(picked) = grouped_icon_picker.picked() {
                    let item = group
                        .icon_list()
                        .snapshot()
                        .into_iter()
                        .map(|o| o.downcast::<StringObject>().unwrap().string())
                        .enumerate()
                        .find(|(_, s)| s == &picked);

                    if let Some((i, _)) = item {
                        // Current group contains child, select it.
                        let child = flowbox.child_at_index(i as i32).unwrap();
                        flowbox.select_child(&child);
                        grouped_icon_picker.set_selection_label_text(generate_display_name(picked.as_str()));
                    } else {
                        // Current group does not contain child, unselect all children.
                        flowbox.unselect_all();
                    }
                } else {
                    // Selection is None, unselect all children.
                    flowbox.unselect_all();
                }
            }),
        );
    }
}
