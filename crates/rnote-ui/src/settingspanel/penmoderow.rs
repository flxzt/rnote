// Imports
use super::penshortcutmodels::{
    ChangePenStyleIconFactory, ChangePenStyleListFactory, ChangePenStyleListModel,
};
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{glib, glib::subclass::*, CompositeTemplate};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use rnote_engine::pens::PenStyle;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penmoderow.ui")]
    pub(crate) struct RnPenModeRow {
        pub(crate) action: RefCell<PenStyle>,
        pub(crate) changepenstyle_model: ChangePenStyleListModel,

        #[template_child]
        pub(crate) mode: TemplateChild<gtk4::Switch>,
    }

    impl Default for RnPenModeRow {
        fn default() -> Self {
            Self {
                action: RefCell::new(PenStyle::Eraser),
                changepenstyle_model: ChangePenStyleListModel::default(),

                mode: TemplateChild::default(),
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

            obj.connect_selected_item_notify(move |row| {
                let new_pen_style = row.pen_style();
                let trigger_action: bool = {
                    let current_style_res = row.imp().action.try_borrow_mut();
                    match current_style_res {
                        Ok(mut current_style) => {
                            // when set from the canvas, both are changed at the same time
                            // it's not the case when a user change the selection
                            if *current_style != new_pen_style {
                                *current_style = new_pen_style;
                                true
                            } else {
                                false
                            }
                        }
                        Err(_) => false, // already used somewhere else, aborting
                    }
                };
                if trigger_action {
                    row.emit_by_name::<()>("action-changed", &[]);
                }
            });
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("action-changed").build()]);
            SIGNALS.as_ref()
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

    /// get the action immutably
    pub(crate) fn get_action(&self) -> PenStyle {
        *self.imp().action.borrow()
    }

    /// incoming change of state (from the canvas to the settings panel)
    pub(crate) fn set_action(&self, action: PenStyle) {
        match self.imp().action.try_borrow_mut() {
            Ok(mut value) => {
                *value = action;
                self.set_pen_style(action);
            }
            Err(e) => {
                tracing::debug!("Error borrowing action L136 {:?}", e)
            }
        }
    }

    pub(crate) fn pen_style(&self) -> PenStyle {
        PenStyle::try_from(self.selected()).unwrap()
    }

    pub(crate) fn set_pen_style(&self, style: PenStyle) {
        self.set_selected(style.to_u32().unwrap())
    }

    pub(crate) fn set_lock_state(&self, state: bool) {
        self.imp().mode.get().set_state(state);
        self.imp().mode.get().set_active(state);
    }
}
