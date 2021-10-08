mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, SpinButton,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage/ellipseconfigpage.ui")]
    pub struct EllipseConfigPage {
        #[template_child]
        pub width_resetbutton: TemplateChild<Button>,
        #[template_child]
        pub width_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub stroke_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub fill_colorpicker: TemplateChild<ColorPicker>,
    }

    impl Default for EllipseConfigPage {
        fn default() -> Self {
            Self {
                width_resetbutton: TemplateChild::<Button>::default(),
                width_adj: TemplateChild::<Adjustment>::default(),
                width_spinbutton: TemplateChild::<SpinButton>::default(),
                stroke_colorpicker: TemplateChild::<ColorPicker>::default(),
                fill_colorpicker: TemplateChild::<ColorPicker>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EllipseConfigPage {
        const NAME: &'static str = "EllipseConfigPage";
        type Type = super::EllipseConfigPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EllipseConfigPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for EllipseConfigPage {}
}

use crate::pens::shaper::EllipseConfig;
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use crate::utils;
use gtk4::gdk;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, Button, Orientable,
    SpinButton, Widget,
};

glib::wrapper! {
    pub struct EllipseConfigPage(ObjectSubclass<imp::EllipseConfigPage>)
        @extends Widget, @implements Orientable;
}

impl Default for EllipseConfigPage {
    fn default() -> Self {
        Self::new()
    }
}

impl EllipseConfigPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create EllipseConfigPage")
    }

    pub fn width_resetbutton(&self) -> Button {
        imp::EllipseConfigPage::from_instance(self)
            .width_resetbutton
            .get()
    }

    pub fn width_adj(&self) -> Adjustment {
        imp::EllipseConfigPage::from_instance(self).width_adj.get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::EllipseConfigPage::from_instance(self)
            .width_spinbutton
            .get()
    }

    pub fn stroke_colorpicker(&self) -> ColorPicker {
        imp::EllipseConfigPage::from_instance(self)
            .stroke_colorpicker
            .get()
    }

    pub fn fill_colorpicker(&self) -> ColorPicker {
        imp::EllipseConfigPage::from_instance(self)
            .fill_colorpicker
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let width_adj = self.width_adj();

        self.width_adj().set_lower(EllipseConfig::WIDTH_MIN);

        self.width_adj().set_upper(EllipseConfig::WIDTH_MAX);

        self.width_adj().set_value(EllipseConfig::WIDTH_DEFAULT);

        self.stroke_colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
            let color = stroke_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().shaper.ellipse_config.color = Some(utils::Color::from_gdk(color));
        }));

        self.fill_colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
            let color = stroke_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().shaper.ellipse_config.fill = Some(utils::Color::from_gdk(color));
        }));

        self.width_resetbutton().connect_clicked(
            clone!(@weak width_adj, @weak appwindow => move |_| {
                appwindow.canvas().pens().borrow_mut().shaper.ellipse_config.set_width(EllipseConfig::WIDTH_DEFAULT);
                width_adj.set_value(EllipseConfig::WIDTH_DEFAULT);
            }),
        );

        self.width_adj()
            .connect_value_changed(clone!(@weak appwindow => move |width_adj| {
                appwindow.canvas().pens().borrow_mut().shaper.ellipse_config.set_width(width_adj.value());
            }));
    }
}
