mod imp {
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate, SpinButton};

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/eraserpage.ui")]
    pub struct EraserPage {
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

use crate::appwindow::RnoteAppWindow;
use gtk4::{glib, glib::clone, subclass::prelude::*, SpinButton};
use rnote_engine::pens::eraser::Eraser;

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

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::EraserPage::from_instance(self).width_spinbutton.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.width_spinbutton().set_increments(1.0, 5.0);
        self.width_spinbutton()
            .set_range(Eraser::WIDTH_MIN, Eraser::WIDTH_MAX);
        self.width_spinbutton().set_value(Eraser::WIDTH_DEFAULT);

        self.width_spinbutton().connect_value_changed(
            clone!(@weak appwindow => move |width_spinbutton| {
                appwindow.canvas().pens().borrow_mut().eraser.width = width_spinbutton.value();
            }),
        );
    }
}
