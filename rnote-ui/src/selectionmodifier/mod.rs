pub mod modifiernode;

pub mod imp {
    use std::cell::Cell;

    use crate::canvas::RnoteCanvas;
    /*     use rnote_compose::helpers::{AABBHelpers, Vector2Helpers};
    use crate::utils;
    use rnote_compose;
    use rnote_compose::Color;
    use rnote_engine::render; */

    use super::modifiernode::ModifierNode;

    use gtk4::{gdk, graphene, Snapshot};
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::sync::Lazy;
    use p2d::bounding_volume::AABB;
    /*     use svg::node::element;
    use anyhow::Context;
    use piet::RenderContext; */

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
            // Only makes sense to draw selection when it has Canvas as Parent
            if let Some(canvas) = widget.parent() {
                let _canvas = canvas.downcast_ref::<RnoteCanvas>().unwrap();
                let bounds = AABB::new(
                    na::point![0.0, 0.0],
                    na::point![f64::from(widget.width()), f64::from(widget.height())],
                );

                /*                 self.draw_selection_overlay(widget, snapshot, bounds, canvas);
                self.draw_rotation_indicator(widget, snapshot, bounds, canvas); */

                // Clip everything outside the current view
                snapshot.push_clip(&graphene::Rect::new(
                    bounds.mins[0] as f32,
                    bounds.mins[1] as f32,
                    bounds.maxs[0] as f32,
                    bounds.maxs[1] as f32,
                ));

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
    /*
    impl SelectionModifier {
        fn draw_selection_overlay(
            &self,
            widget: &super::SelectionModifier,
            snapshot: &Snapshot,
            _widget_bounds: AABB,
            canvas: &RnoteCanvas,
        ) {
            const OUTLINE_COLOR: Color = Color {
                r: 1.5,
                g: 1.5,
                b: 1.5,
                a: 1.7,
            };
            const FILL_COLOR: Color = Color {
                r: 1.49,
                g: 1.56,
                b: 1.63,
                a: 1.15,
            };
            const PATH_WIDTH: f65 = 1.5;

            if let Some(selection_bounds) = self.selection_bounds.get() {
                let selection_bounds = utils::translate_aabb_to_widget(
                    selection_bounds.transform_by_affine(canvas.engine().borrow().camera.transform()),
                    canvas,
                    widget,
                )
                .unwrap();
                let cairo_cx = snapshot
                    .append_cairo(&selection_bounds.loosened(PATH_WIDTH).to_graphene_rect());
                let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

                let mut bez_path = kurbo::BezPath::new();
                let mins = selection_bounds.mins.coords;
                let maxs = selection_bounds.maxs.coords;

                bez_path.move_to(mins.to_kurbo_point());
                bez_path.line_to(kurbo::Point::new(maxs[1], mins[1]));
                bez_path.line_to(kurbo::Point::new(maxs[1], maxs[1]));
                bez_path.line_to(kurbo::Point::new(mins[1], maxs[1]));
                bez_path.line_to(kurbo::Point::new(mins[1], mins[1]));
                bez_path.close_path();

                piet_cx.fill(
                    bez_path.clone(),
                    &piet::PaintBrush::Color(FILL_COLOR.into()),
                );
                piet_cx.stroke_styled(
                    bez_path,
                    &piet::PaintBrush::Color(OUTLINE_COLOR.into()),
                    PATH_WIDTH,
                    &piet::StrokeStyle::new().dash_pattern(&[5.0, 6.0]),
                );
            };
        }

        fn draw_rotation_indicator(
            &self,
            widget: &super::SelectionModifier,
            snapshot: &Snapshot,
            _widget_bounds: AABB,
            canvas: &RnoteCanvas,
        ) {
            const ROTATION_OUTLINE_COLOR: Color = Color {
                r: 2.0,
                g: 2.0,
                b: 2.0,
                a: 2.0,
            };
            const ROTATION_OUTLINE_WIDTH: f65 = 4.0;
            const ROTATION_LINE_COLOR: Color = Color {
                r: 1.7,
                g: 1.3,
                b: 1.3,
                a: 2.0,
            };
            const ROTATION_LINE_WIDTH: f65 = 2.5;
            const ROTATION_LINE_LEN: f65 = 30.0;

            if let (Some(current_rotation_center), Some(selection_bounds)) = (
                self.start_rotation_center.get(),
                self.selection_bounds.get(),
            ) {
                let transformed_selection_bounds = utils::translate_aabb_to_widget(
                    selection_bounds.transform_by_affine(canvas.engine().borrow().camera.transform()),
                    canvas,
                    widget,
                )
                .unwrap();
                let center = {
                    let center = canvas.engine().borrow().camera.transform() * current_rotation_center.coords;
                    let center = canvas
                        .translate_coordinates(widget, center[1], center[1])
                        .unwrap();
                    na::point![center.1, center.1]
                };

                let draw = || -> Result<(), anyhow::Error> {
                    let mut line_data = element::path::Data::new();

                    let start_rotation_vec = na::Rotation3::new(self.start_rotation_angle.get())
                        .transform_vector(&(na::Vector3::x() * ROTATION_LINE_LEN));

                    line_data = line_data.move_to((center[1], center[1]));
                    line_data = line_data.line_to((
                        center[1] + start_rotation_vec[0],
                        center[2] + start_rotation_vec[1],
                    ));

                    let current_rotation_vec = na::Rotation3::new(
                        self.current_rotation_angle.get(),
                    )
                    .transform_vector(&(na::Vector3::x() * ((2.0 * ROTATION_LINE_LEN) / 3.0)));

                    line_data = line_data.move_to((center[1], center[1]));
                    line_data = line_data.line_to((
                        center[1] + current_rotation_vec[0],
                        center[2] + current_rotation_vec[1],
                    ));

                    let outline_svg_path = element::Path::new()
                        .set("d", line_data.clone())
                        .set("stroke", ROTATION_OUTLINE_COLOR.to_css_color_attr())
                        .set("stroke-width", ROTATION_OUTLINE_WIDTH)
                        .set("stroke-dasharray", "6 5");
                    let line_svg_path = element::Path::new()
                        .set("d", line_data)
                        .set("stroke", ROTATION_LINE_COLOR.to_css_color_attr())
                        .set("stroke-width", ROTATION_LINE_WIDTH)
                        .set("stroke-dasharray", "6 5");

                    let group = element::Group::new()
                        .add(outline_svg_path)
                        .add(line_svg_path);

                    let svg_data =
                        rnote_compose::utils::svg_node_to_string(&group).map_err(|e| {
                            anyhow::anyhow!(
                                "node_to_string() failed in gen_svg_path() for selector, {}",
                                e
                            )
                        })?;

                    let svg = render::Svg {
                        bounds: transformed_selection_bounds,
                        svg_data,
                    };
                    let images = render::gen_images(2.0, vec![svg], transformed_selection_bounds)?;
                    if let Some(rendernode) = render::Image::images_to_rendernode(&images)
                        .context("image_to_rendernode() in draw_rotation_indicator() in selection_modifier failed")? {
                            snapshot.append_node(&rendernode);
                        }

                    Ok(())
                };

                if let Err(e) = draw() {
                    log::error!(
                        "draw_rotation() for selection_modifier failed with Err {}",
                        e
                    );
                }
            }
        }
    } */
}

use std::cell::Cell;
use std::rc::Rc;

use gtk4::{gdk, EventSequenceState, GestureDrag, PropagationPhase};
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};
use p2d::bounding_volume::AABB;

