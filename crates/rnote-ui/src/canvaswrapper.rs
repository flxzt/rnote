// Imports
use crate::{RnAppWindow, RnCanvas, RnContextMenu, canvas::reject_pointer_input};
use gtk4::{
    CompositeTemplate, CornerType, EventControllerMotion, EventControllerScroll,
    EventControllerScrollFlags, EventSequenceState, GestureClick, GestureDrag, GestureLongPress,
    GestureRotate, GestureZoom, PropagationPhase, ScrolledWindow, Widget, gdk, glib, glib::clone,
    graphene, prelude::*, subclass::prelude::*,
};
use once_cell::sync::Lazy;
use p2d::math::Vector2;
use rnote_compose::penevent::ShortcutKey;
use rnote_engine::Camera;
use rnote_engine::ext::GraphenePointExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

/// State recorded at the start of a two-finger ruler gesture. The rotation
/// pivot (projected centroid) is fixed for the duration of the gesture; the
/// anchor/angle/centroid at begin let us compute the new state each frame by
/// (1) rotating around the pivot and (2) translating by the centroid delta.
/// All position values are in scroller (window-relative) pixel coordinates.
#[derive(Clone, Copy)]
struct RulerDragBegin {
    pivot: Vector2,
    anchor_begin: Vector2,
    centroid_begin: Vector2,
    angle_begin: f64,
}

/// How long after the last scroll event a session stays "active" (= the pivot
/// is held). Beyond this, the next scroll begins a fresh session.
const RULER_SCROLL_SESSION_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(400);

/// Speed-adaptive scaling for mouse-wheel rotation. Mouse-wheel `dy` is always
/// ±1 per click, so to give the user "precise when slow, fast when fast"
/// behaviour we look at the time since the last click: short intervals (fast
/// scrolling) get a multiplier > 1, long intervals (slow scrolling) get one
/// < 1. Trackpad events already encode speed in `|dy|` per event so they're
/// not adjusted.
const WHEEL_SPEED_BASE_DT_MS: f64 = 200.0;
const WHEEL_SPEED_MIN_MULT: f64 = 0.4;
const WHEEL_SPEED_MAX_MULT: f64 = 3.0;

/// How long after the last scroll event a session's pivot can still be reused
/// — even though the session is no longer "active" for speed adaptation, the
/// dial position is held so the next scroll continues the same gesture
/// instead of either snapping the dial elsewhere or slipping into canvas pan.
const RULER_SCROLL_REVIVAL_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(3);

/// Active scroll-rotation session: the pivot is locked for the duration of the
/// session so the dial doesn't move and the user keeps "grip" even if the
/// ruler rotates out from under the pointer.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScrollRotationSession {
    pivot: Vector2,
    last_event_time: Instant,
}

/// Rotate the ruler in response to a scroll event. Returns `true` if the event
/// was consumed (caller should stop propagation). The rotation pivot is locked
/// for the duration of a session — set on the first scroll while the pointer
/// is over the ruler body, kept for subsequent scrolls that arrive within
/// `RULER_SCROLL_SESSION_TIMEOUT`.
///
/// No angle snap is applied here — scrolling is purely an absolute rotation
/// control. Mouse-wheel events get a speed-adaptive multiplier (precise when
/// slow, faster when scrolled rapidly); trackpad events use `dy` as-is.
fn rotate_ruler_with_scroll(
    canvaswrapper: &RnCanvasWrapper,
    controller: &EventControllerScroll,
    dy: f64,
) -> bool {
    let now = Instant::now();
    let imp = canvaswrapper.imp();

    let prev_session_raw = imp.ruler_scroll_session.get();

    let canvas = canvaswrapper.canvas();
    let config_shared = canvas.engine_ref().engine_config().clone();

    // Speed-adaptive scaling for mouse wheel only. `controller.unit()` is
    // `Wheel` for mouse, `Surface` for trackpad / precision scrolling.
    let is_wheel = controller.unit() == gdk::ScrollUnit::Wheel;
    let speed_mult = if is_wheel {
        // Time since the previous scroll event from the same session lineage.
        // First event of a fresh session has no history → assume baseline.
        let dt_ms = prev_session_raw
            .map(|s| now.duration_since(s.last_event_time).as_millis().max(1) as f64)
            .unwrap_or(WHEEL_SPEED_BASE_DT_MS);
        (WHEEL_SPEED_BASE_DT_MS / dt_ms).clamp(WHEEL_SPEED_MIN_MULT, WHEEL_SPEED_MAX_MULT)
    } else {
        1.0
    };

    let pivot = {
        let mut config = config_shared.write();
        let ruler = &mut config.pens_config.brush_config.ruler_config;
        if !ruler.visible {
            return false;
        }

        // Validate the cached session pivot is still on the current ruler
        // centerline. If something else rotated/moved the ruler in between
        // (e.g. a two-finger gesture), the cached pivot may now be off the
        // line — using it would rotate around an off-line point and put the
        // dial outside the band.
        const ON_LINE_TOLERANCE_PX: f64 = 0.5;
        let session_on_line = prev_session_raw.filter(|s| {
            (s.pivot - ruler.anchor).dot(ruler.normal()).abs() <= ON_LINE_TOLERANCE_PX
        });
        // Re-derive the windowed sessions from the validated value.
        let prev_session = session_on_line.filter(|s| {
            now.duration_since(s.last_event_time) < RULER_SCROLL_SESSION_TIMEOUT
        });
        let revivable_session = session_on_line.filter(|s| {
            now.duration_since(s.last_event_time) < RULER_SCROLL_REVIVAL_TIMEOUT
        });

        let pivot = if let Some(session) = prev_session {
            // Within the short active-session timeout — definitely reuse.
            session.pivot
        } else if let Some(session) = revivable_session {
            // Session timed out for speed-adaptation purposes, but still recent
            // enough that we should hold the pivot. Only re-target if the
            // pointer is clearly somewhere else (and is known).
            let pointer = imp.pointer_pos.get();
            let pointer_far = pointer
                .map(|p| (p - session.pivot).length() > ruler.body_half_width * 1.5)
                .unwrap_or(false);
            if pointer_far {
                // User deliberately moved the pointer to a new spot.
                let p = pointer.unwrap();
                let rel = p - ruler.anchor;
                if rel.dot(ruler.normal()).abs() > ruler.body_half_width {
                    return false;
                }
                let along = rel.dot(ruler.direction());
                let projected = ruler.anchor + along * ruler.direction();
                ruler.dial_pos = projected;
                projected
            } else {
                // Either pointer is near the old pivot, or pointer_pos is
                // currently `None`. Either way, the user is continuing the
                // same gesture — hold the dial in place.
                session.pivot
            }
        } else {
            // No recent session at all: this is a fresh start. Need pointer.
            let pointer = match imp.pointer_pos.get() {
                Some(p) => p,
                None => return false,
            };
            // Dead zone: if the cursor is close to the current dial position
            // (and the dial sits on the current centerline, which it should),
            // reuse it instead of resetting to a new spot. Same behaviour as
            // session revival, so the very first scroll after a long pause
            // doesn't snap the dial to a slightly different place.
            let dial_on_line =
                (ruler.dial_pos - ruler.anchor).dot(ruler.normal()).abs() <= ON_LINE_TOLERANCE_PX;
            if dial_on_line
                && (pointer - ruler.dial_pos).length() <= ruler.body_half_width * 1.5
            {
                ruler.dial_pos
            } else {
                // Need a fresh pivot at the cursor position. Hit-test the body.
                let rel = pointer - ruler.anchor;
                if rel.dot(ruler.normal()).abs() > ruler.body_half_width {
                    return false;
                }
                let along = rel.dot(ruler.direction());
                let projected = ruler.anchor + along * ruler.direction();
                ruler.dial_pos = projected;
                projected
            }
        };

        let delta_rad = dy * speed_mult * ruler.scroll_rotation_step_deg.to_radians();
        let new_angle = ruler.angle + delta_rad;
        let effective_delta = delta_rad;

        // Rotate anchor around the locked pivot.
        let v = ruler.anchor - pivot;
        let cos_a = effective_delta.cos();
        let sin_a = effective_delta.sin();
        let v_rotated = Vector2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a);

        ruler.angle = new_angle;
        ruler.anchor = pivot + v_rotated;
        // dial_pos was set above (on new session) or stays at the locked pivot.
        pivot
    };

    imp.ruler_scroll_session.set(Some(ScrollRotationSession {
        pivot,
        last_event_time: now,
    }));
    canvas.queue_draw();
    true
}

