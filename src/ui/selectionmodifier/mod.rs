pub mod modifiernode;

pub mod imp {
    use std::cell::Cell;

    use super::modifiernode::ModifierNode;

    use gtk4::gdk;
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/selectionmodifier.ui")]
    pub struct SelectionModifier {
        pub bounds: Cell<Option<p2d::bounding_volume::AABB>>,

        #[template_child]
        pub resize_tl: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_tr: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_bl: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_br: TemplateChild<ModifierNode>,
        #[template_child]
        pub translate_node: TemplateChild<gtk4::Box>,
        #[template_child]
        pub rotate_node: TemplateChild<ModifierNode>,
    }

    impl Default for SelectionModifier {
        fn default() -> Self {
            Self {
                bounds: Cell::new(None),
                resize_tl: TemplateChild::default(),
                resize_tr: TemplateChild::default(),
                resize_bl: TemplateChild::default(),
                resize_br: TemplateChild::default(),
                translate_node: TemplateChild::default(),
                rotate_node: TemplateChild::default(),
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

            obj.set_focusable(true);

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

            self.rotate_node
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.translate_node.set_cursor(
                gdk::Cursor::from_name("grab", gdk::Cursor::from_name("default", None).as_ref())
                    .as_ref(),
            );
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for SelectionModifier {}
}

use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};
use gtk4::{EventSequenceState, GestureDrag, PropagationPhase};

use crate::compose::geometry;
use crate::{ui::appwindow::RnoteAppWindow, ui::selectionmodifier::modifiernode::ModifierNode};

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
    pub const RESIZE_NODE_SIZE: i32 = 22;
    pub const SELECTION_MIN: f64 = 3.0; // Must be >= TRANSLATE_NODE_SIZE_MIN

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

    pub fn rotate_node(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self)
            .rotate_node
            .get()
    }

    pub fn translate_node(&self) -> gtk4::Box {
        imp::SelectionModifier::from_instance(self)
            .translate_node
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.init_resize_tl_node(appwindow);
        self.init_resize_tr_node(appwindow);
        self.init_resize_bl_node(appwindow);
        self.init_resize_br_node(appwindow);
        self.init_rotate_node(appwindow);
        self.init_translate_node(appwindow);
    }

