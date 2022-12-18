use std::cell::Cell;
use std::rc::Rc;

use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, Button, CompositeTemplate,
    EventControllerScroll, EventControllerScrollFlags, EventSequenceState, GestureDrag,
    GestureZoom, Inhibit, ProgressBar, PropagationPhase, Revealer, ScrolledWindow, Widget,
};
use rnote_engine::Camera;

use crate::{RnoteAppWindow, RnoteCanvas};

mod imp {
    use std::cell::{Cell, RefCell};

    use once_cell::sync::Lazy;

    use super::*;

    #[allow(missing_debug_implementations)]
    #[derive(CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/canvaswrapper.ui")]
    pub(crate) struct RnoteCanvasWrapper {
        pub(crate) permanently_hide_scrollbars: Cell<bool>,

        pub(crate) progresspulse_source_id: RefCell<Option<glib::SourceId>>,
        pub(crate) canvas_touch_drag_gesture: GestureDrag,
        pub(crate) canvas_drag_empty_area_gesture: GestureDrag,
        pub(crate) canvas_zoom_gesture: GestureZoom,
        pub(crate) canvas_zoom_scroll_controller: EventControllerScroll,
        pub(crate) canvas_mouse_drag_middle_gesture: GestureDrag,
        pub(crate) canvas_alt_drag_gesture: GestureDrag,
        pub(crate) canvas_alt_shift_drag_gesture: GestureDrag,

