// Imports
use super::penshortcutmodels::{
    FOCUS_MODE_ENTRY, PenShortcutActionIconFactory, PenShortcutActionListFactory,
    PenShortcutActionListModel,
};
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{CompositeTemplate, DropDown, ListBoxRow, Widget, glib, glib::clone, glib::subclass::*};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use rnote_engine::pens::PenStyle;
use rnote_engine::pens::shortcuts::ShortcutAction;
use rnote_engine::pens::shortcuts::ShortcutMode;
use std::cell::RefCell;
use std::str::FromStr;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penshortcutrow.ui")]
    pub(crate) struct RnPenShortcutRow {
        pub(crate) action: RefCell<ShortcutAction>,
        pub(crate) shortcut_actions_model: PenShortcutActionListModel,

        #[template_child]
        pub(crate) mode_dropdown: TemplateChild<DropDown>,
    }

    impl Default for RnPenShortcutRow {
        fn default() -> Self {
            Self {
                action: RefCell::new(ShortcutAction::ChangePenStyle {
                    style: PenStyle::Eraser,
                    mode: ShortcutMode::Temporary,
                }),
                shortcut_actions_model: PenShortcutActionListModel::default(),

                mode_dropdown: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnPenShortcutRow {
        const NAME: &'static str = "RnPenShortcutRow";
        type Type = super::RnPenShortcutRow;
        type ParentType = adw::ComboRow;
        type Interfaces = ();

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnPenShortcutRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let list_factory = PenShortcutActionListFactory::default();
            let icon_factory = PenShortcutActionIconFactory::default();

            obj.set_model(Some(&*self.shortcut_actions_model));
            obj.set_list_factory(Some(&*list_factory));
            obj.set_factory(Some(&*icon_factory));

            obj.connect_selected_item_notify(move |row| {
                let previous_action = row.action();

                if row.selected_is_focus_mode() {
                    row.imp().mode_dropdown.set_visible(false);
                    *row.imp().action.borrow_mut() = ShortcutAction::ToggleFocusMode;
                } else {
                    row.imp().mode_dropdown.set_visible(true);
                    let mode = match previous_action {
                        ShortcutAction::ChangePenStyle { mode, .. } => mode,
                        ShortcutAction::ToggleFocusMode => ShortcutMode::Temporary,
                    };
                    *row.imp().action.borrow_mut() = ShortcutAction::ChangePenStyle {
                        style: row.selected_pen_style().unwrap(),
                        mode,
                    };
                }

                row.emit_by_name::<()>("action-changed", &[]);
            });

            self.mode_dropdown.get().connect_selected_notify(clone!(
                #[weak(rename_to=penshortcutrow)]
                obj,
                move |_| {
                    match &mut *penshortcutrow.imp().action.borrow_mut() {
                        ShortcutAction::ChangePenStyle { mode, .. } => {
                            *mode = penshortcutrow.shortcut_mode();
                        }
                        ShortcutAction::ToggleFocusMode => {}
                    }
                    penshortcutrow.emit_by_name::<()>("action-changed", &[]);
                }
            ));

            obj.connect_local(
                "action-changed",
                false,
                clone!(
                    #[weak(rename_to=penshortcutrow)]
                    obj,
                    #[upgrade_or]
                    None,
                    move |_values| {
                        penshortcutrow.update_ui();
                        None
                    }
                ),
            );
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

    impl WidgetImpl for RnPenShortcutRow {}
    impl ListBoxRowImpl for RnPenShortcutRow {}
    impl PreferencesRowImpl for RnPenShortcutRow {}
    impl ActionRowImpl for RnPenShortcutRow {}
    impl ComboRowImpl for RnPenShortcutRow {}
}

glib::wrapper! {
    pub(crate) struct RnPenShortcutRow(ObjectSubclass<imp::RnPenShortcutRow>)
        @extends adw::ComboRow, adw::ActionRow, adw::PreferencesRow, ListBoxRow, Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl RnPenShortcutRow {
    #[allow(clippy::new_without_default)]
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        glib::Object::new()
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

    pub(crate) fn selected_is_focus_mode(&self) -> bool {
        self.imp()
            .shortcut_actions_model
            .string(self.selected())
            .is_some_and(|string| string == FOCUS_MODE_ENTRY)
    }

    fn focus_mode_index(&self) -> Option<u32> {
        let index = self.imp().shortcut_actions_model.find(FOCUS_MODE_ENTRY);
        (index != u32::MAX).then_some(index)
    }

    pub(crate) fn selected_pen_style(&self) -> Option<PenStyle> {
        self.imp()
            .shortcut_actions_model
            .string(self.selected())
            .and_then(|string| PenStyle::from_str(&string).ok())
    }

    fn style_index(&self, style: PenStyle) -> Option<u32> {
        let index = self.imp().shortcut_actions_model.find(&style.to_string());
        (index != u32::MAX).then_some(index)
    }

    pub(crate) fn set_pen_style(&self, style: PenStyle) {
        if let Some(index) = self.style_index(style) {
            self.set_selected(index);
        }
    }

    pub(crate) fn shortcut_mode(&self) -> ShortcutMode {
        ShortcutMode::try_from(self.imp().mode_dropdown.selected()).unwrap()
    }

    pub(crate) fn set_shortcut_mode(&self, mode: ShortcutMode) {
        self.imp()
            .mode_dropdown
            .set_selected(mode.to_u32().unwrap())
    }

    fn update_ui(&self) {
        match self.action() {
            ShortcutAction::ChangePenStyle { style, mode } => {
                self.set_pen_style(style);
                self.set_shortcut_mode(mode);
                self.imp().mode_dropdown.set_visible(true);
            }
            ShortcutAction::ToggleFocusMode => {
                if let Some(index) = self.focus_mode_index() {
                    self.set_selected(index);
                }
                self.imp().mode_dropdown.set_visible(false);
            }
        }
    }
}
