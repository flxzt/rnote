mod actions;

use crate::RnoteAppWindow;
use gtk4::{
    gdk, gio, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, DragSource,
    GestureClick, GestureLongPress, Image, Label, MenuButton, PopoverMenu, Widget,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/filerow.ui")]
    pub struct FileRow {
        pub current_file: RefCell<Option<gio::File>>,
        pub drag_source: DragSource,
        pub action_group: gio::SimpleActionGroup,

        #[template_child]
        pub file_image: TemplateChild<Image>,
        #[template_child]
        pub file_label: TemplateChild<Label>,
        #[template_child]
        pub menubutton_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub popovermenu: TemplateChild<PopoverMenu>,
    }

    impl Default for FileRow {
        fn default() -> Self {
            let drag_source = DragSource::builder()
                .name("workspacebrowser-file-drag-source")
                .actions(gdk::DragAction::COPY)
                .build();

            Self {
                action_group: gio::SimpleActionGroup::new(),
                current_file: RefCell::new(None),
                drag_source,
                file_image: TemplateChild::<Image>::default(),
                file_label: TemplateChild::<Label>::default(),
                menubutton_box: TemplateChild::<gtk4::Box>::default(),
                menubutton: TemplateChild::<MenuButton>::default(),
                popovermenu: TemplateChild::<PopoverMenu>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileRow {
        const NAME: &'static str = "FileRow";
        type Type = super::FileRow;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FileRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.set_widget_name("filerow");

            Self::setup_input(obj);
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
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "current-file" => {
                    let current_file = value
                        .get::<Option<gio::File>>()
                        .expect("The value needs to be of type `Option<gio::File>`.");
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

    impl WidgetImpl for FileRow {}

    impl FileRow {
        fn setup_input(obj: &super::FileRow) {
            obj.add_controller(&obj.imp().drag_source);

            let rightclick_gesture = GestureClick::builder()
                .name("rightclick_gesture")
                .button(gdk::BUTTON_SECONDARY)
                .build();
            obj.add_controller(&rightclick_gesture);
            rightclick_gesture.connect_pressed(
                clone!(@weak obj => move |_rightclick_gesture, _n_press, _x, _y| {
                    obj.imp().popovermenu.popup();
                }),
            );

            let longpress_gesture = GestureLongPress::builder()
                .name("longpress_gesture")
                .touch_only(true)
                .build();
            obj.add_controller(&longpress_gesture);
            longpress_gesture.group_with(&rightclick_gesture);

            longpress_gesture.connect_pressed(
                clone!(@weak obj => move |_rightclick_gesture, _x, _y| {
                    obj.imp().popovermenu.popup();
                }),
            );
        }
    }
}

glib::wrapper! {
    pub struct FileRow(ObjectSubclass<imp::FileRow>)
        @extends gtk4::Widget;
}

impl Default for FileRow {
    fn default() -> Self {
        Self::new()
    }
}

impl FileRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create `FileRow`")
    }

    pub fn current_file(&self) -> Option<gio::File> {
        self.property::<Option<gio::File>>("current-file")
    }

    pub fn set_current_file(&self, current_file: Option<gio::File>) {
        self.set_property("current-file", current_file.to_value());
    }

    pub fn action_group(&self) -> gio::SimpleActionGroup {
        self.imp().action_group.clone()
    }

    pub fn file_image(&self) -> Image {
        self.imp().file_image.clone()
    }

    pub fn file_label(&self) -> Label {
        self.imp().file_label.clone()
    }

    pub fn drag_source(&self) -> DragSource {
        self.imp().drag_source.clone()
    }

    pub fn menubutton_box(&self) -> gtk4::Box {
        self.imp().menubutton_box.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.setup_actions(appwindow);
    }

    fn setup_actions(&self, appwindow: &RnoteAppWindow) {
        self.insert_action_group("filerow", Some(&self.imp().action_group));

        self.imp()
            .action_group
            .add_action(&self.get_open_action(appwindow));
        self.imp()
            .action_group
            .add_action(&self.get_rename_action());
        self.imp().action_group.add_action(&self.get_trash_action());
    }
}
