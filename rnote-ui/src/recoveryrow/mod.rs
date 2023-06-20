#![warn(clippy::todo)]
// modules
mod actions;

// Imports
use adw::{
    prelude::{ButtonExt, WidgetExt},
    subclass::prelude::CompositeTemplateDisposeExt,
};
use cairo::glib::{self, clone};
use gtk4::{
    subclass::{
        prelude::{
            ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt,
            WidgetClassSubclassExt,
        },
        widget::{CompositeTemplate, CompositeTemplateInitializingExt, WidgetImpl},
    },
    Button, CompositeTemplate, Label, TemplateChild, Widget,
};
use rnote_fileformats::recovery_metadata::RecoveryMetadata;
use std::{cell::RefCell, path::PathBuf};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::appwindow::RnAppWindow;

mod imp {

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/github/flxzt/rnote/ui/recoveryentry.ui")]
    pub(crate) struct RnRecoveryRow {
        pub(crate) last_changed_format: String,
        pub(crate) meta: RefCell<Option<RecoveryMetadata>>,
        pub(crate) meta_path: RefCell<Option<PathBuf>>,

        #[template_child]
        pub(crate) document_name_label: TemplateChild<Label>,
        #[template_child]
        pub(crate) document_path_label: TemplateChild<Label>,
        #[template_child]
        pub(crate) last_changed_label: TemplateChild<Label>,
        #[template_child]
        pub(crate) recover_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) save_as_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) discard_button: TemplateChild<Button>,
    }

    impl From<RecoveryMetadata> for RnRecoveryRow {
        fn from(meta: RecoveryMetadata) -> Self {
            Self {
                last_changed_format: format_unix_timestamp(meta.last_changed()),
                meta: RefCell::new(Some(meta)),
                ..Default::default()
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnRecoveryRow {
        const NAME: &'static str = "RnRecoveryEntry";
        type Type = super::RnRecoveryRow;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl WidgetImpl for RnRecoveryRow {}
    impl ObjectImpl for RnRecoveryRow {
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
        // fn properties() -> &'static [glib::ParamSpec] {
        //     static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
        //         vec![
        //             // this is nullable, so it can be used to represent Option<gio::File>
        //             glib::ParamSpecObject::builder::<Path>("current-file").build(),
        //         ]
        //     });
        //     PROPERTIES.as_ref()
        // }

        // fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        //     match pspec.name() {
        //         "current-file" => {
        //             let current_file = value
        //                 .get::<Option<gio::File>>()
        //                 .expect("The value needs to be of type `Option<gio::File>`");
        //             self.current_file.replace(current_file);
        //         }
        //         _ => unimplemented!(),
        //     }
        // }

        // fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        //     match pspec.name() {
        //         "current-file" => self.current_file.borrow().to_value(),
        //         _ => unimplemented!(),
        //     }
        // }
    }

    impl RnRecoveryRow {
        fn setup_input(&self) {}
    }
}

fn format_unix_timestamp(unix: i64) -> String {
    // Shows occuring errors in timesptamp label field instead of crashing
    match OffsetDateTime::from_unix_timestamp(unix) {
        Err(e) => {
            log::error!("Failed to get time from unix time: {e}");
            String::from("Error getting time")
        }
        Ok(ts) => ts.format(&Rfc2822).unwrap_or_else(|e| {
            log::error!("Failed to format time: {e}");
            String::from("Error formatting time")
        }),
    }
}

glib::wrapper! {
    pub(crate) struct RnRecoveryRow(ObjectSubclass<imp::RnRecoveryRow>)
        @extends gtk4::Widget;
}
impl Default for RnRecoveryRow {
    fn default() -> Self {
        Self::new()
    }
}

impl RnRecoveryRow {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.setup_actions(appwindow);
    }

    fn setup_actions(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        imp.recover_button.connect_clicked(
            clone!(@weak self as recoveryrow => move |_button| actions::recover(&recoveryrow)),
        );
        imp.discard_button.connect_clicked(
            clone!(@weak self as recoveryrow => move |_button| actions::discard(&recoveryrow)),
        );
        imp.save_as_button.connect_clicked(
            clone!(@weak self as recoveryrow => move |_button| actions::save_as(&recoveryrow)),
        );
    }
}
