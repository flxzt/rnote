use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib};
use once_cell::sync::Lazy;
use rnote_compose::color;
use rnote_engine::utils::GdkRGBAHelpers;
use std::cell::RefCell;
use std::path::PathBuf;

use self::imp::WorkspaceListEntryInner;

mod imp {
    use super::*;

    #[derive(Debug, Clone, glib::Variant, serde::Serialize, serde::Deserialize)]
    #[serde(default, rename = "workspacelistentryinner")]
    pub(crate) struct WorkspaceListEntryInner {
        #[serde(rename = "dir")]
        pub(crate) dir: PathBuf,
        #[serde(rename = "icon")]
        pub(crate) icon: String,
        #[serde(rename = "color")]
        pub(crate) color: u32,
        #[serde(rename = "name")]
        pub(crate) name: String,
    }

    impl Default for WorkspaceListEntryInner {
        fn default() -> Self {
            Self {
                dir: PathBuf::from("./"),
                icon: String::from("workspacelistentryicon-folder-symbolic"),
                color: super::WorkspaceListEntry::COLOR_DEFAULT.as_rgba_u32(),
                name: String::from("default"),
            }
        }
    }

    #[derive(Debug, Default)]
    pub(crate) struct WorkspaceEntry {
        pub(crate) inner: RefCell<WorkspaceListEntryInner>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WorkspaceEntry {
        const NAME: &'static str = "WorkspaceListEntry";
        type Type = super::WorkspaceListEntry;
    }

    impl ObjectImpl for WorkspaceEntry {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "dir",
                        "dir",
                        "dir",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "icon",
                        "icon",
                        "icon",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoxed::new(
                        "color",
                        "color",
                        "color",
                        gdk::RGBA::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "name",
                        "name",
                        "name",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "dir" => {
                    let dir = value
                        .get::<String>()
                        .expect("The value needs to be of type `String`.");

                    self.inner.borrow_mut().dir = PathBuf::from(dir);
                }
                "icon" => {
                    let icon = value.get::<String>().expect("value not of type `String`");
                    self.inner.borrow_mut().icon = icon;
                }
                "color" => {
                    let color = value
                        .get::<gdk::RGBA>()
                        .expect("value not of type `gdk::RGBA`");
                    self.inner.borrow_mut().color = color.into_compose_color().into();
                }
                "name" => {
                    let name = value.get::<String>().expect("value not of type `String`");
                    self.inner.borrow_mut().name = name;
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "dir" => self
                    .inner
                    .borrow()
                    .dir
                    .to_string_lossy()
                    .to_string()
                    .to_value(),
                "icon" => self.inner.borrow().icon.to_value(),
                "color" => gdk::RGBA::from_compose_color(rnote_compose::Color::from(
                    self.inner.borrow().color,
                ))
                .to_value(),
                "name" => self.inner.borrow().name.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct WorkspaceListEntry(ObjectSubclass<imp::WorkspaceEntry>);
}

impl Default for WorkspaceListEntry {
    fn default() -> Self {
        Self::new(WorkspaceListEntryInner::default())
    }
}

impl WorkspaceListEntry {
    pub(crate) const COLOR_DEFAULT: piet::Color = color::GNOME_BLUES[4];

    pub(crate) fn new(inner: WorkspaceListEntryInner) -> Self {
        glib::Object::new(&[
            ("dir", &inner.dir.to_string_lossy().to_string().to_value()),
            ("icon", &inner.icon.to_value()),
            (
                "color",
                &gdk::RGBA::from_compose_color(rnote_compose::Color::from(inner.color)).to_value(),
            ),
            ("name", &inner.name.to_value()),
        ])
    }

    pub(crate) fn replace_data(&self, entry: &Self) {
        self.set_name(entry.name());
        self.set_icon(entry.icon());
        self.set_color(entry.color());
        self.set_dir(entry.dir());
    }

    pub(crate) fn dir(&self) -> String {
        self.property::<String>("dir")
    }

    pub(crate) fn set_dir(&self, dir: String) {
        self.set_property("dir", dir.to_value());
    }

    pub(crate) fn icon(&self) -> String {
        self.property::<String>("icon")
    }

    pub(crate) fn set_icon(&self, icon: String) {
        self.set_property("icon", icon.to_value());
    }

    pub(crate) fn color(&self) -> gdk::RGBA {
        self.property::<gdk::RGBA>("color")
    }

    pub(crate) fn set_color(&self, color: gdk::RGBA) {
        self.set_property("color", color.to_value());
    }

    pub(crate) fn name(&self) -> String {
        self.property::<String>("name")
    }

    pub(crate) fn set_name(&self, name: String) {
        self.set_property("name", name.to_value());
    }
}

impl glib::StaticVariantType for WorkspaceListEntry {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        let ty = WorkspaceListEntryInner::static_variant_type();
        let variant_type = glib::VariantType::new(ty.as_str()).unwrap();
        std::borrow::Cow::from(variant_type)
    }
}

impl glib::ToVariant for WorkspaceListEntry {
    fn to_variant(&self) -> glib::Variant {
        self.imp().inner.borrow().to_variant()
    }
}

impl glib::FromVariant for WorkspaceListEntry {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        Some(Self::new(WorkspaceListEntryInner::from_variant(variant)?))
    }
}