/// Apply the two-finger gesture update to the ruler. The ruler:
/// - is rotated by `angle_delta` around the fixed `pivot` (set at begin),
/// - then translated by the current centroid offset from begin.
///
/// On placement (`translation = 0`, `angle_delta = 0`) nothing moves except
/// `dial_pos`, which is `pivot` (set in the begin handler).
fn apply_two_finger_ruler_update(canvaswrapper: &RnCanvasWrapper, begin: RulerDragBegin) {
    let canvas = canvaswrapper.canvas();
    let zoom_gesture = &canvaswrapper.imp().canvas_zoom_gesture;
    let rotate_gesture = &canvaswrapper.imp().canvas_rotate_gesture;

    let Some((cx, cy)) = zoom_gesture.bounding_box_center() else {
        return;
    };
    let centroid_now = Vector2::new(cx, cy);
    let translation = centroid_now - begin.centroid_begin;

    let angle_delta = rotate_gesture.angle_delta();
    let raw_new_angle = begin.angle_begin + angle_delta;
    let config_shared = canvas.engine_ref().engine_config().clone();
    let new_angle = if config_shared.read().pens_config.brush_config.ruler_config.angle_snap_enabled {
        // Use hysteresis against the angle at gesture begin: once the user has
        // moved into a snap, finger jitter shouldn't keep them locked there.
        rnote_engine::pens::pensconfig::rulerconfig::RulerConfig::snap_angle_hysteretic(
            raw_new_angle,
            begin.angle_begin,
        )
    } else {
        raw_new_angle
    };
    // The rotation applied to the anchor's offset-from-pivot uses the EFFECTIVE
    // angular change (after snap), not the raw gesture delta, so the anchor
    // tracks the snapped ruler line.
    let effective_delta = new_angle - begin.angle_begin;
    let cos_a = effective_delta.cos();
    let sin_a = effective_delta.sin();
    let v = begin.anchor_begin - begin.pivot;
    let v_rotated = Vector2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a);
    let new_anchor = begin.pivot + v_rotated + translation;
    let new_dial_pos = begin.pivot + translation;

    {
        let mut c = config_shared.write();
        let r = &mut c.pens_config.brush_config.ruler_config;
        r.anchor = new_anchor;
        r.dial_pos = new_dial_pos;
        r.angle = new_angle;
    }
    canvas.queue_draw();
}

