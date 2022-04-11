mod modifiernode;

// Re-exports
pub use modifiernode::ModifierNode;

use super::canvas::RnoteCanvas;
use crate::appwindow::RnoteAppWindow;
use gtk4::{
    gdk, glib, glib::clone, graphene, prelude::*, subclass::prelude::*, CompositeTemplate,
    EventSequenceState, GestureDrag, PropagationPhase, Snapshot,
};
use once_cell::sync::Lazy;
use p2d::bounding_volume::BoundingVolume;
use p2d::bounding_volume::AABB;
use piet::RenderContext;
use rnote_compose::helpers::{self, AABBHelpers, Affine2Helpers, Vector2Helpers};
use rnote_compose::Color;
use rnote_engine::pens::Selector;
use rnote_engine::utils::GrapheneRectHelpers;
use rnote_engine::Camera;
use std::cell::Cell;
use std::rc::Rc;

pub mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/selectionmodifier.ui")]
    pub struct SelectionModifier {
        #[template_child]
        pub resize_tl_node: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_tr_node: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_bl_node: TemplateChild<ModifierNode>,
        #[template_child]
        pub resize_br_node: TemplateChild<ModifierNode>,
        #[template_child]
        pub translate_node: TemplateChild<gtk4::Box>,
        #[template_child]
        pub rotate_node: TemplateChild<ModifierNode>,

        pub resize_lock_aspectratio: Cell<bool>,

        // Internal state for allocation, drawing
        pub(super) selection_bounds: Cell<Option<AABB>>,
        pub(super) start_rotation_center: Cell<Option<na::Point2<f64>>>,
        pub(super) start_rotation_angle: Cell<f64>,
        pub(super) current_rotation_angle: Cell<f64>,
    }

    impl Default for SelectionModifier {
        fn default() -> Self {
            Self {
                resize_tl_node: TemplateChild::default(),
                resize_tr_node: TemplateChild::default(),
                resize_bl_node: TemplateChild::default(),
                resize_br_node: TemplateChild::default(),
                translate_node: TemplateChild::default(),
                rotate_node: TemplateChild::default(),

                resize_lock_aspectratio: Cell::new(false),

                selection_bounds: Cell::new(None),
                start_rotation_center: Cell::new(None),
                start_rotation_angle: Cell::new(0.0),
                current_rotation_angle: Cell::new(0.0),
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

            self.resize_tl_node
                .image()
                .set_icon_name(Some("modifiernode-resize-northwest-symbolic"));
            self.resize_tl_node
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_tr_node
                .image()
                .set_icon_name(Some("modifiernode-resize-northeast-symbolic"));
            self.resize_tr_node
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_bl_node
                .image()
                .set_icon_name(Some("modifiernode-resize-northeast-symbolic"));
            self.resize_bl_node
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.resize_br_node
                .image()
                .set_icon_name(Some("modifiernode-resize-northwest-symbolic"));
            self.resize_br_node
                .image()
                .set_pixel_size(super::SelectionModifier::RESIZE_NODE_SIZE);

            self.rotate_node
                .image()
                .set_icon_name(Some("modifiernode-rotate-symbolic"));
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

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    // The margin of the sheet in px when zoom = 1.0
                    glib::ParamSpecBoolean::new(
                        "resize-lock-aspectratio",
                        "resize-lock-aspectratio",
                        "resize-lock-aspectratio",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "resize-lock-aspectratio" => self.resize_lock_aspectratio.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "resize-lock-aspectratio" => {
                    let resize_locked_aspectratio = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`.");

                    self.resize_lock_aspectratio
                        .replace(resize_locked_aspectratio);
                }
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SelectionModifier {
        fn snapshot(&self, widget: &Self::Type, snapshot: &Snapshot) {
            //return;

            if let Some(canvas) = widget.parent() {
                let canvas = canvas.downcast_ref::<RnoteCanvas>().unwrap();
                let widget_bounds = AABB::new(
                    na::point![0.0, 0.0],
                    na::point![f64::from(widget.width()), f64::from(widget.height())],
                );

                let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_aabb(widget_bounds));
                let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

                let translation_to_canvas = widget.translate_coordinates(canvas, 0.0, 0.0).unwrap();
                let transform =
                    na::Translation2::new(-translation_to_canvas.0, -translation_to_canvas.1)
                        * canvas.engine().borrow().camera.transform();
                piet_cx.transform(transform.to_kurbo());

                piet_cx.save().unwrap();
                self.draw_selection_overlay(&mut piet_cx, &canvas.engine().borrow().camera);
                piet_cx.restore().unwrap();
                piet_cx.save().unwrap();
                self.draw_rotation_indicator(&mut piet_cx, &canvas.engine().borrow().camera);
                piet_cx.restore().unwrap();

                // Clip everything outside the current view
                snapshot.push_clip(&graphene::Rect::from_aabb(widget_bounds));

                widget.snapshot_child(&self.resize_tl_node.get(), snapshot);
                widget.snapshot_child(&self.resize_tr_node.get(), snapshot);
                widget.snapshot_child(&self.resize_bl_node.get(), snapshot);
                widget.snapshot_child(&self.resize_br_node.get(), snapshot);
                widget.snapshot_child(&self.rotate_node.get(), snapshot);
                widget.snapshot_child(&self.translate_node.get(), snapshot);

                snapshot.pop();
            }
        }
    }

    impl SelectionModifier {
        fn draw_selection_overlay(&self, piet_cx: &mut impl RenderContext, camera: &Camera) {
            let total_zoom = camera.total_zoom();

            if let Some(selection_bounds) = self.selection_bounds.get() {
                let rect = selection_bounds
                    .tightened(Selector::PATH_WIDTH / total_zoom)
                    .to_kurbo_rect();

                piet_cx.fill(
                    rect.clone(),
                    &piet::PaintBrush::Color(Selector::FILL_COLOR.into()),
                );
                piet_cx.stroke(
                    rect,
                    &piet::PaintBrush::Color(Selector::OUTLINE_COLOR.into()),
                    Selector::PATH_WIDTH / total_zoom,
                );
            };
        }

        fn draw_rotation_indicator(&self, piet_cx: &mut impl RenderContext, camera: &Camera) {
            const CENTER_CROSS_COLOR: Color = Color {
                r: 0.964,
                g: 0.380,
                b: 0.317,
                a: 1.0,
            };
            let total_zoom = camera.total_zoom();
            let center_cross_radius: f64 = 10.0 / total_zoom;
            let center_cross_path_width: f64 = 1.0 / total_zoom;

            if let (Some(rotation_center), Some(_selection_bounds)) = (
                self.start_rotation_center.get(),
                self.selection_bounds.get(),
            ) {
                let mut center_cross = kurbo::BezPath::new();
                center_cross.move_to(
                    (rotation_center.coords + na::vector![-center_cross_radius, 0.0])
                        .to_kurbo_point(),
                );
                center_cross.line_to(
                    (rotation_center.coords + na::vector![center_cross_radius, 0.0])
                        .to_kurbo_point(),
                );
                center_cross.move_to(
                    (rotation_center.coords + na::vector![0.0, -center_cross_radius])
                        .to_kurbo_point(),
                );
                center_cross.line_to(
                    (rotation_center.coords + na::vector![0.0, center_cross_radius])
                        .to_kurbo_point(),
                );

                piet_cx.save().unwrap();
                piet_cx.transform(
                    kurbo::Affine::translate(rotation_center.coords.to_kurbo_vec())
                        * kurbo::Affine::rotate(
                            self.current_rotation_angle.get() - self.start_rotation_angle.get(),
                        )
                        * kurbo::Affine::translate(-rotation_center.coords.to_kurbo_vec()),
                );

                piet_cx.stroke(
                    center_cross,
                    &piet::Color::from(CENTER_CROSS_COLOR),
                    center_cross_path_width,
                );
                piet_cx.restore().unwrap();
            }
        }
    }
}

glib::wrapper! {
    pub struct SelectionModifier(ObjectSubclass<imp::SelectionModifier>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for SelectionModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionModifier {
    pub const RESIZE_NODE_SIZE: i32 = 18;
    // must not be < 2 * RESIZE_NODE_SIZE + its margins
    pub const SELECTION_BOUNDS_MIN: f64 = 60.0;

    pub fn new() -> Self {
        let selection_modifier: Self =
            glib::Object::new(&[]).expect("Failed to create `SelectionModifier`");
        selection_modifier
    }

    pub fn resize_tl_node(&self) -> ModifierNode {
        self.imp().resize_tl_node.get()
    }

    pub fn resize_tr_node(&self) -> ModifierNode {
        self.imp().resize_tr_node.get()
    }

    pub fn resize_bl_node(&self) -> ModifierNode {
        self.imp().resize_bl_node.get()
    }

    pub fn resize_br_node(&self) -> ModifierNode {
        self.imp().resize_br_node.get()
    }

    pub fn rotate_node(&self) -> ModifierNode {
        self.imp().rotate_node.get()
    }

    pub fn translate_node(&self) -> gtk4::Box {
        self.imp().translate_node.get()
    }

    pub fn resize_lock_aspectratio(&self) -> bool {
        self.property::<bool>("resize-lock-aspectratio")
    }

    pub fn set_resize_lock_aspectratio(&self, resize_lock_aspectratio: bool) {
        self.set_property::<bool>("resize-lock-aspectratio", resize_lock_aspectratio);
    }

    pub fn selection_bounds(&self) -> Option<AABB> {
        self.imp().selection_bounds.get()
    }

    pub fn set_selection_bounds(&self, bounds: Option<AABB>) {
        self.imp()
            .selection_bounds
            .set(bounds.map(|bounds| AABB::new_positive(bounds.mins, bounds.maxs)));
    }

    /// Updates the internal state for measuring the widgets size, allocation, etc.
    pub fn update_state(&self, canvas: &RnoteCanvas) {
        self.set_selection_bounds(
            canvas
                .engine()
                .borrow()
                .strokes_state
                .gen_selection_bounds(),
        );
        self.set_visible(self.selection_bounds().is_some());

        if let Some(selection_bounds) = self.selection_bounds() {
            let zoom = canvas.engine().borrow().camera.zoom();

            self.imp()
                .translate_node
                .get()
                .set_width_request((selection_bounds.extents()[0] * zoom).round() as i32);
            self.imp()
                .translate_node
                .get()
                .set_height_request((selection_bounds.extents()[1] * zoom).round() as i32);
        };

        self.queue_resize();
    }

    pub fn update_translate_node_size_request(&self, canvas: &RnoteCanvas) {
        if let Some(selection_bounds) = self.selection_bounds() {
            let total_zoom = canvas.engine().borrow().camera.total_zoom();

            self.imp()
                .translate_node
                .get()
                .set_width_request((selection_bounds.extents()[0] * total_zoom).ceil() as i32);
            self.imp()
                .translate_node
                .get()
                .set_height_request((selection_bounds.extents()[1] * total_zoom).ceil() as i32);
        };

        self.queue_resize();
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
        let resize_tl_drag_gesture = GestureDrag::builder()
            .name("resize_tl_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.imp()
            .resize_tl_node
            .add_controller(&resize_tl_drag_gesture);

        let start_bounds: Rc<Cell<Option<AABB>>> = Rc::new(Cell::new(None));

        resize_tl_drag_gesture.connect_drag_begin(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
                start_bounds.set(selection_modifier.selection_bounds());

                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        resize_tl_drag_gesture.connect_drag_update(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, x, y| {
                if let (Some(selection_bounds), Some(start_bounds)) = (selection_modifier.selection_bounds(), start_bounds.get()) {
                    let image_scale = appwindow.canvas().engine().borrow().camera.image_scale();
                    let viewport_extended = appwindow.canvas().engine().borrow().camera.viewport_extended();
                    let zoom = appwindow.canvas().engine().borrow().camera.zoom();
                    let offset = na::vector![-x.round() / zoom, -y.round() / zoom];

                    // Lock aspectratio when property is set or with left click drag + ctrl
                    let new_extents = if selection_modifier.resize_lock_aspectratio()
                        || (drag_gesture.current_event_state() == gdk::ModifierType::BUTTON1_MASK | gdk::ModifierType::SHIFT_MASK) {
                            helpers::scale_w_locked_aspectratio(start_bounds.extents(), selection_bounds.extents() + offset)
                    } else {
                        selection_bounds.extents() + offset
                    };
                    let new_extents = new_extents.maxs(&na::Vector2::from_element(Self::SELECTION_BOUNDS_MIN / zoom));

                    let new_bounds = AABB::new(
                        na::point![
                            start_bounds.maxs[0] - new_extents[0],
                            start_bounds.maxs[1] - new_extents[1]
                        ],
                        na::point![
                            start_bounds.maxs[0],
                            start_bounds.maxs[1]
                        ]
                    );

                    let selection_keys = appwindow.canvas().engine().borrow().strokes_state.selection_keys_as_rendered();
                    appwindow.canvas().engine().borrow_mut().strokes_state.resize_strokes(&selection_keys, selection_bounds, new_bounds);
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_in_viewport_threaded(false, viewport_extended, image_scale);
                    selection_modifier.set_selection_bounds(Some(new_bounds));

                    selection_modifier.update_translate_node_size_request(&appwindow.canvas());
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_tl_drag_gesture.connect_drag_end(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |_drag_gesture, _x, _y| {
                start_bounds.set(None);

                appwindow.canvas().engine().borrow_mut().strokes_state.update_geometry_selection_strokes();
                appwindow.canvas().regenerate_content(false, true);

                selection_modifier.update_state(&appwindow.canvas());
                appwindow.canvas().queue_draw();
            }),
        );
    }

    pub fn init_resize_tr_node(&self, appwindow: &RnoteAppWindow) {
        let resize_tr_drag_gesture = GestureDrag::builder()
            .name("resize_tr_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.imp()
            .resize_tr_node
            .add_controller(&resize_tr_drag_gesture);

        let start_bounds: Rc<Cell<Option<AABB>>> = Rc::new(Cell::new(None));

        resize_tr_drag_gesture.connect_drag_begin(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
                start_bounds.set(selection_modifier.selection_bounds());

                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        resize_tr_drag_gesture.connect_drag_update(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, x, y| {
                if let (Some(selection_bounds), Some(start_bounds)) = (selection_modifier.selection_bounds(), start_bounds.get()) {
                    let image_scale = appwindow.canvas().engine().borrow().camera.image_scale();
                    let viewport_extended = appwindow.canvas().engine().borrow().camera.viewport_extended();
                    let zoom = appwindow.canvas().engine().borrow().camera.zoom();
                    let offset = na::vector![x.round() / zoom, -y.round() / zoom];

                    // Lock aspectratio when property is set or with left click drag + ctrl
                    let new_extents = if selection_modifier.resize_lock_aspectratio()
                        || (drag_gesture.current_event_state() == gdk::ModifierType::BUTTON1_MASK | gdk::ModifierType::SHIFT_MASK) {
                            helpers::scale_w_locked_aspectratio(start_bounds.extents(), selection_bounds.extents() + offset)
                    } else {
                        selection_bounds.extents() + offset
                    };
                    let new_extents = new_extents.maxs(&na::Vector2::from_element(Self::SELECTION_BOUNDS_MIN / zoom));

                    let new_bounds = AABB::new(
                        na::point![
                            start_bounds.mins[0],
                            start_bounds.maxs[1] - new_extents[1]
                        ],
                        na::point![
                            start_bounds.mins[0] + new_extents[0],
                            start_bounds.maxs[1]
                        ]
                    );

                    let selection_keys = appwindow.canvas().engine().borrow().strokes_state.selection_keys_as_rendered();
                    appwindow.canvas().engine().borrow_mut().strokes_state.resize_strokes(&selection_keys, selection_bounds, new_bounds);
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_in_viewport_threaded(false, viewport_extended, image_scale);
                    selection_modifier.set_selection_bounds(Some(new_bounds));

                    selection_modifier.update_translate_node_size_request(&appwindow.canvas());
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_tr_drag_gesture.connect_drag_end(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |_drag_gesture, _x, _y| {
                start_bounds.set(None);

                appwindow.canvas().engine().borrow_mut().strokes_state.update_geometry_selection_strokes();
                appwindow.canvas().regenerate_content(false, true);

                selection_modifier.update_state(&appwindow.canvas());
                appwindow.canvas().queue_draw();
            }),
        );
    }

    pub fn init_resize_bl_node(&self, appwindow: &RnoteAppWindow) {
        let resize_bl_drag_gesture = GestureDrag::builder()
            .name("resize_bl_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.imp()
            .resize_bl_node
            .add_controller(&resize_bl_drag_gesture);

        let start_bounds: Rc<Cell<Option<AABB>>> = Rc::new(Cell::new(None));

        resize_bl_drag_gesture.connect_drag_begin(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
                start_bounds.set(selection_modifier.selection_bounds());

                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        resize_bl_drag_gesture.connect_drag_update(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, x, y| {
                if let (Some(selection_bounds), Some(start_bounds)) = (selection_modifier.selection_bounds(), start_bounds.get()) {
                    let image_scale = appwindow.canvas().engine().borrow().camera.image_scale();
                    let viewport_extended = appwindow.canvas().engine().borrow().camera.viewport_extended();
                    let zoom = appwindow.canvas().engine().borrow().camera.zoom();
                    let offset = na::vector![-x.round() / zoom, y.round() / zoom];

                    // Lock aspectratio when property is set or with left click drag + ctrl
                    let new_extents = if selection_modifier.resize_lock_aspectratio()
                        || (drag_gesture.current_event_state() == gdk::ModifierType::BUTTON1_MASK | gdk::ModifierType::SHIFT_MASK) {
                            helpers::scale_w_locked_aspectratio(start_bounds.extents(), selection_bounds.extents() + offset)
                    } else {
                        selection_bounds.extents() + offset
                    };
                    let new_extents = new_extents.maxs(&na::Vector2::from_element(Self::SELECTION_BOUNDS_MIN / zoom));

                    let new_bounds = AABB::new(
                        na::point![
                            start_bounds.maxs[0] - new_extents[0],
                            start_bounds.mins[1]
                        ],
                        na::point![
                            start_bounds.maxs[0],
                            start_bounds.mins[1] + new_extents[1]
                        ]
                    );

                    let selection_keys = appwindow.canvas().engine().borrow().strokes_state.selection_keys_as_rendered();
                    appwindow.canvas().engine().borrow_mut().strokes_state.resize_strokes(&selection_keys, selection_bounds, new_bounds);
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_in_viewport_threaded(false, viewport_extended, image_scale);
                    selection_modifier.set_selection_bounds(Some(new_bounds));

                    selection_modifier.update_translate_node_size_request(&appwindow.canvas());
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_bl_drag_gesture.connect_drag_end(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |_drag_gesture, _x, _y| {
                start_bounds.set(None);

                appwindow.canvas().engine().borrow_mut().strokes_state.update_geometry_selection_strokes();
                appwindow.canvas().regenerate_content(false, true);

                selection_modifier.update_state(&appwindow.canvas());
                appwindow.canvas().queue_draw();
            }),
        );
    }

    pub fn init_resize_br_node(&self, appwindow: &RnoteAppWindow) {
        let resize_br_drag_gesture = GestureDrag::builder()
            .name("resize_br_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.imp()
            .resize_br_node
            .add_controller(&resize_br_drag_gesture);

        let start_bounds: Rc<Cell<Option<AABB>>> = Rc::new(Cell::new(None));

        resize_br_drag_gesture.connect_drag_begin(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
                start_bounds.set(selection_modifier.selection_bounds());

                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        resize_br_drag_gesture.connect_drag_update(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, x, y| {
                if let (Some(selection_bounds), Some(start_bounds)) = (selection_modifier.selection_bounds(), start_bounds.get()) {
                    let image_scale = appwindow.canvas().engine().borrow().camera.image_scale();
                    let viewport_extended = appwindow.canvas().engine().borrow().camera.viewport_extended();
                    let zoom = appwindow.canvas().engine().borrow().camera.zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

                    // Lock aspectratio when property is set or with left click drag + ctrl
                    let new_extents = if selection_modifier.resize_lock_aspectratio()
                        || (drag_gesture.current_event_state() == gdk::ModifierType::BUTTON1_MASK | gdk::ModifierType::SHIFT_MASK) {
                            helpers::scale_w_locked_aspectratio(start_bounds.extents(), selection_bounds.extents() + offset)
                    } else {
                        selection_bounds.extents() + offset
                    };
                    let new_extents = new_extents.maxs(&na::Vector2::from_element(Self::SELECTION_BOUNDS_MIN / zoom));

                    let new_bounds = AABB::new(
                        na::point![
                            start_bounds.mins[0],
                            start_bounds.mins[1]
                        ],
                        na::point![
                            start_bounds.mins[0] + new_extents[0],
                            start_bounds.mins[1] + new_extents[1]
                        ]
                    );

                    let selection_keys = appwindow.canvas().engine().borrow().strokes_state.selection_keys_as_rendered();
                    appwindow.canvas().engine().borrow_mut().strokes_state.resize_strokes(&selection_keys, selection_bounds, new_bounds);
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_in_viewport_threaded(false, viewport_extended, image_scale);
                    selection_modifier.set_selection_bounds(Some(new_bounds));

                    selection_modifier.update_translate_node_size_request(&appwindow.canvas());
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_br_drag_gesture.connect_drag_end(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |_drag_gesture, _x, _y| {
                start_bounds.set(None);

                appwindow.canvas().engine().borrow_mut().strokes_state.update_geometry_selection_strokes();
                appwindow.canvas().regenerate_content(false, true);

                selection_modifier.update_state(&appwindow.canvas());
                appwindow.canvas().queue_draw();
            }),
        );
    }

    pub fn init_translate_node(&self, appwindow: &RnoteAppWindow) {
        let translate_node_drag_gesture = GestureDrag::builder()
            .name("translate_drag")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.imp()
            .translate_node
            .add_controller(&translate_node_drag_gesture);

        translate_node_drag_gesture.connect_drag_begin(
            clone!(@weak self as selection_modifier, @weak appwindow => move |translate_node_drag_gesture, _x, _y| {
                translate_node_drag_gesture.set_state(EventSequenceState::Claimed);

                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        translate_node_drag_gesture.connect_drag_update(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_translate_node_drag_gesture, x, y| {
                let zoom = appwindow.canvas().engine().borrow().camera.zoom();
                let offset = na::vector![x.round() / zoom, y.round() / zoom];

                let selection_keys = appwindow.canvas().engine().borrow().strokes_state.selection_keys_as_rendered();
                appwindow.canvas().engine().borrow_mut().strokes_state.translate_strokes(&selection_keys, offset);
                selection_modifier.set_selection_bounds(selection_modifier.selection_bounds().map(|selection_bounds| selection_bounds.translate(offset)));

                selection_modifier.update_translate_node_size_request(&appwindow.canvas());
                appwindow.canvas().queue_draw();
            }),
        );
        translate_node_drag_gesture.connect_drag_end(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_translate_node_drag_gesture, _x, _y| {
                selection_modifier.update_state(&appwindow.canvas());
                appwindow.canvas().queue_draw();
            }),
        );
    }

    pub fn init_rotate_node(&self, appwindow: &RnoteAppWindow) {
        let rotate_node_drag_gesture = GestureDrag::builder()
            .name("rotate_node_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.imp()
            .rotate_node
            .add_controller(&rotate_node_drag_gesture);

        let start_bounds: Rc<Cell<Option<AABB>>> = Rc::new(Cell::new(None));

        rotate_node_drag_gesture.connect_drag_begin(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
                selection_modifier.update_state(&appwindow.canvas());

                start_bounds.set(selection_modifier.selection_bounds());
                if let (Some(start_bounds), Some(start_point)) = (start_bounds.get(), drag_gesture.start_point()) {
                    selection_modifier.imp().start_rotation_center.set(Some(start_bounds.center()));

                    let current_pos = {
                        let pos = selection_modifier.rotate_node().translate_coordinates(&appwindow.canvas(), start_point.0, start_point.1).unwrap();
                        (appwindow.canvas().engine().borrow().camera.transform().inverse() *  na::point![pos.0, pos.1]).coords
                    };
                    let vec = current_pos - start_bounds.center().coords;
                    let angle = na::Vector2::x().angle_ahead(&vec);

                    selection_modifier.imp().start_rotation_angle.set(angle);
                    selection_modifier.imp().current_rotation_angle.set(angle);
                }
            }),
        );
        rotate_node_drag_gesture.connect_drag_update(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, x, y| {
                let image_scale = appwindow.canvas().engine().borrow().camera.image_scale();
                let viewport_extended = appwindow.canvas().engine().borrow().camera.viewport_extended();

                if let (Some(start_bounds), Some(start_point)) = (start_bounds.get(), drag_gesture.start_point()) {
                    let current_pos = {
                        let pos = selection_modifier.rotate_node().translate_coordinates(&appwindow.canvas(), start_point.0 + x, start_point.1 + y).unwrap();
                        (appwindow.canvas().engine().borrow().camera.transform().inverse() *  na::point![pos.0, pos.1]).coords
                    };
                    let vec = current_pos - start_bounds.center().coords;
                    let angle = na::Vector2::x().angle_ahead(&vec);

                    let angle_delta = angle - selection_modifier.imp().current_rotation_angle.get();

                    let selection_keys = appwindow.canvas().engine().borrow().strokes_state.selection_keys_as_rendered();
                    appwindow.canvas().engine().borrow_mut().strokes_state.rotate_strokes(&selection_keys, angle_delta, start_bounds.center());
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_in_viewport_threaded(false, viewport_extended, image_scale);
                    selection_modifier.update_state(&appwindow.canvas());

                    selection_modifier.imp().current_rotation_angle.set(angle);

                    selection_modifier.update_translate_node_size_request(&appwindow.canvas());
                    appwindow.canvas().queue_draw();
                }
            }),
        );
        rotate_node_drag_gesture.connect_drag_end(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_drag_gesture, _x, _y| {
                selection_modifier.imp().start_rotation_center.set(None);
                selection_modifier.imp().start_rotation_angle.set(0.0);
                selection_modifier.imp().current_rotation_angle.set(0.0);

                selection_modifier.update_state(&appwindow.canvas());
                appwindow.canvas().queue_draw();
            }),
        );
    }
}
