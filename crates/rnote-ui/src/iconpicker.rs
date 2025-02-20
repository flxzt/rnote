// Imports
use cairo::glib::closure;
use gtk4::{
    Align, CompositeTemplate, ConstantExpression, GridView, IconSize, Image, Label, ListItem,
    PropertyExpression, SignalListItemFactory, SingleSelection, StringList, StringObject, Widget,
    glib, glib::clone, prelude::*, subclass::prelude::*,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/iconpicker.ui")]
    pub(crate) struct RnIconPicker {
        pub(crate) list: RefCell<Option<StringList>>,
        pub(crate) selection: RefCell<Option<SingleSelection>>,
        pub(crate) selected_handlerid: RefCell<Option<glib::SignalHandlerId>>,
        pub(crate) picked: RefCell<Option<String>>,

        #[template_child]
        pub(crate) gridview: TemplateChild<GridView>,
        #[template_child]
        pub(crate) selection_label: TemplateChild<Label>,
    }

    impl Default for RnIconPicker {
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
    impl ObjectSubclass for RnIconPicker {
        const NAME: &'static str = "RnIconPicker";
        type Type = super::RnIconPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnIconPicker {
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

                    self.obj().set_picked_intern(picked.clone());
                    self.picked.replace(picked);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnIconPicker {}
}

glib::wrapper! {
    pub(crate) struct RnIconPicker(ObjectSubclass<imp::RnIconPicker>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnIconPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl RnIconPicker {
    pub(crate) fn new() -> Self {
        glib::Object::new()
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

    #[allow(unused)]
    fn set_selection_label_text(&self, text: String) {
        self.imp().selection_label.get().set_text(text.as_str());
    }

    #[allow(unused)]
    fn set_selection_label_visible(&self, visible: bool) {
        self.imp().selection_label.get().set_visible(visible);
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

    /// Binds a list containing the icon names, optionally using a function that returns the i18n string for a given icon name
    pub(crate) fn set_list(
        &self,
        list: StringList,
        generate_display_name_option: Option<fn(&str) -> String>,
        show_selection_label: bool,
    ) {
        let single_selection = SingleSelection::builder()
            .model(&list)
            // Ensures nothing is selected when initially setting the list
            .selected(gtk4::INVALID_LIST_POSITION)
            .can_unselect(true)
            .build();

        if let Some(old_id) = self.imp().selected_handlerid.borrow_mut().take() {
            self.disconnect(old_id);
        }

        let show_display_name = generate_display_name_option.is_some();
        let generate_display_name = generate_display_name_option.unwrap_or(|_| String::new());

        self.imp().selected_handlerid.borrow_mut().replace(
            single_selection.connect_selected_item_notify(clone!(
                #[weak(rename_to=iconpicker)]
                self,
                move |_| {
                    let pick = iconpicker.picked_intern();

                    if show_display_name && show_selection_label {
                        if let Some(icon_name) = &pick {
                            iconpicker.set_selection_label_visible(true);
                            iconpicker.set_selection_label_text(generate_display_name(
                                icon_name.as_str(),
                            ));
                        } else {
                            iconpicker.set_selection_label_visible(false);
                        }
                    }

                    iconpicker.set_picked(pick);
                }
            )),
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

            let list_item_expr = ConstantExpression::new(list_item);

            let icon_name_expr =
                PropertyExpression::new(ListItem::static_type(), Some(&list_item_expr), "item")
                    .chain_property::<StringObject>("string");

            icon_name_expr.bind(&icon_image, "icon-name", Widget::NONE);

            if show_display_name {
                let tooltip_expr = icon_name_expr.chain_closure::<String>(closure!(
                    |_: Option<glib::Object>, icon_name: &str| generate_display_name(icon_name)
                ));

                tooltip_expr.bind(&icon_image, "tooltip-text", Widget::NONE);
            }
        });

        self.imp().gridview.get().set_model(Some(&single_selection));
        self.imp().gridview.get().set_factory(Some(&icon_factory));
        self.imp().list.borrow_mut().replace(list);
        self.imp().selection.borrow_mut().replace(single_selection);
    }
}
