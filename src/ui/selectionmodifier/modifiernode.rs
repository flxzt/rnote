mod imp {
    use gtk4::{gdk, glib, prelude::*, subclass::prelude::*, BinLayout, Image};

    #[derive(Debug)]
    pub struct ModifierNode {
        pub image: Image,
    }

    impl Default for ModifierNode {
        fn default() -> Self {
            let image = Image::builder()
                .name("image")
                .icon_name("modifiernode-default-symbolic")
                .build();

            Self { image }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ModifierNode {
        const NAME: &'static str = "ModifierNode";
        type Type = super::ModifierNode;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<BinLayout>();
        }
    }

    impl ObjectImpl for ModifierNode {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.image.set_parent(obj);

            obj.set_css_classes(&["modifiernode"]);
            obj.set_cursor(gdk::Cursor::from_name("default", None).as_ref());
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for ModifierNode {}
}
use gtk4::{glib, subclass::prelude::*, Image};

glib::wrapper! {
    pub struct ModifierNode(ObjectSubclass<imp::ModifierNode>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for ModifierNode {
    fn default() -> Self {
        Self::new()
    }
}

impl ModifierNode {
    pub fn new() -> Self {
        let modifiernode: ModifierNode =
            glib::Object::new(&[]).expect("Failed to create SelectionModifier");
        modifiernode
    }

    pub fn image(&self) -> Image {
        imp::ModifierNode::from_instance(self).image.clone()
    }
}
