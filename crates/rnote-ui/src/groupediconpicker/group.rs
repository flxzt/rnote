// Imports
use crate::RnGroupedIconPicker;
use cairo::glib::clone;
use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use gtk4::{subclass::prelude::ObjectSubclass, ListBoxRow};
use gtk4::{
    Align, Box, FlowBox, FlowBoxChild, IconSize, Image, Label, StringList, StringObject,
    TemplateChild, Widget,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;

pub(crate) struct GroupedIconPickerGroupData {
    pub(crate) name: String,
    pub(crate) icons: StringList,
}

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/groupediconpicker/groupediconpickergroup.ui")]
    pub(crate) struct RnGroupedIconPickerGroup {
        pub(crate) name: RefCell<String>,
        pub(crate) icons: RefCell<StringList>,

        #[template_child]
        pub(crate) flowbox: TemplateChild<FlowBox>,
        #[template_child]
        pub(crate) name_label: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnGroupedIconPickerGroup {
        const NAME: &'static str = "RnGroupedIconPickerGroup";
        type Type = super::RnGroupedIconPickerGroup;
        type ParentType = ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnGroupedIconPickerGroup {
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
    impl WidgetImpl for RnGroupedIconPickerGroup {}
    impl BoxImpl for RnGroupedIconPickerGroup {}
    impl ListBoxRowImpl for RnGroupedIconPickerGroup {}
}

glib::wrapper! {
    pub(crate) struct RnGroupedIconPickerGroup(ObjectSubclass<imp::RnGroupedIconPickerGroup>)
        @extends Widget, Box, ListBoxRow;
}

#[gtk4::template_callbacks]
impl RnGroupedIconPickerGroup {
    pub(crate) fn new(
        name: &String,
        icons: &StringList,
        grouped_icon_picker: &RnGroupedIconPicker,
        generate_display_name: fn(&str) -> String,
    ) -> Self {
        let widget = glib::Object::builder::<Self>()
            .property("name", name)
            .property("icons", icons)
            .build();
        widget.init(grouped_icon_picker, generate_display_name);
        widget
    }

    #[allow(unused)]
    pub(crate) fn icon_list(&self) -> StringList {
        self.property::<StringList>("icons")
    }

    #[allow(unused)]
    pub(crate) fn name(&self) -> String {
        self.property::<String>("name")
    }

    fn init(
        &self,
        grouped_icon_picker: &RnGroupedIconPicker,
        generate_display_name: fn(&str) -> String,
    ) {
        let imp = self.imp();
        let model = self.icon_list();

        imp.name_label.set_text(self.name().as_str());

        imp.flowbox.bind_model(Some(&model), move |object| {
            let icon_name = object
                .downcast_ref::<StringObject>()
                .expect(
                    "Binding GroupIconPickerFlowBox model failed, item has to be of type `StringObject`",
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
                let icon_name = flowbox_child
                    .child()
                    .expect("GroupIconPickerFlowBox child activated signal callback failed, child has to exist")
                    .downcast_ref::<Image>()
                    .expect("GroupIconPickerFlowBox child activated signal callback failed, child has to be of type `Image`")
                    .icon_name()
                    .expect("GroupIconPickerFlowBox child activated signal callback failed, child `Image` has to have an icon name");

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
