mod imp {
    use gtk4::{glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/canvassettings.ui")]
    pub struct CanvasSettings {
        #[template_child]
        pub settings_closebutton: TemplateChild<Button>,
        #[template_child]
        pub format_chooser: TemplateChild<adw::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasSettings {
        const NAME: &'static str = "CanvasSettings";
        type Type = super::CanvasSettings;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CanvasSettings {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for CanvasSettings {}

    impl CanvasSettings {}
}

use gtk4::{glib, Widget};

use super::appwindow::RnoteAppWindow;

glib::wrapper! {
    pub struct CanvasSettings(ObjectSubclass<imp::CanvasSettings>)
    @extends Widget;
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasSettings {
    pub fn new() -> Self {
        let canvasmenu: CanvasSettings =
            glib::Object::new(&[]).expect("Failed to create CanvasSettings");
        canvasmenu
    }

    pub fn init(&self, _appwindow: &RnoteAppWindow) {}
}
