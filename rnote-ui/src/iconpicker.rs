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
    pub struct IconPicker {
        pub list: RefCell<Option<StringList>>,
        pub selection: RefCell<Option<SingleSelection>>,
        pub selected_handlerid: RefCell<Option<glib::SignalHandlerId>>,

        #[template_child]
        pub gridview: TemplateChild<GridView>,
    }

    impl Default for IconPicker {
        fn default() -> Self {
            Self {
                list: RefCell::new(None),
                selection: RefCell::new(None),
                selected_handlerid: RefCell::new(None),

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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![glib::subclass::Signal::builder(
                    "icon-picked",
                    // Emits the icon name string
                    &[String::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for IconPicker {}
}

glib::wrapper! {
    pub struct IconPicker(ObjectSubclass<imp::IconPicker>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for IconPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl IconPicker {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create `IconPicker`")
    }

    pub fn list(&self) -> Option<StringList> {
        self.imp().list.borrow().clone()
    }

    pub fn picked(&self) -> Option<String> {
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

    /// Binds a list containing the icon names
    pub fn set_list(&self, list: StringList) {
        let single_selection = SingleSelection::builder()
            .model(&list)
            // Ensures nothing is selected when initially setting the list
            .selected(gtk4::INVALID_LIST_POSITION)
            .build();

        if let Some(old_id) = self.imp().selected_handlerid.borrow_mut().take() {
            self.disconnect(old_id);
        }

        self.imp().selected_handlerid.borrow_mut().replace(
            single_selection.connect_selected_item_notify(
                clone!(@weak self as iconpicker => move |_| {
                    if let Some(picked_str) = iconpicker.picked() {
                        iconpicker.emit_by_name::<()>("icon-picked", &[&picked_str]);
                    }
                }),
            ),
        );

        // Factory
        let icon_factory = SignalListItemFactory::new();
        icon_factory.connect_setup(move |_factory, list_item| {
            let icon_image = Image::builder()
                .halign(Align::Center)
                .valign(Align::Center)
                .icon_size(IconSize::Large)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(6)
                .margin_end(6)
                .build();

            list_item.set_child(Some(&icon_image));

            let list_item_expr = ConstantExpression::new(list_item);
            let string_expr =
                PropertyExpression::new(ListItem::static_type(), Some(&list_item_expr), "item")
                    .chain_property::<StringObject>("string");

            string_expr.bind(&icon_image, "icon-name", Widget::NONE);
        });

        self.imp().gridview.get().set_model(Some(&single_selection));
        self.imp().gridview.get().set_factory(Some(&icon_factory));
        self.imp().list.borrow_mut().replace(list);
        self.imp().selection.borrow_mut().replace(single_selection);
    }
}
