pub mod ellipseconfigpage;
pub mod lineconfigpage;
pub mod rectangleconfigpage;

mod imp {
    use super::{
        ellipseconfigpage::EllipseConfigPage, lineconfigpage::LineConfigPage,
        rectangleconfigpage::RectangleConfigPage,
    };
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use gtk4::{
        Adjustment, MenuButton, Popover, Revealer, SpinButton, Stack, StackPage, Switch,
        ToggleButton,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage/shaperpage.ui")]
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
        pub roughconfig_multistroke_switch: TemplateChild<Switch>,
        #[template_child]
        pub roughconfig_curvestepcount_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_curvestepcount_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub shapes_togglebox: TemplateChild<gtk4::Box>,
        #[template_child]
        pub line_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub rectangle_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub ellipse_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub shaperconfig_stack: TemplateChild<Stack>,
        #[template_child]
        pub lineconfig_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub lineconfig_page: TemplateChild<LineConfigPage>,
        #[template_child]
        pub rectangleconfig_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub rectangleconfig_page: TemplateChild<RectangleConfigPage>,
        #[template_child]
        pub ellipseconfig_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub ellipseconfig_page: TemplateChild<EllipseConfigPage>,
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
                roughconfig_multistroke_switch: TemplateChild::<Switch>::default(),
                roughconfig_curvestepcount_spinbutton: TemplateChild::<SpinButton>::default(),
                roughconfig_curvestepcount_adj: TemplateChild::<Adjustment>::default(),
                shapes_togglebox: TemplateChild::<gtk4::Box>::default(),
                line_toggle: TemplateChild::<ToggleButton>::default(),
                rectangle_toggle: TemplateChild::<ToggleButton>::default(),
                ellipse_toggle: TemplateChild::<ToggleButton>::default(),
                shaperconfig_stack: TemplateChild::<Stack>::default(),
                lineconfig_stackpage: TemplateChild::<StackPage>::default(),
                lineconfig_page: TemplateChild::<LineConfigPage>::default(),
                rectangleconfig_stackpage: TemplateChild::<StackPage>::default(),
                rectangleconfig_page: TemplateChild::<RectangleConfigPage>::default(),
                ellipseconfig_stackpage: TemplateChild::<StackPage>::default(),
                ellipseconfig_page: TemplateChild::<EllipseConfigPage>::default(),
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

use crate::ui::appwindow::RnoteAppWindow;
use ellipseconfigpage::EllipseConfigPage;
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, Orientable, Widget};
use gtk4::{MenuButton, Popover, Revealer, Stack, StackPage, ToggleButton};
use lineconfigpage::LineConfigPage;
use rectangleconfigpage::RectangleConfigPage;

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

    pub fn shaperconfig_stack(&self) -> Stack {
        imp::ShaperPage::from_instance(self)
            .shaperconfig_stack
            .get()
    }

    pub fn lineconfig_stackpage(&self) -> StackPage {
        imp::ShaperPage::from_instance(self)
            .rectangleconfig_stackpage
            .get()
    }

    pub fn lineconfig_page(&self) -> LineConfigPage {
        imp::ShaperPage::from_instance(self).lineconfig_page.get()
    }

    pub fn rectangleconfig_stackpage(&self) -> StackPage {
        imp::ShaperPage::from_instance(self)
            .rectangleconfig_stackpage
            .get()
    }

    pub fn rectangleconfig_page(&self) -> RectangleConfigPage {
        imp::ShaperPage::from_instance(self)
            .rectangleconfig_page
            .get()
    }

    pub fn ellipseconfig_stackpage(&self) -> StackPage {
        imp::ShaperPage::from_instance(self)
            .ellipseconfig_stackpage
            .get()
    }

    pub fn ellipseconfig_page(&self) -> EllipseConfigPage {
        imp::ShaperPage::from_instance(self)
            .ellipseconfig_page
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::ShaperPage::from_instance(self);
        self.lineconfig_page().init(appwindow);
        self.rectangleconfig_page().init(appwindow);
        self.ellipseconfig_page().init(appwindow);

        self.drawstyle_rough_toggle()
            .set_group(Some(&self.drawstyle_smooth_toggle()));

        self.rectangle_toggle().set_group(Some(&self.line_toggle()));
        self.ellipse_toggle().set_group(Some(&self.line_toggle()));

        self.shaperconfig_stack()
            .set_visible_child_name("rectangleconfig_page");

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

        // Multistroke
        priv_.roughconfig_multistroke_switch.get().connect_state_notify(clone!(@weak appwindow => move |roughconfig_multistroke_switch| {
            appwindow.canvas().pens().borrow_mut().shaper.roughconfig.set_multistroke(roughconfig_multistroke_switch.state());
        }));

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
        self.line_toggle().connect_active_notify(clone!(@weak appwindow => move |line_toggle| {
            if line_toggle.is_active() {
                appwindow.application().unwrap().activate_action("current-shape", Some(&"line".to_variant()));
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

        self.shaperconfig_stack().connect_visible_child_name_notify(
            clone!(@weak appwindow => move |shaperconfig_stack| {
                if let Some(child_name) = shaperconfig_stack.visible_child_name() {
                    match child_name.to_value().get::<String>().unwrap().as_str() {
                        "lineconfig_page" => {
                            appwindow.application().unwrap().activate_action("current-shape", Some(&"line".to_variant()));
                        },
                        "rectangleconfig_page" => {
                            appwindow.application().unwrap().activate_action("current-shape", Some(&"rectangle".to_variant()));
                        },
                        "ellipseconfig_page" => {
                            appwindow.application().unwrap().activate_action("current-shape", Some(&"ellipse".to_variant()));
                        },
                        _ => {}
                    };
                };
            }),
        );
    }
}
