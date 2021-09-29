mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, SpinButton,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage/lineconfigpage.ui")]
    pub struct LineConfigPage {
        #[template_child]
        pub width_resetbutton: TemplateChild<Button>,
        #[template_child]
        pub width_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub stroke_colorpicker: TemplateChild<ColorPicker>,
    }

    impl Default for LineConfigPage {
        fn default() -> Self {
            Self {
                width_resetbutton: TemplateChild::<Button>::default(),
                width_adj: TemplateChild::<Adjustment>::default(),
                width_spinbutton: TemplateChild::<SpinButton>::default(),
                stroke_colorpicker: TemplateChild::<ColorPicker>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LineConfigPage {
        const NAME: &'static str = "LineConfigPage";
        type Type = super::LineConfigPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LineConfigPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for LineConfigPage {}
}

use crate::pens::shaper::LineConfig;
use crate::strokes;
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use gtk4::gdk;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, Button, Orientable,
    SpinButton, Widget,
};

glib::wrapper! {
    pub struct LineConfigPage(ObjectSubclass<imp::LineConfigPage>)
        @extends Widget, @implements Orientable;
}

impl Default for LineConfigPage {
    fn default() -> Self {
        Self::new()
    }
}

impl LineConfigPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create LineConfigPage")
    }

    pub fn width_resetbutton(&self) -> Button {
        imp::LineConfigPage::from_instance(self)
            .width_resetbutton
            .get()
    }

    pub fn width_adj(&self) -> Adjustment {
        imp::LineConfigPage::from_instance(self).width_adj.get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::LineConfigPage::from_instance(self)
            .width_spinbutton
            .get()
    }

    pub fn stroke_colorpicker(&self) -> ColorPicker {
        imp::LineConfigPage::from_instance(self)
            .stroke_colorpicker
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let width_adj = self.width_adj();

        self.width_adj().set_lower(LineConfig::WIDTH_MIN);

        self.width_adj().set_upper(LineConfig::WIDTH_MAX);

        self.width_adj().set_value(LineConfig::WIDTH_DEFAULT);

        self.stroke_colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
            let color = stroke_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().shaper.line_config.color = Some(strokes::Color::from_gdk(color));
        }));

        self.width_resetbutton().connect_clicked(
            clone!(@weak width_adj, @weak appwindow => move |_| {
                appwindow.canvas().pens().borrow_mut().shaper.line_config.set_width(LineConfig::WIDTH_DEFAULT);
                width_adj.set_value(LineConfig::WIDTH_DEFAULT);
            }),
        );

        self.width_adj()
            .connect_value_changed(clone!(@weak appwindow => move |width_adj| {
                appwindow.canvas().pens().borrow_mut().shaper.line_config.set_width(width_adj.value());
            }));
    }
}
