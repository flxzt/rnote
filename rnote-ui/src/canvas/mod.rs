mod canvaslayout;
mod input;

// Re-exports
pub use canvaslayout::CanvasLayout;

// Imports
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::config;
use rnote_engine::RnoteEngine;

use gtk4::{
    gdk, gio, glib, glib::clone, graphene, prelude::*, subclass::prelude::*, AccessibleRole,
    Adjustment, DropTarget, EventControllerKey, EventSequenceState, GestureDrag, GestureStylus,
    Inhibit, PropagationPhase, Scrollable, ScrollablePolicy, Widget,
};

use crate::appwindow::RnoteAppWindow;
use futures::StreamExt;
use once_cell::sync::Lazy;
use p2d::bounding_volume::AABB;
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::penpath::Element;
use rnote_engine::utils::GrapheneRectHelpers;
use rnote_engine::Sheet;

use std::collections::VecDeque;
use std::time;

mod imp {
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
        pub cursor: gdk::Cursor,
        pub motion_cursor: gdk::Cursor,
        pub stylus_drawing_gesture: GestureStylus,
        pub mouse_drawing_gesture: GestureDrag,
        pub touch_drawing_gesture: GestureDrag,
        pub key_controller: EventControllerKey,

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
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            // Gesture grouping
            mouse_drawing_gesture.group_with(&stylus_drawing_gesture);
            touch_drawing_gesture.group_with(&stylus_drawing_gesture);

