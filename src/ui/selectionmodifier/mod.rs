pub mod modifiernode;

pub mod imp {
    use std::cell::Cell;

    use crate::compose::geometry;
    use crate::ui::canvas::Canvas;
    use crate::{compose, render, utils};

    use super::modifiernode::ModifierNode;

    use anyhow::Context;
    use gtk4::{gdk, graphene, Orientation, SizeRequestMode, Snapshot};
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use p2d::bounding_volume::AABB;
    use svg::node::element;

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

        // Internal state for allocation, drawing
        pub(super) selection_bounds: Cell<Option<AABB>>,
        pub(super) current_rotation_center: Cell<Option<na::Point2<f64>>>,
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

                selection_bounds: Cell::new(None),
                current_rotation_center: Cell::new(None),
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
    }
    impl WidgetImpl for SelectionModifier {
        fn request_mode(&self, _widget: &Self::Type) -> SizeRequestMode {
            SizeRequestMode::ConstantSize
        }

        fn measure(
            &self,
            widget: &Self::Type,
            orientation: Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            // Only makes sense to draw selection when it has Canvas as Parent
            if let (Some(canvas), Some(selection_bounds)) =
                (widget.parent(), self.selection_bounds.get())
            {
                let canvas = canvas.downcast_ref::<Canvas>().unwrap();
                widget.set_visible(true);

                let selection_bounds_zoomed =
                    geometry::aabb_scale(selection_bounds, canvas.total_zoom());

                if orientation == Orientation::Vertical {
                    let natural_height = selection_bounds_zoomed.extents()[1].round() as i32
                        + 2 * super::SelectionModifier::RESIZE_NODE_SIZE;

                    (0, natural_height, -1, -1)
                } else {
                    let natural_width = selection_bounds_zoomed.extents()[0].round() as i32
                        + 2 * super::SelectionModifier::RESIZE_NODE_SIZE;

                    (0, natural_width, -1, -1)
                }
            } else {
                widget.set_visible(false);
                (0, 0, -1, -1)
            }
        }

        fn snapshot(&self, widget: &Self::Type, snapshot: &Snapshot) {
            let bounds = AABB::new(
                na::point![0.0, 0.0],
                na::point![f64::from(widget.width()), f64::from(widget.height())],
            );

            // Only makes sense to draw selection when it has Canvas as Parent
            if let Some(canvas) = widget.parent() {
                let canvas = canvas.downcast_ref::<Canvas>().unwrap();

                self.draw_selection_overlay(widget, snapshot, bounds, canvas);
                self.draw_rotation_indicator(widget, snapshot, bounds, canvas);
            }

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

    impl SelectionModifier {
        fn draw_selection_overlay(
            &self,
            widget: &super::SelectionModifier,
            snapshot: &Snapshot,
            _widget_bounds: AABB,
            canvas: &Canvas,
        ) {
            const SELECTION_BOUNDS_COLOR: utils::Color = utils::Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 0.7,
            };
            const SELECTION_BOUNDS_FILL: utils::Color = utils::Color {
                r: 0.49,
                g: 0.56,
                b: 0.63,
                a: 0.15,
            };
            const SELECTION_BOUNDS_WIDTH: f64 = 4.0;

            if let Some(selection_bounds) = self.selection_bounds.get() {
                let transformed_selection_bounds = utils::translate_aabb_to_widget(
                    canvas.transform_sheet_aabb_to_canvas(selection_bounds),
                    canvas,
                    widget,
                )
                .unwrap();

                let draw = || -> Result<(), anyhow::Error> {
                    let mut data = element::path::Data::new();

                    data = data.move_to((
                        f64::from(super::SelectionModifier::RESIZE_NODE_SIZE),
                        f64::from(super::SelectionModifier::RESIZE_NODE_SIZE),
                    ));
                    data = data.line_to((
                        transformed_selection_bounds.extents()[0]
                            + f64::from(super::SelectionModifier::RESIZE_NODE_SIZE),
                        f64::from(super::SelectionModifier::RESIZE_NODE_SIZE),
                    ));
                    data = data.line_to((
                        transformed_selection_bounds.extents()[0]
                            + f64::from(super::SelectionModifier::RESIZE_NODE_SIZE),
                        transformed_selection_bounds.extents()[1]
                            + f64::from(super::SelectionModifier::RESIZE_NODE_SIZE),
                    ));
                    data = data.line_to((
                        f64::from(super::SelectionModifier::RESIZE_NODE_SIZE),
                        transformed_selection_bounds.extents()[1]
                            + f64::from(super::SelectionModifier::RESIZE_NODE_SIZE),
                    ));
                    data = data.close();

                    let svg_path = element::Path::new()
                        .set("d", data)
                        .set("stroke", SELECTION_BOUNDS_COLOR.to_css_color())
                        .set("fill", SELECTION_BOUNDS_FILL.to_css_color())
                        .set("stroke-width", SELECTION_BOUNDS_WIDTH)
                        .set("stroke-dasharray", "6 10");

                    let svg_data = compose::node_to_string(&svg_path).map_err(|e| {
                        anyhow::anyhow!(
                            "node_to_string() failed in gen_svg_path() for selector, {}",
                            e
                        )
                    })?;

                    let svg = render::Svg {
                        bounds: transformed_selection_bounds,
                        svg_data,
                    };
                    let image = canvas
                        .sheet()
                        .strokes_state()
                        .borrow()
                        .renderer
                        .read()
                        .unwrap()
                        .gen_image(1.0, &[svg], transformed_selection_bounds)?;
                    let rendernode = render::image_to_rendernode(&image, 1.0).context(
                        "image_to_rendernode() in draw_selection() in selection_modifier failed",
                    )?;
                    snapshot.append_node(&rendernode);
                    Ok(())
                };

                if let Err(e) = draw() {
                    log::error!(
                        "draw_rotation() for selection_modifier failed with Err {}",
                        e
                    );
                }
            } else {
                log::debug!("draw in draw_rotation() of selection_modifier while selection_boundse are None");
            };
        }

        fn draw_rotation_indicator(
            &self,
            widget: &super::SelectionModifier,
            snapshot: &Snapshot,
            _widget_bounds: AABB,
            canvas: &Canvas,
        ) {
            const ROTATION_LINESTART_COLOR: utils::Color = utils::Color {
                r: 0.7,
                g: 0.3,
                b: 0.3,
                a: 0.7,
            };
            const ROTATION_LINESTART_WIDTH: f64 = 3.0;
            const ROTATION_LINE_LEN: f64 = 150.0;

            if let (Some(current_rotation_center), Some(selection_bounds)) = (
                self.current_rotation_center.get(),
                self.selection_bounds.get(),
            ) {
                let transformed_selection_bounds = utils::translate_aabb_to_widget(
                    canvas.transform_sheet_aabb_to_canvas(selection_bounds),
                    canvas,
                    widget,
                )
                .unwrap();
                let center = {
                    let center = canvas
                        .transform_sheet_coords_to_canvas_coords(current_rotation_center.coords);
                    let center = canvas
                        .translate_coordinates(widget, center[0], center[1])
                        .unwrap();
                    na::point![center.0, center.1]
                };

                let draw = || -> Result<(), anyhow::Error> {
                    let mut data = element::path::Data::new();

                    data = data.move_to((center[0], center[1]));
                    data = data.line_to((center[0] + ROTATION_LINE_LEN, center[1]));

                    let rotation_vec = na::Rotation2::new(self.current_rotation_angle.get())
                        .transform_vector(&(na::Vector2::x() * ((2.0 * ROTATION_LINE_LEN) / 3.0)));

                    data = data.move_to((center[0], center[1]));
                    data = data.line_to((center[0] + rotation_vec[0], center[1] + rotation_vec[1]));
                    data = data.close();

                    let svg_path = element::Path::new()
                        .set("d", data)
                        .set("stroke", ROTATION_LINESTART_COLOR.to_css_color())
                        .set("stroke-width", ROTATION_LINESTART_WIDTH)
                        .set("stroke-dasharray", "5 5");

                    let svg_data = compose::node_to_string(&svg_path).map_err(|e| {
                        anyhow::anyhow!(
                            "node_to_string() failed in gen_svg_path() for selector, {}",
                            e
                        )
                    })?;

                    let svg = render::Svg {
                        bounds: transformed_selection_bounds,
                        svg_data,
                    };
                    let image = canvas
                        .sheet()
                        .strokes_state()
                        .borrow()
                        .renderer
                        .read()
                        .unwrap()
                        .gen_image(1.0, &[svg], transformed_selection_bounds)?;
                    let rendernode = render::image_to_rendernode(&image, 1.0)
                        .context("image_to_rendernode() in draw_rotation_indicator() in selection_modifier failed")?;
                    snapshot.append_node(&rendernode);
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
    }
}

use std::cell::Cell;
use std::rc::Rc;

use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};
use gtk4::{EventSequenceState, GestureDrag, PropagationPhase};
use p2d::bounding_volume::AABB;

use crate::compose::geometry;
use crate::{ui::appwindow::RnoteAppWindow, ui::selectionmodifier::modifiernode::ModifierNode};

use super::canvas::Canvas;

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
    pub const SELECTION_MIN: f64 = 3.0;

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

    pub fn selection_bounds(&self) -> Option<AABB> {
        self.imp().selection_bounds.get()
    }

    pub fn update_state(&self, canvas: &Canvas) {
        let priv_ = self.imp();

        priv_
            .selection_bounds
            .set(canvas.sheet().strokes_state().borrow().gen_selection_bounds());
        self.set_visible(priv_.selection_bounds.get().is_some());

        if let Some(selection_bounds) = priv_.selection_bounds.get() {

            let total_zoom = canvas.total_zoom();
            priv_
                .translate_node
                .get()
                .set_width_request((selection_bounds.extents()[0] * total_zoom).round() as i32);
            priv_
                .translate_node
                .get()
                .set_height_request((selection_bounds.extents()[1] * total_zoom).round() as i32);
        };
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
        priv_.resize_tl_node.add_controller(&resize_tl_drag_gesture);

        resize_tl_drag_gesture.connect_drag_begin(
            clone!(@weak self as selection_modifier, @weak appwindow => move |resize_tl_drag_gesture, _x, _y| {
                resize_tl_drag_gesture.set_state(EventSequenceState::Claimed);
                
                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        resize_tl_drag_gesture.connect_drag_update(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_resize_tl_drag_gesture, x, y| {
                if let Some(selection_bounds) = selection_modifier.selection_bounds() {
                    let zoom = appwindow.canvas().zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

                    let new_bounds = AABB::new(
                        na::point![
                        selection_bounds.mins[0] + offset[0], selection_bounds.mins[1] + offset[1]],
                        na::point![selection_bounds.maxs[0], selection_bounds.maxs[1]]
                    );
                    let min_bounds = AABB::new(
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

                    selection_modifier.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_tl_drag_gesture.connect_drag_end(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_resize_tl_drag_gesture, _x, _y| {
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_geometry_selection_strokes();
                appwindow.canvas().regenerate_content(false, true);
                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
    }

    pub fn init_resize_tr_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let resize_tr_drag_gesture = GestureDrag::builder()
            .name("resize_tr_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.resize_tr_node.add_controller(&resize_tr_drag_gesture);

        resize_tr_drag_gesture.connect_drag_begin(
            clone!(@weak self as selection_modifier, @weak appwindow => move |resize_tr_drag_gesture, _x, _y| {
                resize_tr_drag_gesture.set_state(EventSequenceState::Claimed);
                
                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        resize_tr_drag_gesture.connect_drag_update(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_resize_tr_drag_gesture, x, y| {
                if let Some(selection_bounds) = selection_modifier.selection_bounds() {
                    let zoom = appwindow.canvas().zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

                    let new_bounds = AABB::new(
                        na::point![
                        selection_bounds.mins[0], selection_bounds.mins[1] + offset[1]],
                        na::point![selection_bounds.maxs[0] + offset[0], selection_bounds.maxs[1]]
                    );
                    let min_bounds = AABB::new(
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

                    selection_modifier.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_tr_drag_gesture.connect_drag_end(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_resize_tr_drag_gesture, _x, _y| {
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_geometry_selection_strokes();
                appwindow.canvas().regenerate_content(false, true);
                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
    }

    pub fn init_resize_bl_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let resize_bl_drag_gesture = GestureDrag::builder()
            .name("resize_bl_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.resize_bl_node.add_controller(&resize_bl_drag_gesture);

        resize_bl_drag_gesture.connect_drag_begin(
            clone!(@weak self as selection_modifier, @weak appwindow => move |resize_bl_drag_gesture, _x, _y| {
                resize_bl_drag_gesture.set_state(EventSequenceState::Claimed);
                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        resize_bl_drag_gesture.connect_drag_update(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_resize_bl_drag_gesture, x, y| {
                if let Some(selection_bounds) = selection_modifier.selection_bounds() {
                    let zoom = appwindow.canvas().zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

                    let new_bounds = AABB::new(
                        na::point![
                        selection_bounds.mins[0] + offset[0], selection_bounds.mins[1]],
                        na::point![selection_bounds.maxs[0], selection_bounds.maxs[1] + offset[1]]
                    );
                    let min_bounds = AABB::new(
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

                    selection_modifier.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_bl_drag_gesture.connect_drag_end(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_resize_bl_drag_gesture, _x, _y| {
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_geometry_selection_strokes();
                appwindow.canvas().regenerate_content(false, true);
                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
    }

    pub fn init_resize_br_node(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SelectionModifier::from_instance(self);

        let resize_br_drag_gesture = GestureDrag::builder()
            .name("resize_br_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        priv_.resize_br_node.add_controller(&resize_br_drag_gesture);

        resize_br_drag_gesture.connect_drag_begin(
            clone!(@weak self as selection_modifier, @weak appwindow => move |resize_br_drag_gesture, _x, _y| {
                resize_br_drag_gesture.set_state(EventSequenceState::Claimed);

                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        resize_br_drag_gesture.connect_drag_update(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_resize_br_drag_gesture, x, y| {
                if let Some(selection_bounds) = selection_modifier.selection_bounds() {
                    let zoom = appwindow.canvas().zoom();
                    let offset = na::vector![x.round() / zoom, y.round() / zoom];

                    let new_bounds = AABB::new(
                        na::point![
                        selection_bounds.mins[0], selection_bounds.mins[1]],
                        na::point![selection_bounds.maxs[0] + offset[0], selection_bounds.maxs[1] + offset[1]]
                    );
                    let min_bounds = AABB::new(
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

                    selection_modifier.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            })
        );
        resize_br_drag_gesture.connect_drag_end(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_resize_br_drag_gesture, _x, _y| {
                appwindow.canvas().sheet().strokes_state().borrow_mut().update_geometry_selection_strokes();
                appwindow.canvas().regenerate_content(false, true);
                selection_modifier.update_state(&appwindow.canvas());
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
            clone!(@weak self as selection_modifier, @weak appwindow => move |translate_node_drag_gesture, _x, _y| {
                translate_node_drag_gesture.set_state(EventSequenceState::Claimed);

                selection_modifier.update_state(&appwindow.canvas());
            }),
        );
        translate_node_drag_gesture.connect_drag_update(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_translate_node_drag_gesture, x, y| {
                let zoom = appwindow.canvas().zoom();
                let offset = na::vector![x.round() / zoom, y.round() / zoom];

                let selection_keys = appwindow.canvas().sheet().strokes_state().borrow().selection_keys();
                appwindow.canvas().sheet().strokes_state().borrow_mut().translate_strokes(&selection_keys, offset);

                selection_modifier.queue_resize();
                appwindow.canvas().queue_draw();
            }),
        );
        translate_node_drag_gesture.connect_drag_end(
            clone!(@weak self as selection_modifier, @weak appwindow => move |_translate_node_drag_gesture, _x, _y| {
                selection_modifier.update_state(&appwindow.canvas());
                selection_modifier.queue_resize();
                appwindow.canvas().queue_draw();
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

        let start_bounds: Rc<Cell<Option<AABB>>> = Rc::new(Cell::new(None));
        let angle_prev = Rc::new(Cell::new(0.0));

        rotate_node_drag_gesture.connect_drag_begin(
            clone!(@strong start_bounds, @strong angle_prev, @weak self as selection_modifier, @weak appwindow => move |drag_gesture, _x, _y| {
                drag_gesture.set_state(EventSequenceState::Claimed);
                selection_modifier.update_state(&appwindow.canvas());

                start_bounds.set(selection_modifier.selection_bounds());
                if let Some(start_bounds) = start_bounds.get() {
                    selection_modifier.imp().current_rotation_center.set(Some(start_bounds.center()));
                }
                angle_prev.set(0.0);
            }),
        );
        rotate_node_drag_gesture.connect_drag_update(
            clone!(@strong start_bounds, @strong angle_prev, @weak self as selection_modifier, @weak appwindow => move |rotate_node_drag_gesture, x, y| {
                if let (Some(start_bounds), Some(start_point)) = (start_bounds.get(), rotate_node_drag_gesture.start_point()) {
                    let current_pos = {
                        let pos = selection_modifier.rotate_node().translate_coordinates(&appwindow.canvas(), start_point.0 + x, start_point.1 + y).unwrap();
                        appwindow.canvas().transform_canvas_coords_to_sheet_coords(na::vector![pos.0, pos.1])
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

                    selection_modifier.imp().current_rotation_angle.set(angle);

                    let angle_delta = angle - angle_prev.get();

                    let selection_keys = appwindow.canvas().sheet().strokes_state().borrow().selection_keys();
                    appwindow.canvas().sheet().strokes_state().borrow_mut().rotate_strokes(&selection_keys, angle_delta, start_bounds.center());
                    selection_modifier.update_state(&appwindow.canvas());

                    angle_prev.set(angle);

                    selection_modifier.queue_resize();
                    appwindow.canvas().queue_draw();
                }
            }),
        );
        rotate_node_drag_gesture.connect_drag_end(
            clone!(@strong angle_prev, @weak self as selection_modifier, @weak appwindow => move |_drag_gesture, _x, _y| {
                angle_prev.set(0.0);
                selection_modifier.imp().current_rotation_center.set(None);
                selection_modifier.imp().current_rotation_angle.set(0.0);
                selection_modifier.update_state(&appwindow.canvas());

                selection_modifier.queue_resize();
                appwindow.canvas().queue_draw();
            }),
        );
    }
}
