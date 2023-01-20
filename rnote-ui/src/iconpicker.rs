use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, GridView, Widget,
};
use gtk4::{
    Align, IconSize, Image, Label, ListItem, SignalListItemFactory, SingleSelection, StringList,
    StringObject,
};
use once_cell::sync::Lazy;

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/iconpicker.ui")]
    pub(crate) struct IconPicker {
        pub(crate) list: RefCell<Option<StringList>>,
        pub(crate) selection: RefCell<Option<SingleSelection>>,
        pub(crate) selected_handlerid: RefCell<Option<glib::SignalHandlerId>>,
        pub(crate) picked: RefCell<Option<String>>,

        #[template_child]
        pub(crate) gridview: TemplateChild<GridView>,
        #[template_child]
        pub(crate) selection_label: TemplateChild<Label>,
    }

    impl Default for IconPicker {
        fn default() -> Self {
            Self {
                list: RefCell::new(None),
                selection: RefCell::new(None),
                selected_handlerid: RefCell::new(None),
                picked: RefCell::new(None),

                gridview: TemplateChild::<GridView>::default(),
                selection_label: TemplateChild::<Label>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconPicker {
        const NAME: &'static str = "IconPicker";
        type Type = super::IconPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for IconPicker {
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

                    self.instance().set_picked_intern(picked.clone());
                    self.picked.replace(picked);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for IconPicker {}
}

glib::wrapper! {
    pub(crate) struct IconPicker(ObjectSubclass<imp::IconPicker>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for IconPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl IconPicker {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn list(&self) -> Option<StringList> {
        self.imp().list.borrow().clone()
    }

    #[allow(unused)]
    pub(crate) fn picked(&self) -> Option<String> {
        self.property::<Option<String>>("picked")
    }

    #[allow(unused)]
    pub(crate) fn set_picked(&self, picked: Option<String>) {
        self.set_property("picked", picked.to_value());
    }

    fn set_selection_label_text(&self, text: String) {
        self.imp().selection_label.get().set_text(text.as_str());
    }

    /// Internal function to retrieve the picked icon from the selection
    fn picked_intern(&self) -> Option<String> {
        self.imp()
            .selection
            .borrow()
            .as_ref()
            .and_then(|selection| {
                let selected = selection.selected();

                self.list()
                    .as_ref()
                    .and_then(|l| l.string(selected).map(|s| s.to_string()))
            })
    }

    /// Internal function to set the picked icon
    fn set_picked_intern(&self, picked: Option<String>) {
        if let (Some(selection), Some(list)) =
            (&*self.imp().selection.borrow(), &*self.imp().list.borrow())
        {
            if let Some(picked) = picked {
                let item = list
                    .snapshot()
                    .into_iter()
                    .map(|o| o.downcast::<StringObject>().unwrap().string())
                    .enumerate()
                    .find(|(_, s)| s == &picked);

                if let Some((i, _)) = item {
                    selection.set_selected(i as u32);
                } else {
                    selection.set_selected(gtk4::INVALID_LIST_POSITION);
                }
            } else {
                selection.set_selected(gtk4::INVALID_LIST_POSITION);
            }
        }
    }

    /// Binds a list containing the icon names, using a function that returns the i18n string for a given icon name
    pub(crate) fn set_list_localized(&self, list: StringList, localize: fn(&str) -> String) {
        let single_selection = SingleSelection::builder()
            .model(&list)
            // Ensures nothing is selected when initially setting the list
            .selected(gtk4::INVALID_LIST_POSITION)
            .can_unselect(true)
            .build();

        if let Some(old_id) = self.imp().selected_handlerid.borrow_mut().take() {
            self.disconnect(old_id);
        }

        self.imp().selected_handlerid.borrow_mut().replace(
            single_selection.connect_selected_item_notify(
                clone!(@weak self as iconpicker => move |_| {
                    let pick = iconpicker.picked_intern();

                    if let Some(icon_name) = &pick {
                        iconpicker.set_selection_label_text(localize(icon_name.as_str()));
                    }

                    iconpicker.set_picked(pick);
                }),
            ),
        );

        // Factory
        let icon_factory = SignalListItemFactory::new();
        icon_factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let string = list_item
                .item()
                .expect("IconPickerListFactory bind() failed, item is None")
                .downcast_ref::<StringObject>()
                .expect(
                    "IconPickerListFactory bind() failed, item has to be of type `StringObject`",
                )
                .string();

            let icon_image = Image::builder()
                .halign(Align::Center)
                .valign(Align::Center)
                .icon_size(IconSize::Large)
                .icon_name(string.as_str())
                .tooltip_text(localize(string.as_str()).as_str())
                .margin_top(3)
                .margin_bottom(3)
                .margin_start(3)
                .margin_end(3)
                .build();

            list_item.set_child(Some(&icon_image));
        });

        self.imp().selection_label.get().set_visible(true);
        self.imp().gridview.get().set_model(Some(&single_selection));
        self.imp().gridview.get().set_factory(Some(&icon_factory));
        self.imp().list.borrow_mut().replace(list);
        self.imp().selection.borrow_mut().replace(single_selection);
    }

    /// Binds a list containing the icon names
    pub(crate) fn set_list(&self, list: StringList) {
        let single_selection = SingleSelection::builder()
            .model(&list)
            // Ensures nothing is selected when initially setting the list
            .selected(gtk4::INVALID_LIST_POSITION)
            .can_unselect(true)
            .build();

        if let Some(old_id) = self.imp().selected_handlerid.borrow_mut().take() {
            self.disconnect(old_id);
        }

        self.imp().selected_handlerid.borrow_mut().replace(
            single_selection.connect_selected_item_notify(
                clone!(@weak self as iconpicker => move |_| {
                    iconpicker.set_picked(iconpicker.picked_intern());
                }),
            ),
        );

        // Factory
        let icon_factory = SignalListItemFactory::new();
        icon_factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let string = list_item
                .item()
                .expect("IconPickerListFactory bind() failed, item is None")
                .downcast_ref::<StringObject>()
                .expect(
                    "IconPickerListFactory bind() failed, item has to be of type `StringObject`",
                )
                .string();

            let icon_image = Image::builder()
                .halign(Align::Center)
                .valign(Align::Center)
                .icon_size(IconSize::Large)
                .icon_name(string.as_str())
                .margin_top(3)
                .margin_bottom(3)
                .margin_start(3)
                .margin_end(3)
                .build();

            list_item.set_child(Some(&icon_image));
        });

        self.imp().selection_label.get().set_visible(false);
        self.imp().gridview.get().set_model(Some(&single_selection));
        self.imp().gridview.get().set_factory(Some(&icon_factory));
        self.imp().list.borrow_mut().replace(list);
        self.imp().selection.borrow_mut().replace(single_selection);
    }
}
