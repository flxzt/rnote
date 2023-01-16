mod canvaslayout;
pub(crate) mod imexport;
mod input;

// Re-exports
pub(crate) use canvaslayout::CanvasLayout;
use gettextrs::gettext;
use rnote_engine::pens::PenMode;

// Imports
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::config;
use rnote_engine::{RnoteEngine, WidgetFlags};

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, glib::Boxed)]
#[boxed_type(name = "WidgetFlagsBoxed")]
struct WidgetFlagsBoxed(WidgetFlags);

#[derive(Debug, Default)]
pub(crate) struct Handlers {
    pub(crate) hadjustment: Option<glib::SignalHandlerId>,
    pub(crate) vadjustment: Option<glib::SignalHandlerId>,
    pub(crate) zoom_timeout: Option<glib::SourceId>,
    pub(crate) tab_page_output_file: Option<glib::Binding>,
    pub(crate) tab_page_unsaved_changes: Option<glib::Binding>,
    pub(crate) appwindow_output_file: Option<glib::SignalHandlerId>,
    pub(crate) appwindow_scalefactor: Option<glib::SignalHandlerId>,
    pub(crate) appwindow_unsaved_changes: Option<glib::SignalHandlerId>,
    pub(crate) appwindow_touch_drawing: Option<glib::Binding>,
    pub(crate) appwindow_regular_cursor: Option<glib::Binding>,
    pub(crate) appwindow_drawing_cursor: Option<glib::Binding>,
    pub(crate) appwindow_drop_target: Option<glib::SignalHandlerId>,
    pub(crate) appwindow_zoom_changed: Option<glib::SignalHandlerId>,
    pub(crate) appwindow_handle_widget_flags: Option<glib::SignalHandlerId>,
}

mod imp {
    use super::*;

    #[allow(missing_debug_implementations)]
    pub(crate) struct RnoteCanvas {
        pub(crate) handlers: RefCell<Handlers>,

        pub(crate) hadjustment: RefCell<Option<Adjustment>>,
        pub(crate) vadjustment: RefCell<Option<Adjustment>>,
        pub(crate) hscroll_policy: Cell<ScrollablePolicy>,
        pub(crate) vscroll_policy: Cell<ScrollablePolicy>,
        pub(crate) regular_cursor: RefCell<gdk::Cursor>,
        pub(crate) regular_cursor_icon_name: RefCell<String>,
        pub(crate) drawing_cursor: RefCell<gdk::Cursor>,
        pub(crate) drawing_cursor_icon_name: RefCell<String>,
        pub(crate) stylus_drawing_gesture: GestureStylus,
        pub(crate) mouse_drawing_gesture: GestureDrag,
        pub(crate) touch_drawing_gesture: GestureDrag,
        pub(crate) key_controller: EventControllerKey,
        pub(crate) key_controller_im_context: IMMulticontext,
        pub(crate) drop_target: DropTarget,

        pub(crate) engine: Rc<RefCell<RnoteEngine>>,

        pub(crate) output_file: RefCell<Option<gio::File>>,
        pub(crate) output_file_monitor: RefCell<Option<gio::FileMonitor>>,
        pub(crate) output_file_monitor_changed_handler: RefCell<Option<glib::SignalHandlerId>>,
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

            let drop_target = DropTarget::builder()
                .name("canvas_drop_target")
                .propagation_phase(PropagationPhase::Capture)
                .actions(gdk::DragAction::COPY)
                .build();

            // The order here is important: first files, then text
            drop_target.set_types(&[gio::File::static_type(), glib::types::Type::STRING]);

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
                handlers: RefCell::new(Handlers::default()),

                hadjustment: RefCell::new(None),
                vadjustment: RefCell::new(None),
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
                drop_target,

                engine: Rc::new(RefCell::new(engine)),

                output_file: RefCell::new(None),
                output_file_monitor: RefCell::new(None),
                output_file_monitor_changed_handler: RefCell::new(None),
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
            inst.add_controller(&self.drop_target);