#[derive(Debug, Default)]
struct Connections {
    appwindow_block_pinch_zoom_bind: Option<glib::Binding>,
    appwindow_show_scrollbars_bind: Option<glib::Binding>,
    appwindow_inertial_scrolling_bind: Option<glib::Binding>,
    appwindow_righthanded_bind: Option<glib::Binding>,
}

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/canvaswrapper.ui")]
    pub(crate) struct RnCanvasWrapper {
        pub(super) connections: RefCell<Connections>,
        pub(crate) canvas_touch_drawing_handler: RefCell<Option<glib::SignalHandlerId>>,
        pub(crate) show_scrollbars: Cell<bool>,
        pub(crate) block_pinch_zoom: Cell<bool>,
        pub(crate) inertial_scrolling: Cell<bool>,
        pub(crate) pointer_pos: Cell<Option<Vector2>>,
        pub(crate) last_contextmenu_pos: Cell<Option<Vector2>>,
        /// Active scroll-to-rotate session on the ruler. Set when the first
        /// scroll-on-ruler event fires; cleared when no scroll event arrives
        /// within `RULER_SCROLL_SESSION_TIMEOUT`. While active, all scroll
        /// events rotate around the locked pivot regardless of pointer position.
        pub(crate) ruler_scroll_session: Cell<Option<ScrollRotationSession>>,

        pub(crate) pointer_motion_controller: EventControllerMotion,
        pub(crate) canvas_drag_gesture: GestureDrag,
        pub(crate) canvas_zoom_gesture: GestureZoom,
        pub(crate) canvas_rotate_gesture: GestureRotate,
        pub(crate) canvas_multi_press_gesture: GestureClick,
        pub(crate) canvas_zoom_scroll_controller: EventControllerScroll,
        pub(crate) canvas_mouse_drag_middle_gesture: GestureDrag,
        pub(crate) canvas_alt_drag_gesture: GestureDrag,
        pub(crate) canvas_alt_shift_drag_gesture: GestureDrag,
        pub(crate) touch_two_finger_long_press_gesture: GestureLongPress,
        pub(crate) touch_long_press_gesture: GestureLongPress,

        #[template_child]
        pub(crate) scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) canvas: TemplateChild<RnCanvas>,
        #[template_child]
        pub(crate) contextmenu: TemplateChild<RnContextMenu>,
    }

    impl Default for RnCanvasWrapper {
        fn default() -> Self {
            let pointer_motion_controller = EventControllerMotion::builder()
                .name("pointer_motion_controller")
                .propagation_phase(PropagationPhase::Capture)
                .build();

            // This allows touch dragging and dragging with pointer in the empty space around the canvas.
            // All relevant pointer events for drawing are captured and denied for propagation before they arrive at this gesture.
            let canvas_drag_gesture = GestureDrag::builder()
                .name("canvas_drag_gesture")
                .button(gdk::BUTTON_PRIMARY)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            let canvas_zoom_gesture = GestureZoom::builder()
                .name("canvas_zoom_gesture")
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let canvas_rotate_gesture = GestureRotate::builder()
                .name("canvas_rotate_gesture")
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let canvas_multi_press_gesture = GestureClick::builder()
                .name("canvas_multi_press_gesture")
                .button(gdk::BUTTON_PRIMARY)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let canvas_zoom_scroll_controller = EventControllerScroll::builder()
                .name("canvas_zoom_scroll_controller")
                // Capture phase: handle scroll BEFORE the canvas's own scrollable
                // behaviour. Otherwise the canvas might consume the event during
                // an active ruler-rotation session and the rotation appears to
                // "slip" into a canvas pan.
                .propagation_phase(PropagationPhase::Capture)
                // Listen on both axes so we can consume the horizontal component
                // of a trackpad scroll while we're rotating the ruler (otherwise
                // the canvas pans sideways from that component).
                .flags(
                    EventControllerScrollFlags::VERTICAL
                        | EventControllerScrollFlags::HORIZONTAL,
                )
                .build();

            let canvas_mouse_drag_middle_gesture = GestureDrag::builder()
                .name("canvas_mouse_drag_middle_gesture")
                .button(gdk::BUTTON_MIDDLE)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            // alt + drag for panning with pointer
            let canvas_alt_drag_gesture = GestureDrag::builder()
                .name("canvas_alt_drag_gesture")
                .button(gdk::BUTTON_PRIMARY)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Capture)
                .build();

            // alt + shift + drag for zooming with pointer
            let canvas_alt_shift_drag_gesture = GestureDrag::builder()
                .name("canvas_alt_shift_drag_gesture")
                .button(gdk::BUTTON_PRIMARY)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let touch_two_finger_long_press_gesture = GestureLongPress::builder()
                .name("touch_two_finger_long_press_gesture")
                .touch_only(true)
                .n_points(2)
                // activate a bit quicker
                .delay_factor(0.8)
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let touch_long_press_gesture = GestureLongPress::builder()
                .name("touch_long_press_gesture")
                .touch_only(true)
                .build();

            Self {
                connections: RefCell::new(Connections::default()),
                canvas_touch_drawing_handler: RefCell::new(None),
                show_scrollbars: Cell::new(false),
                block_pinch_zoom: Cell::new(false),
                inertial_scrolling: Cell::new(true),
                pointer_pos: Cell::new(None),
                last_contextmenu_pos: Cell::new(None),
                ruler_scroll_session: Cell::new(None),

                pointer_motion_controller,
                canvas_drag_gesture,
                canvas_zoom_gesture,
                canvas_rotate_gesture,
                canvas_multi_press_gesture,
                canvas_zoom_scroll_controller,
                canvas_mouse_drag_middle_gesture,
                canvas_alt_drag_gesture,
                canvas_alt_shift_drag_gesture,
                touch_two_finger_long_press_gesture,
                touch_long_press_gesture,

                scroller: TemplateChild::<ScrolledWindow>::default(),
                canvas: TemplateChild::<RnCanvas>::default(),
                contextmenu: TemplateChild::<RnContextMenu>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnCanvasWrapper {
        const NAME: &'static str = "RnCanvasWrapper";
        type Type = super::RnCanvasWrapper;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnCanvasWrapper {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Add input controllers
            self.scroller
                .add_controller(self.pointer_motion_controller.clone());
            self.scroller
                .add_controller(self.canvas_drag_gesture.clone());
            self.scroller
                .add_controller(self.canvas_zoom_gesture.clone());
            self.scroller
                .add_controller(self.canvas_rotate_gesture.clone());
            self.scroller
                .add_controller(self.canvas_multi_press_gesture.clone());
            self.scroller
                .add_controller(self.canvas_zoom_scroll_controller.clone());
            self.scroller
                .add_controller(self.canvas_mouse_drag_middle_gesture.clone());
            self.scroller
                .add_controller(self.canvas_alt_drag_gesture.clone());
            self.scroller
                .add_controller(self.canvas_alt_shift_drag_gesture.clone());
            self.scroller
                .add_controller(self.touch_two_finger_long_press_gesture.clone());
            self.canvas
                .add_controller(self.touch_long_press_gesture.clone());

            // group
            self.touch_two_finger_long_press_gesture
                .group_with(&self.canvas_zoom_gesture);
            self.canvas_rotate_gesture
                .group_with(&self.canvas_zoom_gesture);

            self.setup_input();

            let canvas_touch_drawing_handler = self.canvas.connect_notify_local(
                Some("touch-drawing"),
                clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_canvas, _pspec| {
                        // Disable the zoom gesture and kinetic scrolling when touch drawing is enabled.
                        canvaswrapper.imp().canvas_kinetic_scrolling_update();
                        canvaswrapper.imp().canvas_zoom_gesture_update();
                    }
                ),
            );

            self.canvas_touch_drawing_handler
                .replace(Some(canvas_touch_drawing_handler));
        }

        fn dispose(&self) {
            self.obj().disconnect_connections();

            if let Some(handler) = self.canvas_touch_drawing_handler.take() {
                self.canvas.disconnect(handler);
            }

            // the engine task handler needs to be be aborted here,
            // else a reference of the canvas is held forever in the handler causing a memory leak.
            self.canvas.abort_engine_task_handler();

            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::builder("show-scrollbars")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("block-pinch-zoom")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("inertial-scrolling")
                        .default_value(true)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "show-scrollbars" => self.show_scrollbars.get().to_value(),
                "block-pinch-zoom" => self.block_pinch_zoom.get().to_value(),
                "inertial-scrolling" => self.inertial_scrolling.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "show-scrollbars" => {
                    let show_scrollbars = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.show_scrollbars.replace(show_scrollbars);

                    self.scroller.hscrollbar().set_visible(show_scrollbars);
                    self.scroller.vscrollbar().set_visible(show_scrollbars);
                }
                "block-pinch-zoom" => {
                    let block_pinch_zoom = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.block_pinch_zoom.replace(block_pinch_zoom);
                    self.canvas_zoom_gesture_update();
                }
                "inertial-scrolling" => {
                    let inertial_scrolling = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");

                    self.inertial_scrolling.replace(inertial_scrolling);
                    self.canvas_kinetic_scrolling_update();
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnCanvasWrapper {}

    impl RnCanvasWrapper {
        fn canvas_zoom_gesture_update(&self) {
            if !self.block_pinch_zoom.get() && !self.canvas.touch_drawing() {
                self.canvas_zoom_gesture
                    .set_propagation_phase(PropagationPhase::Capture);
            } else {
                self.canvas_zoom_gesture
                    .set_propagation_phase(PropagationPhase::None);
            }
        }

        fn canvas_kinetic_scrolling_update(&self) {
            self.scroller.set_kinetic_scrolling(
                !self.canvas.touch_drawing() && self.inertial_scrolling.get(),
            );
        }

        fn workaround_disable_kinetic_scrolling(&self) {
            let scroller = self.scroller.get();
            self.canvas
                .workaround_disable_kinetic_scrolling(Some(&scroller));
        }

        fn workaround_restore_kinetic_scrolling(&self) {
            let scroller = self.scroller.get();
            self.canvas
                .workaround_restore_kinetic_scrolling(Some(&scroller));
        }

        pub(super) fn workaround_cancel_kinetic_scrolling_for_zoom(&self) {
            self.workaround_disable_kinetic_scrolling();
            self.workaround_restore_kinetic_scrolling();
        }

        fn setup_input(&self) {
            let obj = self.obj();

            {
                self.pointer_motion_controller.connect_enter(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, x, y| {
                        canvaswrapper.imp().pointer_pos.set(Some(Vector2::new(x, y)));
                    }
                ));

                self.pointer_motion_controller.connect_motion(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, x, y| {
                        canvaswrapper
                            .imp()
                            .pointer_pos
                            .set(Some(Vector2::new(x, y)));
                    }
                ));

                self.pointer_motion_controller.connect_leave(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_| {
                        canvaswrapper.imp().pointer_pos.set(None);
                    }
                ));
            }

            // zoom scrolling with <ctrl> + scroll, or ruler rotation when scrolling
            // over the ruler body (mouse wheel / trackpad).
            {
                self.canvas_zoom_scroll_controller.connect_scroll(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    #[upgrade_or]
                    glib::Propagation::Proceed,
                    move |controller, _, dy| {
                        let modifiers = controller.current_event_state();

                        if !modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                            // No Ctrl: if the pointer is over the ruler body, rotate
                            // the ruler around the pointer (projected onto the
                            // centerline). Otherwise let the event propagate so the
                            // scroller can scroll normally.
                            if rotate_ruler_with_scroll(&canvaswrapper, controller, dy) {
                                return glib::Propagation::Stop;
                            }
                            return glib::Propagation::Proceed;
                        }

                        // workaround for https://gitlab.gnome.org/GNOME/gtk/-/issues/187
                        canvaswrapper
                            .imp()
                            .workaround_cancel_kinetic_scrolling_for_zoom();

                        let canvas = canvaswrapper.canvas();
                        let old_zoom = canvas.engine_ref().camera.total_zoom();
                        let new_zoom = if dy < 0.0 {
                            old_zoom * (1.0 - dy * RnCanvas::ZOOM_SCROLL_STEP)
                        } else {
                            old_zoom * (1.0 / (1.0 + dy * RnCanvas::ZOOM_SCROLL_STEP))
                        };

                        if (Camera::ZOOM_MIN..=Camera::ZOOM_MAX).contains(&new_zoom) {
                            let camera_offset = canvas.engine_ref().camera.offset();
                            let camera_size = canvas.engine_ref().camera.size();
                            let screen_offset = canvaswrapper
                                .imp()
                                .pointer_pos
                                .get()
                                .map(|p| {
                                    let p = canvaswrapper
                                        .compute_point(&canvas, &graphene::Point::from_p2d_vec(p))
                                        .unwrap();
                                    p.to_p2d_vec()
                                })
                                .unwrap_or_else(|| camera_size * 0.5);
                            let new_camera_offset = (((camera_offset + screen_offset) / old_zoom)
                                * new_zoom)
                                - screen_offset;

                            let mut widget_flags = canvas.engine_mut().zoom_w_timeout(new_zoom);
                            widget_flags |= canvas
                                .engine_mut()
                                .camera_set_offset_expand(new_camera_offset);
                            canvas.emit_handle_widget_flags(widget_flags);
                        }

                        glib::Propagation::Stop
                    }
                ));
            }

            // Drag canvas gesture (one-finger touch drag, or mouse drag in the empty area
            // around the canvas). If the gesture begins on the ruler body, drag the ruler
            // instead of the canvas.
            {
                #[derive(Clone, Copy)]
                enum CanvasDragMode {
                    Canvas(Vector2),
                    /// (anchor_begin, dial_pos_begin) — both translated together.
                    Ruler(Vector2, Vector2),
                }
                let drag_mode: Rc<Cell<Option<CanvasDragMode>>> = Rc::new(Cell::new(None));

                self.canvas_drag_gesture.connect_drag_begin(clone!(
                    #[strong]
                    drag_mode,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, x, y| {
                        // We don't claim the sequence, because we want to allow touch zooming.
                        // When the zoom gesture is recognized, it claims it and denies this touch drag gesture.
                        // The drag gesture coords are already in scroller (window-relative)
                        // coordinates — same space as ruler.anchor / ruler.dial_pos.
                        let canvas = canvaswrapper.canvas();
                        let mode = {
                            let config = canvas.engine_ref().engine_config().clone();
                            let config = config.read();
                            let ruler = &config.pens_config.brush_config.ruler_config;
                            if ruler.visible {
                                let p = Vector2::new(x, y);
                                let rel = p - ruler.anchor;
                                let inside_body =
                                    rel.dot(ruler.normal()).abs()
                                        <= ruler.body_half_width;
                                if inside_body {
                                    CanvasDragMode::Ruler(ruler.anchor, ruler.dial_pos)
                                } else {
                                    CanvasDragMode::Canvas(
                                        canvas.engine_ref().camera.offset(),
                                    )
                                }
                            } else {
                                CanvasDragMode::Canvas(canvas.engine_ref().camera.offset())
                            }
                        };
                        drag_mode.set(Some(mode));
                    }
                ));
                self.canvas_drag_gesture.connect_drag_update(clone!(
                    #[strong]
                    drag_mode,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, x, y| {
                        let canvas = canvaswrapper.canvas();
                        match drag_mode.get() {
                            Some(CanvasDragMode::Ruler(anchor_begin, dial_pos_begin)) => {
                                // Drag offset (x, y) is in scroller pixels — apply directly.
                                let delta = Vector2::new(x, y);
                                let config_shared = canvas.engine_ref().engine_config().clone();
                                {
                                    let mut c = config_shared.write();
                                    let r = &mut c.pens_config.brush_config.ruler_config;
                                    r.anchor = anchor_begin + delta;
                                    r.dial_pos = dial_pos_begin + delta;
                                }
                                canvas.queue_draw();
                            }
                            Some(CanvasDragMode::Canvas(offset_begin)) => {
                                let new_offset = offset_begin - Vector2::new(x, y);
                                let widget_flags =
                                    canvas.engine_mut().camera_set_offset_expand(new_offset);
                                canvas.emit_handle_widget_flags(widget_flags);
                            }
                            None => {}
                        }
                    }
                ));
                self.canvas_drag_gesture.connect_drag_end(clone!(
                    #[strong]
                    drag_mode,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, _, _| {
                        drag_mode.set(None);
                        let widget_flags = canvaswrapper
                            .canvas()
                            .engine_mut()
                            .update_rendering_current_viewport();
                        canvaswrapper
                            .canvas()
                            .emit_handle_widget_flags(widget_flags);
                    }
                ));
            }

            // Move Canvas with middle mouse button
            {
                let mouse_drag_start = Rc::new(Cell::new(Vector2::ZERO));

                self.canvas_mouse_drag_middle_gesture
                    .connect_drag_begin(clone!(
                        #[strong]
                        mouse_drag_start,
                        #[weak(rename_to=canvaswrapper)]
                        obj,
                        move |_, _, _| {
                            mouse_drag_start
                                .set(canvaswrapper.canvas().engine_ref().camera.offset());
                        }
                    ));
                self.canvas_mouse_drag_middle_gesture
                    .connect_drag_update(clone!(
                        #[strong]
                        mouse_drag_start,
                        #[weak(rename_to=canvaswrapper)]
                        obj,
                        move |_, x, y| {
                            let canvas = canvaswrapper.canvas();
                            let new_offset = mouse_drag_start.get() - Vector2::new(x, y);
                            let widget_flags =
                                canvas.engine_mut().camera_set_offset_expand(new_offset);
                            canvas.emit_handle_widget_flags(widget_flags);
                        }
                    ));
                self.canvas_mouse_drag_middle_gesture
                    .connect_drag_end(clone!(
                        #[weak(rename_to=canvaswrapper)]
                        obj,
                        move |_, _, _| {
                            let widget_flags = canvaswrapper
                                .canvas()
                                .engine_mut()
                                .update_rendering_current_viewport();
                            canvaswrapper
                                .canvas()
                                .emit_handle_widget_flags(widget_flags);
                        }
                    ));
            }

            // Canvas gesture zooming with dragging (or ruler manipulation if the
            // gesture begins inside the ruler body).
            {
                let prev_scale = Rc::new(Cell::new(1_f64));
                let zoom_begin = Rc::new(Cell::new(1_f64));
                let new_zoom = Rc::new(Cell::new(1.0));
                let bbcenter_begin: Rc<Cell<Option<Vector2>>> = Rc::new(Cell::new(None));
                let offset_begin = Rc::new(Cell::new(Vector2::ZERO));

                // Shared ruler-drag state across the zoom + rotate gestures. When `Some`,
                // the two-finger gesture is manipulating the ruler (pan + rotate around
                // the centroid) instead of zooming the canvas.
                let ruler_drag: Rc<Cell<Option<RulerDragBegin>>> = Rc::new(Cell::new(None));

                self.canvas_zoom_gesture.connect_begin(clone!(
                    #[strong]
                    zoom_begin,
                    #[strong]
                    new_zoom,
                    #[strong]
                    prev_scale,
                    #[strong]
                    bbcenter_begin,
                    #[strong]
                    offset_begin,
                    #[strong]
                    ruler_drag,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |gesture, _| {
                        gesture.set_state(EventSequenceState::Claimed);

                        // workaround for https://gitlab.gnome.org/GNOME/gtk/-/issues/187
                        canvaswrapper.imp().workaround_disable_kinetic_scrolling();

                        let canvas = canvaswrapper.canvas();
                        let current_zoom = canvas.engine_ref().camera.total_zoom();

                        zoom_begin.set(current_zoom);
                        new_zoom.set(current_zoom);
                        prev_scale.set(1.0);

                        let bbcenter = gesture
                            .bounding_box_center()
                            .map(|(x, y)| Vector2::new(x, y));
                        bbcenter_begin.set(bbcenter);
                        offset_begin.set(canvas.engine_ref().camera.offset());

                        // Check if this gesture should manipulate the ruler. Only
                        // genuine touchscreen gestures are accepted — trackpad pinch /
                        // rotate (which GestureZoom/GestureRotate also fire for)
                        // should NOT enter the touch-style ruler manipulation mode.
                        // All math is in scroller (window-relative) coordinates —
                        // bbcenter is already in that space.
                        ruler_drag.set(None);
                        let from_touchscreen = gesture
                            .current_event_device()
                            .map(|d| d.source() == gdk::InputSource::Touchscreen)
                            .unwrap_or(false);
                        if from_touchscreen && let Some(bbcenter) = bbcenter {
                            let projected_opt = {
                                let config = canvas.engine_ref().engine_config().clone();
                                let config = config.read();
                                let ruler = &config.pens_config.brush_config.ruler_config;
                                if !ruler.visible {
                                    None
                                } else {
                                    // Hit-test in scroller coords directly.
                                    let rel = bbcenter - ruler.anchor;
                                    let inside_body =
                                        rel.dot(ruler.normal()).abs()
                                            <= ruler.body_half_width;
                                    if inside_body {
                                        // Project centroid onto the ruler centerline.
                                        let along = rel.dot(ruler.direction());
                                        let projected =
                                            ruler.anchor + along * ruler.direction();
                                        Some((projected, ruler.anchor, ruler.angle))
                                    } else {
                                        None
                                    }
                                }
                            };
                            if let Some((projected_centroid, anchor_at_begin, angle_begin)) =
                                projected_opt
                            {
                                // Move only the dial to the projected centroid (= the
                                // rotation pivot). The anchor / tick origin stays put.
                                let config_shared =
                                    canvas.engine_ref().engine_config().clone();
                                config_shared
                                    .write()
                                    .pens_config
                                    .brush_config
                                    .ruler_config
                                    .dial_pos = projected_centroid;
                                ruler_drag.set(Some(RulerDragBegin {
                                    pivot: projected_centroid,
                                    anchor_begin: anchor_at_begin,
                                    centroid_begin: bbcenter,
                                    angle_begin,
                                }));
                            }
                        }
                    }
                ));

                self.canvas_zoom_gesture.connect_scale_changed(clone!(
                    #[strong]
                    zoom_begin,
                    #[strong]
                    new_zoom,
                    #[strong]
                    prev_scale,
                    #[strong]
                    bbcenter_begin,
                    #[strong]
                    offset_begin,
                    #[strong]
                    ruler_drag,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |gesture, scale| {
                        let canvas = canvaswrapper.canvas();

                        // When a two-finger gesture begins on the ruler, suppress canvas
                        // zoom and apply the combined translate + rotate around the
                        // (live) centroid to the ruler.
                        if let Some(begin) = ruler_drag.get() {
                            apply_two_finger_ruler_update(&canvaswrapper, begin);
                            return;
                        }

                        if (Camera::ZOOM_MIN..=Camera::ZOOM_MAX)
                            .contains(&(zoom_begin.get() * scale))
                        {
                            new_zoom.set(zoom_begin.get() * scale);
                            prev_scale.set(scale);
                        }

                        let mut widget_flags = canvas.engine_mut().zoom_w_timeout(new_zoom.get());

                        if let Some(bbcenter_current) = gesture
                            .bounding_box_center()
                            .map(|(x, y)| Vector2::new(x, y))
                        {
                            let bbcenter_begin = if let Some(bbcenter_begin) = bbcenter_begin.get()
                            {
                                bbcenter_begin
                            } else {
                                // Set the center if not set by gesture begin handler
                                bbcenter_begin.set(Some(bbcenter_current));
                                bbcenter_current
                            };
                            let bbcenter_delta =
                                bbcenter_current - bbcenter_begin * prev_scale.get();
                            let new_offset = offset_begin.get() * prev_scale.get() - bbcenter_delta;
                            widget_flags |=
                                canvas.engine_mut().camera_set_offset_expand(new_offset);
                        }

                        canvas.emit_handle_widget_flags(widget_flags);
                    }
                ));

                self.canvas_rotate_gesture.connect_angle_changed(clone!(
                    #[strong]
                    ruler_drag,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_gesture, _angle, _angle_delta| {
                        let Some(begin) = ruler_drag.get() else {
                            return;
                        };
                        apply_two_finger_ruler_update(&canvaswrapper, begin);
                    }
                ));

                self.canvas_zoom_gesture.connect_end(clone!(
                    #[strong]
                    ruler_drag,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_gesture, _event_sequence| {
                        // workaround for https://gitlab.gnome.org/GNOME/gtk/-/issues/187
                        canvaswrapper.imp().workaround_restore_kinetic_scrolling();
                        ruler_drag.set(None);

                        let widget_flags = canvaswrapper
                            .canvas()
                            .engine_mut()
                            .update_rendering_current_viewport();
                        canvaswrapper
                            .canvas()
                            .emit_handle_widget_flags(widget_flags);
                    }
                ));

                self.canvas_zoom_gesture.connect_cancel(clone!(
                    #[strong]
                    ruler_drag,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |gesture, _event_sequence| {
                        gesture.set_state(EventSequenceState::Denied);

                        // workaround for https://gitlab.gnome.org/GNOME/gtk/-/issues/187
                        canvaswrapper.imp().workaround_restore_kinetic_scrolling();
                        ruler_drag.set(None);

                        let widget_flags = canvaswrapper
                            .canvas()
                            .engine_mut()
                            .update_rendering_current_viewport();
                        canvaswrapper
                            .canvas()
                            .emit_handle_widget_flags(widget_flags);
                    }
                ));
            }

            // Pan with alt + drag
            {
                let offset_start = Rc::new(Cell::new(Vector2::ZERO));

                self.canvas_alt_drag_gesture.connect_drag_begin(clone!(
                    #[strong]
                    offset_start,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |gesture, _, _| {
                        let modifiers = gesture.current_event_state();

                        // At the start BUTTON1_MASK is not included
                        if modifiers.contains(gdk::ModifierType::ALT_MASK)
                            && !modifiers.contains(gdk::ModifierType::SHIFT_MASK)
                        {
                            gesture.set_state(EventSequenceState::Claimed);
                            offset_start.set(canvaswrapper.canvas().engine_ref().camera.offset());
                        } else {
                            gesture.set_state(EventSequenceState::Denied);
                        }
                    }
                ));

                self.canvas_alt_drag_gesture.connect_drag_update(clone!(
                    #[strong]
                    offset_start,
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, offset_x, offset_y| {
                        let canvas = canvaswrapper.canvas();
                        let new_offset = offset_start.get() - Vector2::new(offset_x, offset_y);
                        let widget_flags = canvas.engine_mut().camera_set_offset_expand(new_offset);
                        canvas.emit_handle_widget_flags(widget_flags);
                    }
                ));

                self.canvas_alt_drag_gesture.connect_drag_end(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, _, _| {
                        let widget_flags = canvaswrapper
                            .canvas()
                            .engine_mut()
                            .update_rendering_current_viewport();
                        canvaswrapper
                            .canvas()
                            .emit_handle_widget_flags(widget_flags);
                    }
                ));
            }

            // Double press to select word, triple press to select line
            {
                self.canvas_multi_press_gesture.connect_pressed(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |signal, n_press, _, _| {
                        // cycle through 0, 1, 2 - single, double, triple press
                        let action = (n_press - 1) % 3;

                        if action <= 0 {
                            // Single press or invalid press count
                            return;
                        }

                        let canvas = canvaswrapper.canvas();

                        if signal.current_event().is_none_or(|event| {
                            reject_pointer_input(&event, canvas.touch_drawing())
                        }) {
                            // Reject certain kinds of input (same behavior as canvas)
                            return;
                        }

                        match action {
                            // Double press
                            1 => canvas.engine_mut().text_select_closest_word(),
                            // Triple press
                            2 => canvas.engine_mut().text_select_closest_line(),
                            _ => unreachable!(),
                        }
                    }
                ));
            }

            // Zoom with alt + shift + drag
            {
                let zoom_begin = Rc::new(Cell::new(1_f64));
                let prev_offset = Rc::new(Cell::new(Vector2::ZERO));

                self.canvas_alt_shift_drag_gesture
                    .connect_drag_begin(clone!(
                        #[strong]
                        zoom_begin,
                        #[strong]
                        prev_offset,
                        #[weak(rename_to=canvaswrapper)]
                        obj,
                        move |gesture, _, _| {
                            let modifiers = gesture.current_event_state();

                            // At the start BUTTON1_MASK is not included
                            if modifiers.contains(gdk::ModifierType::ALT_MASK)
                                && modifiers.contains(gdk::ModifierType::SHIFT_MASK)
                            {
                                gesture.set_state(EventSequenceState::Claimed);

                                // workaround for https://gitlab.gnome.org/GNOME/gtk/-/issues/187
                                canvaswrapper.imp().workaround_disable_kinetic_scrolling();

                                let current_zoom =
                                    canvaswrapper.canvas().engine_ref().camera.total_zoom();
                                zoom_begin.set(current_zoom);
                                prev_offset.set(Vector2::ZERO);
                            } else {
                                gesture.set_state(EventSequenceState::Denied);
                            }
                        }
                    ));

                self.canvas_alt_shift_drag_gesture
                    .connect_drag_update(clone!(
                        #[strong]
                        prev_offset,
                        #[weak(rename_to=canvaswrapper)]
                        obj,
                        move |_, offset_x, offset_y| {
                            let canvas = canvaswrapper.canvas();
                            let new_offset = Vector2::new(offset_x, offset_y);
                            let current_total_zoom =
                                canvaswrapper.canvas().engine_ref().camera.total_zoom();
                            // drag down zooms out, drag up zooms in
                            let new_zoom = current_total_zoom
                                * (1.0
                                    - (new_offset[1] - prev_offset.get()[1])
                                        * Camera::DRAG_ZOOM_MAGN_ZOOM_FACTOR);

                            if (Camera::ZOOM_MIN..=Camera::ZOOM_MAX).contains(&new_zoom) {
                                let viewport_center = canvas.engine_ref().camera.viewport_center();

                                let mut widget_flags = canvas.engine_mut().zoom_w_timeout(new_zoom);
                                widget_flags |= canvas
                                    .engine_mut()
                                    .camera
                                    .set_viewport_center(viewport_center);
                                widget_flags |= canvas.engine_mut().doc_expand_autoexpand();
                                canvas.emit_handle_widget_flags(widget_flags);
                            }

                            prev_offset.set(new_offset);
                        }
                    ));

                self.canvas_alt_shift_drag_gesture.connect_drag_end(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, _, _| {
                        // workaround for https://gitlab.gnome.org/GNOME/gtk/-/issues/187
                        canvaswrapper.imp().workaround_restore_kinetic_scrolling();

                        let widget_flags = canvaswrapper
                            .canvas()
                            .engine_mut()
                            .update_rendering_current_viewport();
                        canvaswrapper
                            .canvas()
                            .emit_handle_widget_flags(widget_flags);
                    }
                ));
            }

            {
                // Shortcut with touch two-finger long-press.
                self.touch_two_finger_long_press_gesture
                    .connect_pressed(clone!(
                        #[weak(rename_to=canvaswrapper)]
                        obj,
                        move |_gesture, _, _| {
                            let (_, widget_flags) = canvaswrapper
                                .canvas()
                                .engine_mut()
                                .handle_pressed_shortcut_key(
                                    ShortcutKey::TouchTwoFingerLongPress,
                                    Instant::now(),
                                );
                            canvaswrapper
                                .canvas()
                                .emit_handle_widget_flags(widget_flags);
                        }
                    ));
            }

            {
                // Context menu
                self.touch_long_press_gesture.connect_pressed(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_gesture, x, y| {
                        let popover = canvaswrapper.contextmenu().popover();
                        canvaswrapper
                            .imp()
                            .last_contextmenu_pos
                            .set(Some(Vector2::new(x, y)));
                        popover
                            .set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 4, 4)));
                        popover.popup();
                    }
                ));
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnCanvasWrapper(ObjectSubclass<imp::RnCanvasWrapper>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnCanvasWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RnCanvasWrapper {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn show_scrollbars(&self) -> bool {
        self.property::<bool>("show-scrollbars")
    }

    #[allow(unused)]
    pub(crate) fn set_show_scrollbars(&self, show_scrollbars: bool) {
        self.set_property("show-scrollbars", show_scrollbars.to_value());
    }
    #[allow(unused)]
    pub(crate) fn block_pinch_zoom(&self) -> bool {
        self.property::<bool>("block-pinch-zoom")
    }

    #[allow(unused)]
    pub(crate) fn set_block_pinch_zoom(&self, block_pinch_zoom: bool) {
        self.set_property("block-pinch-zoom", block_pinch_zoom);
    }

    #[allow(unused)]
    pub(crate) fn inertial_scrolling(&self) -> bool {
        self.property::<bool>("inertial-scrolling")
    }

    #[allow(unused)]
    pub(crate) fn set_inertial_scrolling(&self, inertial_scrolling: bool) {
        self.set_property("inertial-scrolling", inertial_scrolling);
    }

    pub(crate) fn pointer_pos(&self) -> Option<Vector2> {
        self.imp().pointer_pos.get()
    }

    pub(crate) fn last_contextmenu_pos(&self) -> Option<Vector2> {
        self.imp().last_contextmenu_pos.get()
    }

    pub(crate) fn scroller(&self) -> ScrolledWindow {
        self.imp().scroller.get()
    }

    pub(crate) fn workaround_cancel_kinetic_scrolling_for_zoom(&self) {
        self.imp().workaround_cancel_kinetic_scrolling_for_zoom();
    }

    pub(crate) fn canvas(&self) -> RnCanvas {
        self.imp().canvas.get()
    }

    pub(crate) fn contextmenu(&self) -> RnContextMenu {
        self.imp().contextmenu.get()
    }

    /// Initializes for the given appwindow. Usually `init()` is only called once,
    /// but because this widget can be moved across appwindows through tabs,
    /// this function also disconnects and replaces all existing old connections
    ///
    /// The same method of the canvas child is chained up in here.
    pub(crate) fn init_reconnect(&self, appwindow: &RnAppWindow) {
        self.imp().canvas.init_reconnect(appwindow);

        let appwindow_block_pinch_zoom_bind = appwindow
            .bind_property("block-pinch-zoom", self, "block_pinch_zoom")
            .sync_create()
            .build();

        let appwindow_show_scrollbars_bind = appwindow
            .sidebar()
            .settings_panel()
            .general_show_scrollbars_row()
            .bind_property("active", self, "show-scrollbars")
            .sync_create()
            .build();

        let appwindow_inertial_scrolling_bind = appwindow
            .sidebar()
            .settings_panel()
            .general_inertial_scrolling_row()
            .bind_property("active", self, "inertial-scrolling")
            .sync_create()
            .build();

        let appwindow_righthanded_bind = appwindow
            .bind_property("righthanded", &self.scroller(), "window-placement")
            .transform_to(|_, righthanded: bool| {
                if righthanded {
                    Some(CornerType::BottomRight)
                } else {
                    Some(CornerType::BottomLeft)
                }
            })
            .sync_create()
            .build();

        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections
            .appwindow_block_pinch_zoom_bind
            .replace(appwindow_block_pinch_zoom_bind)
        {
            old.unbind()
        }
        if let Some(old) = connections
            .appwindow_show_scrollbars_bind
            .replace(appwindow_show_scrollbars_bind)
        {
            old.unbind();
        }
        if let Some(old) = connections
            .appwindow_inertial_scrolling_bind
            .replace(appwindow_inertial_scrolling_bind)
        {
            old.unbind();
        }
        if let Some(old) = connections
            .appwindow_righthanded_bind
            .replace(appwindow_righthanded_bind)
        {
            old.unbind();
        }
    }

    /// This disconnects all connections with references to external objects,
    /// to prepare moving the widget to another appwindow.
    ///
    /// The same method of the canvas child is chained up in here.
    pub(crate) fn disconnect_connections(&self) {
        self.canvas().disconnect_connections();

        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections.appwindow_block_pinch_zoom_bind.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_show_scrollbars_bind.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_inertial_scrolling_bind.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_righthanded_bind.take() {
            old.unbind();
        }
    }

    /// When the widget is the child of a tab page, we want to connect the title, icons, ..
    ///
    /// disconnects existing connections to old tab pages.
    ///
    /// The same method of the canvas child is chained up in here.
    pub(crate) fn connect_to_tab_page(&self, page: &adw::TabPage) {
        self.canvas().connect_to_tab_page(page);
    }
}
