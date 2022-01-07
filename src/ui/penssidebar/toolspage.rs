mod imp {
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate, ToggleButton};

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/toolspage.ui")]
    pub struct ToolsPage {
        #[template_child]
        pub toolstyle_expandsheet_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub toolstyle_dragproximity_toggle: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ToolsPage {
        const NAME: &'static str = "ToolsPage";
        type Type = super::ToolsPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ToolsPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ToolsPage {}
}

use crate::ui::appwindow::RnoteAppWindow;
use gtk4::{prelude::*, glib, glib::clone, subclass::prelude::*, Orientable, ToggleButton, Widget};

glib::wrapper! {
    pub struct ToolsPage(ObjectSubclass<imp::ToolsPage>)
        @extends Widget, @implements Orientable;
}

impl Default for ToolsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolsPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ToolsPage")
    }

    pub fn toolstyle_expandsheet_toggle(&self) -> ToggleButton {
        imp::ToolsPage::from_instance(self)
            .toolstyle_expandsheet_toggle
            .get()
    }

    pub fn toolstyle_dragproximity_toggle(&self) -> ToggleButton {
        imp::ToolsPage::from_instance(self)
            .toolstyle_dragproximity_toggle
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.toolstyle_expandsheet_toggle().connect_active_notify(clone!(@weak appwindow => move |toolstyle_expandsheet_toggle| {
            if toolstyle_expandsheet_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "tool-style", Some(&"expandsheet".to_variant()));
            }
        }));

        self.toolstyle_dragproximity_toggle().connect_active_notify(clone!(@weak appwindow => move |toolstyle_dragproximity_toggle| {
            if toolstyle_dragproximity_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "tool-style", Some(&"dragproximity".to_variant()));
            }
        }));
    }
}