            // receive and handling engine tasks
            glib::MainContext::default().spawn_local(
                clone!(@weak inst as canvas => async move {
                    let mut task_rx = canvas.engine().borrow_mut().regenerate_channel();

                    loop {
                        if let Some(task) = task_rx.next().await {
                            let (widget_flags, quit) = canvas.engine().borrow_mut().handle_engine_task(task);
                            canvas.emit_handle_widget_flags(widget_flags);

                            if quit {
                                break;
                            }
                        }
                    }
                }),
            );

            self.setup_input();
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

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![
                    glib::subclass::Signal::builder("zoom-changed").build(),
                    glib::subclass::Signal::builder("handle-widget-flags")
                        .param_types([WidgetFlagsBoxed::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
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

    impl RnoteCanvas {
        fn setup_input(&self) {
            let inst = self.instance();

            // Stylus Drawing
            self.stylus_drawing_gesture.connect_down(clone!(@weak inst as canvas => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture down");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }
            stylus_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retrieve_stylus_elements(stylus_drawing_gesture, x, y);
           Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retrieve_stylus_pen_mode(stylus_drawing_gesture);

            let mut widget_flags = WidgetFlags::default();
            for element in data_entries {
                widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), pen_mode, Instant::now(), ));
            }

            canvas.emit_handle_widget_flags(widget_flags);
        }));

            self.stylus_drawing_gesture.connect_motion(clone!(@weak inst as canvas => move |stylus_drawing_gesture, x, y| {
            //log::debug!("stylus_drawing_gesture motion");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries: VecDeque<Element> = input::retrieve_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retrieve_stylus_pen_mode(stylus_drawing_gesture);

            let mut widget_flags = WidgetFlags::default();
            for element in data_entries {
                widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), pen_mode, Instant::now()));
            }
            canvas.emit_handle_widget_flags(widget_flags);
        }));

            self.stylus_drawing_gesture.connect_up(clone!(@weak inst as canvas => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture up");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries = input::retrieve_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retrieve_stylus_pen_mode(stylus_drawing_gesture);

            if let Some(last) = data_entries.pop_back() {
                let mut widget_flags = WidgetFlags::default();
                for element in data_entries {
                    widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), pen_mode, Instant::now()));
                }
                widget_flags.merge(input::process_pen_up(&canvas, last, shortcut_keys, pen_mode, Instant::now()));
                canvas.emit_handle_widget_flags(widget_flags);
            }
        }));

            self.stylus_drawing_gesture.connect_proximity(clone!(@weak inst as canvas => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture proximity");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries = input::retrieve_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retrieve_stylus_pen_mode(stylus_drawing_gesture);

            let mut widget_flags = WidgetFlags::default();
            for element in data_entries {
                widget_flags.merge(input::process_pen_proximity(&canvas, element, shortcut_keys.clone(), pen_mode, Instant::now()));
            }
            canvas.emit_handle_widget_flags(widget_flags);
        }));

            // Mouse drawing
            self.mouse_drawing_gesture.connect_drag_begin(clone!(@weak inst as canvas => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture begin");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }
            mouse_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retrieve_pointer_elements(mouse_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_mouse_shortcut_keys(mouse_drawing_gesture);

            let mut widget_flags = WidgetFlags::default();
            for element in data_entries {
                widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now()));
            }
            canvas.emit_handle_widget_flags(widget_flags);
        }));

            self.mouse_drawing_gesture.connect_drag_update(clone!(@weak inst as canvas => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture motion");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retrieve_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retrieve_mouse_shortcut_keys(mouse_drawing_gesture);

                let mut widget_flags = WidgetFlags::default();
                for element in data_entries {
                    widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now()));
                }
                canvas.emit_handle_widget_flags(widget_flags);
            }
        }));

            self.mouse_drawing_gesture.connect_drag_end(clone!(@weak inst as canvas => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture end");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retrieve_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1) );

                let shortcut_keys = input::retrieve_mouse_shortcut_keys(mouse_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    let mut widget_flags = WidgetFlags::default();
                    for element in data_entries {
                        widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now()));
                    }
                    widget_flags.merge(input::process_pen_up(&canvas, last, shortcut_keys, Some(PenMode::Pen), Instant::now()));
                    canvas.emit_handle_widget_flags(widget_flags);
                }
            }
        }));

            // Touch drawing
            self.touch_drawing_gesture.connect_drag_begin(clone!(@weak inst as canvas => move |touch_drawing_gesture, x, y| {
            //log::debug!("touch_drawing_gesture begin");

            if input::filter_touch_input(touch_drawing_gesture) { return; }
            touch_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retrieve_pointer_elements(touch_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retrieve_touch_shortcut_keys(touch_drawing_gesture);

            let mut widget_flags = WidgetFlags::default();
            for element in data_entries {
                widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now()));
            }
            canvas.emit_handle_widget_flags(widget_flags);
        }));

            self.touch_drawing_gesture.connect_drag_update(clone!(@weak inst as canvas => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture motion");

                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retrieve_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retrieve_touch_shortcut_keys(touch_drawing_gesture);

                let mut widget_flags = WidgetFlags::default();
                for element in data_entries {
                    widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now()));
                }
                canvas.emit_handle_widget_flags(widget_flags);
            }
        }));

            self.touch_drawing_gesture.connect_drag_end(clone!(@weak inst as canvas => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture end");

                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retrieve_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retrieve_touch_shortcut_keys(touch_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    let mut widget_flags = WidgetFlags::default();
                    for element in data_entries {
                        widget_flags.merge(input::process_pen_down(&canvas, element, shortcut_keys.clone(), Some(PenMode::Pen), Instant::now()));
                    }
                    widget_flags.merge(input::process_pen_up(&canvas, last, shortcut_keys, Some(PenMode::Pen), Instant::now()));
                    canvas.emit_handle_widget_flags(widget_flags);
                }
            }
        }));

            // Key controller

            self.key_controller.connect_key_pressed(clone!(@weak inst as canvas => @default-return Inhibit(false), move |_key_controller, key, _raw, modifier| {
            //log::debug!("key pressed - key: {:?}, raw: {:?}, modifier: {:?}", key, raw, modifier);
            canvas.grab_focus();

            let keyboard_key = input::retrieve_keyboard_key(key);
            let shortcut_keys = input::retrieve_modifier_shortcut_key(modifier);

            //log::debug!("keyboard key: {:?}", keyboard_key);

            let widget_flags = input::process_keyboard_key_pressed(&canvas, keyboard_key, shortcut_keys, Instant::now());
            canvas.emit_handle_widget_flags(widget_flags);

            Inhibit(true)
        }));

            // For unicode text the input is committed from the IM context, and won't trigger the key_pressed signal
            self.key_controller_im_context.connect_commit(clone!(@weak inst as canvas => move |_cx, text| {
            let widget_flags = input::process_keyboard_text(&canvas, text.to_string(), Instant::now());
            canvas.emit_handle_widget_flags(widget_flags);
        }));

            /*
            self.imp().key_controller.connect_key_released(clone!(@weak inst as canvas => move |_key_controller, _key, _raw, _modifier| {
                //log::debug!("key released - key: {:?}, raw: {:?}, modifier: {:?}", key, raw, modifier);
            }));

            self.imp().key_controller.connect_modifiers(clone!(@weak inst as canvas, @weak appwindow => @default-return Inhibit(false), move |_key_controller, modifier| {
                //log::debug!("key_controller modifier pressed: {:?}", modifier);

                let shortcut_keys = input::retrieve_modifier_shortcut_key(modifier);
                canvas.grab_focus();

                let mut widget_flags = WidgetFlags::default();
                for shortcut_key in shortcut_keys {
                    log::debug!("shortcut key pressed: {:?}", shortcut_key);

                    widget_flags.merge(input::process_shortcut_key_pressed(self, shortcut_key));
                }
                canvas.emit_handle_widget_flags(widget_flags);

                Inhibit(true)
            }));
            */
        }
    }
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

