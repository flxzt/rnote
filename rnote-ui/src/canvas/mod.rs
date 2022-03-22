pub mod canvaslayout;
pub mod input;

mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::canvaslayout::CanvasLayout;
    use crate::config;
    use crate::selectionmodifier::SelectionModifier;
    use rnote_engine::{RnoteEngine};

    use gtk4::{
        gdk, glib, graphene, prelude::*, subclass::prelude::*, GestureDrag, GestureStylus,
        PropagationPhase, Widget,
    };
    use gtk4::{AccessibleRole, Adjustment, Scrollable, ScrollablePolicy};

    use once_cell::sync::Lazy;

    #[derive(Debug)]
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
        pub selection_modifier: SelectionModifier,
        pub return_to_center_toast: RefCell<Option<adw::Toast>>,

        pub engine: Rc<RefCell<RnoteEngine>>,

        pub unsaved_changes: Cell<bool>,
        pub empty: Cell<bool>,

        pub touch_drawing: Cell<bool>,
        pub pdf_import_width: Cell<f64>,
        pub pdf_import_as_vector: Cell<bool>,
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
                zoom_timeout_id: RefCell::new(None),
                return_to_center_toast: RefCell::new(None),

                selection_modifier: SelectionModifier::default(),

                engine: Rc::new(RefCell::new(RnoteEngine::default())),

                unsaved_changes: Cell::new(false),
                empty: Cell::new(true),

                touch_drawing: Cell::new(false),
                pdf_import_width: Cell::new(super::RnoteCanvas::PDF_IMPORT_WIDTH_DEFAULT),
                pdf_import_as_vector: Cell::new(true),
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
            self.selection_modifier.set_parent(obj);

            obj.set_hexpand(false);
            obj.set_vexpand(false);
            obj.set_can_target(true);
            obj.set_focusable(true);
            obj.set_can_focus(true);
            obj.set_cursor(Some(&self.cursor));

            obj.add_controller(&self.stylus_drawing_gesture);
            obj.add_controller(&self.mouse_drawing_gesture);
            obj.add_controller(&self.touch_drawing_gesture);
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
                    // import PDFs with with in percentage to sheet width
                    glib::ParamSpecDouble::new(
                        "pdf-import-width",
                        "pdf-import-width",
                        "pdf-import-width",
                        1.0,
                        100.0,
                        super::RnoteCanvas::PDF_IMPORT_WIDTH_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    // import PDFs as vector images ( if false = as bitmap images )
                    glib::ParamSpecBoolean::new(
                        "pdf-import-as-vector",
                        "pdf-import-as-vector",
                        "pdf-import-as-vector",
                        true,
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
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
                "empty" => self.empty.get().to_value(),
                "hadjustment" => self.hadjustment.borrow().to_value(),
                "vadjustment" => self.vadjustment.borrow().to_value(),
                "hscroll-policy" => self.hscroll_policy.get().to_value(),
                "vscroll-policy" => self.vscroll_policy.get().to_value(),
                "touch-drawing" => self.touch_drawing.get().to_value(),
                "pdf-import-width" => self.pdf_import_width.get().to_value(),
                "pdf-import-as-vector" => self.pdf_import_as_vector.get().to_value(),
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
                "pdf-import-width" => {
                    let pdf_import_width = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.")
                        .clamp(1.0, 100.0);

                    self.pdf_import_width.replace(pdf_import_width);
                }
                "pdf-import-as-vector" => {
                    let pdf_import_as_vector = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`.");

                    self.pdf_import_as_vector.replace(pdf_import_as_vector);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnoteCanvas {
        // request_mode(), measure(), allocate() overrides happen in the CanvasLayout LayoutManager

        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk4::Snapshot) {
            if let Err(e) = || -> Result<(), anyhow::Error> {
                let (clip_x, clip_y, clip_width, clip_height) = if let Some(parent) =
                    widget.parent()
                {
                    // unwrapping is fine, because its the parent
                    let (clip_x, clip_y) = parent.translate_coordinates(widget, 0.0, 0.0).unwrap();
                    (
                        clip_x as f32,
                        clip_y as f32,
                        parent.width() as f32,
                        parent.height() as f32,
                    )
                } else {
                    (0.0, 0.0, widget.width() as f32, widget.height() as f32)
                };

                // Clip everything outside the parent (scroller) view
                snapshot.push_clip(&graphene::Rect::new(
                    clip_x,
                    clip_y,
                    clip_width,
                    clip_height,
                ));

                // Save the original coordinate space
                snapshot.save();

                // Draw the entire engine
                self.engine.borrow().draw(&snapshot)?;

                // Restore original coordinate space
                snapshot.restore();

                // Draw the children
                widget.snapshot_child(&self.selection_modifier, snapshot);

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

use crate::selectionmodifier::SelectionModifier;
use crate::{app::RnoteApp, appwindow::RnoteAppWindow};
use futures::StreamExt;
use p2d::bounding_volume::{AABB};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::penpath::Element;
use rnote_engine::{RnoteEngine, Sheet};

use gettextrs::gettext;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time;

use gtk4::{gdk, glib, glib::clone, prelude::*, subclass::prelude::*};
use gtk4::{gio, Adjustment, DropTarget, EventSequenceState, PropagationPhase};

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
    pub const ZOOM_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(300);
    // The default width of imported PDF's in percentage to the sheet width
    pub const PDF_IMPORT_WIDTH_DEFAULT: f64 = 50.0;

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

    pub fn pdf_import_width(&self) -> f64 {
        self.property::<f64>("pdf-import-width")
    }

    pub fn set_pdf_import_width(&self, pdf_import_width: f64) {
        self.set_property("pdf-import-width", pdf_import_width.to_value());
    }

    pub fn pdf_import_as_vector(&self) -> bool {
        self.property::<bool>("pdf-import-as-vector")
    }

    pub fn set_pdf_import_as_vector(&self, as_vector: bool) {
        self.set_property("pdf-import-as-vector", as_vector.to_value());
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
        let self_ = imp::RnoteCanvas::from_instance(self);
        if let Some(signal_id) = self_.hadjustment_signal.borrow_mut().take() {
            let old_adj = self_.hadjustment.borrow().as_ref().unwrap().clone();
            old_adj.disconnect(signal_id);
        }

        if let Some(ref hadjustment) = adj {
            let signal_id = hadjustment.connect_value_changed(
                clone!(@weak self as canvas => move |_hadjustment| {
                    canvas.queue_resize();
                    // Everything is updated in canvaslayout allocate
                }),
            );
            self_.hadjustment_signal.replace(Some(signal_id));
        }
        self_.hadjustment.replace(adj);
    }

    fn set_vadjustment(&self, adj: Option<Adjustment>) {
        let self_ = imp::RnoteCanvas::from_instance(self);
        if let Some(signal_id) = self_.vadjustment_signal.borrow_mut().take() {
            let old_adj = self_.vadjustment.borrow().as_ref().unwrap().clone();
            old_adj.disconnect(signal_id);
        }

        if let Some(ref vadjustment) = adj {
            let signal_id = vadjustment.connect_value_changed(
                clone!(@weak self as canvas => move |_vadjustment| {
                    canvas.queue_resize();
                    // Everything is updated in canvaslayout allocate
                }),
            );
            self_.vadjustment_signal.replace(Some(signal_id));
        }
        self_.vadjustment.replace(adj);
    }

    pub fn selection_modifier(&self) -> SelectionModifier {
        imp::RnoteCanvas::from_instance(self)
            .selection_modifier
            .clone()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.setup_input(appwindow);

        // receive strokes_state tasks
        let main_cx = glib::MainContext::default();

        main_cx.spawn_local(clone!(@strong self as canvas, @strong appwindow => async move {
            let mut task_rx = canvas.engine().borrow_mut().strokes_state.tasks_rx.take().unwrap();

            loop {
                let zoom = canvas.engine().borrow().camera.zoom();
                if let Some(task) = task_rx.next().await {
                    let surface_flags = canvas.engine().borrow_mut().strokes_state.process_received_task(task, zoom);
                    appwindow.handle_surface_flags(surface_flags);
                }
            }
        }));

        self.bind_property(
            "unsaved-changes",
            &appwindow
                .application()
                .unwrap()
                .downcast::<RnoteApp>()
                .unwrap(),
            "unsaved-changes",
        )
        .flags(glib::BindingFlags::DEFAULT)
        .build();

        self.connect_notify_local(Some("unsaved-changes"), clone!(@weak appwindow => move |app, _pspec| {
            appwindow.mainheader().main_title_unsaved_indicator().set_visible(app.unsaved_changes());
            if app.unsaved_changes() {
                appwindow.mainheader().main_title().add_css_class("unsaved_changes");
            } else {
                appwindow.mainheader().main_title().remove_css_class("unsaved_changes");
            }
        }));
    }

    fn setup_input(&self, appwindow: &RnoteAppWindow) {
        // Stylus Drawing
        self.imp().stylus_drawing_gesture.connect_down(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture down");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            // filter out invalid stylus input
            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }
            stylus_drawing_gesture.set_state(EventSequenceState::Claimed);

            let mut data_entries = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
           Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_key = input::retreive_stylus_shortcut_key(&stylus_drawing_gesture);

            if let Some(first) = data_entries.pop_front() {
                input::process_pen_down(first, shortcut_key.clone(), &appwindow);
            }
            input::process_pen_motion(data_entries, shortcut_key, &appwindow);
        }));

        self.imp().stylus_drawing_gesture.connect_motion(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            //log::debug!("stylus_drawing_gesture motion");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            // filter out invalid stylus input
            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }

            let mut data_entries: VecDeque<Element> = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_key = input::retreive_stylus_shortcut_key(&stylus_drawing_gesture);

            input::process_pen_motion(data_entries, shortcut_key, &appwindow);
        }));

        self.imp().stylus_drawing_gesture.connect_up(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            //log::debug!("stylus_drawing_gesture up");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            // filter out invalid stylus input
            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }

            let mut data_entries = input::retreive_stylus_elements(stylus_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_key = input::retreive_stylus_shortcut_key(&stylus_drawing_gesture);

            if let Some(last) = data_entries.pop_back() {
                input::process_pen_motion(data_entries, shortcut_key.clone(), &appwindow);
                input::process_pen_up(last, shortcut_key, &appwindow);
            }
        }));

        // Mouse drawing
        self.imp().mouse_drawing_gesture.connect_drag_begin(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture begin");
            //input::debug_drag_gesture(&mouse_drawing_gesture);

            // filter out invalid point input
            if input::filter_mouse_input(mouse_drawing_gesture) { return; }
            mouse_drawing_gesture.set_state(EventSequenceState::Claimed);

            let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
            Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

            let shortcut_key = input::retreive_mouse_shortcut_key(&mouse_drawing_gesture);

            if let Some(first) = data_entries.pop_front() {
                input::process_pen_down(first, shortcut_key.clone(), &appwindow);
            }
            input::process_pen_motion(data_entries, shortcut_key, &appwindow);
        }));

        self.imp().mouse_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture motion");

            // filter out invalid point input
            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1));

                let shortcut_key = input::retreive_mouse_shortcut_key(&mouse_drawing_gesture);

                input::process_pen_motion(data_entries, shortcut_key, &appwindow);
            }
        }));

        self.imp().mouse_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {
            //log::debug!("mouse_drawing_gesture end");

            // filter out invalid point input
            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_elements(mouse_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse() * na::Translation2::new(start_point.0, start_point.1) );

                let shortcut_key = input::retreive_mouse_shortcut_key(&mouse_drawing_gesture);

                if let Some(last) = data_entries.pop_back() {
                    input::process_pen_motion(data_entries, shortcut_key.clone(), &appwindow);
                    input::process_pen_up(last, shortcut_key, &appwindow);
                }
            }
        }));

        // Touch drawing
        self.imp().touch_drawing_gesture.connect_drag_begin(
            clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
                //log::debug!("touch_drawing_gesture begin");

                // filter out invalid stylus input
                if input::filter_touch_input(touch_drawing_gesture) { return; }
                touch_drawing_gesture.set_state(EventSequenceState::Claimed);

                let mut data_entries = input::retreive_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, canvas.engine().borrow().camera.transform().inverse());

                if let Some(first) = data_entries.pop_front() {
                    input::process_pen_down(first, None, &appwindow);
                }
                input::process_pen_motion(data_entries, None, &appwindow);
            }),
        );

        self.imp().touch_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                //log::debug!("touch_drawing_gesture motion");

                // filter out invalid stylus input
                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retreive_pointer_elements(touch_drawing_gesture, x, y);
                Element::transform_elements(&mut data_entries, na::Translation2::new(start_point.0, start_point.1) * canvas.engine().borrow().camera.transform().inverse());
                input::process_pen_motion(data_entries, None, &appwindow);
            }
        }));

        self.imp().touch_drawing_gesture.connect_drag_end(
            clone!(@weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
                if let Some(start_point) = touch_drawing_gesture.start_point() {
                    //log::debug!("touch_drawing_gesture end");

                    // filter out invalid stylus input
                    if input::filter_touch_input(touch_drawing_gesture) { return; }

                    let mut data_entries = input::retreive_pointer_elements(touch_drawing_gesture, x, y);
                    Element::transform_elements(&mut data_entries, na::Translation2::new(start_point.0, start_point.1) * canvas.engine().borrow().camera.transform().inverse());

                    if let Some(last) = data_entries.pop_back() {
                        input::process_pen_motion(data_entries, None, &appwindow);
                        input::process_pen_up(last, None, &appwindow);
                    }
                }
            }),
        );

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

    // When the camera offset should change, we call this ( for example from touch drag gestures )
    pub fn update_camera_offset(&self, new_offset: na::Vector2<f64>) {
        self.engine().borrow_mut().camera.offset = new_offset;

        self.hadjustment().unwrap().set_value(new_offset[0]);
        self.vadjustment().unwrap().set_value(new_offset[1]);

        self.engine().borrow_mut().resize_new_offset();

        self.queue_resize();
    }

    /// Centers the view around a coord on the sheet. The coord parameter has the coordinate space of the sheet!
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

    /// Centering the view to the first page
    pub fn return_to_origin_page(&self) {
        let zoom = self.engine().borrow().camera.zoom();

        let new_offset = na::vector![
            ((self.engine().borrow().sheet.format.width / 2.0) * zoom)
                - f64::from(self.parent().unwrap().width()) * 0.5,
            -Sheet::SHADOW_WIDTH * zoom
        ];

        self.update_camera_offset(new_offset);
    }

    /// zooms and regenerates the canvas and its contents to a new zoom
    /// is private, zooming from other parts of the app should always be done through the "zoom-to-value" action
    fn zoom_to(&self, new_zoom: f64) {
        // Remove the timeout if existss
        if let Some(zoom_timeout_id) = self.imp().zoom_timeout_id.take() {
            zoom_timeout_id.remove();
        }

        self.engine().borrow_mut().camera.set_temporary_zoom(1.0);
        self.engine().borrow_mut().camera.set_zoom(new_zoom);

        self.engine()
            .borrow_mut()
            .strokes_state
            .reset_regenerate_flag_all_strokes();

        self.engine().borrow_mut().resize_autoexpand();

        self.update_background_rendernode(true);
        self.regenerate_background(false);
        self.regenerate_content(false, true);
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
        self.engine().borrow_mut().camera.set_temporary_zoom(new_temp_zoom);

        self.update_background_rendernode(true);

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
            ));
    }

    /// Update rendernodes of the background. Used when the background itself did not change, but for example the sheet size or the viewport
    pub fn update_background_rendernode(&self, redraw: bool) {
        let sheet_bounds = self.engine().borrow().sheet.bounds();
        let viewport = self.engine().borrow().camera.viewport();

        if let Err(e) = self
            .engine()
            .borrow_mut()
            .sheet
            .background
            .update_rendernode(sheet_bounds, Some(viewport))
        {
            log::error!("failed to update rendernode for background in update_background_rendernode() with Err {}", e);
        }

        if redraw {
            self.queue_resize();
        }
    }
    /// regenerating the background image and rendernode.
    /// use for example when changing the background pattern or zoom
    pub fn regenerate_background(&self, redraw: bool) {
        let sheet_bounds = self.engine().borrow().sheet.bounds();
        let viewport = self.engine().borrow().camera.viewport();
        let zoom = self.engine().borrow().camera.zoom();

        if let Err(e) = self
            .engine()
            .borrow_mut()
            .sheet
            .background
            .regenerate_background(sheet_bounds, Some(viewport), zoom)
        {
            log::error!("failed to regenerate background, {}", e)
        };

        if redraw {
            self.queue_resize();
        }
    }

    /// regenerate the rendernodes of the canvas content. force_regenerate regenerate all images and rendernodes from scratch. redraw: queue canvas redrawing
    pub fn regenerate_content(&self, force_regenerate: bool, redraw: bool) {
        let zoom = self.engine().borrow().camera.zoom();
        let viewport = self.engine().borrow().camera.viewport();

        self.engine()
            .borrow_mut()
            .strokes_state
            .regenerate_rendering_current_view_threaded(Some(viewport), force_regenerate, zoom);

        if redraw {
            self.queue_resize();
        }
    }

    pub fn show_return_to_center_toast(&self) {
        let return_to_center_toast_is_some = self.imp().return_to_center_toast.borrow().is_some();

        if !return_to_center_toast_is_some {
            let return_to_center_toast = adw::Toast::builder()
                .title(&gettext("Return to origin"))
                .timeout(0)
                .button_label(&gettext("Return"))
                .priority(adw::ToastPriority::Normal)
                .action_name("win.return-origin-page")
                .build();

            return_to_center_toast.connect_dismissed(
                clone!(@weak self as canvas => move |_toast| {
                    canvas.imp().return_to_center_toast.borrow_mut().take();
                }),
            );

            self.ancestor(RnoteAppWindow::static_type())
                .unwrap()
                .downcast_ref::<RnoteAppWindow>()
                .unwrap()
                .toast_overlay()
                .add_toast(&return_to_center_toast);
            *self.imp().return_to_center_toast.borrow_mut() = Some(return_to_center_toast);
        }
    }

    pub fn dismiss_return_to_center_toast(&self) {
        // Avoid already borrowed err
        let return_to_center_toast = self.imp().return_to_center_toast.borrow_mut().take();

        if let Some(return_to_center_toast) = return_to_center_toast {
            return_to_center_toast.dismiss();
        }
    }
}
