pub mod modifiernode;

mod imp {
    use std::cell::Cell;

    use super::modifiernode::ModifierNode;
    use crate::ui::canvas::Canvas;

    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::sync::Lazy;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/selectionmodifier.ui")]
    pub struct SelectionModifier {
        #[template_child]
        pub resize_tl: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_tr: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_bl: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_br: TemplateChild<ModifierNode>,
        #[template_child]
        pub translate_node: TemplateChild<ModifierNode>,

        pub scalefactor: Cell<f64>,
    }

    impl Default for SelectionModifier {
        fn default() -> Self {
            ModifierNode::static_type();

            Self {
                resize_tl: TemplateChild::<ModifierNode>::default(),
                resize_tr: TemplateChild::<ModifierNode>::default(),
                resize_bl: TemplateChild::<ModifierNode>::default(),
                resize_br: TemplateChild::<ModifierNode>::default(),
                translate_node: TemplateChild::<ModifierNode>::default(),
                scalefactor: Cell::new(Canvas::SCALE_DEFAULT),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SelectionModifier {
        const NAME: &'static str = "SelectionModifier";
        type Type = super::SelectionModifier;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SelectionModifier {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.resize_tl
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_tr
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_bl
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_br
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.translate_node
                .image()
                .set_icon_name(Some("selection-translate-symbolic"));

            self.translate_node
                .get()
                .set_margin_start(super::SelectionModifier::TRANSLATE_NODE_MARGIN);
            self.translate_node
                .get()
                .set_margin_end(super::SelectionModifier::TRANSLATE_NODE_MARGIN);
            self.translate_node
                .get()
                .set_margin_top(super::SelectionModifier::TRANSLATE_NODE_MARGIN);
            self.translate_node
                .get()
                .set_margin_bottom(super::SelectionModifier::TRANSLATE_NODE_MARGIN);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_double(
                    // Name
                    "scalefactor",
                    // Nickname
                    "scalefactor",
                    // Short description
                    "scalefactor",
                    // Minimum value
                    f64::MIN,
                    // Maximum value
                    f64::MAX,
                    // Default value
                    Canvas::SCALE_DEFAULT,
                    // The property can be read and written to
                    glib::ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "scalefactor" => self.scalefactor.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "scalefactor" => {
                    let scalefactor: f64 = value
                        .get::<f64>()
                        .expect("The value needs to be of type `i32`.")
                        .clamp(Canvas::SCALE_MIN, Canvas::SCALE_MAX);
                    self.scalefactor.replace(scalefactor);

                    obj.queue_draw();
                    obj.queue_resize();
                }
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SelectionModifier {}
}

use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};

use crate::{
    ui::appwindow::RnoteAppWindow, ui::selectionmodifier::modifiernode::ModifierNode, utils,
};

glib::wrapper! {
    pub struct SelectionModifier(ObjectSubclass<imp::SelectionModifier>)
        @extends gtk4::Widget;
}

impl Default for SelectionModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionModifier {
    pub const TRANSLATE_NODE_MARGIN: i32 = 1;
    pub const TRANSLATE_NODE_SIZE_MIN: i32 = 1;
    pub const TRANSLATE_NODE_SIZE_MAX: i32 = 256;
    pub const RESIZE_NODE_SIZE: i32 = 18;
    pub const RESIZE_MIN: f64 = 3.0; // Must be >= TRANSLATE_NODE_SIZE_MIN + 2 * TRANSLATE_NODE_MARGIN

    pub fn new() -> Self {
        let selection_modifier: Self =
            glib::Object::new(&[]).expect("Failed to create `SelectionModifier`");
        selection_modifier
    }

    pub fn resize_tl(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self).resize_tl.get()
    }

    pub fn resize_tr(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self).resize_tr.get()
    }

    pub fn resize_bl(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self).resize_bl.get()
    }

    pub fn resize_br(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self).resize_br.get()
    }

