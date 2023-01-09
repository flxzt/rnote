use crate::settingspanel::penshortcutmodels::{
    ChangePenStyleIconFactory, ChangePenStyleListFactory, ChangePenStyleListModel,
};
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{glib, glib::clone, glib::subclass::*, CheckButton, CompositeTemplate};
use once_cell::sync::Lazy;
use rnote_compose::penevents::ShortcutKey;
use rnote_engine::pens::shortcuts::ShortcutAction;
use rnote_engine::pens::PenStyle;
use std::cell::RefCell;

mod imp {
    use super::*;
    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penshortcutrow.ui")]
    pub(crate) struct PenShortcutRow {
        pub(crate) key: RefCell<Option<ShortcutKey>>,
        pub(crate) action: RefCell<ShortcutAction>,
        pub(crate) changepenstyle_model: ChangePenStyleListModel,

        #[template_child]
        pub(crate) permanent_checker: TemplateChild<CheckButton>,
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
        fn constructed(&self) {
            let inst = self.instance();

            self.parent_constructed();

            let list_factory = ChangePenStyleListFactory::default();
            let icon_factory = ChangePenStyleIconFactory::default();

            inst.set_model(Some(&*self.changepenstyle_model));
            inst.set_list_factory(Some(&*list_factory));
            inst.set_factory(Some(&*icon_factory));

            inst.connect_selected_item_notify(move |obj| {
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
                clone!(@weak inst as penshortcutrow => move |permanent_checker| {
                    match &mut *penshortcutrow.imp().action.borrow_mut() {
                        ShortcutAction::ChangePenStyle { style: _, ref mut permanent } => {
                            *permanent = permanent_checker.is_active();
                        }
                    }
                    penshortcutrow.emit_by_name::<()>("action-changed", &[]);
                }),
            );

            inst.connect_local(
                "key-changed",
                false,
                clone!(@weak inst as penshortcutrow => @default-return None, move |_values| {
                    penshortcutrow.update_ui();
                    None
                }),
            );

            inst.connect_local(
                "action-changed",
                false,
                clone!(@weak inst as penshortcutrow => @default-return None, move |_values| {
                    penshortcutrow.update_ui();
                    None
                }),
            );
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("key-changed").build(),
                    Signal::builder("action-changed").build(),
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
    pub(crate) struct PenShortcutRow(ObjectSubclass<imp::PenShortcutRow>)
        @extends adw::ComboRow, adw::ActionRow, adw::PreferencesRow, gtk4::ListBoxRow, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Actionable;
}

impl PenShortcutRow {
    #[allow(clippy::new_without_default)]
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    fn update_ui(&self) {
        match self.action() {
            ShortcutAction::ChangePenStyle { style, permanent } => {
                self.set_selected(self.imp().changepenstyle_model.find_position(style as i32));
                self.imp().permanent_checker.set_active(permanent);
            }
        }
    }

    #[allow(unused)]
    pub(crate) fn key(&self) -> Option<ShortcutKey> {
        *self.imp().key.borrow()
    }

    #[allow(unused)]
    pub(crate) fn set_key(&self, key: Option<ShortcutKey>) {
        *self.imp().key.borrow_mut() = key;
        self.emit_by_name::<()>("key-changed", &[]);
    }

    #[allow(unused)]
    pub(crate) fn action(&self) -> ShortcutAction {
        *self.imp().action.borrow()
    }

    #[allow(unused)]
    pub(crate) fn set_action(&self, action: ShortcutAction) {
        *self.imp().action.borrow_mut() = action;
        self.emit_by_name::<()>("action-changed", &[]);
    }
}
