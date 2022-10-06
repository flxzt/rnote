mod canvaslayout;
mod input;

// Re-exports
pub use canvaslayout::CanvasLayout;
use rnote_engine::pens::PenMode;

// Imports
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::config;
use rnote_engine::RnoteEngine;

use gtk4::{
    gdk, gio, glib, glib::clone, graphene, prelude::*, subclass::prelude::*, AccessibleRole,
    Adjustment, DropTarget, EventControllerKey, EventSequenceState, GestureDrag, GestureStylus,
    IMContextSimple, Inhibit, PropagationPhase, Scrollable, ScrollablePolicy, Widget,
};

use crate::appwindow::RnoteAppWindow;
use futures::StreamExt;
use once_cell::sync::Lazy;
use p2d::bounding_volume::AABB;
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::penpath::Element;
use rnote_engine::utils::GrapheneRectHelpers;
use rnote_engine::Document;

use std::collections::VecDeque;
use std::time;

mod imp {
    use std::path::PathBuf;

    use super::*;

    #[allow(missing_debug_implementations)]
    pub struct RnoteCanvas {
        pub hadjustment: RefCell<Option<Adjustment>>,
        pub hadjustment_signal: RefCell<Option<glib::SignalHandlerId>>,
        pub vadjustment: RefCell<Option<Adjustment>>,
        pub vadjustment_signal: RefCell<Option<glib::SignalHandlerId>>,
        pub hscroll_policy: Cell<ScrollablePolicy>,
        pub vscroll_policy: Cell<ScrollablePolicy>,
        pub zoom_timeout_id: RefCell<Option<glib::SourceId>>,
        pub regular_cursor: RefCell<gdk::Cursor>,
        pub regular_cursor_icon_name: RefCell<String>,
        pub motion_cursor: RefCell<gdk::Cursor>,
        pub motion_cursor_icon_name: RefCell<String>,
        pub stylus_drawing_gesture: GestureStylus,
        pub mouse_drawing_gesture: GestureDrag,
        pub touch_drawing_gesture: GestureDrag,
        pub key_controller: EventControllerKey,
        pub key_controller_im_context: IMContextSimple,

        pub engine: Rc<RefCell<RnoteEngine>>,

        pub output_file: RefCell<Option<gio::File>>,
        pub unsaved_changes: Cell<bool>,
        pub empty: Cell<bool>,

        pub touch_drawing: Cell<bool>,
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

