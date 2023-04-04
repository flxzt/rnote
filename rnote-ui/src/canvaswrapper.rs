use gtk4::CornerType;
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate,
    EventControllerScroll, EventControllerScrollFlags, EventSequenceState, GestureDrag,
    GestureLongPress, GestureZoom, Inhibit, PropagationPhase, ScrolledWindow, Widget,
};
use once_cell::sync::Lazy;
use rnote_compose::penevents::ShortcutKey;
use rnote_engine::Camera;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use crate::{RnAppWindow, RnCanvas};

mod imp {

    use super::*;

    #[allow(missing_debug_implementations)]
    #[derive(CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/canvaswrapper.ui")]
    pub(crate) struct RnCanvasWrapper {
        pub(crate) show_scrollbars: Cell<bool>,
        pub(crate) block_pinch_zoom: Cell<bool>,

        pub(crate) appwindow_show_scrollbars_bind: RefCell<Option<glib::Binding>>,
        pub(crate) appwindow_righthanded_bind: RefCell<Option<glib::Binding>>,

        pub(crate) canvas_drag_gesture: GestureDrag,
        pub(crate) canvas_zoom_gesture: GestureZoom,
        pub(crate) canvas_zoom_scroll_controller: EventControllerScroll,
        pub(crate) canvas_mouse_drag_middle_gesture: GestureDrag,
        pub(crate) canvas_alt_drag_gesture: GestureDrag,
        pub(crate) canvas_alt_shift_drag_gesture: GestureDrag,
        pub(crate) touch_two_finger_long_press_gesture: GestureLongPress,

