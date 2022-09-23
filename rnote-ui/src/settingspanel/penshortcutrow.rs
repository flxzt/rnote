use crate::settingspanel::penshortcutmodels::{
    ChangePenStyleIconFactory, ChangePenStyleListFactory, ChangePenStyleListModel,
};
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{glib, glib::clone, glib::subclass::*, CheckButton, CompositeTemplate};
use once_cell::sync::Lazy;
use rnote_compose::penhelpers::ShortcutKey;
use rnote_engine::pens::penholder::PenStyle;
use rnote_engine::pens::shortcuts::ShortcutAction;
use std::cell::RefCell;

mod imp {
    use super::*;
    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penshortcutrow.ui")]
    pub struct PenShortcutRow {
        pub key: RefCell<Option<ShortcutKey>>,
        pub action: RefCell<ShortcutAction>,
        pub changepenstyle_model: ChangePenStyleListModel,

        #[template_child]
        pub permanent_checker: TemplateChild<CheckButton>,
    }

    impl Default for PenShortcutRow {
        fn default() -> Self {
            Self {
                key: RefCell::new(None),
                action: RefCell::new(ShortcutAction::ChangePenStyle {
                    style: PenStyle::Eraser,
                    permanent: false,
                }),
                permanent_checker: TemplateChild::<CheckButton>::default(),
                changepenstyle_model: ChangePenStyleListModel::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PenShortcutRow {
        const NAME: &'static str = "PenShortcutRow";
        type Type = super::PenShortcutRow;
        type ParentType = adw::ComboRow;
        type Interfaces = ();

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PenShortcutRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let list_factory = ChangePenStyleListFactory::default();
            let icon_factory = ChangePenStyleIconFactory::default();

            obj.set_model(Some(&*self.changepenstyle_model));
            obj.set_list_factory(Some(&*list_factory));
            obj.set_factory(Some(&*icon_factory));

            obj.connect_selected_item_notify(move |obj| {
                if let Some(selected_item) = obj.selected_item() {
                    let new_pen_style = PenStyle::try_from(
                        selected_item
                            .downcast::<adw::EnumListItem>()
                            .unwrap()
                            .value() as u32,
                    )
                    .unwrap();

                    match &mut *obj.imp().action.borrow_mut() {
                        ShortcutAction::ChangePenStyle {
                            ref mut style,
                            permanent: _,
                        } => {
                            *style = new_pen_style;
                        }
                    }
                    obj.emit_by_name::<()>("action-changed", &[]);
                }
            });

            self.permanent_checker.get().connect_toggled(
                clone!(@weak obj => move |permanent_checker| {
                    match &mut *obj.imp().action.borrow_mut() {
                        ShortcutAction::ChangePenStyle { style: _, ref mut permanent } => {
                            *permanent = permanent_checker.is_active();
                        }
                    }
                    obj.emit_by_name::<()>("action-changed", &[]);
                }),
            );

            obj.connect_local(
                "key-changed",
                false,
                clone!(@weak obj => @default-return None, move |_values| {
                    obj.update_ui();
                    None
                }),
            );

            obj.connect_local(
                "action-changed",
                false,
                clone!(@weak obj => @default-return None, move |_values| {
                    obj.update_ui();
                    None
                }),
            );
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("key-changed", &[], <()>::static_type().into()).build(),
                    Signal::builder("action-changed", &[], <()>::static_type().into()).build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for PenShortcutRow {}
    impl ListBoxRowImpl for PenShortcutRow {}
    impl PreferencesRowImpl for PenShortcutRow {}
    impl ActionRowImpl for PenShortcutRow {}
    impl ComboRowImpl for PenShortcutRow {}

    impl PenShortcutRow {}
}

glib::wrapper! {
    pub struct PenShortcutRow(ObjectSubclass<imp::PenShortcutRow>)
        @extends adw::ComboRow, adw::ActionRow, adw::PreferencesRow, gtk4::ListBoxRow, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Actionable;
}

impl PenShortcutRow {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    fn update_ui(&self) {
        let action = self.action();
        match action {
            ShortcutAction::ChangePenStyle { style, permanent } => {
                self.set_selected(self.imp().changepenstyle_model.find_position(style as i32));
                self.imp().permanent_checker.set_active(permanent);
            }
        }
    }

    pub fn key(&self) -> Option<ShortcutKey> {
        *self.imp().key.borrow()
    }

    pub fn set_key(&self, key: Option<ShortcutKey>) {
        *self.imp().key.borrow_mut() = key;
        self.emit_by_name::<()>("key-changed", &[]);
    }

    pub fn action(&self) -> ShortcutAction {
        *self.imp().action.borrow()
    }

    pub fn set_action(&self, action: ShortcutAction) {
        *self.imp().action.borrow_mut() = action;
        self.emit_by_name::<()>("action-changed", &[]);
    }
}
