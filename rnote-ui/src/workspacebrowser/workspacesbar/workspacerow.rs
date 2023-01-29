use crate::RnoteAppWindow;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, CssProvider, Image,
    Label, Widget,
};
use once_cell::sync::Lazy;
use rnote_compose::{color, Color};
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
            let inst = self.instance();

            self.entry.borrow().connect_notify_local(
                Some("dir"),
                clone!(@weak inst as workspacerow => move |_, _| {
                    workspacerow.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("icon"),
                clone!(@weak inst as workspacerow => move |_, _| {
                    workspacerow.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("color"),
                clone!(@weak inst as workspacerow => move |_, _| {
                    workspacerow.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("name"),
                clone!(@weak inst as workspacerow => move |_, _| {
                    workspacerow.imp().update_apearance();
                }),
            );
        }

        fn update_apearance(&self) {
            let dir = self.entry.borrow().dir();
            let icon = self.entry.borrow().icon();
            let color = self.entry.borrow().color().into_compose_color();
            let name = self.entry.borrow().name();

            let workspacerow_color = format!(
                "rgba({0}, {1}, {2}, {3:.3})",
                (color.r * 255.0) as i32,
                (color.g * 255.0) as i32,
                (color.b * 255.0) as i32,
                (color.a * 1000.0).round() / 1000.0,
            );

            let workspacerow_fg_color = if color == Color::TRANSPARENT {
                String::from("@window_fg_color")
            } else if color.luma() < color::FG_LUMINANCE_THRESHOLD {
                String::from("@light_1")
            } else {
                String::from("@dark_5")
            };

            let css = CssProvider::new();

            self.name_label
                .set_label(name.graphemes(true).take(2).collect::<String>().as_str());
            self.instance()
                .set_tooltip_text(Some(format!("{name}\n{dir}").as_str()));

            self.folder_image.set_icon_name(Some(&icon));

            let custom_css = format!(
                "@define-color workspacerow_color {workspacerow_color};@define-color workspacerow_fg_color {workspacerow_fg_color};",
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

    pub(crate) fn init(&self, _appwindow: &RnoteAppWindow) {
        // TODO: add gestures / menu for editing the row
    }
}