            let key_controller_im_context = IMContextSimple::new();

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
            let motion_cursor_icon_name = String::from("cursor-dot-small");
            let motion_cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH)
                        + "icons/scalable/actions/cursor-dot-small.svg")
                        .as_str(),
                ),
                32,
                32,
                gdk::Cursor::from_name("default", None).as_ref(),
            );

            let engine = RnoteEngine::new(Some(PathBuf::from(config::PKG_DATA_DIR)));

            Self {
                hadjustment: RefCell::new(None),
                hadjustment_signal: RefCell::new(None),
                vadjustment: RefCell::new(None),
                vadjustment_signal: RefCell::new(None),
                hscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                vscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                regular_cursor: RefCell::new(regular_cursor),
                regular_cursor_icon_name: RefCell::new(regular_cursor_icon_name),
                motion_cursor: RefCell::new(motion_cursor),
                motion_cursor_icon_name: RefCell::new(motion_cursor_icon_name),
                stylus_drawing_gesture,
                mouse_drawing_gesture,
                touch_drawing_gesture,
                key_controller,
                key_controller_im_context,
                zoom_timeout_id: RefCell::new(None),

                engine: Rc::new(RefCell::new(engine)),

                output_file: RefCell::new(None),
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_hexpand(false);
            obj.set_vexpand(false);
            obj.set_can_target(true);
            obj.set_focusable(true);
            obj.set_can_focus(true);

            obj.set_cursor(Some(&*self.regular_cursor.borrow()));

            obj.add_controller(&self.stylus_drawing_gesture);
            obj.add_controller(&self.mouse_drawing_gesture);
            obj.add_controller(&self.touch_drawing_gesture);
            obj.add_controller(&self.key_controller);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    // Flag for any unsaved changes on the canvas. Propagates to the application 'unsaved-changes' property
                    glib::ParamSpecObject::new(
                        "output-file",
                        "output-file",
                        "output-file",
                        Option::<gio::File>::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "unsaved-changes",
                        "unsaved-changes",
                        "unsaved-changes",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Wether the canvas is empty
                    glib::ParamSpecBoolean::new(
                        "empty",
                        "empty",
                        "empty",
                        true,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Wether to enable touch drawing
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
                        "motion-cursor",
                        "motion-cursor",
                        "motion-cursor",
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

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
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
                "motion-cursor" => self.motion_cursor_icon_name.borrow().to_value(),
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
                        obj.set_unsaved_changes(false);
                    }
                }
                "hadjustment" => {
                    let hadjustment = value.get().unwrap();
                    obj.set_hadjustment(hadjustment);
                }
                "hscroll-policy" => {
                    let hscroll_policy = value.get().unwrap();
                    self.hscroll_policy.replace(hscroll_policy);
                }
                "vadjustment" => {
                    let vadjustment = value.get().unwrap();
                    obj.set_vadjustment(vadjustment);
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
                }
                "motion-cursor" => {
                    let icon_name = value.get().unwrap();
                    self.motion_cursor_icon_name.replace(icon_name);

                    let cursor = gdk::Cursor::from_texture(
                        &gdk::Texture::from_resource(
                            (String::from(config::APP_IDPATH)
                                + &format!(
                                    "icons/scalable/actions/{}.svg",
                                    self.motion_cursor_icon_name.borrow()
                                ))
                                .as_str(),
                        ),
                        32,
                        32,
                        gdk::Cursor::from_name("default", None).as_ref(),
                    );

                    self.motion_cursor.replace(cursor);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnoteCanvas {
        // request_mode(), measure(), allocate() overrides happen in the CanvasLayout LayoutManager

        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk4::Snapshot) {
            if let Err(e) = || -> anyhow::Result<()> {
                let clip_bounds = if let Some(parent) = widget.parent() {
                    // unwrapping is fine, because its the parent
                    let (clip_x, clip_y) = parent.translate_coordinates(widget, 0.0, 0.0).unwrap();
                    AABB::new_positive(
                        na::point![clip_x, clip_y],
                        na::point![f64::from(parent.width()), f64::from(parent.height())],
                    )
                } else {
                    widget.bounds()
                };
                // pushing the clip
                snapshot.push_clip(&graphene::Rect::from_p2d_aabb(clip_bounds));

                // Save the original coordinate space
                snapshot.save();

                // Draw the entire engine
                self.engine
                    .borrow()
                    .draw_on_snapshot(snapshot, widget.bounds())?;

                // Restore original coordinate space
                snapshot.restore();
                // End the clip of widget bounds
                snapshot.pop();
                Ok(())
            }() {
                log::error!("canvas snapshot() failed with Err {}", e);
            }
        }
    }

    impl ScrollableImpl for RnoteCanvas {}

    impl RnoteCanvas {}
}

