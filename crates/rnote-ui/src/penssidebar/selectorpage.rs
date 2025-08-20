// Imports
use crate::RnAppWindow;
use gtk4::{CompositeTemplate, ToggleButton, glib, glib::clone, prelude::*, subclass::prelude::*};
use rnote_engine::pens::pensconfig::selectorconfig::SelectorStyle;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/selectorpage.ui")]
    pub(crate) struct RnSelectorPage {
        #[template_child]
        pub(crate) selectorstyle_polygon_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selectorstyle_rect_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selectorstyle_single_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selectorstyle_intersectingpath_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) resize_lock_aspectratio_togglebutton: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnSelectorPage {
        const NAME: &'static str = "RnSelectorPage";
        type Type = super::RnSelectorPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnSelectorPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnSelectorPage {}
}

glib::wrapper! {
    pub(crate) struct RnSelectorPage(ObjectSubclass<imp::RnSelectorPage>)
        @extends gtk4::Widget;
}

impl Default for RnSelectorPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnSelectorPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn selector_style(&self) -> Option<SelectorStyle> {
        if self.imp().selectorstyle_polygon_toggle.is_active() {
            Some(SelectorStyle::Polygon)
        } else if self.imp().selectorstyle_rect_toggle.is_active() {
            Some(SelectorStyle::Rectangle)
        } else if self.imp().selectorstyle_single_toggle.is_active() {
            Some(SelectorStyle::Single)
        } else if self.imp().selectorstyle_intersectingpath_toggle.is_active() {
            Some(SelectorStyle::IntersectingPath)
        } else {
            None
        }
    }

    #[allow(unused)]
    pub(crate) fn set_selector_style(&self, style: SelectorStyle) {
        match style {
            SelectorStyle::Polygon => self.imp().selectorstyle_polygon_toggle.set_active(true),
            SelectorStyle::Rectangle => self.imp().selectorstyle_rect_toggle.set_active(true),
            SelectorStyle::Single => self.imp().selectorstyle_single_toggle.set_active(true),
            SelectorStyle::IntersectingPath => self
                .imp()
                .selectorstyle_intersectingpath_toggle
                .set_active(true),
        }
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.selectorstyle_polygon_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .selector_config
                    .style = SelectorStyle::Polygon;
            }
        ));

        imp.selectorstyle_rect_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .selector_config
                    .style = SelectorStyle::Rectangle;
            }
        ));

        imp.selectorstyle_single_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .selector_config
                    .style = SelectorStyle::Single;
            }
        ));

        imp.selectorstyle_intersectingpath_toggle
            .connect_toggled(clone!(
                #[weak]
                appwindow,
                move |toggle| {
                    if !toggle.is_active() {
                        return;
                    }
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .selector_config
                        .style = SelectorStyle::IntersectingPath;
                }
            ));

        imp.resize_lock_aspectratio_togglebutton
            .connect_toggled(clone!(
                #[weak]
                appwindow,
                move |toggle| {
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .selector_config
                        .resize_lock_aspectratio = toggle.is_active();
                }
            ));
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        let selector_config = appwindow
            .engine_config()
            .read()
            .pens_config
            .selector_config
            .clone();

        self.set_selector_style(selector_config.style);

        imp.resize_lock_aspectratio_togglebutton
            .set_active(selector_config.resize_lock_aspectratio);
    }
}