    pub fn translate_node(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self)
            .translate_node
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        self.bind_property("visible", &appwindow.canvas().sheet().selection(), "shown")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        priv_
            .resize_tl
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow  => @default-return None, move |args| {

                    let selection_bounds = appwindow.canvas().sheet().selection().bounds().borrow().to_owned();
                    if let Some(selection_bounds) = selection_bounds {
                        let scalefactor = appwindow.canvas().scalefactor();
                        let offset = args[1].get::<utils::BoxedPos>().unwrap();
                        let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                        let new_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                            selection_bounds.mins[0] + offset[0], selection_bounds.mins[1] + offset[1]],
                            na::point![selection_bounds.maxs[0], selection_bounds.maxs[1]]
                        );
                        let min_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                                new_bounds.maxs[0] - Self::RESIZE_MIN,
                                new_bounds.maxs[1] - Self::RESIZE_MIN
                            ],
                            na::point![
                                new_bounds.maxs[0],
                                new_bounds.maxs[1]
                            ]
                        );
                        let new_bounds = utils::aabb_clamp(new_bounds, Some(min_bounds), None);

                        appwindow.canvas().sheet().selection().resize_selection(new_bounds);
                    }
                    None
                }),
            )
            .unwrap();

        priv_
            .resize_tr
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow => @default-return None, move |args| {

                    let selection_bounds = appwindow.canvas().sheet().selection().bounds().borrow().to_owned();
                    if let Some(selection_bounds) = selection_bounds {
                        let scalefactor = appwindow.canvas().scalefactor();
                        let offset = args[1].get::<utils::BoxedPos>().unwrap();
                        let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                        let new_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                            selection_bounds.mins[0], selection_bounds.mins[1] + offset[1]],
                            na::point![selection_bounds.maxs[0] + offset[0], selection_bounds.maxs[1]]
                        );
                        let min_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                                new_bounds.mins[0],
                                new_bounds.maxs[1] - Self::RESIZE_MIN
                            ],
                            na::point![
                                new_bounds.mins[0] + Self::RESIZE_MIN,
                                new_bounds.maxs[1]
                            ]
                        );
                        let new_bounds = utils::aabb_clamp(new_bounds, Some(min_bounds), None);

                        appwindow.canvas().sheet().selection().resize_selection(new_bounds);
                    }
                    None
                }),
            )
            .unwrap();

        priv_
            .resize_bl
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow => @default-return None, move |args| {

                    let selection_bounds = appwindow.canvas().sheet().selection().bounds().borrow().to_owned();
                    if let Some(selection_bounds) = selection_bounds {
                        let scalefactor = appwindow.canvas().scalefactor();
                        let offset = args[1].get::<utils::BoxedPos>().unwrap();
                        let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                        let new_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                            selection_bounds.mins[0] + offset[0], selection_bounds.mins[1]],
                            na::point![selection_bounds.maxs[0], selection_bounds.maxs[1] + offset[1]]
                        );
                        let min_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                                new_bounds.maxs[0] - Self::RESIZE_MIN,
                                new_bounds.mins[1]
                            ],
                            na::point![
                                new_bounds.maxs[0],
                                new_bounds.mins[1] + Self::RESIZE_MIN
                            ]
                        );
                        let new_bounds = utils::aabb_clamp(new_bounds, Some(min_bounds), None);

                        appwindow.canvas().sheet().selection().resize_selection(new_bounds);
                    }
                    None
                }),
            )
            .unwrap();

        priv_
            .resize_br
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow => @default-return None, move |args| {

                    let selection_bounds = appwindow.canvas().sheet().selection().bounds().borrow().to_owned();
                    if let Some(selection_bounds) = selection_bounds {
                        let scalefactor = appwindow.canvas().scalefactor();
                        let offset = args[1].get::<utils::BoxedPos>().unwrap();
                        let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                        let new_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                            selection_bounds.mins[0], selection_bounds.mins[1]],
                            na::point![selection_bounds.maxs[0] + offset[0], selection_bounds.maxs[1] + offset[1]]
                        );
                        let min_bounds = p2d::bounding_volume::AABB::new(
                            na::point![
                                new_bounds.mins[0],
                                new_bounds.mins[1]
                            ],
                            na::point![
                                new_bounds.mins[0] + Self::RESIZE_MIN,
                                new_bounds.mins[1] + Self::RESIZE_MIN
                            ]
                        );
                        let new_bounds = utils::aabb_clamp(new_bounds, Some(min_bounds), None);

                        appwindow.canvas().sheet().selection().resize_selection(new_bounds);
                    }
                    None
                }),
            )
            .unwrap();

        priv_
            .translate_node
            .get()
            .connect_local(
                "offset-update",
                false,
                clone!(@weak self as obj, @weak appwindow => @default-return None, move |args| {
                    let scalefactor = appwindow.canvas().scalefactor();
                    let offset = args[1].get::<utils::BoxedPos>().unwrap();
                    let offset = na::vector![offset.x.round() / scalefactor, offset.y.round() / scalefactor];

                    appwindow.canvas().sheet().selection().translate_selection(offset);
                    None
                }),
            )
            .unwrap();
    }
}
