mod imp {
    use gtk4::{glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/settingspanel.ui")]
    pub struct SettingsPanel {
        #[template_child]
        pub settings_closebutton: TemplateChild<Button>,
        #[template_child]
        pub format_chooser: TemplateChild<adw::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsPanel {
        const NAME: &'static str = "SettingsPanel";
        type Type = super::SettingsPanel;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsPanel {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for SettingsPanel {}

    impl SettingsPanel {}
}

use gtk4::{glib, Widget};

use super::appwindow::RnoteAppWindow;

glib::wrapper! {
    pub struct SettingsPanel(ObjectSubclass<imp::SettingsPanel>)
    @extends Widget;
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsPanel {
    pub fn new() -> Self {
        let canvasmenu: SettingsPanel =
            glib::Object::new(&[]).expect("Failed to create SettingsPanel");
        canvasmenu
    }

    pub fn init(&self, _appwindow: &RnoteAppWindow) {}
}
