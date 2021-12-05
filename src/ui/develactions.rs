mod imp {
    use gtk4::ToggleButton;
    use gtk4::{glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate, Widget};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/develactions.ui")]
    pub struct DevelActions {
        #[template_child]
        pub element_stepback_button: TemplateChild<Button>,
        #[template_child]
        pub element_stepforward_button: TemplateChild<Button>,
        #[template_child]
        pub visual_debug_toggle: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DevelActions {
        const NAME: &'static str = "DevelActions";
        type Type = super::DevelActions;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DevelActions {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for DevelActions {}
}

use crate::ui::appwindow::RnoteAppWindow;

use gtk4::ToggleButton;
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, Button, Widget};

glib::wrapper! {
    pub struct DevelActions(ObjectSubclass<imp::DevelActions>)
        @extends Widget;
}

impl Default for DevelActions {
    fn default() -> Self {
        Self::new()
    }
}

impl DevelActions {
    pub fn new() -> Self {
        let mainheader: DevelActions =
            glib::Object::new(&[]).expect("Failed to create DevelActions");
        mainheader
    }

    pub fn element_stepback_button(&self) -> Button {
        imp::DevelActions::from_instance(self)
            .element_stepback_button
            .get()
    }

    pub fn element_stepforward_button(&self) -> Button {
        imp::DevelActions::from_instance(self)
            .element_stepforward_button
            .get()
    }

    pub fn visual_debug_toggle(&self) -> ToggleButton {
        imp::DevelActions::from_instance(self)
            .visual_debug_toggle
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.element_stepback_button()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                //appwindow.canvas().sheet().undo_elements_last_stroke(1, appwindow.canvas().zoom(), &*appwindow.canvas().renderer().borrow());

                appwindow.canvas().queue_draw();
            }));

        self.element_stepforward_button()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                //appwindow.canvas().sheet().redo_elements_last_stroke(1, appwindow.canvas().zoom(), &*appwindow.canvas().renderer().borrow());

                appwindow.canvas().queue_draw();
            }));

        self.visual_debug_toggle().connect_toggled(clone!(@weak appwindow => move |visual_debug_toggle| {
            appwindow.app_settings().set_boolean("visual-debug", visual_debug_toggle.is_active()).unwrap();
        }));
    }
}
