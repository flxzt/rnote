mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use gtk4::{Adjustment, Button, MenuButton, Popover, Revealer, SpinButton, Switch, ToggleButton};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage.ui")]
    pub struct ShaperPage {
        #[template_child]
        pub drawstyle_smooth_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub drawstyle_rough_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub roughconfig_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub roughconfig_popover: TemplateChild<Popover>,
        #[template_child]
        pub roughconfig_revealer: TemplateChild<Revealer>,
        #[template_child]
        pub roughconfig_roughness_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_roughness_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub roughconfig_bowing_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_bowing_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub roughconfig_curvestepcount_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_curvestepcount_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub roughconfig_multistroke_switch: TemplateChild<Switch>,
        #[template_child]
        pub width_resetbutton: TemplateChild<Button>,
        #[template_child]
        pub width_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub stroke_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub fill_revealer: TemplateChild<Revealer>,
        #[template_child]
        pub fill_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub shapes_togglebox: TemplateChild<gtk4::Box>,
        #[template_child]
        pub line_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub rectangle_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub ellipse_toggle: TemplateChild<ToggleButton>,
    }

    impl Default for ShaperPage {
        fn default() -> Self {
            Self {
                drawstyle_smooth_toggle: TemplateChild::<ToggleButton>::default(),
                drawstyle_rough_toggle: TemplateChild::<ToggleButton>::default(),
                roughconfig_menubutton: TemplateChild::<MenuButton>::default(),
                roughconfig_popover: TemplateChild::<Popover>::default(),
                roughconfig_revealer: TemplateChild::<Revealer>::default(),
                roughconfig_roughness_spinbutton: TemplateChild::<SpinButton>::default(),
                roughconfig_roughness_adj: TemplateChild::<Adjustment>::default(),
                roughconfig_bowing_spinbutton: TemplateChild::<SpinButton>::default(),
                roughconfig_bowing_adj: TemplateChild::<Adjustment>::default(),
                roughconfig_curvestepcount_spinbutton: TemplateChild::<SpinButton>::default(),
                roughconfig_curvestepcount_adj: TemplateChild::<Adjustment>::default(),
                roughconfig_multistroke_switch: TemplateChild::<Switch>::default(),
                width_resetbutton: TemplateChild::<Button>::default(),
                width_adj: TemplateChild::<Adjustment>::default(),
                stroke_colorpicker: TemplateChild::<ColorPicker>::default(),
                fill_revealer: TemplateChild::<Revealer>::default(),
                fill_colorpicker: TemplateChild::<ColorPicker>::default(),
                shapes_togglebox: TemplateChild::<gtk4::Box>::default(),
                line_toggle: TemplateChild::<ToggleButton>::default(),
                rectangle_toggle: TemplateChild::<ToggleButton>::default(),
                ellipse_toggle: TemplateChild::<ToggleButton>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShaperPage {
        const NAME: &'static str = "ShaperPage";
        type Type = super::ShaperPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ShaperPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ShaperPage {}
}

use crate::pens::shaper::{Shaper};
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use crate::utils;
use gtk4::{Adjustment, Button, MenuButton, Popover, Revealer, ToggleButton, gdk};
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, Orientable, Widget};

glib::wrapper! {
    pub struct ShaperPage(ObjectSubclass<imp::ShaperPage>)
        @extends Widget, @implements Orientable;
}

impl Default for ShaperPage {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaperPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ShaperPage")
    }

    pub fn drawstyle_smooth_toggle(&self) -> ToggleButton {
        imp::ShaperPage::from_instance(self)
            .drawstyle_smooth_toggle
            .get()
    }

    pub fn drawstyle_rough_toggle(&self) -> ToggleButton {
        imp::ShaperPage::from_instance(self)
            .drawstyle_rough_toggle
            .get()
    }

    pub fn roughconfig_menubutton(&self) -> MenuButton {
        imp::ShaperPage::from_instance(self)
            .roughconfig_menubutton
            .get()
    }

    pub fn roughconfig_popover(&self) -> Popover {
        imp::ShaperPage::from_instance(self)
            .roughconfig_popover
            .get()
    }

    pub fn roughconfig_revealer(&self) -> Revealer {
        imp::ShaperPage::from_instance(self)
            .roughconfig_revealer
            .get()
    }

    pub fn width_resetbutton(&self) -> Button {
        imp::ShaperPage::from_instance(self)
            .width_resetbutton
            .get()
    }

    pub fn width_adj(&self) -> Adjustment {
        imp::ShaperPage::from_instance(self).width_adj.get()
    }

    pub fn stroke_colorpicker(&self) -> ColorPicker {
        imp::ShaperPage::from_instance(self)
            .stroke_colorpicker
            .get()
    }

    pub fn fill_revealer(&self) -> Revealer {
        imp::ShaperPage::from_instance(self).fill_revealer.get()
    }

    pub fn fill_colorpicker(&self) -> ColorPicker {
        imp::ShaperPage::from_instance(self).fill_colorpicker.get()
    }

    pub fn shapes_togglebox(&self) -> gtk4::Box {
        imp::ShaperPage::from_instance(self).shapes_togglebox.get()
    }

    pub fn line_toggle(&self) -> ToggleButton {
        imp::ShaperPage::from_instance(self).line_toggle.get()
    }

    pub fn rectangle_toggle(&self) -> ToggleButton {
        imp::ShaperPage::from_instance(self).rectangle_toggle.get()
    }

    pub fn ellipse_toggle(&self) -> ToggleButton {
        imp::ShaperPage::from_instance(self).ellipse_toggle.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::ShaperPage::from_instance(self);
        let width_adj = self.width_adj();

        // Shape stroke width
        self.width_resetbutton().connect_clicked(
            clone!(@weak width_adj, @weak appwindow => move |_| {
                appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.set_width(Shaper::WIDTH_DEFAULT);
                width_adj.set_value(Shaper::WIDTH_DEFAULT);
                appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.set_width(Shaper::WIDTH_DEFAULT);
                width_adj.set_value(Shaper::WIDTH_DEFAULT);
                appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.set_width(Shaper::WIDTH_DEFAULT);
                width_adj.set_value(Shaper::WIDTH_DEFAULT);
            }),
        );


        self.width_adj().set_lower(Shaper::WIDTH_MIN);

        self.width_adj().set_upper(Shaper::WIDTH_MAX);

        self.width_adj().set_value(Shaper::WIDTH_DEFAULT);
        self.width_adj()
            .connect_value_changed(clone!(@weak appwindow => move |width_adj| {
                appwindow.canvas().pens().borrow_mut().shaper.line_config.set_width(width_adj.value());
                appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.set_width(width_adj.value());
                appwindow.canvas().pens().borrow_mut().shaper.ellipse_config.set_width(width_adj.value());
            }));

        // Stroke color
        self.stroke_colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
            let color = stroke_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().shaper.line_config.color = Some(utils::Color::from(color));
            appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.color = Some(utils::Color::from(color));
            appwindow.canvas().pens().borrow_mut().shaper.ellipse_config.color = Some(utils::Color::from(color));
        }));

        // Fill color
        self.fill_colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |fill_colorpicker, _paramspec| {
            let color = fill_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().shaper.rectangle_config.fill = Some(utils::Color::from(color));
            appwindow.canvas().pens().borrow_mut().shaper.ellipse_config.fill = Some(utils::Color::from(color));
        }));

        // Roughness
        priv_
            .roughconfig_roughness_adj
            .get()
            .set_lower(rough_rs::options::Options::ROUGHNESS_MIN);
        priv_
            .roughconfig_roughness_adj
            .get()
            .set_upper(rough_rs::options::Options::ROUGHNESS_MAX);
        priv_
            .roughconfig_roughness_adj
            .get()
            .set_value(rough_rs::options::Options::ROUGHNESS_DEFAULT);

        priv_.roughconfig_roughness_adj.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_roughness_adj| {
                appwindow.canvas().pens().borrow_mut().shaper.roughconfig.set_roughness(roughconfig_roughness_adj.value());
            }),
        );

        // Bowing
        priv_
            .roughconfig_bowing_adj
            .get()
            .set_lower(rough_rs::options::Options::BOWING_MIN);
        priv_
            .roughconfig_bowing_adj
            .get()
            .set_upper(rough_rs::options::Options::BOWING_MAX);
        priv_
            .roughconfig_bowing_adj
            .get()
            .set_value(rough_rs::options::Options::BOWING_DEFAULT);

        priv_.roughconfig_bowing_adj.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_bowing_adj| {
                appwindow.canvas().pens().borrow_mut().shaper.roughconfig.set_bowing(roughconfig_bowing_adj.value());
            }),
        );

        // Curve stepcount
        priv_
            .roughconfig_curvestepcount_adj
            .get()
            .set_lower(rough_rs::options::Options::CURVESTEPCOUNT_MIN);
        priv_
            .roughconfig_curvestepcount_adj
            .get()
            .set_upper(rough_rs::options::Options::CURVESTEPCOUNT_MAX);
        priv_
            .roughconfig_curvestepcount_adj
            .get()
            .set_value(rough_rs::options::Options::CURVESTEPCOUNT_DEFAULT);

        priv_.roughconfig_curvestepcount_adj.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_curvestepcount_adj| {
                appwindow.canvas().pens().borrow_mut().shaper.roughconfig.set_curve_stepcount(roughconfig_curvestepcount_adj.value());
            }),
        );

        // Multistroke
        priv_.roughconfig_multistroke_switch.get().connect_state_notify(clone!(@weak appwindow => move |roughconfig_multistroke_switch| {
            appwindow.canvas().pens().borrow_mut().shaper.roughconfig.set_multistroke(roughconfig_multistroke_switch.state());
        }));

        // Smooth / Rough shape toggle
        self.drawstyle_smooth_toggle().connect_active_notify(clone!(@weak appwindow => move |drawstyle_smooth_toggle| {
            if drawstyle_smooth_toggle.is_active() {
                appwindow.application().unwrap().activate_action("shaper-drawstyle", Some(&"smooth".to_variant()));
                appwindow.penssidebar().shaper_page().roughconfig_revealer().set_reveal_child(false);
            }
        }));

        self.drawstyle_rough_toggle().connect_active_notify(clone!(@weak appwindow => move |drawstyle_rough_toggle| {
            if drawstyle_rough_toggle.is_active() {
                appwindow.application().unwrap().activate_action("shaper-drawstyle", Some(&"rough".to_variant()));
                appwindow.penssidebar().shaper_page().roughconfig_revealer().set_reveal_child(true);
            }
        }));

        // Shape toggles
        self.line_toggle().connect_active_notify(clone!(@weak self as shaperpage, @weak appwindow => move |line_toggle| {
            if line_toggle.is_active() {
                appwindow.application().unwrap().activate_action("current-shape", Some(&"line".to_variant()));
            } else {
                shaperpage.fill_revealer().set_reveal_child(true);
            }
        }));

        self.rectangle_toggle().connect_active_notify(clone!(@weak appwindow => move |rectangle_toggle| {
            if rectangle_toggle.is_active() {
                appwindow.application().unwrap().activate_action("current-shape", Some(&"rectangle".to_variant()));
            }
        }));

        self.ellipse_toggle().connect_active_notify(clone!(@weak appwindow => move |ellipse_toggle| {
            if ellipse_toggle.is_active() {
                appwindow.application().unwrap().activate_action("current-shape", Some(&"ellipse".to_variant()));
            }
        }));
    }
}