pub(crate) static OUTPUT_FILE_NEW_TITLE: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| gettext("New Document"));
pub(crate) static OUTPUT_FILE_NEW_SUBTITLE: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| gettext("Draft"));

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
    pub(crate) fn set_regular_cursor(&self, regular_cursor: &str) {
        self.set_property("regular-cursor", regular_cursor.to_value());
    }

    #[allow(unused)]
    pub(crate) fn drawing_cursor(&self) -> String {
        self.property::<String>("drawing-cursor")
    }

    #[allow(unused)]
    pub(crate) fn set_drawing_cursor(&self, drawing_cursor: &str) {
        self.set_property("drawing-cursor", drawing_cursor.to_value());
    }

    #[allow(unused)]
    pub(crate) fn output_file(&self) -> Option<gio::File> {
        self.property::<Option<gio::File>>("output-file")
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

    #[allow(unused)]
    fn emit_zoom_changed(&self) {
        self.emit_by_name::<()>("zoom-changed", &[]);
    }

    #[allow(unused)]
    fn emit_handle_widget_flags(&self, widget_flags: WidgetFlags) {
        self.emit_by_name::<()>("handle-widget-flags", &[&WidgetFlagsBoxed(widget_flags)]);
    }

    pub(crate) fn engine(&self) -> Rc<RefCell<RnoteEngine>> {
        self.imp().engine.clone()
    }

    fn set_hadjustment(&self, adj: Option<Adjustment>) {
        if let Some(signal_id) = self.imp().handlers.borrow_mut().hadjustment.take() {
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

            self.imp()
                .handlers
                .borrow_mut()
                .hadjustment
                .replace(signal_id);
        }
        self.imp().hadjustment.replace(adj);
    }

    fn set_vadjustment(&self, adj: Option<Adjustment>) {
        if let Some(signal_id) = self.imp().handlers.borrow_mut().vadjustment.take() {
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

            self.imp()
                .handlers
                .borrow_mut()
                .vadjustment
                .replace(signal_id);
        }
        self.imp().vadjustment.replace(adj);
    }

    pub(crate) fn stylus_drawing_gesture(&self) -> GestureStylus {
        self.imp().stylus_drawing_gesture.clone()
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

    pub(crate) fn clear_output_file_monitor(&self) {
        if let Some(old_output_file_monitor) = self.imp().output_file_monitor.take() {
            if let Some(handler) = self.imp().output_file_monitor_changed_handler.take() {
                old_output_file_monitor.disconnect(handler);
            }

            old_output_file_monitor.cancel();
        }
    }

    pub(crate) fn dismiss_output_file_modified_toast(&self) {
        if let Some(output_file_modified_toast) =
            self.imp().output_file_modified_toast_singleton.take()
        {
            output_file_modified_toast.dismiss();
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

    /// The document title for display. Can be used to get a string for the existing / a new save file.
    ///
    /// When there is no output-file, falls back to the "New document" string
    pub(crate) fn doc_title_display(&self) -> String {
        self.output_file()
            .map(|f| {
                f.basename()
                    .and_then(|t| Some(t.file_stem()?.to_string_lossy().to_string()))
                    .unwrap_or_else(|| gettext("- invalid file name -"))
            })
            .unwrap_or_else(|| OUTPUT_FILE_NEW_TITLE.to_string())
    }

    /// The document folder path for display. To get the actual path, use output-file
    ///
    /// When there is no output-file, falls back to the "Draft" string
    pub(crate) fn doc_folderpath_display(&self) -> String {
        self.output_file()
            .map(|f| {
                f.parent()
                    .and_then(|p| Some(p.path()?.display().to_string()))
                    .unwrap_or_else(|| gettext("- invalid folder path -"))
            })
            .unwrap_or_else(|| OUTPUT_FILE_NEW_SUBTITLE.to_string())
    }

    pub(crate) fn create_output_file_monitor(&self, file: &gio::File, appwindow: &RnoteAppWindow) {
        let new_monitor =
            match file.monitor_file(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE) {
                Ok(output_file_monitor) => output_file_monitor,
                Err(e) => {
                    self.clear_output_file_monitor();
                    log::error!(
                        "creating a file monitor for the new output file failed with Err: {e:?}"
                    );
                    return;
                }
            };

        let new_handler = new_monitor.connect_changed(
            glib::clone!(@weak self as canvas, @weak appwindow => move |_monitor, file, other_file, event| {
                let dispatch_toast_reload_modified_file = || {
                    canvas.set_unsaved_changes(true);

                    appwindow.overlays().dispatch_toast_w_button_singleton(
                        &gettext("Opened file was modified on disk."),
                        &gettext("Reload"),
                        clone!(@weak canvas, @weak appwindow => move |_reload_toast| {
                            glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                                appwindow.overlays().start_pulsing_progressbar();

                                if let Err(e) = canvas.reload_from_disk().await {
                                    appwindow.overlays().dispatch_toast_error(&gettext("Reloading .rnote file from disk failed."));
                                    log::error!("failed to reload current output file, {}", e);
                                }

                                appwindow.overlays().finish_progressbar();
                            }));
                        }),
                        0,
                    &mut canvas.imp().output_file_modified_toast_singleton.borrow_mut());
                };

                log::debug!("canvas with title: `{}` - output-file monitor emitted `changed` - file: {:?}, other_file: {:?}, event: {event:?}", canvas.doc_title_display(), file.path(), other_file.map(|f| f.path()));

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
                        if crate::utils::is_goutputstream_file(file) {
                            // => file has been modified, handle it the same as the Changed event.
                            dispatch_toast_reload_modified_file();
                        } else {
                            // => file has been renamed.

                            // other_file *should* never be none.
                            if other_file.is_none() {
                                canvas.set_unsaved_changes(true);
                            }

                            canvas.set_output_file(other_file.cloned());

                            appwindow.overlays().dispatch_toast_text(&gettext("Opened file was renamed on disk."))
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

                        appwindow.overlays().dispatch_toast_text(&gettext("Opened file was moved or deleted on disk."));
                    },
                    _ => {},
                }

                // The expect_write flag can't be cleared after any event has been fired, because some actions emit multiple
                // events - not all of which are handled. The flag should stick around until a handled event has been blocked by it,
                // otherwise it will likely miss its purpose.
            }),
        );

        if let Some(old_monitor) = self
            .imp()
            .output_file_monitor
            .borrow_mut()
            .replace(new_monitor)
        {
            if let Some(old_handler) = self
                .imp()
                .output_file_monitor_changed_handler
                .borrow_mut()
                .replace(new_handler)
            {
                old_monitor.disconnect(old_handler);
            }

            old_monitor.cancel();
        }
    }

    /// Replaces and installs a new file monitor when there is an output file present
    fn reinstall_output_file_monitor(&self, appwindow: &RnoteAppWindow) {
        if let Some(output_file) = self.output_file() {
            self.create_output_file_monitor(&output_file, appwindow);
        } else {
            self.clear_output_file_monitor();
        }
    }

    /// Initializes for the given appwindow. Usually `init()` is only called once, but since this widget can be moved between appwindows through tabs,
    /// this function also disconnects and replaces all existing old connections
    pub(crate) fn init_reconnect(&self, appwindow: &RnoteAppWindow) {
        // Initial file monitor, (e.g. needed when reiniting the widget on a new appwindow)
        self.reinstall_output_file_monitor(appwindow);

        let appwindow_output_file = self.connect_notify_local(
            Some("output-file"),
            clone!(@weak appwindow => move |canvas, _pspec| {
                if let Some(output_file) = canvas.output_file(){
                    canvas.create_output_file_monitor(&output_file, &appwindow);
                } else {
                    canvas.clear_output_file_monitor();
                    canvas.dismiss_output_file_modified_toast();
                }

                appwindow.refresh_titles_active_tab();
            }),
        );

        // set scalefactor initially
        self.engine().borrow_mut().camera.scale_factor = f64::from(self.scale_factor());
        // and connect
        let appwindow_scalefactor =
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

        // Update titles when there are changes
        let appwindow_unsaved_changes = self.connect_notify_local(
            Some("unsaved-changes"),
            clone!(@weak appwindow => move |_canvas, _pspec| {
                appwindow.refresh_titles_active_tab();
            }),
        );

        // one per-appwindow property for touch-drawing
        let appwindow_touch_drawing = appwindow
            .bind_property("touch-drawing", self, "touch_drawing")
            .sync_create()
            .build();

        // bind cursors
        let appwindow_regular_cursor = appwindow
            .settings_panel()
            .general_regular_cursor_picker()
            .bind_property("picked", self, "regular-cursor")
            .transform_to(|_, v: Option<String>| v)
            .sync_create()
            .build();

        let appwindow_drawing_cursor = appwindow
            .settings_panel()
            .general_drawing_cursor_picker()
            .bind_property("picked", self, "drawing-cursor")
            .transform_to(|_, v: Option<String>| v)
            .sync_create()
            .build();

        // Drop Target
        let appwindow_drop_target = self.imp().drop_target.connect_drop(
            clone!(@weak self as canvas, @weak appwindow => @default-return false, move |_drop_target, value, x, y| {
                let pos = (canvas.engine().borrow().camera.transform().inverse() *
                    na::point![x,y]).coords;

                if value.is::<gio::File>() {
                    appwindow.open_file_w_dialogs(value.get::<gio::File>().unwrap(), Some(pos), true);

                    return true;
                } else if value.is::<String>() {
                    if let Err(e) = canvas.load_in_text(value.get::<String>().unwrap(), Some(pos)) {
                        log::error!("failed to insert dropped in text, Err: {e:?}");
                    }
                }

                false
            }),
        );

        // update ui when zoom changes
        let appwindow_zoom_changed = self.connect_local("zoom-changed", false, clone!(@weak self as canvas, @weak appwindow => @default-return None, move |_| {
            let total_zoom = canvas.engine().borrow().camera.total_zoom();
            appwindow.mainheader().canvasmenu().zoom_reset_button().set_label(format!("{:.0}%", (100.0 * total_zoom).round()).as_str());
            None
        }));

        // handle widget flags
        let appwindow_handle_widget_flags = self.connect_local(
            "handle-widget-flags",
            false,
            clone!(@weak self as canvas, @weak appwindow => @default-return None, move |args| {
                // first argument is RnoteCanvas
                let widget_flags = args[1].get::<WidgetFlagsBoxed>().unwrap().0;

                appwindow.handle_widget_flags(widget_flags, &canvas);
                None
            }),
        );

        // Replace old handlers
        let mut handlers = self.imp().handlers.borrow_mut();
        if let Some(old) = handlers
            .appwindow_output_file
            .replace(appwindow_output_file)
        {
            self.disconnect(old);
        }
        if let Some(old) = handlers
            .appwindow_scalefactor
            .replace(appwindow_scalefactor)
        {
            self.disconnect(old);
        }
        if let Some(old) = handlers
            .appwindow_unsaved_changes
            .replace(appwindow_unsaved_changes)
        {
            self.disconnect(old);
        }
        if let Some(old) = handlers
            .appwindow_touch_drawing
            .replace(appwindow_touch_drawing)
        {
            old.unbind();
        }
        if let Some(old) = handlers
            .appwindow_regular_cursor
            .replace(appwindow_regular_cursor)
        {
            old.unbind();
        }
        if let Some(old) = handlers
            .appwindow_drawing_cursor
            .replace(appwindow_drawing_cursor)
        {
            old.unbind();
        }
        if let Some(old) = handlers
            .appwindow_drop_target
            .replace(appwindow_drop_target)
        {
            self.imp().drop_target.disconnect(old);
        }
        if let Some(old) = handlers
            .appwindow_zoom_changed
            .replace(appwindow_zoom_changed)
        {
            self.disconnect(old);
        }
        if let Some(old) = handlers
            .appwindow_handle_widget_flags
            .replace(appwindow_handle_widget_flags)
        {
            self.disconnect(old);
        }
    }

    /// This disconnects all handlers with references to external objects, to prepare moving the widget to another appwindow.
    pub(crate) fn disconnect_handlers(&self, _appwindow: &RnoteAppWindow) {
        self.clear_output_file_monitor();

        let mut handlers = self.imp().handlers.borrow_mut();
        if let Some(old) = handlers.appwindow_output_file.take() {
            self.disconnect(old);
        }
        if let Some(old) = handlers.appwindow_scalefactor.take() {
            self.disconnect(old);
        }
        if let Some(old) = handlers.appwindow_unsaved_changes.take() {
            self.disconnect(old);
        }
        if let Some(old) = handlers.appwindow_touch_drawing.take() {
            old.unbind();
        }
        if let Some(old) = handlers.appwindow_regular_cursor.take() {
            old.unbind();
        }
        if let Some(old) = handlers.appwindow_drawing_cursor.take() {
            old.unbind();
        }
        if let Some(old) = handlers.appwindow_drop_target.take() {
            self.imp().drop_target.disconnect(old);
        }
        if let Some(old) = handlers.appwindow_zoom_changed.take() {
            self.disconnect(old);
        }
        if let Some(old) = handlers.appwindow_handle_widget_flags.take() {
            self.disconnect(old);
        }

        // tab page connections
        if let Some(old) = handlers.tab_page_output_file.take() {
            old.unbind();
        }
        if let Some(old) = handlers.tab_page_unsaved_changes.take() {
            old.unbind();
        }
    }

    /// When the widget is the child of a tab page, we want to connect their titles, icons, ..
    ///
    /// disconnects existing bindings / handlers to old tab pages.
    pub(crate) fn connect_to_tab_page(&self, page: &adw::TabPage) {
        // update the tab title whenever the canvas output file changes
        let tab_page_output_file = self
            .bind_property("output-file", page, "title")
            .sync_create()
            .transform_to(|b, _output_file: Option<gio::File>| {
                Some(
                    b.source()?
                        .downcast::<RnoteCanvas>()
                        .unwrap()
                        .doc_title_display(),
                )
            })
            .build();

        // display unsaved changes as icon
        let tab_page_unsaved_changes = self
            .bind_property("unsaved-changes", page, "icon")
            .transform_to(|_, from: bool| {
                Some(from.then_some(gio::ThemedIcon::new("dot-symbolic")))
            })
            .sync_create()
            .build();

        let mut handlers = self.imp().handlers.borrow_mut();
        if let Some(old) = handlers.tab_page_output_file.replace(tab_page_output_file) {
            old.unbind();
        }

        if let Some(old) = handlers
            .tab_page_unsaved_changes
            .replace(tab_page_unsaved_changes)
        {
            old.unbind();
        }
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
        let Some(parent) = self.parent() else {
            log::debug!("self.parent() is None in `center_around_coord_on_doc().");
            return
        };

        let (parent_width, parent_height) = (f64::from(parent.width()), f64::from(parent.height()));
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
        let Some(parent) = self.parent() else {
            log::debug!("self.parent() is None in `return_to_origin_page().");
            return
        };

        let new_offset =
            if self.engine().borrow().document.format.width * zoom <= f64::from(parent.width()) {
                na::vector![
                    (self.engine().borrow().document.format.width * 0.5 * zoom)
                        - f64::from(parent.width()) * 0.5,
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
        if let Some(source_id) = self.imp().handlers.borrow_mut().zoom_timeout.take() {
            source_id.remove();
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
    pub(crate) fn zoom_temporarily_then_scale_to_after_timeout(&self, new_zoom: f64) {
        if let Some(handler_id) = self.imp().handlers.borrow_mut().zoom_timeout.take() {
            handler_id.remove();
        }

        let old_perm_zoom = self.engine().borrow().camera.zoom();

        // Zoom temporarily
        let new_temp_zoom = new_zoom / old_perm_zoom;
        self.engine()
            .borrow_mut()
            .camera
            .set_temporary_zoom(new_temp_zoom);

        self.emit_zoom_changed();

        // In resize we render the strokes that came into view
        self.queue_resize();

        if let Some(source_id) = self.imp().handlers.borrow_mut().zoom_timeout.replace(
            glib::source::timeout_add_local_once(
                Self::ZOOM_TIMEOUT_TIME,
                clone!(@weak self as canvas => move || {

                    // After timeout zoom permanent
                    canvas.zoom_to(new_zoom);

                    // Removing the timeout id
                    let mut handlers = canvas.imp().handlers.borrow_mut();
                    if let Some(source_id) = handlers.zoom_timeout.take() {
                        source_id.remove();
                    }
                }),
            ),
        ) {
            source_id.remove();
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
