// Imports
use crate::RnAppWindow;
use gtk4::{
    Button, CompositeTemplate, TemplateChild, ToggleButton, Widget, glib, glib::clone, prelude::*,
    subclass::prelude::*,
};
use rnote_engine::pens::PenStyle;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penpicker.ui")]
    pub(crate) struct RnPenPicker {
        #[template_child]
        pub(crate) brush_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) shaper_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) typewriter_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) eraser_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selector_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) tools_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) undo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) redo_button: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnPenPicker {
        const NAME: &'static str = "RnPenPicker";
        type Type = super::RnPenPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnPenPicker {
        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnPenPicker {}
}

glib::wrapper! {
    pub(crate) struct RnPenPicker(ObjectSubclass<imp::RnPenPicker>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnPenPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl RnPenPicker {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn brush_toggle(&self) -> ToggleButton {
        self.imp().brush_toggle.get()
    }

    pub(crate) fn shaper_toggle(&self) -> ToggleButton {
        self.imp().shaper_toggle.get()
    }

    pub(crate) fn typewriter_toggle(&self) -> ToggleButton {
        self.imp().typewriter_toggle.get()
    }

    pub(crate) fn eraser_toggle(&self) -> ToggleButton {
        self.imp().eraser_toggle.get()
    }

    pub(crate) fn selector_toggle(&self) -> ToggleButton {
        self.imp().selector_toggle.get()
    }

    pub(crate) fn tools_toggle(&self) -> ToggleButton {
        self.imp().tools_toggle.get()
    }

    pub(crate) fn undo_button(&self) -> Button {
        self.imp().undo_button.get()
    }

    pub(crate) fn redo_button(&self) -> Button {
        self.imp().redo_button.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.brush_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |brush_toggle| {
                if brush_toggle.is_active() {
                    appwindow.set_pen_style(PenStyle::Brush);
                }
            }
        ));

        imp.shaper_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |shaper_toggle| {
                if shaper_toggle.is_active() {
                    appwindow.set_pen_style(PenStyle::Shaper);
                }
            }
        ));

        imp.typewriter_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |typewriter_toggle| {
                if typewriter_toggle.is_active() {
                    appwindow.set_pen_style(PenStyle::Typewriter);
                }
            }
        ));

        imp.eraser_toggle.get().connect_toggled(clone!(
            #[weak]
            appwindow,
            move |eraser_toggle| {
                if eraser_toggle.is_active() {
                    appwindow.set_pen_style(PenStyle::Eraser);
                }
            }
        ));

        imp.selector_toggle.get().connect_toggled(clone!(
            #[weak]
            appwindow,
            move |selector_toggle| {
                if selector_toggle.is_active() {
                    appwindow.set_pen_style(PenStyle::Selector);
                }
            }
        ));

        imp.tools_toggle.get().connect_toggled(clone!(
            #[weak]
            appwindow,
            move |tools_toggle| {
                if tools_toggle.is_active() {
                    appwindow.set_pen_style(PenStyle::Tools);
                }
            }
        ));
    }
}
