mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use gtk4::{MenuButton, Popover, Revealer, SpinButton, Switch, ToggleButton};

    #[derive(Default, Debug, CompositeTemplate)]
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
        pub roughconfig_roughness_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_bowing_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_curvestepcount_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_multistroke_switch: TemplateChild<Switch>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
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

use crate::compose::color::Color;
use crate::compose::rough::roughoptions::{self, RoughOptions};
use crate::pens::shaper::ShaperDrawStyle;
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use gtk4::{gdk, MenuButton, Popover, Revealer, SpinButton, Switch, ToggleButton};
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};

glib::wrapper! {
    pub struct ShaperPage(ObjectSubclass<imp::ShaperPage>)
        @extends gtk4::Widget;
}

impl Default for ShaperPage {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaperPage {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 2.0;
    /// The min width
    pub const WIDTH_MIN: f64 = 0.1;
    /// The max width
    pub const WIDTH_MAX: f64 = 1000.0;

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

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::ShaperPage::from_instance(self).width_spinbutton.get()
    }

    pub fn roughconfig_roughness_spinbutton(&self) -> SpinButton {
        imp::ShaperPage::from_instance(self)
            .roughconfig_roughness_spinbutton
            .get()
    }

    pub fn roughconfig_bowing_spinbutton(&self) -> SpinButton {
        imp::ShaperPage::from_instance(self)
            .roughconfig_bowing_spinbutton
            .get()
    }

    pub fn roughconfig_curvestepcount_spinbutton(&self) -> SpinButton {
        imp::ShaperPage::from_instance(self)
            .roughconfig_curvestepcount_spinbutton
            .get()
    }

