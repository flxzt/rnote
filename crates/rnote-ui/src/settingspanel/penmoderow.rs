// Imports
use super::penshortcutmodels::{
    ChangePenStyleIconFactory, ChangePenStyleListFactory, ChangePenStyleListModel,
};
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{CompositeTemplate, glib};
use num_traits::ToPrimitive;
use rnote_engine::pens::PenStyle;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penmoderow.ui")]
    pub(crate) struct RnPenModeRow {
        pub(crate) changepenstyle_model: ChangePenStyleListModel,

        #[template_child]
        pub(crate) mode: TemplateChild<gtk4::Switch>,
    }

    impl Default for RnPenModeRow {
        fn default() -> Self {
            Self {
                changepenstyle_model: ChangePenStyleListModel::default(),
                mode: TemplateChild::default(),
                // will probably be more featured later
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnPenModeRow {
        const NAME: &'static str = "RnPenModeRow";
        type Type = super::RnPenModeRow;
        type ParentType = adw::ComboRow;
        type Interfaces = ();

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnPenModeRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let list_factory = ChangePenStyleListFactory::default();
            let icon_factory = ChangePenStyleIconFactory::default();

            obj.set_model(Some(&*self.changepenstyle_model));
            obj.set_list_factory(Some(&*list_factory));
            obj.set_factory(Some(&*icon_factory));
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnPenModeRow {}
    impl ListBoxRowImpl for RnPenModeRow {}
    impl PreferencesRowImpl for RnPenModeRow {}
    impl ActionRowImpl for RnPenModeRow {}
    impl ComboRowImpl for RnPenModeRow {}
}

glib::wrapper! {
    pub(crate) struct RnPenModeRow(ObjectSubclass<imp::RnPenModeRow>)
        @extends adw::ComboRow, adw::ActionRow, adw::PreferencesRow, gtk4::ListBoxRow, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl RnPenModeRow {
    #[allow(clippy::new_without_default)]
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn pen_style(&self) -> PenStyle {
        PenStyle::try_from(self.selected()).unwrap()
    }

    pub(crate) fn set_pen_style(&self, style: PenStyle) {
        self.set_selected(style.to_u32().unwrap())
    }
}
