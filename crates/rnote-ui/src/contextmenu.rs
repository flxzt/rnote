// Imports
use gtk4::{CompositeTemplate, PopoverMenu, Widget, glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/contextmenu.ui")]
    pub(crate) struct RnContextMenu {
        #[template_child]
        pub(crate) popover: TemplateChild<PopoverMenu>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnContextMenu {
        const NAME: &'static str = "RnContextMenu";
        type Type = super::RnContextMenu;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnContextMenu {
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

    impl WidgetImpl for RnContextMenu {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.popover.get().present();
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnContextMenu(ObjectSubclass<imp::RnContextMenu>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl RnContextMenu {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn popover(&self) -> PopoverMenu {
        self.imp().popover.get()
    }
}
