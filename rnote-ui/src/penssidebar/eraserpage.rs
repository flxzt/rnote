use crate::appwindow::RnoteAppWindow;
use adw::prelude::*;
use gtk4::{glib, glib::clone, subclass::prelude::*, CompositeTemplate, SpinButton, ToggleButton};
use rnote_engine::pens::eraser::Eraser;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/eraserpage.ui")]
    pub struct EraserPage {
        #[template_child]
        pub eraserstyle_trash_colliding_strokes_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub eraserstyle_split_colliding_strokes_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EraserPage {
        const NAME: &'static str = "EraserPage";
        type Type = super::EraserPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EraserPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for EraserPage {}
}

glib::wrapper! {
    pub struct EraserPage(ObjectSubclass<imp::EraserPage>)
        @extends gtk4::Widget;
}

impl Default for EraserPage {
    fn default() -> Self {
        Self::new()
    }
}

impl EraserPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create EraserPage")
    }

    pub fn eraserstyle_trash_colliding_strokes_toggle(&self) -> ToggleButton {
        self.imp().eraserstyle_trash_colliding_strokes_toggle.get()
    }

    pub fn eraserstyle_split_colliding_strokes_toggle(&self) -> ToggleButton {
        self.imp().eraserstyle_split_colliding_strokes_toggle.get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        self.imp().width_spinbutton.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.eraserstyle_trash_colliding_strokes_toggle().connect_toggled(clone!(@weak appwindow => move |eraserstyle_trash_colliding_strokes_toggle| {
            if eraserstyle_trash_colliding_strokes_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "eraser-style", Some(&"trash-colliding-strokes".to_variant()));
            }
        }));

        self.eraserstyle_split_colliding_strokes_toggle().connect_toggled(clone!(@weak appwindow => move |eraserstyle_split_colliding_strokes_toggle| {
            if eraserstyle_split_colliding_strokes_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "eraser-style", Some(&"split-colliding-strokes".to_variant()));
            }
        }));

        self.width_spinbutton().set_increments(1.0, 5.0);
        self.width_spinbutton()
            .set_range(Eraser::WIDTH_MIN, Eraser::WIDTH_MAX);
        self.width_spinbutton().set_value(Eraser::WIDTH_DEFAULT);

        self.width_spinbutton().connect_value_changed(
            clone!(@weak appwindow => move |width_spinbutton| {
                appwindow.canvas().engine().borrow_mut().penholder.eraser.width = width_spinbutton.value();
            }),
        );
    }
}
