mod imp {
    use gtk4::ToggleButton;
    use gtk4::{glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate};

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/selectorpage.ui")]
    pub struct SelectorPage {
        #[template_child]
        pub selectorstyle_polygon_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub selectorstyle_rect_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub resize_lock_aspectratio_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub delete_button: TemplateChild<Button>,
        #[template_child]
        pub duplicate_button: TemplateChild<Button>,
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

use crate::pens::selector::Selector;
use crate::ui::appwindow::RnoteAppWindow;
use gtk4::ToggleButton;
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, Button, Orientable, Widget};

glib::wrapper! {
    pub struct SelectorPage(ObjectSubclass<imp::SelectorPage>)
        @extends Widget, @implements Orientable;
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
        imp::SelectorPage::from_instance(self)
            .selectorstyle_polygon_toggle
            .get()
    }

    pub fn selectorstyle_rect_toggle(&self) -> ToggleButton {
        imp::SelectorPage::from_instance(self)
            .selectorstyle_rect_toggle
            .get()
    }

    pub fn resize_lock_aspectratio_togglebutton(&self) -> ToggleButton {
        imp::SelectorPage::from_instance(self)
            .resize_lock_aspectratio_togglebutton
            .get()
    }

    pub fn delete_button(&self) -> Button {
        imp::SelectorPage::from_instance(self).delete_button.get()
    }

    pub fn duplicate_button(&self) -> Button {
        imp::SelectorPage::from_instance(self)
            .duplicate_button
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        // selecting with Polygon / Rect toggles
        self.selectorstyle_polygon_toggle().connect_active_notify(clone!(@weak appwindow => move |selectorstyle_polygon_toggle| {
            if selectorstyle_polygon_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "selector-style", Some(&"polygon".to_variant()));
            }
        }));

        self.selectorstyle_rect_toggle().connect_active_notify(clone!(@weak appwindow => move |selectorstyle_rect_toggle| {
            if selectorstyle_rect_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "selector-style", Some(&"rectangle".to_variant()));
            }
        }));

        self.resize_lock_aspectratio_togglebutton()
            .bind_property(
                "active",
                &appwindow.canvas().selection_modifier(),
                "resize-lock-aspectratio",
            )
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        self.delete_button()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "delete-selection", None);
            }));

        self.duplicate_button().connect_clicked(clone!(@weak appwindow => move |_| {
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "duplicate-selection", None);
        }));
    }

    pub fn load_from_selector(&self, _selector: Selector) {}
}
