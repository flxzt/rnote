mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, SpinButton,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(
        resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage/rectangleconfigpage.ui"
    )]
    pub struct RectangleConfigPage {
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

    impl Default for RectangleConfigPage {
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
    impl ObjectSubclass for RectangleConfigPage {
        const NAME: &'static str = "RectangleConfigPage";
        type Type = super::RectangleConfigPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RectangleConfigPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RectangleConfigPage {}
}

use crate::pens::shaper::RectangleConfig;
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use crate::utils;
use gtk4::gdk;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, Button, Orientable,
    SpinButton, Widget,
};

glib::wrapper! {
    pub struct RectangleConfigPage(ObjectSubclass<imp::RectangleConfigPage>)
        @extends Widget, @implements Orientable;
}

impl Default for RectangleConfigPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RectangleConfigPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create RectangleConfigPage")
    }

    pub fn width_resetbutton(&self) -> Button {
        imp::RectangleConfigPage::from_instance(self)
            .width_resetbutton
            .get()
    }

    pub fn width_adj(&self) -> Adjustment {
        imp::RectangleConfigPage::from_instance(self)
            .width_adj
            .get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::RectangleConfigPage::from_instance(self)
            .width_spinbutton
            .get()
    }

    pub fn stroke_colorpicker(&self) -> ColorPicker {
        imp::RectangleConfigPage::from_instance(self)
            .stroke_colorpicker
            .get()
    }

    pub fn fill_colorpicker(&self) -> ColorPicker {
        imp::RectangleConfigPage::from_instance(self)
            .fill_colorpicker
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let width_adj = self.width_adj();

        self.width_adj().set_lower(RectangleConfig::WIDTH_MIN);

        self.width_adj().set_upper(RectangleConfig::WIDTH_MAX);

        self.width_adj().set_value(RectangleConfig::WIDTH_DEFAULT);

        self.stroke_colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
            let color = stroke_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.color = Some(utils::Color::from_gdk(color));
        }));

        self.fill_colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
            let color = stroke_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.fill = Some(utils::Color::from_gdk(color));
        }));

        self.width_resetbutton().connect_clicked(
            clone!(@weak width_adj, @weak appwindow => move |_| {
                appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.set_width(RectangleConfig::WIDTH_DEFAULT);
                width_adj.set_value(RectangleConfig::WIDTH_DEFAULT);
            }),
        );

        self.width_adj()
            .connect_value_changed(clone!(@weak appwindow => move |width_adj| {
                appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.set_width(width_adj.value());
            }));
    }
}