        #[template_child]
        pub(crate) scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) canvas: TemplateChild<RnCanvas>,
    }

    impl Default for RnCanvasWrapper {
        fn default() -> Self {
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

            let canvas_zoom_scroll_controller = EventControllerScroll::builder()
                .name("canvas_zoom_scroll_controller")
                .propagation_phase(PropagationPhase::Bubble)
                .flags(EventControllerScrollFlags::VERTICAL)
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

            Self {
                show_scrollbars: Cell::new(false),
                block_pinch_zoom: Cell::new(false),

                appwindow_show_scrollbars_bind: RefCell::new(None),
                appwindow_righthanded_bind: RefCell::new(None),

                canvas_drag_gesture,
                canvas_zoom_gesture,
                canvas_zoom_scroll_controller,
                canvas_mouse_drag_middle_gesture,
                canvas_alt_drag_gesture,
                canvas_alt_shift_drag_gesture,
                touch_two_finger_long_press_gesture,

                scroller: TemplateChild::<ScrolledWindow>::default(),
                canvas: TemplateChild::<RnCanvas>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnCanvasWrapper {
        const NAME: &'static str = "RnCanvasWrapper";
        type Type = super::RnCanvasWrapper;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
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
                .add_controller(self.canvas_drag_gesture.clone());
            self.scroller
                .add_controller(self.canvas_zoom_gesture.clone());
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

            // group
            self.touch_two_finger_long_press_gesture
                .group_with(&self.canvas_zoom_gesture);

            self.setup_input();

            self.canvas.connect_notify_local(
                Some("touch-drawing"),
                clone!(@weak obj as canvaswrapper => move |_canvas, _pspec| {
                    // Disable the zoom gesture when touch drawing is enabled
                    canvaswrapper.canvas_zoom_gesture_update();
                }),
            );
        }

        fn dispose(&self) {
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
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "show-scrollbars" => self.show_scrollbars.get().to_value(),
                "block-pinch-zoom" => self.block_pinch_zoom.get().to_value(),
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
                }

                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnCanvasWrapper {}

    impl RnCanvasWrapper {
        fn setup_input(&self) {
            let obj = self.obj();

            // zoom scrolling with <ctrl> + scroll
            {
                self.canvas_zoom_scroll_controller.connect_scroll(
                    clone!(@weak obj as canvaswrapper => @default-return Inhibit(false), move |controller, _, dy| {
                    if controller.current_event_state() == gdk::ModifierType::CONTROL_MASK {
                        let canvas = canvaswrapper.canvas();
                        let new_zoom = canvas.engine().borrow().camera.total_zoom() * (1.0 - dy * RnCanvas::ZOOM_STEP);
                        let center_offset = canvas.current_view_center_coords();
                        canvas.zoom_temporarily_then_scale_to_after_timeout(new_zoom);
                        canvas.center_view_around_coords(center_offset);

                        // Stop event propagation
                        Inhibit(true)
                    } else {
                        Inhibit(false)
                    }
                }));
            }

            // Drag canvas gesture
            {
                let touch_drag_start = Rc::new(Cell::new(na::vector![0.0, 0.0]));

                self.canvas_drag_gesture.connect_drag_begin(
                    clone!(@strong touch_drag_start, @weak obj as canvaswrapper => move |_, _, _| {
                        // We don't claim the sequence, because we we want to allow touch zooming.
                        // When the zoom gesture is recognized, it claims it and denies this touch drag gesture.

                        touch_drag_start.set(na::vector![
                            canvaswrapper.canvas().hadjustment().unwrap().value(),
                            canvaswrapper.canvas().vadjustment().unwrap().value()
                        ]);
                    }),
                );
                self.canvas_drag_gesture.connect_drag_update(
                    clone!(@strong touch_drag_start, @weak obj as canvaswrapper => move |_, x, y| {
                        let new_offset = touch_drag_start.get() - na::vector![x,y];
                        canvaswrapper.canvas().update_camera_offset(new_offset);
                    }),
                );
                self.canvas_drag_gesture.connect_drag_end(
                    clone!(@weak obj as canvaswrapper => move |_, _, _| {
                        canvaswrapper.canvas().update_engine_rendering();
                    }),
                );
            }

            // Move Canvas with middle mouse button
            {
                let mouse_drag_start = Rc::new(Cell::new(na::vector![0.0, 0.0]));

                self.canvas_mouse_drag_middle_gesture.connect_drag_begin(
                    clone!(@strong mouse_drag_start, @weak obj as canvaswrapper => move |_, _, _| {
                        mouse_drag_start.set(canvaswrapper.canvas().engine().borrow().camera.offset);
                    }),
                );
                self.canvas_mouse_drag_middle_gesture.connect_drag_update(
                    clone!(@strong mouse_drag_start, @weak obj as canvaswrapper => move |_, x, y| {
                        let new_offset = mouse_drag_start.get() - na::vector![x,y];
                        canvaswrapper.canvas().update_camera_offset(new_offset);
                    }),
                );
                self.canvas_mouse_drag_middle_gesture.connect_drag_end(
                    clone!(@weak obj as canvaswrapper => move |_, _, _| {
                        canvaswrapper.canvas().update_engine_rendering();
                    }),
                );
            }

            // Canvas gesture zooming with dragging
            {
                let prev_scale = Rc::new(Cell::new(1_f64));
                let zoom_begin = Rc::new(Cell::new(1_f64));
                let new_zoom = Rc::new(Cell::new(1.0));
                let bbcenter_begin: Rc<Cell<Option<na::Vector2<f64>>>> = Rc::new(Cell::new(None));
                let offset_begin = Rc::new(Cell::new(na::vector![0.0, 0.0]));

                self.canvas_zoom_gesture.connect_begin(clone!(
                    @strong zoom_begin,
                    @strong new_zoom,
                    @strong prev_scale,
                    @strong bbcenter_begin,
                    @strong offset_begin,
                    @weak obj as canvaswrapper => move |gesture, _| {
                        gesture.set_state(EventSequenceState::Claimed);
                        let current_zoom = canvaswrapper.canvas().engine().borrow().camera.total_zoom();

                        zoom_begin.set(current_zoom);
                        new_zoom.set(current_zoom);
                        prev_scale.set(1.0);

                        bbcenter_begin.set(gesture.bounding_box_center().map(|coords| na::vector![coords.0, coords.1]));
                        offset_begin.set(canvaswrapper.canvas().engine().borrow().camera.offset);
                    })
                );

                self.canvas_zoom_gesture.connect_scale_changed(clone!(
                    @strong zoom_begin,
                    @strong new_zoom,
                    @strong prev_scale,
                    @strong bbcenter_begin,
                    @strong offset_begin,
                    @weak obj as canvaswrapper => move |gesture, scale| {
                        if (Camera::ZOOM_MIN..=Camera::ZOOM_MAX).contains(&(zoom_begin.get() * scale)) {
                            new_zoom.set(zoom_begin.get() * scale);
                            prev_scale.set(scale);
                        }
                        canvaswrapper.canvas().zoom_temporarily_then_scale_to_after_timeout(new_zoom.get());

                        if let Some(bbcenter_current) = gesture.bounding_box_center().map(|coords| na::vector![coords.0, coords.1]) {
                            let bbcenter_begin = if let Some(bbcenter_begin) = bbcenter_begin.get() {
                                bbcenter_begin
                            } else {
                                // Set the center if not set by gesture begin handler
                                bbcenter_begin.set(Some(bbcenter_current));
                                bbcenter_current
                            };
                            let bbcenter_delta = bbcenter_current - bbcenter_begin * prev_scale.get();
                            let new_offset = offset_begin.get() * prev_scale.get() - bbcenter_delta;
                            canvaswrapper.canvas().update_camera_offset(new_offset);
                        }
                    })
                );

                self.canvas_zoom_gesture.connect_end(
                    clone!(@weak obj as canvaswrapper => move |gesture, _event_sequence| {
                        gesture.set_state(EventSequenceState::Denied);
                        canvaswrapper.canvas().update_engine_rendering();
                    }),
                );

                self.canvas_zoom_gesture.connect_cancel(
                    clone!(@weak obj as canvaswrapper => move |gesture, _event_sequence| {
                        gesture.set_state(EventSequenceState::Denied);
                        canvaswrapper.canvas().update_engine_rendering();
                    }),
                );
            }

            // Pan with alt + drag
            {
                let offset_start = Rc::new(Cell::new(na::Vector2::<f64>::zeros()));

                self.canvas_alt_drag_gesture.connect_drag_begin(clone!(
                    @strong offset_start,
                    @weak obj as canvaswrapper => move |gesture, _, _| {
                        let modifiers = gesture.current_event_state();

                        // At the start BUTTON1_MASK is not included
                        if modifiers == gdk::ModifierType::ALT_MASK {
                            gesture.set_state(EventSequenceState::Claimed);
                            offset_start.set(canvaswrapper.canvas().engine().borrow().camera.offset);
                        } else {
                            gesture.set_state(EventSequenceState::Denied);
                        }
                }));

                self.canvas_alt_drag_gesture.connect_drag_update(
                    clone!(@strong offset_start, @weak obj as canvaswrapper => move |_, offset_x, offset_y| {
                        let new_offset = offset_start.get() - na::vector![offset_x, offset_y];
                        canvaswrapper.canvas().update_camera_offset(new_offset);
                    })
                );

                self.canvas_alt_drag_gesture.connect_drag_end(
                    clone!(@weak obj as canvaswrapper => move |_, _, _| {
                        canvaswrapper.canvas().update_engine_rendering();
                    }),
                );
            }

            // Zoom with alt + shift + drag
            {
                let zoom_begin = Rc::new(Cell::new(1_f64));
                let prev_offset = Rc::new(Cell::new(na::Vector2::<f64>::zeros()));

                self
                .canvas_alt_shift_drag_gesture
                .connect_drag_begin(clone!(
                    @strong zoom_begin,
                    @strong prev_offset,
                    @weak obj as canvaswrapper => move |gesture, _, _| {
                        let modifiers = gesture.current_event_state();

                        // At the start BUTTON1_MASK is not included
                        if modifiers == (gdk::ModifierType::SHIFT_MASK | gdk::ModifierType::ALT_MASK) {
                            gesture.set_state(EventSequenceState::Claimed);
                            let current_zoom = canvaswrapper.canvas().engine().borrow().camera.total_zoom();
                            zoom_begin.set(current_zoom);
                            prev_offset.set(na::Vector2::<f64>::zeros());
                        } else {
                            gesture.set_state(EventSequenceState::Denied);
                        }
                    })
                );

                self.canvas_alt_shift_drag_gesture.connect_drag_update(clone!(
                    @strong zoom_begin,
                    @strong prev_offset,
                    @weak obj as canvaswrapper => move |_, offset_x, offset_y| {
                        // 0.5% zoom for every pixel in y dir
                        const OFFSET_MAGN_ZOOM_LVL_FACTOR: f64 = 0.005;

                        let new_offset = na::vector![offset_x, offset_y];
                        let cur_zoom = canvaswrapper.canvas().engine().borrow().camera.total_zoom();
                        // Drag down zooms out, drag up zooms in
                        let new_zoom = cur_zoom * (1.0 + (prev_offset.get()[1] - new_offset[1]) * OFFSET_MAGN_ZOOM_LVL_FACTOR);

                        if (Camera::ZOOM_MIN..=Camera::ZOOM_MAX).contains(&new_zoom) {
                            let current_doc_center = canvaswrapper.canvas().current_view_center_coords();
                            canvaswrapper.canvas().zoom_temporarily_then_scale_to_after_timeout(new_zoom);
                            canvaswrapper.canvas().center_view_around_coords(current_doc_center);
                        }

                        prev_offset.set(new_offset);
                    })
                );

                self.canvas_alt_shift_drag_gesture.connect_drag_end(
                    clone!(@weak obj as canvaswrapper => move |_, _, _| {
                        canvaswrapper.canvas().update_engine_rendering();
                    }),
                );
            }

            {
                // Shortcut with touch two-finger long-press.
                self.touch_two_finger_long_press_gesture.connect_pressed(clone!(@weak obj as canvaswrapper => move |_gesture, _, _| {
                    let widget_flags = canvaswrapper.canvas()
                        .engine()
                        .borrow_mut()
                        .handle_pressed_shortcut_key(ShortcutKey::TouchTwoFingerLongPress, Instant::now());
                    canvaswrapper.canvas().emit_handle_widget_flags(widget_flags);
                }));

                self.touch_two_finger_long_press_gesture.connect_end(
                    clone!(@weak obj as canvaswrapper => move |gesture, event_sequence| {
                        // Only deny the sequence that is actually handled.
                        // Because this gesture is grouped with the zoom gesture, denying all
                        // sequences within the group ( by calling `set_state()` ) might result in a segfault in certain cases
                        if let Some(event_sequence) = event_sequence {
                            gesture.set_sequence_state(event_sequence, EventSequenceState::Denied);
                        }
                    }),
                );

                self.touch_two_finger_long_press_gesture.connect_cancel(
                    clone!(@weak obj as canvaswrapper => move |gesture, event_sequence| {
                        if let Some(event_sequence) = event_sequence {
                            gesture.set_sequence_state(event_sequence, EventSequenceState::Denied);
                        }
                    }),
                );
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnCanvasWrapper(ObjectSubclass<imp::RnCanvasWrapper>)
    @extends Widget;
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
        self.canvas_zoom_gesture_update();
    }

    pub(crate) fn scroller(&self) -> ScrolledWindow {
        self.imp().scroller.get()
    }

    pub(crate) fn canvas(&self) -> RnCanvas {
        self.imp().canvas.get()
    }

    /// Initializes for the given appwindow. Usually `init()` is only called once, but since this widget can be moved across appwindows through tabs,
    /// this function also disconnects and replaces all existing old connections
    pub(crate) fn init_reconnect(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        self.imp().canvas.init_reconnect(appwindow);

        let appwindow_show_scrollbars_bind = appwindow
            .settings_panel()
            .general_show_scrollbars_switch()
            .bind_property("state", self, "show-scrollbars")
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

        if let Some(old) = imp
            .appwindow_show_scrollbars_bind
            .borrow_mut()
            .replace(appwindow_show_scrollbars_bind)
        {
            old.unbind();
        }

        if let Some(old) = imp
            .appwindow_righthanded_bind
            .borrow_mut()
            .replace(appwindow_righthanded_bind)
        {
            old.unbind();
        }
    }

    /// This disconnects all handlers with references to external objects, to prepare moving the widget to another appwindow.
    pub(crate) fn disconnect_handlers(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        self.canvas().disconnect_handlers(appwindow);

        if let Some(old) = imp.appwindow_show_scrollbars_bind.borrow_mut().take() {
            old.unbind();
        }

        if let Some(old) = imp.appwindow_righthanded_bind.borrow_mut().take() {
            old.unbind();
        }
    }

    /// When the widget is the child of a tab page, we want to connect their titles, icons, ..
    ///
    /// disconnects existing bindings / handlers to old tab pages.
    pub(crate) fn connect_to_tab_page(&self, page: &adw::TabPage) {
        self.canvas().connect_to_tab_page(page);
    }

    pub(crate) fn canvas_zoom_gesture_update(&self) {
        if !(self.block_pinch_zoom() || self.canvas().touch_drawing()) {
            self.imp()
                .canvas_zoom_gesture
                .set_propagation_phase(PropagationPhase::Capture);
        } else {
            self.imp()
                .canvas_zoom_gesture
                .set_propagation_phase(PropagationPhase::None);
        }
    }
}
