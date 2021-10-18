mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, SpinButton,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/markerpage.ui")]
    pub struct MarkerPage {
        #[template_child]
        pub width_resetbutton: TemplateChild<Button>,
        #[template_child]
        pub width_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub colorpicker: TemplateChild<ColorPicker>,
    }

    impl Default for MarkerPage {
        fn default() -> Self {
            Self {
                width_resetbutton: TemplateChild::<Button>::default(),
                width_adj: TemplateChild::<Adjustment>::default(),
                width_spinbutton: TemplateChild::<SpinButton>::default(),
                colorpicker: TemplateChild::<ColorPicker>::default(),
            }
        }
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

use crate::pens::marker::Marker;
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use crate::utils;
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, Button, Orientable,
    SpinButton, Widget,
};

glib::wrapper! {
    pub struct MarkerPage(ObjectSubclass<imp::MarkerPage>)
        @extends Widget, @implements Orientable;
}

impl Default for MarkerPage {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkerPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MarkerPage")
    }

    pub fn width_resetbutton(&self) -> Button {
        imp::MarkerPage::from_instance(self).width_resetbutton.get()
    }

    pub fn width_adj(&self) -> Adjustment {
        imp::MarkerPage::from_instance(self).width_adj.get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::MarkerPage::from_instance(self).width_spinbutton.get()
    }

    pub fn colorpicker(&self) -> ColorPicker {
        imp::MarkerPage::from_instance(self).colorpicker.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let width_adj = self.width_adj();

        self.width_adj().set_lower(Marker::WIDTH_MIN);

        self.width_adj().set_upper(Marker::WIDTH_MAX);

        self.width_adj().set_value(Marker::WIDTH_DEFAULT);

        self.colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |colorpicker, _paramspec| {
            let color = colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().marker.color = utils::Color::from(color);
        }));

        self.width_resetbutton().connect_clicked(
            clone!(@weak width_adj, @weak appwindow => move |_| {
                appwindow.canvas().pens().borrow_mut().marker.set_width(Marker::WIDTH_DEFAULT);
                width_adj.set_value(Marker::WIDTH_DEFAULT);
            }),
        );

        self.width_adj()
            .connect_value_changed(clone!(@weak appwindow => move |width_adj| {
                appwindow.canvas().pens().borrow_mut().marker.set_width(width_adj.value());
            }));
    }
}
