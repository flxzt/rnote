use crate::RnoteAppWindow;
use gtk4::{
    gdk, gio, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, GestureClick,
    GestureLongPress, Image, Widget,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacerow.ui")]
    pub struct WorkspaceRow {
        pub current_file: RefCell<Option<gio::File>>,

        #[template_child]
        pub folder_image: TemplateChild<Image>,
    }

    impl Default for WorkspaceRow {
        fn default() -> Self {
            Self {
                current_file: RefCell::new(None),
                folder_image: TemplateChild::<Image>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WorkspaceRow {
        const NAME: &'static str = "WorkspaceRow";
        type Type = super::WorkspaceRow;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WorkspaceRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.set_widget_name("filerow");
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "current-file",
                    "current-file",
                    "current-file",
                    Option::<gio::File>::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "current-file" => {
                    let current_file = value
                        .get::<Option<gio::File>>()
                        .expect("The value needs to be of type `Option<gio::File>`.");

                    // Set the tooltip text to the current path
                    let s = current_file
                        .as_ref()
                        .and_then(|f| f.path().map(|p| p.to_string_lossy().to_string()));
                    obj.set_tooltip_text(s.as_ref().map(|s| s.as_str()));

                    self.current_file.replace(current_file);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "current-file" => self.current_file.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for WorkspaceRow {}
}

glib::wrapper! {
    pub struct WorkspaceRow(ObjectSubclass<imp::WorkspaceRow>)
        @extends gtk4::Widget;
}

impl Default for WorkspaceRow {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create `WorkspaceRow`")
    }

    pub fn from_file(file: &gio::File) -> Self {
        glib::Object::new(&[("current-file", &file.to_value())])
            .expect("Failed to create `WorkspaceRow` from file")
    }

    pub fn current_file(&self) -> Option<gio::File> {
        self.property::<Option<gio::File>>("current-file")
    }

    pub fn set_current_file(&self, current_file: Option<gio::File>) {
        self.set_property("current-file", current_file.to_value());
    }

    pub fn folder_image(&self) -> Image {
        self.imp().folder_image.clone()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let rightclick_gesture = GestureClick::builder()
            .name("rightclick_gesture")
            .button(gdk::BUTTON_SECONDARY)
            .build();
        self.add_controller(&rightclick_gesture);
        rightclick_gesture.connect_pressed(
            clone!(@weak appwindow => move |_rightclick_gesture, _n_press, _x, _y| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "edit-workspace", None);
            }),
        );

        let longpress_gesture = GestureLongPress::builder()
            .name("longpress_gesture")
            .touch_only(true)
            .build();
        self.add_controller(&longpress_gesture);
        longpress_gesture.group_with(&rightclick_gesture);

        longpress_gesture.connect_pressed(
            clone!(@weak appwindow => move |_rightclick_gesture, _x, _y| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "edit-workspace", None);
            }),
        );
    }
}