            let cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH) + "icons/scalable/actions/canvas-cursor.svg")
                        .as_str(),
                ),
                8,
                8,
                gdk::Cursor::from_name("default", None).as_ref(),
            );
            let motion_cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH)
                        + "icons/scalable/actions/canvas-motion-cursor.svg")
                        .as_str(),
                ),
                8,
                8,
                gdk::Cursor::from_name("default", None).as_ref(),
            );

            Self {
                hadjustment: RefCell::new(None),
                hadjustment_signal: RefCell::new(None),
                vadjustment: RefCell::new(None),
                vadjustment_signal: RefCell::new(None),
                hscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                vscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                cursor,
                motion_cursor,
                stylus_drawing_gesture,
                mouse_drawing_gesture,
                touch_drawing_gesture,
                key_controller,
                zoom_timeout_id: RefCell::new(None),

                engine: Rc::new(RefCell::new(RnoteEngine::default())),

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
            obj.set_cursor(Some(&self.cursor));

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
                self.engine.borrow().draw(&snapshot, widget.bounds())?;

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

    pub fn cursor(&self) -> gdk::Cursor {
        self.imp().cursor.clone()
    }

    pub fn motion_cursor(&self) -> gdk::Cursor {
        self.imp().motion_cursor.clone()
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
                    canvas.update_engine_rendering();
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
                    canvas.update_engine_rendering();
                }),
            );
            self.imp().vadjustment_signal.replace(Some(signal_id));
        }
        self.imp().vadjustment.replace(adj);
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.setup_input(appwindow);

        glib::MainContext::default().spawn_local(clone!(@strong self as canvas, @strong appwindow => async move {
            let mut task_rx = canvas.engine().borrow_mut().tasks_rx.take().unwrap();

            loop {
                if let Some(task) = task_rx.next().await {
                    let surface_flags = canvas.engine().borrow_mut().process_received_task(task);
                    appwindow.handle_surface_flags(surface_flags);
                }
            }
        }));

        self.connect_notify_local(
            Some("output-file"),
            clone!(@weak appwindow => move |canvas, _pspec| {
                appwindow.mainheader().set_title_for_file(canvas.output_file().as_ref());
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
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }
            stylus_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
           Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_stylus_shortcut_keys(&stylus_drawing_gesture);

            if let Some(first) = data_entries.pop_front() {
                input::process_pen_down(first, shortcut_keys.clone(), &appwindow);
            }
            input::process_pen_motion(data_entries, shortcut_keys, &appwindow);
        }));

        self.imp().stylus_drawing_gesture.connect_motion(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            //log::debug!("stylus_drawing_gesture motion");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }

            let mut data_entries: VecDeque<Element> = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_stylus_shortcut_keys(&stylus_drawing_gesture);

            input::process_pen_motion(data_entries, shortcut_keys, &appwindow);
        }));

        self.imp().stylus_drawing_gesture.connect_up(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture up");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }

            let mut data_entries = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_stylus_shortcut_keys(&stylus_drawing_gesture);

            if let Some(last) = data_entries.pop_back() {
                input::process_pen_motion(data_entries, shortcut_keys.clone(), &appwindow);
                input::process_pen_up(last, shortcut_keys, &appwindow);
            }
        }));

        self.imp().stylus_drawing_gesture.connect_proximity(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture proximity");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }

            let mut data_entries = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_stylus_shortcut_keys(&stylus_drawing_gesture);

            input::process_pen_proximity(data_entries, shortcut_keys.clone(), &appwindow);
        }));

        // Mouse drawing
        self.imp().mouse_drawing_gesture.connect_drag_begin(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture begin");
            //input::debug_drag_gesture(&mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }
            mouse_drawing_gesture.set_state(EventSequenceState::Claimed);
            canvas.grab_focus();

            let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_keys = input::retreive_mouse_shortcut_keys(&mouse_drawing_gesture);

            if let Some(first) = data_entries.pop_front() {
                input::process_pen_down(first, shortcut_keys.clone(), &appwindow);
            }
            input::process_pen_motion(data_entries, shortcut_keys, &appwindow);
        }));

        self.imp().mouse_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture motion");
            //input::debug_drag_gesture(&mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retreive_mouse_shortcut_keys(&mouse_drawing_gesture);

                input::process_pen_motion(data_entries, shortcut_keys, &appwindow);
            }
        }));

        self.imp().mouse_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture end");
            //input::debug_drag_gesture(&mouse_drawing_gesture);

            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1) );

                let shortcut_keys = input::retreive_mouse_shortcut_keys(&mouse_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    input::process_pen_motion(data_entries, shortcut_keys.clone(), &appwindow);
                    input::process_pen_up(last, shortcut_keys, &appwindow);
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

            let shortcut_keys = input::retreive_touch_shortcut_keys(&touch_drawing_gesture);

            if let Some(first) = data_entries.pop_front() {
                input::process_pen_down(first, shortcut_keys.clone(), &appwindow);
            }
            input::process_pen_motion(data_entries, shortcut_keys, &appwindow);
        }));

        self.imp().touch_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture motion");

                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retreive_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retreive_touch_shortcut_keys(&touch_drawing_gesture);

                input::process_pen_motion(data_entries, shortcut_keys, &appwindow);
            }
        }));

        self.imp().touch_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture end");

                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retreive_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_keys = input::retreive_touch_shortcut_keys(&touch_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    input::process_pen_motion(data_entries, shortcut_keys.clone(), &appwindow);
                    input::process_pen_up(last, shortcut_keys, &appwindow);
                }
            }
        }));

        // Key controller

        // modifiers not really working in connect_key_pressed, use connect_modifiers for it
        self.imp().key_controller.connect_key_pressed(clone!(@weak self as canvas, @weak appwindow => @default-return Inhibit(false), move |_key_controller, key, _raw, _modifier| {
            //log::debug!("key_pressed: {:?}, {:?}, {:?}", key.to_unicode(), raw, modifier);

            if let Some(shortcut_key) = input::retreive_keyboard_key_shortcut_key(key) {
                input::process_keyboard_pressed(shortcut_key, &appwindow);
            }

            Inhibit(true)
        }));
        self.imp().key_controller.connect_modifiers(clone!(@weak self as canvas, @weak appwindow => @default-return Inhibit(false), move |_key_controller, modifier| {
            //log::debug!("key_controller modifier: {:?}", modifier);

            let shortcut_keys = input::retreive_modifier_shortcut_key(modifier);

            for shortcut_key in shortcut_keys {
                input::process_keyboard_pressed(shortcut_key, &appwindow);
            }

            Inhibit(true)
        }));

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

        self.hadjustment().unwrap().set_value(new_offset[0]);
        self.vadjustment().unwrap().set_value(new_offset[1]);

        self.queue_resize();
    }

    /// returns the center of the current view on the sheet
    // update_engine_rendering() then needs to be called.
    pub fn current_center_on_sheet(&self) -> na::Vector2<f64> {
        (self.engine().borrow().camera.transform().inverse()
            * na::point![
                f64::from(self.width()) * 0.5,
                f64::from(self.height()) * 0.5
            ])
        .coords
    }

    /// Centers the view around a coord on the sheet. The coord parameter has the coordinate space of the sheet!
    // update_engine_rendering() then needs to be called.
    pub fn center_around_coord_on_sheet(&self, coord: na::Vector2<f64>) {
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

        let new_offset = if self.engine().borrow().sheet.format.width * zoom
            <= f64::from(self.parent().unwrap().width())
        {
            na::vector![
                (self.engine().borrow().sheet.format.width * 0.5 * zoom)
                    - f64::from(self.parent().unwrap().width()) * 0.5,
                -Sheet::SHADOW_WIDTH * zoom
            ]
        } else {
            // If the zoomed format width is larger than the displayed surface, we zoom to a fixed origin
            na::vector![-Sheet::SHADOW_WIDTH * zoom, -Sheet::SHADOW_WIDTH * zoom]
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
    }

    /// Zooms temporarily and then scale the canvas and its contents to a new zoom after a given time.
    /// Repeated calls to this function reset the timeout.
    /// should only be called in the "zoom-to-value" action.
    pub fn zoom_temporarily_then_scale_to_after_timeout(
        &self,
        zoom: f64,
        timeout_time: time::Duration,
    ) {
        if let Some(zoom_timeout_id) = self.imp().zoom_timeout_id.take() {
            zoom_timeout_id.remove();
        }

        // Zoom temporarily
        let new_temp_zoom = zoom / self.engine().borrow().camera.zoom();
        self.engine()
            .borrow_mut()
            .camera
            .set_temporary_zoom(new_temp_zoom);

        self.queue_resize();

        if let Some(zoom_timeout_id) =
            self.imp()
                .zoom_timeout_id
                .borrow_mut()
                .replace(glib::source::timeout_add_local_once(
                    timeout_time,
                    clone!(@weak self as canvas => move || {

                        // After timeout zoom permanent
                        canvas.zoom_to(zoom);

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
        // Updating the engine rendering in the layout manager.
        self.queue_resize();
    }

    /// updates the background pattern and rendering for the current viewport.
    /// to be called for example when changing the background pattern or zoom.
    pub fn regenerate_background_pattern(&self) {
        let viewport = self.engine().borrow().camera.viewport();
        let image_scale = self.engine().borrow().camera.image_scale();

        if let Err(e) = self
            .engine()
            .borrow_mut()
            .sheet
            .background
            .regenerate_pattern(viewport, image_scale)
        {
            log::error!("failed to regenerate background, {}", e)
        };

        self.queue_draw();
    }
}
