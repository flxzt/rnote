mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate, SpinButton};

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/markerpage.ui")]
    pub struct MarkerPage {
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub colorpicker: TemplateChild<ColorPicker>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MarkerPage {
        const NAME: &'static str = "MarkerPage";
        type Type = super::MarkerPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MarkerPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for MarkerPage {}
}

use crate::compose::color::Color;
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, SpinButton,
};

glib::wrapper! {
    pub struct MarkerPage(ObjectSubclass<imp::MarkerPage>)
        @extends gtk4::Widget;
}

impl Default for MarkerPage {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkerPage {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 6.0;
    /// The min width
    pub const WIDTH_MIN: f64 = 0.1;
    /// The max width
    pub const WIDTH_MAX: f64 = 1000.0;

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MarkerPage")
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::MarkerPage::from_instance(self).width_spinbutton.get()
    }

    pub fn colorpicker(&self) -> ColorPicker {
        imp::MarkerPage::from_instance(self).colorpicker.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.width_spinbutton().set_increments(0.1, 2.0);
        self.width_spinbutton()
            .set_range(Self::WIDTH_MIN, Self::WIDTH_MAX);
        // Must be after set_range() !
        self.width_spinbutton().set_value(Self::WIDTH_DEFAULT);

        self.colorpicker().connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |colorpicker, _paramspec| {
                let color = colorpicker.property::<gdk::RGBA>("current-color");
                appwindow.canvas().pens().borrow_mut().marker.options.stroke_color = Some(Color::from(color));
            }),
        );

        self.width_spinbutton().connect_value_changed(
            clone!(@weak appwindow => move |width_spinbutton| {
                appwindow.canvas().pens().borrow_mut().marker.options.width = width_spinbutton.value();
            }),
        );
    }
}
