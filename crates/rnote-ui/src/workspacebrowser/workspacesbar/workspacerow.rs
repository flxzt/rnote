// Imports
use super::RnWorkspaceListEntry;
use crate::RnAppWindow;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, CssProvider, Image,
    Label, Widget,
};
use once_cell::sync::Lazy;
use rnote_compose::{color, Color};
use rnote_engine::ext::GdkRGBAExt;
use std::cell::RefCell;
use unicode_segmentation::UnicodeSegmentation;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacesbar/workspacerow.ui")]
    pub(crate) struct RnWorkspaceRow {
        pub(crate) entry: RefCell<RnWorkspaceListEntry>,
        #[template_child]
        pub(crate) folder_image: TemplateChild<Image>,
        #[template_child]
        pub(crate) name_label: TemplateChild<Label>,
    }

    impl Default for RnWorkspaceRow {
        fn default() -> Self {
            Self {
                entry: RefCell::new(RnWorkspaceListEntry::default()),
                folder_image: TemplateChild::<Image>::default(),
                name_label: TemplateChild::<Label>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnWorkspaceRow {
        const NAME: &'static str = "RnWorkspaceRow";
        type Type = super::RnWorkspaceRow;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnWorkspaceRow {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_css_classes(&["workspacerow"]);

            self.connect_entry();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<RnWorkspaceListEntry>("entry").build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "entry" => {
                    let entry = value
                        .get::<RnWorkspaceListEntry>()
                        .expect("The value needs to be of type `WorkspaceListEntry`");

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

    impl WidgetImpl for RnWorkspaceRow {}

    impl RnWorkspaceRow {
        fn connect_entry(&self) {
            let obj = self.obj();

            self.entry.borrow().connect_notify_local(
                Some("dir"),
                clone!(@weak obj as workspacerow => move |_, _| {
                    workspacerow.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("icon"),
                clone!(@weak obj as workspacerow => move |_, _| {
                    workspacerow.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("color"),
                clone!(@weak obj as workspacerow => move |_, _| {
                    workspacerow.imp().update_apearance();
                }),
            );

            self.entry.borrow().connect_notify_local(
                Some("name"),
                clone!(@weak obj as workspacerow => move |_, _| {
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
            self.obj()
                .set_tooltip_text(Some(format!("{name}\n{dir}").as_str()));

            self.folder_image.set_icon_name(Some(&icon));

            let custom_css = format!(
                "@define-color workspacerow_color {workspacerow_color};@define-color workspacerow_fg_color {workspacerow_fg_color};",
            );

            css.load_from_string(&custom_css);

            // adding custom css is deprecated.
            // TODO: We should refactor to drawing through snapshot().
            #[allow(deprecated)]
            self.obj()
                .style_context()
                .add_provider(&css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            self.obj().queue_draw();
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnWorkspaceRow(ObjectSubclass<imp::RnWorkspaceRow>)
        @extends gtk4::Widget;
}

impl Default for RnWorkspaceRow {
    fn default() -> Self {
        Self::new(&RnWorkspaceListEntry::default())
    }
}

impl RnWorkspaceRow {
    pub(crate) fn new(entry: &RnWorkspaceListEntry) -> Self {
        glib::Object::builder()
            .property("entry", entry.to_value())
            .build()
    }

    #[allow(unused)]
    pub(crate) fn entry(&self) -> RnWorkspaceListEntry {
        self.property::<RnWorkspaceListEntry>("entry")
    }

    #[allow(unused)]
    pub(crate) fn set_entry(&self, entry: RnWorkspaceListEntry) {
        self.set_property("entry", entry.to_value());
    }

    pub(crate) fn init(&self, _appwindow: &RnAppWindow) {
        // TODO: add gestures / menu for editing the row
    }
}