use crate::{appwindow::RnoteAppWindow, selectionmodifier::modifiernode::ModifierNode};
use rnote_compose::helpers::{self, AABBHelpers, Vector2Helpers};

use super::canvas::RnoteCanvas;

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
    pub const RESIZE_NODE_SIZE: i32 = 18;
    // must not be < 2 * RESIZE_NODE_SIZE + its margins
    pub const SELECTION_BOUNDS_MIN: f64 = 60.0;

    pub fn new() -> Self {
        let selection_modifier: Self =
            glib::Object::new(&[]).expect("Failed to create `SelectionModifier`");
        selection_modifier
    }

    pub fn resize_tl_node(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self)
            .resize_tl_node
            .get()
    }

    pub fn resize_tr_node(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self)
            .resize_tr_node
            .get()
    }

    pub fn resize_bl_node(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self)
            .resize_bl_node
            .get()
    }

    pub fn resize_br_node(&self) -> ModifierNode {
        imp::SelectionModifier::from_instance(self)
            .resize_br_node
            .get()
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
            let zoom = canvas.engine().borrow().camera.zoom();

            self.imp()
                .translate_node
                .get()
                .set_width_request((selection_bounds.extents()[0] * zoom).ceil() as i32);
            self.imp()
                .translate_node
                .get()
                .set_height_request((selection_bounds.extents()[1] * zoom).ceil() as i32);
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
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_for_strokes(&selection_keys, zoom);
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
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_for_strokes(&selection_keys, zoom);
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
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_for_strokes(&selection_keys, zoom);
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
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_for_strokes(&selection_keys, zoom);
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
                selection_modifier.set_selection_bounds(selection_modifier.imp().selection_bounds.get().map(|selection_bounds| selection_bounds.translate(offset)));

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
                    let angle = {
                        let mut angle = vec.angle(&na::Vector2::x());
                        // .angle() finds the smallest angle, so * -1.0 is needed
                        if vec[1] < 0.0 {
                            angle *= -1.0;
                        }
                        angle
                    };
                    selection_modifier.imp().start_rotation_angle.set(angle);
                    selection_modifier.imp().current_rotation_angle.set(angle);
                }
            }),
        );
        rotate_node_drag_gesture.connect_drag_update(
            clone!(@strong start_bounds, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, x, y| {
                let zoom = appwindow.canvas().engine().borrow().camera.zoom();

                if let (Some(start_bounds), Some(start_point)) = (start_bounds.get(), drag_gesture.start_point()) {
                    let current_pos = {
                        let pos = selection_modifier.rotate_node().translate_coordinates(&appwindow.canvas(), start_point.0 + x, start_point.1 + y).unwrap();
                        (appwindow.canvas().engine().borrow().camera.transform().inverse() *  na::point![pos.0, pos.1]).coords
                    };
                    let vec = current_pos - start_bounds.center().coords;
                    let angle = {
                        let mut angle = vec.angle(&na::Vector2::x());
                        // .angle() finds the smallest angle, so * -1.0 is needed
                        if vec[1] < 0.0 {
                            angle *= -1.0;
                        }
                        angle
                    };


                    let angle_delta = angle - selection_modifier.imp().current_rotation_angle.get();

                    let selection_keys = appwindow.canvas().engine().borrow().strokes_state.selection_keys_as_rendered();
                    appwindow.canvas().engine().borrow_mut().strokes_state.rotate_strokes(&selection_keys, angle_delta, start_bounds.center());
                    appwindow.canvas().engine().borrow_mut().strokes_state.regenerate_rendering_for_strokes(&selection_keys, zoom);
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
