use adw::{prelude::*, subclass::prelude::*};
use gtk4::{glib, glib::clone, glib::subclass::*, CompositeTemplate, DropDown};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use rnote_engine::pens::shortcuts::ShortcutAction;
use rnote_engine::pens::shortcuts::ShortcutMode;
use rnote_engine::pens::PenStyle;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penshortcutrow.ui")]
    pub(crate) struct RnPenShortcutRow {
        pub(crate) action: RefCell<ShortcutAction>,

        #[template_child]
        pub(crate) style_dropdown: TemplateChild<DropDown>,
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

                style_dropdown: TemplateChild::default(),
                mode_dropdown: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnPenShortcutRow {
        const NAME: &'static str = "RnPenShortcutRow";
        type Type = super::RnPenShortcutRow;
        type ParentType = adw::ActionRow;
        type Interfaces = ();

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnPenShortcutRow {
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();

            self.style_dropdown.get().connect_selected_notify(
                clone!(@weak inst as penshortcutrow => move |_| {
                    match &mut *penshortcutrow.imp().action.borrow_mut() {
                        ShortcutAction::ChangePenStyle { style, .. } => {
                            *style = penshortcutrow.pen_style();
                        }
                    }
                    penshortcutrow.emit_by_name::<()>("action-changed", &[]);
                }),
            );

            self.mode_dropdown.get().connect_selected_notify(
                clone!(@weak inst as penshortcutrow => move |_| {
                    match &mut *penshortcutrow.imp().action.borrow_mut() {
                        ShortcutAction::ChangePenStyle { mode, .. } => {
                            *mode = penshortcutrow.shortcut_mode();
                        }
                    }
                    penshortcutrow.emit_by_name::<()>("action-changed", &[]);
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
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("action-changed").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for RnPenShortcutRow {}
    impl ListBoxRowImpl for RnPenShortcutRow {}
    impl PreferencesRowImpl for RnPenShortcutRow {}
    impl ActionRowImpl for RnPenShortcutRow {}

    impl RnPenShortcutRow {}
}

glib::wrapper! {
    pub(crate) struct RnPenShortcutRow(ObjectSubclass<imp::RnPenShortcutRow>)
        @extends adw::ActionRow, adw::PreferencesRow, gtk4::ListBoxRow, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Actionable;
}

impl RnPenShortcutRow {
    #[allow(clippy::new_without_default)]
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
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

    pub(crate) fn pen_style(&self) -> PenStyle {
        PenStyle::try_from(self.imp().style_dropdown.selected()).unwrap()
    }

    pub(crate) fn set_pen_style(&self, style: PenStyle) {
        self.imp()
            .style_dropdown
            .set_selected(style.to_u32().unwrap())
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
            }
        }
    }
}
