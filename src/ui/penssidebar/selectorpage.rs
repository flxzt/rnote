mod imp {
    use gtk4::{glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/selectorpage.ui")]
    pub struct SelectorPage {
        #[template_child]
        pub delete_button: TemplateChild<Button>,
    }

    impl Default for SelectorPage {
        fn default() -> Self {
            Self {
                delete_button: TemplateChild::<Button>::default(),
            }
        }
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

use crate::app::RnoteApp;
use crate::ui::appwindow::RnoteAppWindow;
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

    pub fn delete_button(&self) -> Button {
        imp::SelectorPage::from_instance(self).delete_button.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.delete_button().connect_clicked(clone!(@weak appwindow => move |_| {
            appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().activate_action("delete-selection", None);
        }));
    }
}
