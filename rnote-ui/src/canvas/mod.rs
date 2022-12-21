mod canvaslayout;
mod input;

// Re-exports
pub(crate) use canvaslayout::CanvasLayout;
use gettextrs::gettext;
use rnote_engine::pens::PenMode;

// Imports
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::config;
use crate::utils::FileType;
use rnote_engine::RnoteEngine;

use gtk4::{
    gdk, gio, glib, glib::clone, graphene, prelude::*, subclass::prelude::*, AccessibleRole,
    Adjustment, DropTarget, EventControllerKey, EventSequenceState, GestureDrag, GestureStylus,
    IMMulticontext, Inhibit, PropagationPhase, Scrollable, ScrollablePolicy, Widget,
};

use crate::appwindow::RnoteAppWindow;
use futures::StreamExt;
use once_cell::sync::Lazy;
use p2d::bounding_volume::Aabb;
use rnote_compose::helpers::AabbHelpers;
use rnote_compose::penpath::Element;
use rnote_engine::utils::GrapheneRectHelpers;
use rnote_engine::Document;

use std::collections::VecDeque;
use std::time::{self, Instant};

mod imp {
    use super::*;

    #[allow(missing_debug_implementations)]
    pub(crate) struct RnoteCanvas {
        pub(crate) hadjustment: RefCell<Option<Adjustment>>,
        pub(crate) hadjustment_signal: RefCell<Option<glib::SignalHandlerId>>,
        pub(crate) vadjustment: RefCell<Option<Adjustment>>,
        pub(crate) vadjustment_signal: RefCell<Option<glib::SignalHandlerId>>,
        pub(crate) hscroll_policy: Cell<ScrollablePolicy>,
        pub(crate) vscroll_policy: Cell<ScrollablePolicy>,
        pub(crate) zoom_timeout_id: RefCell<Option<glib::SourceId>>,
        pub(crate) regular_cursor: RefCell<gdk::Cursor>,
        pub(crate) regular_cursor_icon_name: RefCell<String>,
        pub(crate) drawing_cursor: RefCell<gdk::Cursor>,
        pub(crate) drawing_cursor_icon_name: RefCell<String>,
        pub(crate) stylus_drawing_gesture: GestureStylus,
        pub(crate) mouse_drawing_gesture: GestureDrag,
        pub(crate) touch_drawing_gesture: GestureDrag,
        pub(crate) key_controller: EventControllerKey,
        pub(crate) key_controller_im_context: IMMulticontext,

        pub(crate) engine: Rc<RefCell<RnoteEngine>>,

        pub(crate) output_file: RefCell<Option<gio::File>>,
        pub(crate) output_file_monitor: RefCell<Option<gio::FileMonitor>>,
        pub(crate) output_file_modified_toast_singleton: RefCell<Option<adw::Toast>>,
        pub(crate) output_file_expect_write: Cell<bool>,
        pub(crate) unsaved_changes: Cell<bool>,
        pub(crate) empty: Cell<bool>,

        pub(crate) touch_drawing: Cell<bool>,
    }