glib::wrapper! {
    pub struct RnoteCanvas(ObjectSubclass<imp::RnoteCanvas>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Scrollable;
}

impl Default for RnoteCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl RnoteCanvas {
    /// The zoom amount when activating the zoom-in / zoom-out action
    pub const ZOOM_ACTION_DELTA: f64 = 0.1;
    // the zoom timeout time
    pub const ZOOM_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(300);
    // Sets the canvas zoom scroll step in % for one unit of the event controller delta
    pub const ZOOM_STEP: f64 = 0.1;

    pub fn new() -> Self {
        let canvas: RnoteCanvas = glib::Object::new(&[]).expect("Failed to create RnoteCanvas");

        canvas
    }

    pub fn regular_cursor(&self) -> String {
        self.property::<String>("regular-cursor")
    }

    pub fn set_regular_cursor(&self, regular_cursor: String) {
        self.set_property("regular-cursor", regular_cursor.to_value());
    }

    pub fn motion_cursor(&self) -> String {
        self.property::<String>("motion-cursor")
    }

    pub fn set_motion_cursor(&self, motion_cursor: String) {
        self.set_property("motion-cursor", motion_cursor.to_value());
    }

    /// Only change the engine state in actions to avoid nested mutable borrows!
    pub fn engine(&self) -> Rc<RefCell<RnoteEngine>> {
        self.imp().engine.clone()
    }

    pub fn output_file(&self) -> Option<gio::File> {
        self.property::<Option<gio::File>>("output-file")
    }

    pub fn set_output_file(&self, output_file: Option<gio::File>) {
        self.set_property("output-file", output_file.to_value());
    }

    pub fn unsaved_changes(&self) -> bool {
        self.property::<bool>("unsaved-changes")
    }

    pub fn set_unsaved_changes(&self, unsaved_changes: bool) {
        self.set_property("unsaved-changes", unsaved_changes.to_value());
    }

    pub fn empty(&self) -> bool {
        self.property::<bool>("empty")
    }

    pub fn set_empty(&self, empty: bool) {
        self.set_property("empty", empty.to_value());
    }

    pub fn touch_drawing(&self) -> bool {
        self.property::<bool>("touch-drawing")
    }

    pub fn set_touch_drawing(&self, touch_drawing: bool) {
        self.set_property("touch-drawing", touch_drawing.to_value());
    }

    fn set_hadjustment(&self, adj: Option<Adjustment>) {
        if let Some(signal_id) = self.imp().hadjustment_signal.borrow_mut().take() {
            let old_adj = self.imp().hadjustment.borrow().as_ref().unwrap().clone();
            old_adj.disconnect(signal_id);
        }

        if let Some(ref hadjustment) = adj {
            let signal_id = hadjustment.connect_value_changed(
                clone!(@weak self as canvas => move |_hadjustment| {
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
                    canvas.queue_resize();
                }),
            );

            self.imp().vadjustment_signal.replace(Some(signal_id));
        }
        self.imp().vadjustment.replace(adj);
    }

    pub fn set_text_preprocessing(&self, enable: bool) {
        if enable {
            self.imp()
                .key_controller
                .set_im_context(Some(&self.imp().key_controller_im_context));
        } else {
            self.imp()
                .key_controller
                .set_im_context(None::<&IMContextSimple>);
        }
    }

    /// Switches between the regular and the motion cursor
    pub fn switch_between_cursors(&self, in_motion: bool) {
        if in_motion {
            self.set_cursor(Some(&*self.imp().motion_cursor.borrow()));
        } else {
            self.set_cursor(Some(&*self.imp().regular_cursor.borrow()));
        }
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
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

                appwindow.update_titles_for_file(output_file.as_ref());
            }),
        );

        self.bind_property("unsaved-changes", appwindow, "unsaved-changes")
            .flags(glib::BindingFlags::DEFAULT)
            .build();

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
            "motion-cursor",
            &appwindow
                .settings_panel()
                .general_motion_cursor_picker_image(),
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

            canvas
                .engine()
                .borrow_mut()
                .store
                .set_rendering_dirty_all_keys();

            canvas.regenerate_background_pattern();
            canvas.update_engine_rendering();
        });
    }

    fn setup_input(&self, appwindow: &RnoteAppWindow) {
        // Stylus Drawing
        self.imp().stylus_drawing_gesture.connect_down(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture down");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }
            stylus_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
           Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retreive_stylus_pen_mode(stylus_drawing_gesture);

            for element in data_entries {
                input::process_pen_down(element, shortcut_keys.clone(), pen_mode, &appwindow);
            }
        }));

        self.imp().stylus_drawing_gesture.connect_motion(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            //log::debug!("stylus_drawing_gesture motion");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries: VecDeque<Element> = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retreive_stylus_pen_mode(stylus_drawing_gesture);

            for element in data_entries {
                input::process_pen_down(element, shortcut_keys.clone(), pen_mode, &appwindow);
            }
        }));

        self.imp().stylus_drawing_gesture.connect_up(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture up");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retreive_stylus_pen_mode(stylus_drawing_gesture);

            if let Some(last) = data_entries.pop_back() {
                for element in data_entries {
                    input::process_pen_down(element, shortcut_keys.clone(), pen_mode, &appwindow);
                }
                input::process_pen_up(last, shortcut_keys, pen_mode, &appwindow);
            }
        }));

        self.imp().stylus_drawing_gesture.connect_proximity(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture proximity");
            //input::debug_stylus_gesture(stylus_drawing_gesture);

            if input::filter_stylus_input(stylus_drawing_gesture) { return; }

            let mut data_entries = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_stylus_shortcut_keys(stylus_drawing_gesture);
            let pen_mode = input::retreive_stylus_pen_mode(stylus_drawing_gesture);

            for element in data_entries {
                input::process_pen_proximity(element, shortcut_keys.clone(), pen_mode, &appwindow);
            }
        }));

        // Mouse drawing
        self.imp().mouse_drawing_gesture.connect_drag_begin(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture begin");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }
            mouse_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_mouse_shortcut_keys(mouse_drawing_gesture);

            for element in data_entries {
                input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), &appwindow);
            }
        }));

        self.imp().mouse_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture motion");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retreive_mouse_shortcut_keys(mouse_drawing_gesture);

                for element in data_entries {
                    input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), &appwindow);
                }
            }
        }));

        self.imp().mouse_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture end");
            //input::debug_drag_gesture(mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1) );

                let shortcut_keys = input::retreive_mouse_shortcut_keys(mouse_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    for element in data_entries {
                        input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), &appwindow);
                    }
                    input::process_pen_up(last, shortcut_keys, Some(PenMode::Pen), &appwindow);
                }
            }
        }));

        // Touch drawing
        self.imp().touch_drawing_gesture.connect_drag_begin(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            //log::debug!("touch_drawing_gesture begin");

            if input::filter_touch_input(touch_drawing_gesture) { return; }
            touch_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retreive_pointer_elements(touch_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_touch_shortcut_keys(touch_drawing_gesture);

            for element in data_entries {
                input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), &appwindow);
            }
        }));

        self.imp().touch_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture motion");

                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retreive_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retreive_touch_shortcut_keys(touch_drawing_gesture);

                for element in data_entries {
                    input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), &appwindow);
                }
            }
        }));

        self.imp().touch_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture end");

                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retreive_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retreive_touch_shortcut_keys(touch_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    for element in data_entries {
                        input::process_pen_down(element, shortcut_keys.clone(), Some(PenMode::Pen), &appwindow);
                    }
                    input::process_pen_up(last, shortcut_keys, Some(PenMode::Pen), &appwindow);
                }
            }
        }));

        // Key controller

        self.imp().key_controller.connect_key_pressed(clone!(@weak self as canvas, @weak appwindow => @default-return Inhibit(false), move |_key_controller, key, _raw, modifier| {
            //log::debug!("key pressed - key: {:?}, raw: {:?}, modifier: {:?}", key, raw, modifier);
            canvas.grab_focus();

            let keyboard_key = input::retreive_keyboard_key(key);
            let shortcut_keys = input::retreive_modifier_shortcut_key(modifier);

            //log::debug!("keyboard key: {:?}", keyboard_key);

            input::process_keyboard_key_pressed(keyboard_key, shortcut_keys, &appwindow);

            Inhibit(true)
        }));

        // For unicode text the input is commited from the IM context, and won't trigger the key_pressed signal
        self.imp().key_controller_im_context.connect_commit(
            clone!(@weak self as canvas, @weak appwindow => move |_cx, text| {
                input::process_keyboard_text(text.to_string(), &appwindow);
            }),
        );

        /*
        self.imp().key_controller.connect_key_released(clone!(@weak self as canvas, @weak appwindow => move |_key_controller, _key, _raw, _modifier| {
            //log::debug!("key released - key: {:?}, raw: {:?}, modifier: {:?}", key, raw, modifier);
        }));

        self.imp().key_controller.connect_modifiers(clone!(@weak self as canvas, @weak appwindow => @default-return Inhibit(false), move |_key_controller, modifier| {
            //log::debug!("key_controller modifier pressed: {:?}", modifier);

            let shortcut_keys = input::retreive_modifier_shortcut_key(modifier);
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
        drop_target.set_types(&[gio::File::static_type()]);
        self.add_controller(&drop_target);

        drop_target.connect_drop(
            clone!(@weak appwindow => @default-return false, move |_drop_target, value, x, y| {
                let pos = (appwindow.canvas().engine().borrow().camera.transform().inverse() *
                    na::point![x,y]).coords;

                if let Ok(file) = value.get::<gio::File>() {
                    appwindow.open_file_w_dialogs(&file, Some(pos))
                }
                true
            }),
        );
    }

    pub fn bounds(&self) -> AABB {
        AABB::new_positive(
            na::point![0.0, 0.0],
            na::point![f64::from(self.width()), f64::from(self.height())],
        )
    }

    // updates the camera offset with a new one ( for example from touch drag gestures )
    // update_engine_rendering() then needs to be called.
    pub fn update_camera_offset(&self, new_offset: na::Vector2<f64>) {
        self.engine().borrow_mut().update_camera_offset(new_offset);

        // By setting new adjustment values, the callback connected to their value property is called,
        // Which is where the engine rendering is updated.
        self.hadjustment().unwrap().set_value(new_offset[0]);
        self.vadjustment().unwrap().set_value(new_offset[1]);
    }

    /// returns the center of the current view on the doc
    // update_engine_rendering() then needs to be called.
    pub fn current_center_on_doc(&self) -> na::Vector2<f64> {
        (self.engine().borrow().camera.transform().inverse()
            * na::point![
                f64::from(self.width()) * 0.5,
                f64::from(self.height()) * 0.5
            ])
        .coords
    }

    /// Centers the view around a coord on the doc. The coord parameter has the coordinate space of the doc.
    // update_engine_rendering() then needs to be called.
    pub fn center_around_coord_on_doc(&self, coord: na::Vector2<f64>) {
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
    pub fn return_to_origin_page(&self) {
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

        self.engine()
            .borrow_mut()
            .store
            .set_rendering_dirty_all_keys();

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
    pub fn zoom_temporarily_then_scale_to_after_timeout(
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
    pub fn update_engine_rendering(&self) {
        // background rendering is updated in the layout manager
        self.queue_resize();

        // Update engine rendering for the new viewport
        self.engine()
            .borrow_mut()
            .update_rendering_current_viewport();

        self.queue_draw();
    }

    /// updates the background pattern and rendering for the current viewport.
    /// to be called for example when changing the background pattern or zoom.
    pub fn regenerate_background_pattern(&self) {
        let viewport = self.engine().borrow().camera.viewport();
        let image_scale = self.engine().borrow().camera.image_scale();

        if let Err(e) = self
            .engine()
            .borrow_mut()
            .document
            .background
            .regenerate_pattern(viewport, image_scale)
        {
            log::error!("failed to regenerate background, {}", e)
        };

        self.queue_draw();
    }
}
