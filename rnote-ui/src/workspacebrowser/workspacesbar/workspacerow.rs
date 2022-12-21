use crate::RnoteAppWindow;
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, CssProvider,
    GestureClick, GestureLongPress, Image, Label, Widget,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use unicode_segmentation::UnicodeSegmentation;

use super::WorkspaceListEntry;

mod imp {
    use rnote_engine::utils::GdkRGBAHelpers;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacerow.ui")]
    pub(crate) struct WorkspaceRow {
        pub(crate) entry: RefCell<WorkspaceListEntry>,
        #[template_child]
        pub(crate) folder_image: TemplateChild<Image>,
        #[template_child]
        pub(crate) name_label: TemplateChild<Label>,
    }

    impl Default for WorkspaceRow {
        fn default() -> Self {
            Self {
                entry: RefCell::new(WorkspaceListEntry::default()),
                folder_image: TemplateChild::<Image>::default(),
                name_label: TemplateChild::<Label>::default(),
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
        fn constructed(&self) {
            self.parent_constructed();

            self.instance().set_css_classes(&["workspacerow"]);

            self.connect_entry();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "entry",
                    "entry",
                    "entry",
                    WorkspaceListEntry::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "entry" => {
                    let entry = value
                        .get::<WorkspaceListEntry>()
                        .expect("The value needs to be of type `WorkspaceListEntry`.");

                    self.entry.replace(entry);
                    self.connect_entry();
                    self.update_apearance();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "entry" => self.entry.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for WorkspaceRow {}

    impl WorkspaceRow {
        fn connect_entry(&self) {
            let obj = self.instance();

            self.entry.borrow().connect_notify_local(
                Some("dir"),
                clone!(@strong obj => move |_, _| {
                    obj.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("icon"),
                clone!(@strong obj => move |_, _| {
                    obj.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("color"),
                clone!(@strong obj => move |_, _| {
                    obj.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("name"),
                clone!(@strong obj => move |_, _| {
                    obj.imp().update_apearance();
                }),
            );
        }

        fn update_apearance(&self) {
            let dir = self.entry.borrow().dir();
            let icon = self.entry.borrow().icon();
            let color = self.entry.borrow().color();
            let name = self.entry.borrow().name();

            let color_str = format!(
                "rgba({0}, {1}, {2}, {3:.3})",
                (color.red() * 255.0) as i32,
                (color.green() * 255.0) as i32,
                (color.blue() * 255.0) as i32,
                (color.alpha() * 1000.0).round() / 1000.0,
            );

            // Check luminosity to either have light or dark fg colors to ensure good contrast
            let fg_color_str = if color.into_compose_color().luma()
                < super::WorkspaceRow::FG_LUMINANCE_THRESHOLD
            {
                String::from("@light_1")
            } else {
                String::from("@dark_5")
            };

            let css = CssProvider::new();

            self.name_label
                .set_label(name.graphemes(true).take(2).collect::<String>().as_str());
            self.instance()
                .set_tooltip_text(Some(format!("{}\n{}", name, dir).as_str()));

            self.folder_image.set_icon_name(Some(&icon));

            let custom_css = format!(
                "@define-color workspacerow_color {};@define-color workspacerow_fg_color {};",
                color_str, fg_color_str
            );

            css.load_from_data(custom_css.as_bytes());

            self.instance()
                .style_context()
                .add_provider(&css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            self.instance().queue_draw();
        }
    }
}

glib::wrapper! {
    pub(crate) struct WorkspaceRow(ObjectSubclass<imp::WorkspaceRow>)
        @extends gtk4::Widget;
}

impl Default for WorkspaceRow {
    fn default() -> Self {
        Self::new(&WorkspaceListEntry::default())
    }
}

impl WorkspaceRow {
    /// The threshold of the luminance of the workspacerow color, deciding if a light or dark fg color is used. Between 0.0 and 1.0
    pub(crate) const FG_LUMINANCE_THRESHOLD: f64 = 0.7;

    pub(crate) fn new(entry: &WorkspaceListEntry) -> Self {
        glib::Object::new(&[("entry", &entry.to_value())])
    }

    #[allow(unused)]
    pub(crate) fn entry(&self) -> WorkspaceListEntry {
        self.property::<WorkspaceListEntry>("entry")
    }

    #[allow(unused)]
    pub(crate) fn set_entry(&self, entry: WorkspaceListEntry) {
        self.set_property("entry", entry.to_value());
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let rightclick_gesture = GestureClick::builder()
            .name("rightclick_gesture")
            .button(gdk::BUTTON_SECONDARY)
            .build();
        self.add_controller(&rightclick_gesture);
        rightclick_gesture.connect_pressed(
            clone!(@weak appwindow => move |_rightclick_gesture, _n_press, _x, _y| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow.workspacebrowser().workspacesbar().action_group(), "edit-selected-workspace", None);
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
                adw::prelude::ActionGroupExt::activate_action(&appwindow.workspacebrowser().workspacesbar().action_group(), "edit-selected-workspace", None);
            }),
        );
    }
}
