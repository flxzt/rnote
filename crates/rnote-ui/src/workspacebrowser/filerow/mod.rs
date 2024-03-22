// Modules
mod actions;

// Imports
use crate::RnAppWindow;
use gtk4::{
    gdk, gio, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, DragSource,
    GestureClick, GestureLongPress, Image, Label, MenuButton, PopoverMenu, Widget,
};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/filerow.ui")]
    pub(crate) struct RnFileRow {
        pub(crate) current_file: RefCell<Option<gio::File>>,
        /// The position in the list.
        pub(crate) position: Cell<u32>,
        pub(crate) drag_source: DragSource,
        pub(crate) action_group: gio::SimpleActionGroup,

        #[template_child]
        pub(crate) file_image: TemplateChild<Image>,
        #[template_child]
        pub(crate) file_label: TemplateChild<Label>,
        #[template_child]
        pub(crate) menubutton_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) popovermenu: TemplateChild<PopoverMenu>,
    }

    impl Default for RnFileRow {
        fn default() -> Self {
            let drag_source = DragSource::builder()
                .name("workspacebrowser-file-drag-source")
                .actions(gdk::DragAction::COPY)
                .build();

            Self {
                action_group: gio::SimpleActionGroup::new(),
                position: Cell::new(0),
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
    impl ObjectSubclass for RnFileRow {
        const NAME: &'static str = "RnFileRow";
        type Type = super::RnFileRow;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnFileRow {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_widget_name("filerow");

            self.setup_input();
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
                    // this is nullable, so it can be used to represent Option<gio::File>
                    glib::ParamSpecObject::builder::<gio::File>("current-file").build(),
                    glib::ParamSpecUInt::builder("position").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "current-file" => {
                    let current_file = value
                        .get::<Option<gio::File>>()
                        .expect("The value needs to be of type `Option<gio::File>`");
                    self.current_file.replace(current_file);
                }
                "position" => {
                    let position = value
                        .get::<u32>()
                        .expect("The value needs to be of type `u32`");
                    self.position.set(position);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "current-file" => self.current_file.borrow().to_value(),
                "position" => self.position.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnFileRow {}

    impl RnFileRow {
        fn setup_input(&self) {
            let obj = self.obj();
            obj.add_controller(self.drag_source.clone());

            let rightclick_gesture = GestureClick::builder()
                .name("rightclick_gesture")
                .button(gdk::BUTTON_SECONDARY)
                .build();
            obj.add_controller(rightclick_gesture.clone());
            rightclick_gesture.connect_pressed(
                clone!(@weak obj as filerow => move |_rightclick_gesture, _n_press, _x, _y| {
                    filerow.imp().popovermenu.popup();
                }),
            );

            let longpress_gesture = GestureLongPress::builder()
                .name("longpress_gesture")
                .touch_only(true)
                .build();
            obj.add_controller(longpress_gesture.clone());
            longpress_gesture.group_with(&rightclick_gesture);

            longpress_gesture.connect_pressed(
                clone!(@weak obj as filerow => move |_rightclick_gesture, _x, _y| {
                    filerow.imp().popovermenu.popup();
                }),
            );
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnFileRow(ObjectSubclass<imp::RnFileRow>)
        @extends gtk4::Widget;
}

impl Default for RnFileRow {
    fn default() -> Self {
        Self::new()
    }
}

impl RnFileRow {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn current_file(&self) -> Option<gio::File> {
        self.property::<Option<gio::File>>("current-file")
    }

    #[allow(unused)]
    pub(crate) fn set_current_file(&self, current_file: Option<gio::File>) {
        self.set_property("current-file", current_file.to_value());
    }

    #[allow(unused)]
    pub(crate) fn position(&self) -> u32 {
        self.property::<u32>("position")
    }

    #[allow(unused)]
    pub(crate) fn set_position(&self, position: u32) {
        self.set_property("position", position);
    }

    pub(crate) fn file_image(&self) -> Image {
        self.imp().file_image.clone()
    }

    pub(crate) fn file_label(&self) -> Label {
        self.imp().file_label.clone()
    }

    pub(crate) fn drag_source(&self) -> DragSource {
        self.imp().drag_source.clone()
    }

    pub(crate) fn menubutton_box(&self) -> gtk4::Box {
        self.imp().menubutton_box.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.setup_actions(appwindow);

        self.imp().popovermenu.connect_visible_notify(
            clone!(@weak self as filerow, @weak appwindow => move |w| {
                if w.get_visible() {
                    appwindow.sidebar().workspacebrowser().files_list_set_selected(Some(filerow.position()));
                }
            }),
        );
    }

    fn setup_actions(&self, appwindow: &RnAppWindow) {
        self.insert_action_group("filerow", Some(&self.imp().action_group));

        self.imp()
            .action_group
            .add_action(&actions::open(self, appwindow));
        self.imp()
            .action_group
            .add_action(&actions::open_in_default_app(self, appwindow));
        self.imp()
            .action_group
            .add_action(&actions::rename(self, appwindow));
        self.imp()
            .action_group
            .add_action(&actions::trash(self, appwindow));
        self.imp()
            .action_group
            .add_action(&actions::duplicate(self, appwindow));
    }
}
