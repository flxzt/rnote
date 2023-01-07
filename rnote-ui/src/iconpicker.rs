use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, GridView, Widget,
};
use gtk4::{
    Align, ConstantExpression, IconSize, Image, ListItem, PropertyExpression,
    SignalListItemFactory, SingleSelection, StringList, StringObject,
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
    }

    impl Default for IconPicker {
        fn default() -> Self {
            Self {
                list: RefCell::new(None),
                selection: RefCell::new(None),
                selected_handlerid: RefCell::new(None),
                picked: RefCell::new(None),

                gridview: TemplateChild::<GridView>::default(),
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

    /// Internal function to retreive the picked icon from the selection
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
        icon_factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let icon_image = Image::builder()
                .halign(Align::Center)
                .valign(Align::Center)
                .icon_size(IconSize::Large)
                .margin_top(3)
                .margin_bottom(3)
                .margin_start(3)
                .margin_end(3)
                .build();

            list_item.set_child(Some(&icon_image));

            let list_item_expr = ConstantExpression::new(&list_item);
            let string_expr =
                PropertyExpression::new(ListItem::static_type(), Some(&list_item_expr), "item")
                    .chain_property::<StringObject>("string");

            string_expr.bind(&icon_image, "icon-name", Widget::NONE);
            string_expr.bind(&icon_image, "tooltip-text", Widget::NONE);
        });

        self.imp().gridview.get().set_model(Some(&single_selection));
        self.imp().gridview.get().set_factory(Some(&icon_factory));
        self.imp().list.borrow_mut().replace(list);
        self.imp().selection.borrow_mut().replace(single_selection);
    }
}