    impl Default for RnoteCanvas {
        fn default() -> Self {
            let stylus_drawing_gesture = GestureStylus::builder()
                .name("stylus_drawing_gesture")
                .propagation_phase(PropagationPhase::Target)
                // Listen for any button
                .button(0)
                .build();

            // mouse gesture handlers have a guard to not handle emulated pointer events ( e.g. coming from touch input )
            // matching different input methods with gdk4::InputSource or gdk4::DeviceToolType did NOT WORK unfortunately, dont know why
            let mouse_drawing_gesture = GestureDrag::builder()
                .name("mouse_drawing_gesture")
                .button(0)
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            let touch_drawing_gesture = GestureDrag::builder()
                .name("touch_drawing_gesture")
                .touch_only(true)
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            let key_controller = EventControllerKey::builder()
                .name("key_controller")
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let key_controller_im_context = IMMulticontext::new();

            // Gesture grouping
            mouse_drawing_gesture.group_with(&stylus_drawing_gesture);
            touch_drawing_gesture.group_with(&stylus_drawing_gesture);

            let regular_cursor_icon_name = String::from("cursor-dot-medium");
            let regular_cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH)
                        + "icons/scalable/actions/cursor-dot-medium.svg")
                        .as_str(),
                ),
                32,
                32,
                gdk::Cursor::from_name("default", None).as_ref(),
            );
            let drawing_cursor_icon_name = String::from("cursor-dot-small");
            let drawing_cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH)
                        + "icons/scalable/actions/cursor-dot-small.svg")
                        .as_str(),
                ),
                32,
                32,
                gdk::Cursor::from_name("default", None).as_ref(),
            );

            let engine = RnoteEngine::default();

            Self {
                hadjustment: RefCell::new(None),
                hadjustment_signal: RefCell::new(None),
                vadjustment: RefCell::new(None),
                vadjustment_signal: RefCell::new(None),
                hscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                vscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                regular_cursor: RefCell::new(regular_cursor),
                regular_cursor_icon_name: RefCell::new(regular_cursor_icon_name),
                drawing_cursor: RefCell::new(drawing_cursor),
                drawing_cursor_icon_name: RefCell::new(drawing_cursor_icon_name),
                stylus_drawing_gesture,
                mouse_drawing_gesture,
                touch_drawing_gesture,
                key_controller,
                key_controller_im_context,
                zoom_timeout_id: RefCell::new(None),

                engine: Rc::new(RefCell::new(engine)),

                output_file: RefCell::new(None),
                output_file_monitor: RefCell::new(None),
                output_file_modified_toast_singleton: RefCell::new(None),
                output_file_expect_write: Cell::new(false),
                unsaved_changes: Cell::new(false),
                empty: Cell::new(true),

                touch_drawing: Cell::new(false),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteCanvas {
        const NAME: &'static str = "RnoteCanvas";
        type Type = super::RnoteCanvas;
        type ParentType = Widget;
        type Interfaces = (Scrollable,);

        fn class_init(klass: &mut Self::Class) {
            klass.set_accessible_role(AccessibleRole::Widget);
            klass.set_layout_manager_type::<CanvasLayout>();
        }

        fn new() -> Self {
            Self::default()
        }
    }

    impl ObjectImpl for RnoteCanvas {
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();

            inst.set_hexpand(false);
            inst.set_vexpand(false);
            inst.set_can_target(true);
            inst.set_focusable(true);
            inst.set_can_focus(true);

            inst.set_cursor(Some(&*self.regular_cursor.borrow()));

            inst.add_controller(&self.stylus_drawing_gesture);
            inst.add_controller(&self.mouse_drawing_gesture);
            inst.add_controller(&self.touch_drawing_gesture);
            inst.add_controller(&self.key_controller);
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "output-file",
                        "output-file",
                        "output-file",
                        Option::<gio::File>::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    // Flag to indicate that there are unsaved changes to the canvas
                    glib::ParamSpecBoolean::new(
                        "unsaved-changes",
                        "unsaved-changes",
                        "unsaved-changes",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Whether the canvas is empty
                    glib::ParamSpecBoolean::new(
                        "empty",
                        "empty",
                        "empty",
                        true,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Whether to enable touch drawing
                    glib::ParamSpecBoolean::new(
                        "touch-drawing",
                        "touch-drawing",
                        "touch-drawing",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "regular-cursor",
                        "regular-cursor",
                        "regular-cursor",
                        Some("cursor-dot-medium"),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "drawing-cursor",
                        "drawing-cursor",
                        "drawing-cursor",
                        Some("cursor-dot-small"),
                        glib::ParamFlags::READWRITE,
                    ),
                    // Scrollable properties
                    glib::ParamSpecOverride::for_interface::<Scrollable>("hscroll-policy"),
                    glib::ParamSpecOverride::for_interface::<Scrollable>("vscroll-policy"),
                    glib::ParamSpecOverride::for_interface::<Scrollable>("hadjustment"),
                    glib::ParamSpecOverride::for_interface::<Scrollable>("vadjustment"),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "output-file" => self.output_file.borrow().to_value(),
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
                "empty" => self.empty.get().to_value(),
                "hadjustment" => self.hadjustment.borrow().to_value(),
                "vadjustment" => self.vadjustment.borrow().to_value(),
                "hscroll-policy" => self.hscroll_policy.get().to_value(),
                "vscroll-policy" => self.vscroll_policy.get().to_value(),
                "touch-drawing" => self.touch_drawing.get().to_value(),
                "regular-cursor" => self.regular_cursor_icon_name.borrow().to_value(),
                "drawing-cursor" => self.drawing_cursor_icon_name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let inst = self.instance();
            /*             let parent_scrolledwindow = obj
            .parent()
            .map(|parent| parent.downcast::<ScrolledWindow>().unwrap()); */

            match pspec.name() {
                "output-file" => {
                    let output_file = value
                        .get::<Option<gio::File>>()
                        .expect("The value needs to be of type `Option<gio::File>`.");
                    self.output_file.replace(output_file);
                }
                "unsaved-changes" => {
                    let unsaved_changes: bool =
                        value.get().expect("The value needs to be of type `bool`.");
                    self.unsaved_changes.replace(unsaved_changes);
                }
                "empty" => {
                    let empty: bool = value.get().expect("The value needs to be of type `bool`.");
                    self.empty.replace(empty);
                    if empty {
                        inst.set_unsaved_changes(false);
                    }
                }
                "hadjustment" => {
                    let hadjustment = value.get().unwrap();
                    inst.set_hadjustment(hadjustment);
                }
                "hscroll-policy" => {
                    let hscroll_policy = value.get().unwrap();
                    self.hscroll_policy.replace(hscroll_policy);
                }
                "vadjustment" => {
                    let vadjustment = value.get().unwrap();
                    inst.set_vadjustment(vadjustment);
                }
                "vscroll-policy" => {
                    let vscroll_policy = value.get().unwrap();
                    self.vscroll_policy.replace(vscroll_policy);
                }
                "touch-drawing" => {
                    let touch_drawing: bool =
                        value.get().expect("The value needs to be of type `bool`.");
                    self.touch_drawing.replace(touch_drawing);
                    if touch_drawing {
                        self.touch_drawing_gesture
                            .set_propagation_phase(PropagationPhase::Bubble);
                    } else {
                        self.touch_drawing_gesture
                            .set_propagation_phase(PropagationPhase::None);
                    }
                }
                "regular-cursor" => {
                    let icon_name = value.get().unwrap();
                    self.regular_cursor_icon_name.replace(icon_name);

                    let cursor = gdk::Cursor::from_texture(
                        &gdk::Texture::from_resource(
                            (String::from(config::APP_IDPATH)
                                + &format!(
                                    "icons/scalable/actions/{}.svg",
                                    self.regular_cursor_icon_name.borrow()
                                ))
                                .as_str(),
                        ),
                        32,
                        32,
                        gdk::Cursor::from_name("default", None).as_ref(),
                    );

                    self.regular_cursor.replace(cursor);

                    inst.set_cursor(Some(&*self.regular_cursor.borrow()));
                }
                "drawing-cursor" => {
                    let icon_name = value.get().unwrap();
                    self.drawing_cursor_icon_name.replace(icon_name);

                    let cursor = gdk::Cursor::from_texture(
                        &gdk::Texture::from_resource(
                            (String::from(config::APP_IDPATH)
                                + &format!(
                                    "icons/scalable/actions/{}.svg",
                                    self.drawing_cursor_icon_name.borrow()
                                ))
                                .as_str(),
                        ),
                        32,
                        32,
                        gdk::Cursor::from_name("default", None).as_ref(),
                    );

                    self.drawing_cursor.replace(cursor);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnoteCanvas {
        // request_mode(), measure(), allocate() overrides happen in the CanvasLayout LayoutManager

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let inst = self.instance();

            if let Err(e) = || -> anyhow::Result<()> {
                let clip_bounds = if let Some(parent) = inst.parent() {
                    // unwrapping is fine, because its the parent
                    let (clip_x, clip_y) = parent.translate_coordinates(&*inst, 0.0, 0.0).unwrap();
                    Aabb::new_positive(
                        na::point![clip_x, clip_y],
                        na::point![f64::from(parent.width()), f64::from(parent.height())],
                    )
                } else {
                    inst.bounds()
                };
                // pushing the clip
                snapshot.push_clip(&graphene::Rect::from_p2d_aabb(clip_bounds));

                // Save the original coordinate space
                snapshot.save();

                // Draw the entire engine
                self.engine
                    .borrow()
                    .draw_on_gtk_snapshot(snapshot, inst.bounds())?;

                // Restore original coordinate space
                snapshot.restore();
                // End the clip of widget bounds
                snapshot.pop();
                Ok(())
            }() {
                log::error!("canvas snapshot() failed with Err: {e:?}");
            }
        }
    }

    impl ScrollableImpl for RnoteCanvas {}

    impl RnoteCanvas {}
}

glib::wrapper! {
    pub(crate) struct RnoteCanvas(ObjectSubclass<imp::RnoteCanvas>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Scrollable;
}

impl Default for RnoteCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl RnoteCanvas {
    // the zoom timeout time
    pub(crate) const ZOOM_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(300);
    // Sets the canvas zoom scroll step in % for one unit of the event controller delta
    pub(crate) const ZOOM_STEP: f64 = 0.1;

    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    #[allow(unused)]
    pub(crate) fn regular_cursor(&self) -> String {
        self.property::<String>("regular-cursor")
    }

    #[allow(unused)]
    pub(crate) fn set_regular_cursor(&self, regular_cursor: String) {
        self.set_property("regular-cursor", regular_cursor.to_value());
    }

    #[allow(unused)]
    pub(crate) fn drawing_cursor(&self) -> String {
        self.property::<String>("drawing-cursor")
    }

    #[allow(unused)]
    pub(crate) fn set_drawing_cursor(&self, drawing_cursor: String) {
        self.set_property("drawing-cursor", drawing_cursor.to_value());
    }

    #[allow(unused)]
    pub(crate) fn output_file(&self) -> Option<gio::File> {
        self.property::<Option<gio::File>>("output-file")
    }

    #[allow(unused)]
    pub(crate) fn clear_output_file_monitor(&self) {
        let mut current_output_file_monitor = self.imp().output_file_monitor.borrow_mut();

        if let Some(old_output_file_monitor) = current_output_file_monitor.take() {
            old_output_file_monitor.cancel();
        }
    }

    #[allow(unused)]
    pub(crate) fn dismiss_output_file_modified_toast(&self) {
        let output_file_modified_toast = self.imp().output_file_modified_toast_singleton.borrow();

        if let Some(output_file_modified_toast) = output_file_modified_toast.as_ref() {
            output_file_modified_toast.dismiss();
        }
    }

    #[allow(unused)]
    pub(crate) fn set_output_file_expect_write(&self, expect_write: bool) {
        self.imp().output_file_expect_write.set(expect_write);
    }

    #[allow(unused)]
    pub(crate) fn output_file_expect_write(&self) -> bool {
        self.imp().output_file_expect_write.get()
    }

    #[allow(unused)]
    pub(crate) fn set_output_file(&self, output_file: Option<gio::File>) {
        self.set_property("output-file", output_file.to_value());
    }

    #[allow(unused)]
    pub(crate) fn unsaved_changes(&self) -> bool {
        self.property::<bool>("unsaved-changes")
    }

    #[allow(unused)]
    pub(crate) fn set_unsaved_changes(&self, unsaved_changes: bool) {
        self.set_property("unsaved-changes", unsaved_changes.to_value());
    }

    #[allow(unused)]
    pub(crate) fn empty(&self) -> bool {
        self.property::<bool>("empty")
    }

    #[allow(unused)]
    pub(crate) fn set_empty(&self, empty: bool) {
        self.set_property("empty", empty.to_value());
    }

    #[allow(unused)]
    pub(crate) fn touch_drawing(&self) -> bool {
        self.property::<bool>("touch-drawing")
    }

    #[allow(unused)]
    pub(crate) fn set_touch_drawing(&self, touch_drawing: bool) {
        self.set_property("touch-drawing", touch_drawing.to_value());
    }

    pub(crate) fn engine(&self) -> Rc<RefCell<RnoteEngine>> {
        self.imp().engine.clone()
    }

    fn set_hadjustment(&self, adj: Option<Adjustment>) {
        if let Some(signal_id) = self.imp().hadjustment_signal.borrow_mut().take() {
            let old_adj = self.imp().hadjustment.borrow().as_ref().unwrap().clone();
            old_adj.disconnect(signal_id);
        }

        if let Some(ref hadjustment) = adj {
            let signal_id = hadjustment.connect_value_changed(
                clone!(@weak self as canvas => move |_hadjustment| {
                    // this triggers a canvaslayout allocate() call, where the strokes rendering is updated based on some conditions
                    canvas.queue_resize();
                }),
            );

            self.imp().hadjustment_signal.replace(Some(signal_id));
        }
        self.imp().hadjustment.replace(adj);
    }

    fn set_vadjustment(&self, adj: Option<Adjustment>) {
        if let Some(signal_id) = self.imp().vadjustment_signal.borrow_mut().take() {
            let old_adj = self.imp().vadjustment.borrow().as_ref().unwrap().clone();
            old_adj.disconnect(signal_id);
        }

        if let Some(ref vadjustment) = adj {
            let signal_id = vadjustment.connect_value_changed(
                clone!(@weak self as canvas => move |_vadjustment| {
                    // this triggers a canvaslayout allocate() call, where the strokes rendering is updated based on some conditions
                    canvas.queue_resize();
                }),
            );

            self.imp().vadjustment_signal.replace(Some(signal_id));
        }
        self.imp().vadjustment.replace(adj);
    }

    pub(crate) fn set_text_preprocessing(&self, enable: bool) {
        if enable {
            self.imp()
                .key_controller
                .set_im_context(Some(&self.imp().key_controller_im_context));
        } else {
            self.imp()
                .key_controller
                .set_im_context(None::<&IMMulticontext>);
        }
    }

    /// Switches between the regular and the drawing cursor
    pub(crate) fn switch_between_cursors(&self, drawing_cursor: bool) {
        if drawing_cursor {
            self.set_cursor(Some(&*self.imp().drawing_cursor.borrow()));
        } else {
            self.set_cursor(Some(&*self.imp().regular_cursor.borrow()));
        }
    }

    pub(crate) fn create_output_file_monitor(&self, file: &gio::File, appwindow: &RnoteAppWindow) {
        match file.monitor_file(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE) {
            Ok(output_file_monitor) => {
                output_file_monitor.connect_changed(
                    glib::clone!(@weak self as canvas, @weak appwindow => move |_monitor, file, other_file, event| {
                        let dispatch_toast_reload_modified_file = || {
                            appwindow.canvas().set_unsaved_changes(true);

                            appwindow.canvas_wrapper().dispatch_toast_w_button_singleton(&gettext("Opened file was modified on disk."), &gettext("Reload"), clone!(@weak appwindow => move |_reload_toast| {
                                if let Some(output_file) = appwindow.canvas().output_file() {
                                    if let Err(e) = appwindow.load_in_file(output_file, None) {
                                        log::error!("failed to reload current output file, {}", e);
                                    }
                                }
                            }), 0, &mut canvas.imp().output_file_modified_toast_singleton.borrow_mut());
                        };

                        match event {
                            gio::FileMonitorEvent::Changed => {
                                if canvas.output_file_expect_write() {
                                    // => file has been modified due to own save, don't do anything.
                                    canvas.set_output_file_expect_write(false);
                                    return;
                                }

                                dispatch_toast_reload_modified_file();
                            },
                            gio::FileMonitorEvent::Renamed => {
                                if canvas.output_file_expect_write() {
                                    // => file has been modified due to own save, don't do anything.
                                    canvas.set_output_file_expect_write(false);
                                    return;
                                }

                                // if previous file name was .goutputstream-<hash>, then the file has been replaced using gio.
                                if FileType::is_goutputstream_file(file) {
                                    // => file has been modified, handle it the same as the Changed event.
                                    dispatch_toast_reload_modified_file();
                                } else {
                                    // => file has been renamed.

                                    // other_file *should* never be none.
                                    if other_file.is_none() {
                                        canvas.set_unsaved_changes(true);
                                    }

                                    canvas.set_output_file(other_file.cloned());

                                    appwindow.canvas_wrapper().dispatch_toast_text(&gettext("Opened file was renamed on disk."))
                                }
                            },
                            gio::FileMonitorEvent::Deleted | gio::FileMonitorEvent::MovedOut => {
                                if canvas.output_file_expect_write() {
                                    // => file has been modified due to own save, don't do anything.
                                    canvas.set_output_file_expect_write(false);
                                    return;
                                }

                                canvas.set_unsaved_changes(true);
                                canvas.set_output_file(None);

                                appwindow.canvas_wrapper().dispatch_toast_text(&gettext("Opened file was moved or deleted on disk."));
                            },
                            _ => (),
                        }

                        // The expect_write flag can't be cleared after any event has been fired, because some actions emit multiple
                        // events - not all of which are handled. The flag should stick around until a handled event has been blocked by it,
                        // otherwise it will likely miss its purpose.
                    }),
                );

                self.imp()
                    .output_file_monitor
                    .borrow_mut()
                    .replace(output_file_monitor);
            }
            Err(e) => {
                self.clear_output_file_monitor();
                log::error!(
                    "creating a file monitor for the new output file failed with Err: {e:?}"
                )
            }
        }
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        self.setup_input(appwindow);

        // receiving and handling engine tasks
        glib::MainContext::default().spawn_local(
            clone!(@strong self as canvas, @strong appwindow => async move {
                let mut task_rx = canvas.engine().borrow_mut().tasks_rx.take().unwrap();

                loop {
                    if let Some(task) = task_rx.next().await {
                        let widget_flags = canvas.engine().borrow_mut().process_received_task(task);
                        if appwindow.handle_widget_flags(widget_flags) {
                            break;
                        }
                    }
                }
            }),
        );

        self.connect_notify_local(
            Some("output-file"),
            clone!(@weak appwindow => move |canvas, _pspec| {
                let output_file = canvas.output_file();

                if let Some(output_file) = &output_file {
                    canvas.create_output_file_monitor(output_file, &appwindow);
                } else {
                    canvas.clear_output_file_monitor();
                    canvas.dismiss_output_file_modified_toast();
                }

                appwindow.update_titles_for_file(output_file.as_ref());
            }),
        );

        self.connect_notify_local(
            Some("touch-drawing"),
            clone!(@weak appwindow => move |canvas, _pspec| {
                let touch_drawing = canvas.touch_drawing();

                // Disable the zoom gesture when touch drawing is enabled
                appwindow.canvas_wrapper().canvas_zoom_gesture_enable(!touch_drawing);
            }),
        );

        self.connect_notify_local(Some("unsaved-changes"), clone!(@weak appwindow => move |canvas, _pspec| {
            appwindow.mainheader().main_title_unsaved_indicator().set_visible(canvas.unsaved_changes());
            if canvas.unsaved_changes() {
                appwindow.mainheader().main_title().add_css_class("unsaved_changes");
            } else {
                appwindow.mainheader().main_title().remove_css_class("unsaved_changes");
            }
        }));

        self.bind_property(
            "regular-cursor",
            &appwindow
                .settings_panel()
                .general_regular_cursor_picker_image(),
            "icon-name",
        )
        .flags(glib::BindingFlags::DEFAULT)
        .build();

        self.bind_property(
            "drawing-cursor",
            &appwindow
                .settings_panel()
                .general_drawing_cursor_picker_image(),
            "icon-name",
        )
        .flags(glib::BindingFlags::DEFAULT)
        .build();

        // set at startup
        self.engine().borrow_mut().camera.scale_factor = f64::from(self.scale_factor());
        // and connect
        self.connect_notify_local(Some("scale-factor"), move |canvas, _pspec| {
            let scale_factor = f64::from(canvas.scale_factor());
            canvas.engine().borrow_mut().camera.scale_factor = scale_factor;

            let all_strokes = canvas.engine().borrow_mut().store.stroke_keys_unordered();
            canvas
                .engine()
                .borrow_mut()
                .store
                .set_rendering_dirty_for_strokes(&all_strokes);

            canvas.regenerate_background_pattern();
            canvas.update_engine_rendering();
        });
    }

    fn setup_input(&self, appwindow: &RnoteAppWindow) {
        // Stylus Drawing
        self.imp().stylus_drawing_gesture.connect_down(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture down");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            // disable drag and zoom gestures entirely while drawing with stylus
            appwindow.canvas_wrapper().canvas_touch_drag_gesture_enable(false);
            appwindow.canvas_wrapper().canvas_zoom_gesture_enable(false);
            appwindow.canvas_wrapper().canvas_drag_empty_area_gesture_enable(false);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }
            stylus_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retrieve_stylus_elements(stylus_drawing_gesture, x, y);
           Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retrieve_stylus_pen_mode(stylus_drawing_gesture);

            for element in data_entries {
                input::process_pen_down(element, shortcut_keys.clone(), pen_mode, Instant::now(), &appwindow);
            }
        }));

        self.imp().stylus_drawing_gesture.connect_motion(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            //log::debug!("stylus_drawing_gesture motion");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries: VecDeque<Element> = input::retrieve_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retrieve_stylus_pen_mode(stylus_drawing_gesture);

            for element in data_entries {
                input::process_pen_down(element, shortcut_keys.clone(), pen_mode, Instant::now(), &appwindow);
            }
        }));

        self.imp().stylus_drawing_gesture.connect_up(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture up");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            // enable drag and zoom gestures again
            appwindow.canvas_wrapper().canvas_touch_drag_gesture_enable(true);
            appwindow.canvas_wrapper().canvas_drag_empty_area_gesture_enable(true);

            if !canvas.touch_drawing() {
                appwindow.canvas_wrapper().canvas_zoom_gesture_enable(true);
            }

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries = input::retrieve_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retrieve_stylus_pen_mode(stylus_drawing_gesture);

            if let Some(last) = data_entries.pop_back() {
                for element in data_entries {
                    input::process_pen_down(element, shortcut_keys.clone(), pen_mode, Instant::now(), &appwindow);
                }
                input::process_pen_up(last, shortcut_keys, pen_mode, Instant::now(), &appwindow);
            }
        }));

        self.imp().stylus_drawing_gesture.connect_proximity(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture proximity");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries = input::retrieve_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retrieve_stylus_pen_mode(stylus_drawing_gesture);

            for element in data_entries {
                input::process_pen_proximity(element, shortcut_keys.clone(), pen_mode, Instant::now(), &appwindow);
            }
        }));

        // Mouse drawing
        self.imp().mouse_drawing_gesture.connect_drag_begin(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture begin");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }
            mouse_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retrieve_pointer_elements(mouse_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_mouse_shortcut_keys(mouse_drawing_gesture);

            for element in data_entries {
                input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now(), &appwindow);
            }
        }));

        self.imp().mouse_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture motion");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retrieve_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retrieve_mouse_shortcut_keys(mouse_drawing_gesture);

                for element in data_entries {
                    input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now(), &appwindow);
                }
            }
        }));

        self.imp().mouse_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture end");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retrieve_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1) );

                let shortcut_keys = input::retrieve_mouse_shortcut_keys(mouse_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    for element in data_entries {
                        input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now(), &appwindow);
                    }
                    input::process_pen_up(last, shortcut_keys, Some(PenMode::Pen), Instant::now(), &appwindow);
                }
            }
        }));

        // Touch drawing
        self.imp().touch_drawing_gesture.connect_drag_begin(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            //log::debug!("touch_drawing_gesture begin");

            if input::filter_touch_input(touch_drawing_gesture) { return; }
            touch_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retrieve_pointer_elements(touch_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_touch_shortcut_keys(touch_drawing_gesture);

            for element in data_entries {
                input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now(), &appwindow);
            }
        }));

        self.imp().touch_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture motion");

                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retrieve_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retrieve_touch_shortcut_keys(touch_drawing_gesture);

                for element in data_entries {
                    input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now(), &appwindow);
                }
            }
        }));

        self.imp().touch_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture end");

                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retrieve_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retrieve_touch_shortcut_keys(touch_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    for element in data_entries {
                        input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now(), &appwindow);
                    }
                    input::process_pen_up(last, shortcut_keys, Some(PenMode::Pen), Instant::now(), &appwindow);
                }
            }
        }));

        // Key controller

        self.imp().key_controller.connect_key_pressed(clone!(@weak self as canvas, @weak appwindow => @default-return Inhibit(false), move |_key_controller, key, _raw, modifier| {
            //log::debug!("key pressed - key: {:?}, raw: {:?}, modifier: {:?}", key, raw, modifier);
            canvas.grab_focus();

            let keyboard_key = input::retrieve_keyboard_key(key);
            let shortcut_keys = input::retrieve_modifier_shortcut_key(modifier);

            //log::debug!("keyboard key: {:?}", keyboard_key);

            input::process_keyboard_key_pressed(keyboard_key, shortcut_keys, Instant::now(), &appwindow);

            Inhibit(true)
        }));

        // For unicode text the input is committed from the IM context, and won't trigger the key_pressed signal
        self.imp().key_controller_im_context.connect_commit(
            clone!(@weak self as canvas, @weak appwindow => move |_cx, text| {
                input::process_keyboard_text(text.to_string(), Instant::now(), &appwindow);
            }),
        );

        /*
        self.imp().key_controller.connect_key_released(clone!(@weak self as canvas, @weak appwindow => move |_key_controller, _key, _raw, _modifier| {
            //log::debug!("key released - key: {:?}, raw: {:?}, modifier: {:?}", key, raw, modifier);
        }));

        self.imp().key_controller.connect_modifiers(clone!(@weak self as canvas, @weak appwindow => @default-return Inhibit(false), move |_key_controller, modifier| {
            //log::debug!("key_controller modifier pressed: {:?}", modifier);

            let shortcut_keys = input::retrieve_modifier_shortcut_key(modifier);
            canvas.grab_focus();

            for shortcut_key in shortcut_keys {
                log::debug!("shortcut key pressed: {:?}", shortcut_key);

                input::process_shortcut_key_pressed(shortcut_key, &appwindow);
            }

            Inhibit(true)
        }));
        */

        // Drop Target
        let drop_target = DropTarget::builder()
            .name("canvas_drop_target")
            .propagation_phase(PropagationPhase::Capture)
            .actions(gdk::DragAction::COPY)
            .build();
        self.add_controller(&drop_target);

        // The order here is important: first files, then text
        drop_target.set_types(&[gio::File::static_type(), glib::types::Type::STRING]);

        drop_target.connect_drop(
            clone!(@weak appwindow => @default-return false, move |_drop_target, value, x, y| {
                let pos = (appwindow.canvas().engine().borrow().camera.transform().inverse() *
                    na::point![x,y]).coords;

                if value.is::<gio::File>() {
                    appwindow.open_file_w_dialogs(value.get::<gio::File>().unwrap(), Some(pos));

                    return true;
                } else if value.is::<String>() {
                    if let Err(e) = appwindow.load_in_text(value.get::<String>().unwrap(), Some(pos)) {
                        log::error!("failed to insert dropped in text, Err: {e:?}");
                    }
                }

                false
            }),
        );
    }

    pub(crate) fn bounds(&self) -> Aabb {
        Aabb::new_positive(
            na::point![0.0, 0.0],
            na::point![f64::from(self.width()), f64::from(self.height())],
        )
    }

    // updates the camera offset with a new one ( for example from touch drag gestures )
    // update_engine_rendering() then needs to be called.
    pub(crate) fn update_camera_offset(&self, new_offset: na::Vector2<f64>) {
        self.engine().borrow_mut().update_camera_offset(new_offset);

        // By setting new adjustment values, the callback connected to their value property is called,
        // Which is where the engine rendering is updated.
        self.hadjustment().unwrap().set_value(new_offset[0]);
        self.vadjustment().unwrap().set_value(new_offset[1]);
    }

    /// returns the center of the current view on the doc
    pub(crate) fn current_center_on_doc(&self) -> na::Vector2<f64> {
        (self.engine().borrow().camera.transform().inverse()
            * na::point![
                f64::from(self.width()) * 0.5,
                f64::from(self.height()) * 0.5
            ])
        .coords
    }

    /// Centers the view around a coord on the doc. The coord parameter has the coordinate space of the doc.
    // update_engine_rendering() then needs to be called.
    pub(crate) fn center_around_coord_on_doc(&self, coord: na::Vector2<f64>) {
        let (parent_width, parent_height) = (
            f64::from(self.parent().unwrap().width()),
            f64::from(self.parent().unwrap().height()),
        );
        let total_zoom = self.engine().borrow().camera.total_zoom();

        let new_offset = na::vector![
            ((coord[0]) * total_zoom) - parent_width * 0.5,
            ((coord[1]) * total_zoom) - parent_height * 0.5
        ];

        self.update_camera_offset(new_offset);
    }

    /// Centering the view to the origin page
    // update_engine_rendering() then needs to be called.
    pub(crate) fn return_to_origin_page(&self) {
        let zoom = self.engine().borrow().camera.zoom();

        let new_offset = if self.engine().borrow().document.format.width * zoom
            <= f64::from(self.parent().unwrap().width())
        {
            na::vector![
                (self.engine().borrow().document.format.width * 0.5 * zoom)
                    - f64::from(self.parent().unwrap().width()) * 0.5,
                -Document::SHADOW_WIDTH * zoom
            ]
        } else {
            // If the zoomed format width is larger than the displayed surface, we zoom to a fixed origin
            na::vector![
                -Document::SHADOW_WIDTH * zoom,
                -Document::SHADOW_WIDTH * zoom
            ]
        };

        self.update_camera_offset(new_offset);
    }

    /// zooms and regenerates the canvas and its contents to a new zoom
    /// is private, zooming from other parts of the app should always be done through the "zoom-to-value" action
    fn zoom_to(&self, new_zoom: f64) {
        // Remove the timeout if exists
        if let Some(zoom_timeout_id) = self.imp().zoom_timeout_id.take() {
            zoom_timeout_id.remove();
        }

        self.engine().borrow_mut().camera.set_temporary_zoom(1.0);
        self.engine().borrow_mut().camera.set_zoom(new_zoom);

        let all_strokes = self.engine().borrow_mut().store.stroke_keys_unordered();
        self.engine()
            .borrow_mut()
            .store
            .set_rendering_dirty_for_strokes(&all_strokes);

        self.regenerate_background_pattern();
        self.update_engine_rendering();

        // We need to update the layout managers internal state after zooming
        self.layout_manager()
            .unwrap()
            .downcast::<CanvasLayout>()
            .unwrap()
            .update_state(self);
    }

    /// Zooms temporarily and then scale the canvas and its contents to a new zoom after a given time.
    /// Repeated calls to this function reset the timeout.
    /// should only be called from the "zoom-to-value" action.
    pub(crate) fn zoom_temporarily_then_scale_to_after_timeout(
        &self,
        new_zoom: f64,
        timeout_time: time::Duration,
    ) {
        if let Some(zoom_timeout_id) = self.imp().zoom_timeout_id.take() {
            zoom_timeout_id.remove();
        }

        let old_perm_zoom = self.engine().borrow().camera.zoom();

        // Zoom temporarily
        let new_temp_zoom = new_zoom / old_perm_zoom;
        self.engine()
            .borrow_mut()
            .camera
            .set_temporary_zoom(new_temp_zoom);

        // In resize we render the strokes that came into view
        self.queue_resize();

        if let Some(zoom_timeout_id) =
            self.imp()
                .zoom_timeout_id
                .borrow_mut()
                .replace(glib::source::timeout_add_local_once(
                    timeout_time,
                    clone!(@weak self as canvas => move || {

                        // After timeout zoom permanent
                        canvas.zoom_to(new_zoom);

                        // Removing the timeout id
                        let mut zoom_timeout_id = canvas.imp().zoom_timeout_id.borrow_mut();
                        if let Some(zoom_timeout_id) = zoom_timeout_id.take() {
                            zoom_timeout_id.remove();
                        }
                    }),
                ))
        {
            zoom_timeout_id.remove();
        }
    }

    /// Updates the rendering of the background and strokes that are flagged for rerendering for the current viewport.
    /// To force the rerendering of the background pattern, call regenerate_background_pattern().
    /// To force the rerendering for all strokes in the current viewport, first flag their rendering as dirty.
    pub(crate) fn update_engine_rendering(&self) {
        // background rendering is updated in the layout manager
        self.queue_resize();

        // Update engine rendering for the new viewport
        if let Err(e) = self
            .engine()
            .borrow_mut()
            .update_rendering_current_viewport()
        {
            log::error!("failed to update engine rendering for current viewport, Err: {e:?}");
        }

        self.queue_draw();
    }

    /// updates the background pattern and rendering for the current viewport.
    /// to be called for example when changing the background pattern or zoom.
    pub(crate) fn regenerate_background_pattern(&self) {
        if let Err(e) = self.engine().borrow_mut().background_regenerate_pattern() {
            log::error!("failed to regenerate background, {e:?}")
        };

        self.queue_draw();
    }
}
