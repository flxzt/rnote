use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, ToggleButton};

use crate::appwindow::RnoteAppWindow;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/selectorpage.ui")]
    pub struct SelectorPage {
        #[template_child]
        pub selectorstyle_polygon_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub selectorstyle_rect_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub resize_lock_aspectratio_togglebutton: TemplateChild<ToggleButton>,
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for SelectorPage {}
}

glib::wrapper! {
    pub struct SelectorPage(ObjectSubclass<imp::SelectorPage>)
        @extends gtk4::Widget;
}

impl Default for SelectorPage {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectorPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create SelectorPage")
    }

    pub fn selectorstyle_polygon_toggle(&self) -> ToggleButton {
        self.imp().selectorstyle_polygon_toggle.get()
    }

    pub fn selectorstyle_rect_toggle(&self) -> ToggleButton {
        self.imp().selectorstyle_rect_toggle.get()
    }

    pub fn resize_lock_aspectratio_togglebutton(&self) -> ToggleButton {
        self.imp().resize_lock_aspectratio_togglebutton.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        // selecting with Polygon / Rect toggles
        self.selectorstyle_polygon_toggle().connect_toggled(clone!(@weak appwindow => move |selectorstyle_polygon_toggle| {
            if selectorstyle_polygon_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "selector-style", Some(&"polygon".to_variant()));
            }
        }));

        self.selectorstyle_rect_toggle().connect_toggled(clone!(@weak appwindow => move |selectorstyle_rect_toggle| {
            if selectorstyle_rect_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "selector-style", Some(&"rectangle".to_variant()));
            }
        }));

        self.resize_lock_aspectratio_togglebutton().connect_toggled(clone!(@weak appwindow = > move |resize_lock_aspectratio_togglebutton| {
            appwindow.canvas().engine().borrow_mut().penholder.selector.resize_lock_aspectratio = resize_lock_aspectratio_togglebutton.is_active();
        }));
    }
}