        #[template_child]
        pub(crate) toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub(crate) progressbar: TemplateChild<ProgressBar>,
        #[template_child]
        pub(crate) quickactions_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) fixedsize_quickactions_revealer: TemplateChild<Revealer>,
        #[template_child]
        pub(crate) undo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) redo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) canvas: TemplateChild<RnoteCanvas>,
    }

    impl Default for RnoteCanvasWrapper {
        fn default() -> Self {
            let canvas_touch_drag_gesture = GestureDrag::builder()
                .name("canvas_touch_drag_gesture")
                .touch_only(true)
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            let canvas_drag_empty_area_gesture = GestureDrag::builder()
                .name("canvas_mouse_drag_empty_area_gesture")
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

            Self {
                permanently_hide_scrollbars: Cell::new(false),

                progresspulse_source_id: RefCell::new(None),
                canvas_touch_drag_gesture,
                canvas_drag_empty_area_gesture,
                canvas_zoom_gesture,
                canvas_zoom_scroll_controller,
                canvas_mouse_drag_middle_gesture,
                canvas_alt_drag_gesture,
                canvas_alt_shift_drag_gesture,

                toast_overlay: TemplateChild::<adw::ToastOverlay>::default(),
                progressbar: TemplateChild::<ProgressBar>::default(),
                quickactions_box: TemplateChild::<gtk4::Box>::default(),
                fixedsize_quickactions_revealer: TemplateChild::<Revealer>::default(),
                undo_button: TemplateChild::<Button>::default(),
                redo_button: TemplateChild::<Button>::default(),
                scroller: TemplateChild::<ScrolledWindow>::default(),
                canvas: TemplateChild::<RnoteCanvas>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteCanvasWrapper {
        const NAME: &'static str = "RnoteCanvasWrapper";
        type Type = super::RnoteCanvasWrapper;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnoteCanvasWrapper {
        fn constructed(&self) {
            self.parent_constructed();

            // Add input controllers
            self.scroller
                .add_controller(&self.canvas_touch_drag_gesture);
            self.scroller
                .add_controller(&self.canvas_drag_empty_area_gesture);
            self.scroller.add_controller(&self.canvas_zoom_gesture);
            self.scroller
                .add_controller(&self.canvas_zoom_scroll_controller);
            self.scroller
                .add_controller(&self.canvas_mouse_drag_middle_gesture);
            self.scroller.add_controller(&self.canvas_alt_drag_gesture);
            self.scroller
                .add_controller(&self.canvas_alt_shift_drag_gesture);
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    // permanently hide canvas scrollbars
                    glib::ParamSpecBoolean::new(
                        "permanently-hide-scrollbars",
                        "permanently-hide-scrollbars",
                        "permanently-hide-scrollbars",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "permanently-hide-scrollbars" => self.permanently_hide_scrollbars.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "permanently-hide-scrollbars" => {
                    let permanently_hide_canvas_scrollbars = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`.");

                    self.permanently_hide_scrollbars
                        .replace(permanently_hide_canvas_scrollbars);

                    if permanently_hide_canvas_scrollbars {
                        self.scroller.hscrollbar().set_visible(false);
                        self.scroller.vscrollbar().set_visible(false);
                    } else {
                        self.scroller.hscrollbar().set_visible(true);
                        self.scroller.vscrollbar().set_visible(true);
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnoteCanvasWrapper {}
}

glib::wrapper! {
    pub(crate) struct RnoteCanvasWrapper(ObjectSubclass<imp::RnoteCanvasWrapper>)
    @extends Widget;
}

impl Default for RnoteCanvasWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RnoteCanvasWrapper {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    #[allow(unused)]
    pub(crate) fn permanently_hide_scrollbars(&self) -> bool {
        self.property::<bool>("permanently-hide-scrollbars")
    }

    #[allow(unused)]
    pub(crate) fn set_permanently_hide_scrollbars(&self, permanently_hide_canvas_scrollbars: bool) {
        self.set_property(
            "permanently-hide-scrollbars",
            permanently_hide_canvas_scrollbars.to_value(),
        );
    }

    pub(crate) fn quickactions_box(&self) -> gtk4::Box {
        self.imp().quickactions_box.get()
    }

    pub(crate) fn fixedsize_quickactions_revealer(&self) -> Revealer {
        self.imp().fixedsize_quickactions_revealer.get()
    }

    pub(crate) fn undo_button(&self) -> Button {
        self.imp().undo_button.get()
    }

    pub(crate) fn redo_button(&self) -> Button {
        self.imp().redo_button.get()
    }

    pub(crate) fn toast_overlay(&self) -> adw::ToastOverlay {
        self.imp().toast_overlay.get()
    }
    pub(crate) fn progressbar(&self) -> ProgressBar {
        self.imp().progressbar.get()
    }

    pub(crate) fn scroller(&self) -> ScrolledWindow {
        self.imp().scroller.get()
    }

    pub(crate) fn canvas(&self) -> RnoteCanvas {
        self.imp().canvas.get()
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        self.setup_input(appwindow);

        imp.canvas.init(appwindow);
    }

    pub(crate) fn setup_input(&self, appwindow: &RnoteAppWindow) {
        // zoom scrolling with <ctrl> + scroll
        {
            self.imp().canvas_zoom_scroll_controller.connect_scroll(clone!(@weak appwindow => @default-return Inhibit(false), move |controller, _, dy| {
                if controller.current_event_state() == gdk::ModifierType::CONTROL_MASK {
                    let new_zoom = appwindow.canvas().engine().borrow().camera.total_zoom() * (1.0 - dy * RnoteCanvas::ZOOM_STEP);

                    let current_doc_center = appwindow.canvas().current_center_on_doc();
                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
                    appwindow.canvas().center_around_coord_on_doc(current_doc_center);

                    // Stop event propagation
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            }));
        }

        // Drag canvas with touch gesture
        {
            let touch_drag_start = Rc::new(Cell::new(na::vector![0.0, 0.0]));

            self.imp().canvas_touch_drag_gesture.connect_drag_begin(
                clone!(@strong touch_drag_start, @weak appwindow => move |_, _, _| {
                    // We don't claim the sequence, because we we want to allow touch zooming. When the zoom gesture is recognized, it claims it and denies this touch drag gesture.

                    touch_drag_start.set(na::vector![
                        appwindow.canvas().hadjustment().unwrap().value(),
                        appwindow.canvas().vadjustment().unwrap().value()
                    ]);
                }),
            );
            self.imp().canvas_touch_drag_gesture.connect_drag_update(
                clone!(@strong touch_drag_start, @weak appwindow => move |_, x, y| {
                    let new_adj_values = touch_drag_start.get() - na::vector![x,y];

                    appwindow.canvas().update_camera_offset(new_adj_values);
                }),
            );
        }

        // Move Canvas with middle mouse button
        {
            let mouse_drag_start = Rc::new(Cell::new(na::vector![0.0, 0.0]));

            self.imp()
                .canvas_mouse_drag_middle_gesture
                .connect_drag_begin(
                    clone!(@strong mouse_drag_start, @weak appwindow => move |_, _, _| {
                        mouse_drag_start.set(na::vector![
                            appwindow.canvas().hadjustment().unwrap().value(),
                            appwindow.canvas().vadjustment().unwrap().value()
                        ]);
                    }),
                );
            self.imp()
                .canvas_mouse_drag_middle_gesture
                .connect_drag_update(
                    clone!(@strong mouse_drag_start, @weak appwindow => move |_, x, y| {
                        let new_adj_values = mouse_drag_start.get() - na::vector![x,y];

                        appwindow.canvas().update_camera_offset(new_adj_values);
                    }),
                );

            self.imp()
                .canvas_mouse_drag_middle_gesture
                .connect_drag_end(clone!(@weak self as appwindow => move |_, _, _| {
                    appwindow.canvas().update_engine_rendering();
                }));
        }

        // Move Canvas by dragging in the empty area around the canvas
        {
            let mouse_drag_empty_area_start = Rc::new(Cell::new(na::vector![0.0, 0.0]));

            self.imp().canvas_drag_empty_area_gesture.connect_drag_begin(clone!(@strong mouse_drag_empty_area_start, @weak appwindow => move |_, _x, _y| {
                mouse_drag_empty_area_start.set(na::vector![
                    appwindow.canvas().hadjustment().unwrap().value(),
                    appwindow.canvas().vadjustment().unwrap().value()
                ]);
            }));
            self.imp()
                .canvas_drag_empty_area_gesture
                .connect_drag_update(
                    clone!(@strong mouse_drag_empty_area_start, @weak appwindow => move |_, x, y| {
                        let new_adj_values = mouse_drag_empty_area_start.get() - na::vector![x,y];

                        appwindow.canvas().update_camera_offset(new_adj_values);
                    }),
                );
        }

        // Canvas gesture zooming with dragging
        {
            let prev_scale = Rc::new(Cell::new(1_f64));
            let zoom_begin = Rc::new(Cell::new(1_f64));
            let new_zoom = Rc::new(Cell::new(1.0));
            let bbcenter_begin: Rc<Cell<Option<na::Vector2<f64>>>> = Rc::new(Cell::new(None));
            let adjs_begin = Rc::new(Cell::new(na::vector![0.0, 0.0]));

            self.imp().canvas_zoom_gesture.connect_begin(clone!(
                @strong zoom_begin,
                @strong new_zoom,
                @strong prev_scale,
                @strong bbcenter_begin,
                @strong adjs_begin,
                @weak self as appwindow => move |gesture, _| {
                    gesture.set_state(EventSequenceState::Claimed);

                    let current_zoom = appwindow.canvas().engine().borrow().camera.total_zoom();

                    zoom_begin.set(current_zoom);
                    new_zoom.set(current_zoom);
                    prev_scale.set(1.0);

                    bbcenter_begin.set(gesture.bounding_box_center().map(|coords| na::vector![coords.0, coords.1]));
                    adjs_begin.set(na::vector![
                        appwindow.canvas().hadjustment().unwrap().value(),
                        appwindow.canvas().vadjustment().unwrap().value()
                        ]);
            }));

            self.imp().canvas_zoom_gesture.connect_scale_changed(clone!(
                @strong zoom_begin,
                @strong new_zoom,
                @strong prev_scale,
                @strong bbcenter_begin,
                @strong adjs_begin,
                @weak appwindow => move |gesture, scale| {
                    if zoom_begin.get() * scale <= Camera::ZOOM_MAX && zoom_begin.get() * scale >= Camera::ZOOM_MIN {
                        new_zoom.set(zoom_begin.get() * scale);
                        prev_scale.set(scale);
                    }

                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.get().to_variant()));

                    if let Some(bbcenter_current) = gesture.bounding_box_center().map(|coords| na::vector![coords.0, coords.1]) {
                        let bbcenter_begin = if let Some(bbcenter_begin) = bbcenter_begin.get() {
                            bbcenter_begin
                        } else {
                            // Set the center if not set by gesture begin handler
                            bbcenter_begin.set(Some(bbcenter_current));
                            bbcenter_current
                        };

                        let bbcenter_delta = bbcenter_current - bbcenter_begin * prev_scale.get();
                        let new_adj_values = adjs_begin.get() * prev_scale.get() - bbcenter_delta;

                        appwindow.canvas().update_camera_offset(new_adj_values);
                    }
            }));

            self.imp().canvas_zoom_gesture.connect_cancel(
                clone!(@weak appwindow => move |canvas_zoom_gesture, _event_sequence| {
                    canvas_zoom_gesture.set_state(EventSequenceState::Denied);
                }),
            );

            self.imp().canvas_zoom_gesture.connect_end(
                clone!(@weak appwindow => move |canvas_zoom_gesture, _event_sequence| {
                    canvas_zoom_gesture.set_state(EventSequenceState::Denied);
                }),
            );
        }

        // Pan with alt + drag
        {
            let adj_start = Rc::new(Cell::new(na::Vector2::<f64>::zeros()));

            self.imp()
                .canvas_alt_drag_gesture
                .connect_drag_begin(clone!(
                    @strong adj_start,
                    @weak self as appwindow => move |gesture, _, _| {
                        let modifiers = gesture.current_event_state();

                        // At the start BUTTON1_MASK is not included
                        if modifiers == gdk::ModifierType::ALT_MASK {
                            gesture.set_state(EventSequenceState::Claimed);

                            adj_start.set(na::vector![
                                appwindow.canvas().hadjustment().unwrap().value(),
                                appwindow.canvas().vadjustment().unwrap().value()
                            ]);
                        } else {
                            gesture.set_state(EventSequenceState::Denied);
                        }
                }));

            self.imp()
                .canvas_alt_drag_gesture
                .connect_drag_update(clone!(
                    @strong adj_start,
                    @weak appwindow => move |_, offset_x, offset_y| {
                        let new_adj_values = adj_start.get() - na::vector![offset_x, offset_y];
                        appwindow.canvas().update_camera_offset(new_adj_values);
                }));
        }

        // Zoom with alt + shift + drag
        {
            let zoom_begin = Rc::new(Cell::new(1_f64));
            let prev_offset = Rc::new(Cell::new(na::Vector2::<f64>::zeros()));

            self.imp()
                .canvas_alt_shift_drag_gesture
                .connect_drag_begin(clone!(
                @strong zoom_begin,
                @strong prev_offset,
                @weak self as appwindow => move |gesture, _, _| {
                    let modifiers = gesture.current_event_state();

                    // At the start BUTTON1_MASK is not included
                    if modifiers == (gdk::ModifierType::SHIFT_MASK | gdk::ModifierType::ALT_MASK) {
                        gesture.set_state(EventSequenceState::Claimed);
                        let current_zoom = appwindow.canvas().engine().borrow().camera.total_zoom();

                        zoom_begin.set(current_zoom);
                        prev_offset.set(na::Vector2::<f64>::zeros());
                    } else {
                        gesture.set_state(EventSequenceState::Denied);
                    }
                }));

            self.imp().canvas_alt_shift_drag_gesture.connect_drag_update(clone!(
                @strong zoom_begin,
                @strong prev_offset,
                @weak appwindow => move |_, offset_x, offset_y| {
                    // 0.5% zoom for every pixel in y dir
                    const OFFSET_MAGN_ZOOM_LVL_FACTOR: f64 = 0.005;

                    let new_offset = na::vector![offset_x, offset_y];
                    let cur_zoom = appwindow.canvas().engine().borrow().camera.total_zoom();

                    // Drag down zooms out, drag up zooms in
                    let new_zoom = cur_zoom * (1.0 + (prev_offset.get()[1] - new_offset[1]) * OFFSET_MAGN_ZOOM_LVL_FACTOR);

                    if new_zoom <= Camera::ZOOM_MAX && new_zoom >= Camera::ZOOM_MIN {
                        let current_doc_center = appwindow.canvas().current_center_on_doc();
                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
                        appwindow.canvas().center_around_coord_on_doc(current_doc_center);
                    }

                    prev_offset.set(new_offset);
            }));
        }
    }

    pub(crate) fn canvas_touch_drag_gesture_enable(&self, enable: bool) {
        if enable {
            self.imp()
                .canvas_touch_drag_gesture
                .set_propagation_phase(PropagationPhase::Bubble);
        } else {
            self.imp()
                .canvas_touch_drag_gesture
                .set_propagation_phase(PropagationPhase::None);
        }
    }

    pub(crate) fn canvas_drag_empty_area_gesture_enable(&self, enable: bool) {
        if enable {
            self.imp()
                .canvas_drag_empty_area_gesture
                .set_propagation_phase(PropagationPhase::Bubble);
        } else {
            self.imp()
                .canvas_drag_empty_area_gesture
                .set_propagation_phase(PropagationPhase::None);
        }
    }

    pub(crate) fn canvas_zoom_gesture_enable(&self, enable: bool) {
        if enable {
            self.imp()
                .canvas_zoom_gesture
                .set_propagation_phase(PropagationPhase::Capture);
        } else {
            self.imp()
                .canvas_zoom_gesture
                .set_propagation_phase(PropagationPhase::None);
        }
    }

    pub(crate) fn start_pulsing_progressbar(&self) {
        const PROGRESS_BAR_PULSE_INTERVAL: std::time::Duration =
            std::time::Duration::from_millis(300);

        if let Some(old_pulse_source) = self.imp().progresspulse_source_id.replace(Some(glib::source::timeout_add_local(
            PROGRESS_BAR_PULSE_INTERVAL,
            clone!(@weak self as appwindow => @default-return glib::source::Continue(false), move || {
                appwindow.progressbar().pulse();

                glib::source::Continue(true)
            })),
        )) {
            old_pulse_source.remove();
        }
    }

    pub(crate) fn finish_progressbar(&self) {
        const PROGRESS_BAR_TIMEOUT_TIME: std::time::Duration =
            std::time::Duration::from_millis(300);

        if let Some(pulse_source) = self.imp().progresspulse_source_id.take() {
            pulse_source.remove();
        }

        self.progressbar().set_fraction(1.0);

        glib::source::timeout_add_local_once(
            PROGRESS_BAR_TIMEOUT_TIME,
            clone!(@weak self as appwindow => move || {
                appwindow.progressbar().set_fraction(0.0);
            }),
        );
    }

    #[allow(unused)]
    pub(crate) fn abort_progressbar(&self) {
        if let Some(pulse_source) = self.imp().progresspulse_source_id.take() {
            pulse_source.remove();
        }

        self.progressbar().set_fraction(0.0);
    }

    pub(crate) fn dispatch_toast_w_button<F: Fn(&adw::Toast) + 'static>(
        &self,
        text: &str,
        button_label: &str,
        button_callback: F,
        timeout: u32,
    ) -> adw::Toast {
        let text_notify_toast = adw::Toast::builder()
            .title(text)
            .priority(adw::ToastPriority::High)
            .button_label(button_label)
            .timeout(timeout)
            .build();

        text_notify_toast.connect_button_clicked(button_callback);
        self.toast_overlay().add_toast(&text_notify_toast);

        text_notify_toast
    }

    /// Ensures that only one toast per `singleton_toast` is queued at the same time by dismissing the previous toast.
    ///
    /// `singleton_toast` is a mutable reference to an `Option<Toast>`. It will always hold the most recently dispatched toast
    /// and it should not be modified, because it's used to keep track of previous toasts.
    pub(crate) fn dispatch_toast_w_button_singleton<F: Fn(&adw::Toast) + 'static>(
        &self,
        text: &str,
        button_label: &str,
        button_callback: F,
        timeout: u32,
        singleton_toast: &mut Option<adw::Toast>,
    ) {
        if let Some(previous_toast) = singleton_toast {
            previous_toast.dismiss();
        }

        let text_notify_toast =
            self.dispatch_toast_w_button(text, button_label, button_callback, timeout);
        *singleton_toast = Some(text_notify_toast);
    }

    pub(crate) fn dispatch_toast_text(&self, text: &str) {
        let text_notify_toast = adw::Toast::builder()
            .title(text)
            .priority(adw::ToastPriority::High)
            .timeout(5)
            .build();

        self.toast_overlay().add_toast(&text_notify_toast);
    }

    pub(crate) fn dispatch_toast_error(&self, error: &String) {
        let text_notify_toast = adw::Toast::builder()
            .title(error.as_str())
            .priority(adw::ToastPriority::High)
            .timeout(0)
            .build();

        log::error!("{error}");

        self.toast_overlay().add_toast(&text_notify_toast);
    }
}
