use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, ToggleButton};
use rnote_engine::pens::pensconfig::selectorconfig::SelectorStyle;

use crate::appwindow::RnoteAppWindow;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/selectorpage.ui")]
    pub(crate) struct SelectorPage {
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
    impl ObjectSubclass for SelectorPage {
        const NAME: &'static str = "SelectorPage";
        type Type = super::SelectorPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SelectorPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for SelectorPage {}
}

glib::wrapper! {
    pub(crate) struct SelectorPage(ObjectSubclass<imp::SelectorPage>)
        @extends gtk4::Widget;
}

impl Default for SelectorPage {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectorPage {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        imp.selectorstyle_polygon_toggle.connect_toggled(clone!(@weak appwindow => move |selectorstyle_polygon_toggle| {
            if selectorstyle_polygon_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().pens_config.selector_config.style = SelectorStyle::Polygon;
            }
        }));

        imp.selectorstyle_rect_toggle.connect_toggled(clone!(@weak appwindow => move |selectorstyle_rect_toggle| {
            if selectorstyle_rect_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().pens_config.selector_config.style = SelectorStyle::Rectangle;
            }
        }));

        imp.selectorstyle_single_toggle.connect_toggled(clone!(@weak appwindow => move |selectorstyle_single_toggle| {
            if selectorstyle_single_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().pens_config.selector_config.style = SelectorStyle::Single;
            }
        }));

        imp.selectorstyle_intersectingpath_toggle.connect_toggled(clone!(@weak appwindow => move |selectorstyle_intersectingpath_toggle| {
            if selectorstyle_intersectingpath_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().pens_config.selector_config.style = SelectorStyle::IntersectingPath;
            }
        }));

        imp.resize_lock_aspectratio_togglebutton.connect_toggled(clone!(@weak appwindow = > move |resize_lock_aspectratio_togglebutton| {
            appwindow.canvas().engine().borrow_mut().pens_config.selector_config.resize_lock_aspectratio = resize_lock_aspectratio_togglebutton.is_active();
        }));
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        let selector_config = appwindow
            .canvas()
            .engine()
            .borrow()
            .pens_config
            .selector_config
            .clone();

        match selector_config.style {
            SelectorStyle::Polygon => imp.selectorstyle_polygon_toggle.set_active(true),
            SelectorStyle::Rectangle => imp.selectorstyle_rect_toggle.set_active(true),
            SelectorStyle::Single => imp.selectorstyle_single_toggle.set_active(true),
            SelectorStyle::IntersectingPath => {
                imp.selectorstyle_intersectingpath_toggle.set_active(true)
            }
        }
        imp.resize_lock_aspectratio_togglebutton
            .set_active(selector_config.resize_lock_aspectratio);
    }
}
