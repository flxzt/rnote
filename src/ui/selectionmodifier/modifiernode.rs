mod imp {
    use gtk4::{
        glib, glib::clone, glib::subclass::*, prelude::*, subclass::prelude::*, BinLayout,
        GestureDrag, Image,
    };
    use once_cell::sync::Lazy;

    use crate::utils;

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

            let gesture_drag = GestureDrag::new();

            gesture_drag.connect_drag_begin(clone!(@weak obj => move |_gesture_drag, x, y| {
                obj.emit_by_name("offset-begin", &[&utils::BoxedPos {x, y} ]).unwrap();
            }));
            gesture_drag.connect_drag_update(clone!(@weak obj => move |_gesture_drag, x, y| {
                obj.emit_by_name("offset-update", &[&utils::BoxedPos {x, y} ]).unwrap();
            }));
            gesture_drag.connect_drag_end(clone!(@weak obj => move |_gesture_drag, x, y| {
                obj.emit_by_name("offset-end", &[&utils::BoxedPos {x, y} ]).unwrap();
            }));

            obj.add_controller(&gesture_drag);
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    // sends absolute coordinates when offset begins
                    Signal::builder(
                        // Signal name
                        "offset-begin",
                        // Types of the values which will be sent to the signal handler
                        &[utils::BoxedPos::static_type().into()],
                        // Type of the value the signal handler sends back
                        <()>::static_type().into(),
                    )
                    .build(),
                    // sends relative coordinates to offset start
                    Signal::builder(
                        // Signal name
                        "offset-update",
                        // Types of the values which will be sent to the signal handler
                        &[utils::BoxedPos::static_type().into()],
                        // Type of the value the signal handler sends back
                        <()>::static_type().into(),
                    )
                    .build(),
                    // sends relative coordinates to offset start
                    Signal::builder(
                        // Signal name
                        "offset-end",
                        // Types of the values which will be sent to the signal handler
                        &[utils::BoxedPos::static_type().into()],
                        // Type of the value the signal handler sends back
                        <()>::static_type().into(),
                    )
                    .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for ModifierNode {}
}
use gtk4::{glib, subclass::prelude::*, Image, Widget};

glib::wrapper! {
    pub struct ModifierNode(ObjectSubclass<imp::ModifierNode>)
        @extends Widget;
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