    pub fn roughconfig_multistroke_switch(&self) -> Switch {
        imp::ShaperPage::from_instance(self)
            .roughconfig_multistroke_switch
            .get()
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
        // Width
        self.width_spinbutton().set_increments(0.1, 2.0);
        self.width_spinbutton()
            .set_range(Self::WIDTH_MIN, Self::WIDTH_MAX);
        self.width_spinbutton().set_value(Self::WIDTH_DEFAULT);

        self.width_spinbutton().connect_value_changed(
            clone!(@weak appwindow => move |width_spinbutton| {
                let shaper_style = appwindow.canvas().pens().borrow_mut().shaper.drawstyle;

                match shaper_style {
                    ShaperDrawStyle::Smooth => appwindow.canvas().pens().borrow_mut().shaper.smooth_options.width = width_spinbutton.value(),
                    ShaperDrawStyle::Rough => appwindow.canvas().pens().borrow_mut().shaper.rough_options.stroke_width = width_spinbutton.value(),
                }
            }),
        );

        // Stroke color
        self.stroke_colorpicker().connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
                let color = Color::from(stroke_colorpicker.property::<gdk::RGBA>("current-color"));
                let shaper_style = appwindow.canvas().pens().borrow_mut().shaper.drawstyle;

                match shaper_style {
                    ShaperDrawStyle::Smooth => appwindow.canvas().pens().borrow_mut().shaper.smooth_options.stroke_color = Some(color),
                    ShaperDrawStyle::Rough => appwindow.canvas().pens().borrow_mut().shaper.rough_options.stroke_color= Some(color),
                }
            }),
        );

        // Fill color
        self.fill_colorpicker().connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |fill_colorpicker, _paramspec| {
                let color = Color::from(fill_colorpicker.property::<gdk::RGBA>("current-color"));
                let shaper_style = appwindow.canvas().pens().borrow_mut().shaper.drawstyle;

                match shaper_style {
                    ShaperDrawStyle::Smooth => appwindow.canvas().pens().borrow_mut().shaper.smooth_options.fill_color = Some(color),
                    ShaperDrawStyle::Rough => appwindow.canvas().pens().borrow_mut().shaper.rough_options.fill_color= Some(color),
                }
            }),
        );

        // Roughness
        self.imp()
            .roughconfig_roughness_spinbutton
            .get()
            .set_increments(0.1, 2.0);
        self.imp()
            .roughconfig_roughness_spinbutton
            .get()
            .set_range(RoughOptions::ROUGHNESS_MIN, RoughOptions::ROUGHNESS_MAX);
        self.imp()
            .roughconfig_roughness_spinbutton
            .get()
            .set_value(RoughOptions::ROUGHNESS_DEFAULT);

        self.imp().roughconfig_roughness_spinbutton.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_roughness_spinbutton| {
                appwindow.canvas().pens().borrow_mut().shaper.rough_options.set_roughness(roughconfig_roughness_spinbutton.value());
            }),
        );

        // Bowing
        self.imp()
            .roughconfig_bowing_spinbutton
            .get()
            .set_increments(0.1, 2.0);
        self.imp()
            .roughconfig_bowing_spinbutton
            .get()
            .set_range(RoughOptions::BOWING_MIN, RoughOptions::BOWING_MAX);
        self.imp()
            .roughconfig_bowing_spinbutton
            .get()
            .set_value(roughoptions::RoughOptions::BOWING_DEFAULT);

        self.imp().roughconfig_bowing_spinbutton.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_bowing_spinbutton| {
                appwindow.canvas().pens().borrow_mut().shaper.rough_options.set_bowing(roughconfig_bowing_spinbutton.value());
            }),
        );

        // Curve stepcount
        self.imp()
            .roughconfig_curvestepcount_spinbutton
            .get()
            .set_increments(1.0, 2.0);
        self.imp()
            .roughconfig_curvestepcount_spinbutton
            .get()
            .set_range(
                RoughOptions::CURVESTEPCOUNT_MIN,
                RoughOptions::CURVESTEPCOUNT_MAX,
            );
        self.imp()
            .roughconfig_curvestepcount_spinbutton
            .get()
            .set_value(roughoptions::RoughOptions::CURVESTEPCOUNT_DEFAULT);

        self.imp().roughconfig_curvestepcount_spinbutton.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_curvestepcount_spinbutton| {
                appwindow.canvas().pens().borrow_mut().shaper.rough_options.set_curve_stepcount(roughconfig_curvestepcount_spinbutton.value());
            }),
        );

        // Multistroke
        self.imp().roughconfig_multistroke_switch.get().connect_state_notify(clone!(@weak appwindow => move |roughconfig_multistroke_switch| {
            appwindow.canvas().pens().borrow_mut().shaper.rough_options.set_multistroke(roughconfig_multistroke_switch.state());
        }));

        // Smooth / Rough shape toggle
        self.drawstyle_smooth_toggle().connect_toggled(clone!(@weak appwindow => move |drawstyle_smooth_toggle| {
            if drawstyle_smooth_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "shaper-drawstyle", Some(&"smooth".to_variant()));
            }
        }));

        self.drawstyle_smooth_toggle()
            .bind_property("active", &self.roughconfig_menubutton(), "sensitive")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::INVERT_BOOLEAN)
            .build();

        self.drawstyle_rough_toggle().connect_toggled(clone!(@weak appwindow => move |drawstyle_rough_toggle| {
            if drawstyle_rough_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "shaper-drawstyle", Some(&"rough".to_variant()));
            }
        }));

        self.drawstyle_rough_toggle()
            .bind_property("active", &self.roughconfig_menubutton(), "sensitive")
            .flags(glib::BindingFlags::DEFAULT)
            .build();

        self.line_toggle()
            .bind_property("active", &self.fill_revealer(), "reveal-child")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::INVERT_BOOLEAN)
            .build();

        self.rectangle_toggle()
            .bind_property("active", &self.fill_revealer(), "reveal-child")
            .flags(glib::BindingFlags::DEFAULT)
            .build();

        self.ellipse_toggle()
            .bind_property("active", &self.fill_revealer(), "reveal-child")
            .flags(glib::BindingFlags::DEFAULT)
            .build();

        // Shape toggles
        self.line_toggle().connect_toggled(clone!(@weak self as shaperpage, @weak appwindow => move |line_toggle| {
            if line_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "shaper-style", Some(&"line".to_variant()));
            }
        }));

        self.rectangle_toggle().connect_toggled(clone!(@weak appwindow => move |rectangle_toggle| {
            if rectangle_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "shaper-style", Some(&"rectangle".to_variant()));
            }
        }));

        self.ellipse_toggle().connect_toggled(clone!(@weak appwindow => move |ellipse_toggle| {
            if ellipse_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "shaper-style", Some(&"ellipse".to_variant()));
            }
        }));
    }
}