    pub fn init_resize_tl_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let resize_tl_drag_gesture = GestureDrag::builder()
            .name("resize_tl_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.resize_tl.add_controller(&resize_tl_drag_gesture);

        resize_tl_drag_gesture.connect_drag_begin(
            clone!(@weak self as obj, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
            }),
        );
        resize_tl_drag_gesture.connect_drag_update(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, x, y| {
                let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;
                if let Some(selection_bounds) = selection_bounds {
                    let zoom = appwindow.canvas().zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

                    let new_bounds = p2d::bounding_volume::AABB::new(
                        na::point![
                        selection_bounds.mins[0] + offset[0], selection_bounds.mins[1] + offset[1]],
                        na::point![selection_bounds.maxs[0], selection_bounds.maxs[1]]
                    );
                    let min_bounds = p2d::bounding_volume::AABB::new(
                        na::point![
                            new_bounds.maxs[0] - Self::SELECTION_MIN,
                            new_bounds.maxs[1] - Self::SELECTION_MIN
                        ],
                        na::point![
                            new_bounds.maxs[0],
                            new_bounds.maxs[1]
                        ]
                    );
                    let selection_keys = appwindow.canvas().sheet().strokes_state().borrow().selection_keys();

                    let new_bounds = geometry::aabb_clamp(new_bounds, Some(min_bounds), None);
                    appwindow.canvas().sheet().strokes_state().borrow_mut().resize_strokes(&selection_keys, selection_bounds, new_bounds);
                    appwindow.canvas().sheet().strokes_state().borrow_mut().selection_bounds = Some(new_bounds);

                    obj.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_tl_drag_gesture.connect_drag_end(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, _x, _y| {
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_geometry_selection_strokes();
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_selection_bounds();
                appwindow.canvas().regenerate_content(false, true);
            }),
        );
    }

    pub fn init_resize_tr_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let resize_tr_drag_gesture = GestureDrag::builder()
            .name("resize_tr_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.resize_tr.add_controller(&resize_tr_drag_gesture);

        resize_tr_drag_gesture.connect_drag_begin(
            clone!(@weak self as obj, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
            }),
        );
        resize_tr_drag_gesture.connect_drag_update(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, x, y| {
                let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;
                if let Some(selection_bounds) = selection_bounds {
                    let zoom = appwindow.canvas().zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

                    let new_bounds = p2d::bounding_volume::AABB::new(
                        na::point![
                        selection_bounds.mins[0], selection_bounds.mins[1] + offset[1]],
                        na::point![selection_bounds.maxs[0] + offset[0], selection_bounds.maxs[1]]
                    );
                    let min_bounds = p2d::bounding_volume::AABB::new(
                        na::point![
                            new_bounds.mins[0],
                            new_bounds.maxs[1] - Self::SELECTION_MIN
                        ],
                        na::point![
                            new_bounds.mins[0] + Self::SELECTION_MIN,
                            new_bounds.maxs[1]
                        ]
                    );
                    let selection_keys = appwindow.canvas().sheet().strokes_state().borrow().selection_keys();

                    let new_bounds = geometry::aabb_clamp(new_bounds, Some(min_bounds), None);
                    appwindow.canvas().sheet().strokes_state().borrow_mut().resize_strokes(&selection_keys, selection_bounds, new_bounds);
                    appwindow.canvas().sheet().strokes_state().borrow_mut().selection_bounds = Some(new_bounds);

                    obj.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            }),
        );
        resize_tr_drag_gesture.connect_drag_end(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, _x, _y| {
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_geometry_selection_strokes();
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_selection_bounds();
                appwindow.canvas().regenerate_content(false, true);
            }),
        );
    }

    pub fn init_resize_bl_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let resize_bl_drag_gesture = GestureDrag::builder()
            .name("resize_bl_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.resize_bl.add_controller(&resize_bl_drag_gesture);

        resize_bl_drag_gesture.connect_drag_begin(
            clone!(@weak self as obj, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
            }),
        );
        resize_bl_drag_gesture.connect_drag_update(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, x, y| {

                let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;
                if let Some(selection_bounds) = selection_bounds {
                    let zoom = appwindow.canvas().zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

                    let new_bounds = p2d::bounding_volume::AABB::new(
                        na::point![
                        selection_bounds.mins[0] + offset[0], selection_bounds.mins[1]],
                        na::point![selection_bounds.maxs[0], selection_bounds.maxs[1] + offset[1]]
                    );
                    let min_bounds = p2d::bounding_volume::AABB::new(
                        na::point![
                            new_bounds.maxs[0] - Self::SELECTION_MIN,
                            new_bounds.mins[1]
                        ],
                        na::point![
                            new_bounds.maxs[0],
                            new_bounds.mins[1] + Self::SELECTION_MIN
                        ]
                    );
                    let selection_keys = appwindow.canvas().sheet().strokes_state().borrow().selection_keys();

                    let new_bounds = geometry::aabb_clamp(new_bounds, Some(min_bounds), None);
                    appwindow.canvas().sheet().strokes_state().borrow_mut().resize_strokes(&selection_keys, selection_bounds, new_bounds);
                    appwindow.canvas().sheet().strokes_state().borrow_mut().selection_bounds = Some(new_bounds);

                    obj.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            }),
        );
        resize_bl_drag_gesture.connect_drag_end(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, _x, _y| {
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_geometry_selection_strokes();
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_selection_bounds();
                appwindow.canvas().regenerate_content(false, true);
            }),
        );
    }

    pub fn init_resize_br_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let resize_br_drag_gesture = GestureDrag::builder()
            .name("resize_br_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.resize_br.add_controller(&resize_br_drag_gesture);

        resize_br_drag_gesture.connect_drag_begin(
            clone!(@weak self as obj, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
            }),
        );
        resize_br_drag_gesture.connect_drag_update(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, x, y| {
                let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;
                if let Some(selection_bounds) = selection_bounds {
                    let zoom = appwindow.canvas().zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

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
                            new_bounds.mins[0] + Self::SELECTION_MIN,
                            new_bounds.mins[1] + Self::SELECTION_MIN
                        ]
                    );
                    let selection_keys = appwindow.canvas().sheet().strokes_state().borrow().selection_keys();

                    let new_bounds = geometry::aabb_clamp(new_bounds, Some(min_bounds), None);
                    appwindow.canvas().sheet().strokes_state().borrow_mut().resize_strokes(&selection_keys, selection_bounds, new_bounds);
                    appwindow.canvas().sheet().strokes_state().borrow_mut().selection_bounds = Some(new_bounds);

                    obj.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            }),
        );
        resize_br_drag_gesture.connect_drag_end(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, _x, _y| {
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_geometry_selection_strokes();
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_selection_bounds();
                appwindow.canvas().regenerate_content(false, true);
            }),
        );
    }

    pub fn init_translate_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let translate_node_drag_gesture = GestureDrag::builder()
            .name("translate_drag")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_
            .translate_node
            .add_controller(&translate_node_drag_gesture);

        translate_node_drag_gesture.connect_drag_begin(
            clone!(@weak self as obj, @weak appwindow => move |translate_node_drag_gesture, _x, _y| {
                translate_node_drag_gesture.set_state(EventSequenceState::Claimed);
            }),
        );
        translate_node_drag_gesture.connect_drag_update(
            clone!(@weak self as obj, @weak appwindow => move |_translate_node_drag_gesture, x, y| {
                let zoom = appwindow.canvas().zoom();
                let offset = na::vector![x.round() / zoom, y.round() / zoom];

                let selection_keys = appwindow.canvas().sheet().strokes_state().borrow().selection_keys();
                appwindow.canvas().sheet().strokes_state().borrow_mut().translate_strokes(&selection_keys, offset);
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_selection_bounds();

                obj.queue_resize();
                appwindow.canvas().queue_draw();
            }),
        );
        translate_node_drag_gesture.connect_drag_end(
            clone!(@weak self as obj, @weak appwindow => move |_translate_node_drag_gesture, _x, _y| {
            }),
        );
    }

    pub fn init_rotate_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let rotate_node_drag_gesture = GestureDrag::builder()
            .name("rotate_node_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.rotate_node.add_controller(&rotate_node_drag_gesture);

        rotate_node_drag_gesture.connect_drag_begin(
            clone!(@weak self as obj, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
            }),
        );
        rotate_node_drag_gesture.connect_drag_update(
            clone!(@weak self as obj, @weak appwindow => move |_rotate_node_drag_gesture, x, y| {
                let selection_bounds = appwindow.canvas().sheet().strokes_state().borrow().selection_bounds;

                if let Some(selection_bounds) = selection_bounds {
                    let angle = na::vector![x, y].magnitude() / (100.0 * std::f64::consts::PI * 2.0);

                    let selection_keys = appwindow.canvas().sheet().strokes_state().borrow().selection_keys();
                    appwindow.canvas().sheet().strokes_state().borrow_mut().rotate_strokes(&selection_keys, angle, selection_bounds.center());
                    appwindow.canvas().sheet().strokes_state().borrow_mut().update_selection_bounds();

                    obj.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            }),
        );
        rotate_node_drag_gesture.connect_drag_end(
            clone!(@weak self as obj, @weak appwindow => move |_drag_gesture, _x, _y| {
            }),
        );
    }
}
