mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::debug;
    use crate::config;
    use crate::pens::{PenStyle, Pens};
    use crate::sheet::Sheet;

    use gtk4::{
        gdk, glib, graphene, gsk, prelude::*, subclass::prelude::*, GestureDrag, GestureStylus,
        Orientation, PropagationPhase, SizeRequestMode, Snapshot, Widget, WidgetPaintable,
    };

    use once_cell::sync::Lazy;

    #[derive(Debug)]
    pub struct Canvas {
        pub pens: Rc<RefCell<Pens>>,
        pub current_pen: Rc<Cell<PenStyle>>,
        pub sheet: Sheet,
        pub scalefactor: Cell<f64>,
        pub temporary_zoom: Cell<f64>,
        pub visual_debug: Cell<bool>,
        pub touch_drawing: Cell<bool>,
        pub unsaved_changes: Cell<bool>,
        pub empty: Cell<bool>,
        pub cursor: gdk::Cursor,
        pub motion_cursor: gdk::Cursor,
        pub stylus_drawing_gesture: GestureStylus,
        pub mouse_drawing_gesture: GestureDrag,
        pub touch_drawing_gesture: GestureDrag,
        pub preview: WidgetPaintable,
        pub texture_buffer: RefCell<Option<gdk::Texture>>,
        pub zoom_timeout: RefCell<Option<glib::SourceId>>,
    }

    impl Default for Canvas {
        fn default() -> Self {
            let stylus_drawing_gesture = GestureStylus::builder()
                .name("stylus_drawing_gesture")
                .propagation_phase(PropagationPhase::Target)
                .build();

            // mouse gesture handlers have a guard to not handle emulated pointer events ( e.g. coming from touch input )
            // matching different input methods with gdk4::InputSource or gdk4::DeviceToolType did NOT WORK unfortunately, dont know why
            let mouse_drawing_gesture = GestureDrag::builder()
                .name("mouse_drawing_gesture")
                .button(gdk::BUTTON_PRIMARY)
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            let touch_drawing_gesture = GestureDrag::builder()
                .name("touch_drawing_gesture")
                .touch_only(true)
                .propagation_phase(PropagationPhase::Target)
                .build();

            Self {
                pens: Rc::new(RefCell::new(Pens::default())),
                current_pen: Rc::new(Cell::new(PenStyle::default())),
                sheet: Sheet::default(),
                scalefactor: Cell::new(super::Canvas::SCALE_DEFAULT),
                temporary_zoom: Cell::new(1.0),
                visual_debug: Cell::new(false),
                touch_drawing: Cell::new(false),
                unsaved_changes: Cell::new(false),
                empty: Cell::new(true),
                cursor: gdk::Cursor::from_texture(
                    &gdk::Texture::from_resource(
                        (String::from(config::APP_IDPATH)
                            + "icons/scalable/actions/canvas-cursor.svg")
                            .as_str(),
                    ),
                    8,
                    8,
                    gdk::Cursor::from_name("default", None).as_ref(),
                ),
                motion_cursor: gdk::Cursor::from_texture(
                    &gdk::Texture::from_resource(
                        (String::from(config::APP_IDPATH)
                            + "icons/scalable/actions/canvas-motion-cursor.svg")
                            .as_str(),
                    ),
                    8,
                    8,
                    gdk::Cursor::from_name("default", None).as_ref(),
                ),
                stylus_drawing_gesture,
                mouse_drawing_gesture,
                touch_drawing_gesture,
                preview: WidgetPaintable::new(None as Option<&Widget>),
                texture_buffer: RefCell::new(None),
                zoom_timeout: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Canvas {
        const NAME: &'static str = "Canvas";
        type Type = super::Canvas;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Canvas {
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

            self.preview.set_widget(Some(obj));
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                // The temporary zoom factor (multiplied on top of scalefactor!) which is used when doing zoom gestures ( to avoid strokes redrawing )
                vec![
                    glib::ParamSpec::new_double(
                        "temporary-zoom",
                        "temporary-zoom",
                        "temporary-zoom",
                        f64::MIN,
                        f64::MAX,
                        1.0,
                        glib::ParamFlags::READWRITE,
                    ),
                    // The scalefactor of the canvas in relation to the sheet
                    glib::ParamSpec::new_double(
                        "scalefactor",
                        "scalefactor",
                        "scalefactor",
                        f64::MIN,
                        f64::MAX,
                        super::Canvas::SCALE_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Visual debugging, which shows bounding boxes, hitboxes, ... (enable in developer action menu)
                    glib::ParamSpec::new_boolean(
                        "visual-debug",
                        "visual-debug",
                        "visual-debug",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Wether to enable touch drawing
                    glib::ParamSpec::new_boolean(
                        "touch-drawing",
                        "touch-drawing",
                        "touch-drawing",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Flag for any unsaved changes on the canvas. Propagates to the application 'unsaved-changes' property
                    glib::ParamSpec::new_boolean(
                        "unsaved-changes",
                        "unsaved-changes",
                        "unsaved-changes",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Wether the canvas is empty
                    glib::ParamSpec::new_boolean(
                        "empty",
                        "empty",
                        "empty",
                        true,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "temporary-zoom" => self.temporary_zoom.get().to_value(),
                "scalefactor" => self.scalefactor.get().to_value(),
                "visual-debug" => self.visual_debug.get().to_value(),
                "touch-drawing" => self.touch_drawing.get().to_value(),
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
                "empty" => self.empty.get().to_value(),
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
            match pspec.name() {
                "temporary-zoom" => {
                    let temporary_zoom = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.")
                        .clamp(
                            super::Canvas::SCALE_MIN / self.scalefactor.get(),
                            super::Canvas::SCALE_MAX / self.scalefactor.get(),
                        );
                    self.temporary_zoom.replace(temporary_zoom);
                    obj.queue_resize();
                    obj.queue_draw();
                }
                "scalefactor" => {
                    let scalefactor: f64 = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.")
                        .clamp(super::Canvas::SCALE_MIN, super::Canvas::SCALE_MAX);
                    self.scalefactor.replace(scalefactor);
                    self.sheet.strokes_state().borrow_mut().scalefactor = scalefactor;

                    obj.queue_resize();
                    obj.queue_draw();
                }
                "visual-debug" => {
                    let visual_debug: bool =
                        value.get().expect("The value needs to be of type `bool`.");
                    self.visual_debug.replace(visual_debug);

                    obj.queue_draw();
                }
                "touch-drawing" => {
                    let touch_drawing: bool =
                        value.get().expect("The value needs to be of type `bool`.");
                    self.touch_drawing.replace(touch_drawing);
                    if touch_drawing {
                        self.touch_drawing_gesture
                            .set_propagation_phase(PropagationPhase::Target);
                    } else {
                        self.touch_drawing_gesture
                            .set_propagation_phase(PropagationPhase::None);
                    }
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
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Canvas {
        fn request_mode(&self, _widget: &Self::Type) -> SizeRequestMode {
            SizeRequestMode::ConstantSize
        }

        fn measure(
            &self,
            _widget: &Self::Type,
            orientation: Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            if orientation == Orientation::Vertical {
                let minimal_height = (f64::from(self.sheet.height())
                    * self.scalefactor.get()
                    * self.temporary_zoom.get()
                    + f64::from(self.sheet.y()))
                .round() as i32;
                let natural_height = minimal_height;

                (minimal_height, natural_height, -1, -1)
            } else {
                let minimal_width = (f64::from(self.sheet.width())
                    * self.scalefactor.get()
                    * self.temporary_zoom.get()
                    + f64::from(self.sheet.x()))
                .round() as i32;
                let natural_width = minimal_width;

                (minimal_width, natural_width, -1, -1)
            }
        }

        fn snapshot(&self, _widget: &Self::Type, snapshot: &gtk4::Snapshot) {
            let temporary_zoom = self.temporary_zoom.get();
            let scalefactor = self.scalefactor.get();

            let sheet_bounds_scaled = graphene::Rect::new(
                self.sheet.x() as f32 * scalefactor as f32,
                self.sheet.y() as f32 * scalefactor as f32,
                self.sheet.width() as f32 * scalefactor as f32,
                self.sheet.height() as f32 * scalefactor as f32,
            );

            if let Some(texture_buffer) = &*self.texture_buffer.borrow() {
                snapshot.scale(temporary_zoom as f32, temporary_zoom as f32);
                snapshot.append_texture(texture_buffer, &sheet_bounds_scaled);
            } else {
                self.draw_shadow(
                    &sheet_bounds_scaled,
                    Self::SHADOW_WIDTH * scalefactor,
                    snapshot,
                );

                let sheet_bounds_scaled = graphene::Rect::new(
                    self.sheet.x() as f32 * scalefactor as f32,
                    self.sheet.y() as f32 * scalefactor as f32,
                    self.sheet.width() as f32 * scalefactor as f32,
                    self.sheet.height() as f32 * scalefactor as f32,
                );

                // Clip sheet and stroke drawing to sheet bounds
                snapshot.push_clip(&sheet_bounds_scaled);

                self.sheet.draw(scalefactor, snapshot);

                self.sheet.strokes_state().borrow().draw_strokes(snapshot);

                snapshot.pop();

                self.sheet
                    .strokes_state()
                    .borrow()
                    .draw_selection(scalefactor, snapshot);

                self.pens
                    .borrow()
                    .draw(self.current_pen.get(), snapshot, scalefactor);

                if self.sheet.format_borders() {
                    self.sheet
                        .format()
                        .draw(self.sheet.bounds(), snapshot, scalefactor);
                }

                if self.visual_debug.get() {
                    self.draw_debug(snapshot);
                }
            }
        }
    }

    impl Canvas {
        pub const SHADOW_WIDTH: f64 = 30.0;

        pub fn draw_shadow(&self, bounds: &graphene::Rect, width: f64, snapshot: &Snapshot) {
            let shadow_color = gdk::RGBA {
                red: 0.1,
                green: 0.1,
                blue: 0.1,
                alpha: 0.3,
            };
            let corner_radius = graphene::Size::new(width as f32 / 4.0, width as f32 / 4.0);

            let rounded_rect = gsk::RoundedRect::new(
                bounds.clone(),
                corner_radius.clone(),
                corner_radius.clone(),
                corner_radius.clone(),
                corner_radius,
            );

            snapshot.append_outset_shadow(
                &rounded_rect,
                &shadow_color,
                0.0,
                0.0,
                width as f32,
                width as f32,
            );
        }

        // Draw bounds, positions, .. for visual debugging purposes
        fn draw_debug(&self, snapshot: &Snapshot) {
            let scalefactor = self.scalefactor.get();

            match self.current_pen.get() {
                PenStyle::Eraser => {
                    if self.pens.borrow().eraser.shown() {
                        if let Some(ref current_input) = self.pens.borrow().eraser.current_input {
                            debug::draw_pos(
                                current_input.pos(),
                                debug::COLOR_POS_ALT,
                                scalefactor,
                                snapshot,
                            );
                        }
                    }
                }
                PenStyle::Selector => {
                    if self.pens.borrow().selector.shown() {
                        if let Some(bounds) = self.pens.borrow().selector.bounds {
                            debug::draw_bounds(
                                bounds,
                                debug::COLOR_SELECTOR_BOUNDS,
                                scalefactor,
                                snapshot,
                            );
                        }
                    }
                }
                PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper | PenStyle::Unkown => {}
            }

            debug::draw_bounds(
                p2d::bounding_volume::AABB::new(
                    na::point![0.0, 0.0],
                    na::point![
                        f64::from(self.sheet.width()),
                        f64::from(self.sheet.height())
                    ],
                ),
                debug::COLOR_SHEET_BOUNDS,
                scalefactor,
                snapshot,
            );

            self.sheet
                .strokes_state()
                .borrow()
                .draw_debug(scalefactor, snapshot);
        }
    }
}

use crate::strokes::strokestyle::{Element, InputData};
use crate::{
    app::RnoteApp, pens::PenStyle, pens::Pens, render, sheet::Sheet, ui::appwindow::RnoteAppWindow,
};
use crate::{geometry, input};

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time;

use gtk4::{gdk, glib, glib::clone, prelude::*, subclass::prelude::*, WidgetPaintable};
use gtk4::{EventSequenceState, Snapshot, Widget};

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<imp::Canvas>)
        @extends gtk4::Widget;
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    pub const SCALE_MIN: f64 = 0.1;
    pub const SCALE_MAX: f64 = 5.0;
    pub const SCALE_DEFAULT: f64 = 1.0;
    pub const ZOOM_ACTION_DELTA: f64 = 0.1;
    pub const ZOOM_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(300);
    pub const INPUT_OVERSHOOT: f64 = 30.0;
    pub const SHADOW_WIDTH: f64 = 30.0;

    pub fn new() -> Self {
        let canvas: Canvas = glib::Object::new(&[]).expect("Failed to create Canvas");

        canvas
    }

    pub fn current_pen(&self) -> Rc<Cell<PenStyle>> {
        let priv_ = imp::Canvas::from_instance(self);
        priv_.current_pen.clone()
    }

    pub fn pens(&self) -> Rc<RefCell<Pens>> {
        let priv_ = imp::Canvas::from_instance(self);
        priv_.pens.clone()
    }

    pub fn cursor(&self) -> gdk::Cursor {
        let priv_ = imp::Canvas::from_instance(self);
        priv_.cursor.clone()
    }

    pub fn motion_cursor(&self) -> gdk::Cursor {
        let priv_ = imp::Canvas::from_instance(self);
        priv_.motion_cursor.clone()
    }

    pub fn sheet(&self) -> Sheet {
        imp::Canvas::from_instance(self).sheet.clone()
    }

    pub fn temporary_zoom(&self) -> f64 {
        self.property("temporary-zoom")
            .unwrap()
            .get::<f64>()
            .unwrap()
    }

    fn set_temporary_zoom(&self, temporary_zoom: f64) {
        self.set_property("temporary-zoom", temporary_zoom.to_value())
            .unwrap();
    }

    pub fn scalefactor(&self) -> f64 {
        self.property("scalefactor").unwrap().get::<f64>().unwrap()
    }

    fn set_scalefactor(&self, scalefactor: f64) {
        self.set_property("scalefactor", scalefactor.to_value())
            .unwrap();
    }

    pub fn preview(&self) -> WidgetPaintable {
        imp::Canvas::from_instance(self).preview.clone()
    }

    pub fn unsaved_changes(&self) -> bool {
        self.property("unsaved-changes")
            .unwrap()
            .get::<bool>()
            .unwrap()
    }

    pub fn set_unsaved_changes(&self, unsaved_changes: bool) {
        match self.set_property("unsaved-changes", unsaved_changes.to_value()) {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "failed to set property `unsaved-changes` of `Canvas`, {}",
                    e
                )
            }
        }
    }

    pub fn empty(&self) -> bool {
        self.property("empty").unwrap().get::<bool>().unwrap()
    }

    pub fn set_empty(&self, empty: bool) {
        match self.set_property("empty", empty.to_value()) {
            Ok(_) => {}
            Err(e) => {
                log::error!("failed to set property `empty` of `Canvas`, {}", e)
            }
        }
    }

    /// The bounds of the sheet scaled to the current canvas scalefactor
    pub fn sheet_bounds_scaled(&self) -> p2d::bounding_volume::AABB {
        let scalefactor = self.scalefactor();

        p2d::bounding_volume::AABB::new(
            na::point![
                f64::from(self.sheet().x()) * scalefactor,
                f64::from(self.sheet().y()) * scalefactor
            ],
            na::point![
                f64::from(self.width()) * scalefactor,
                f64::from(self.height()) * scalefactor
            ],
        )
    }

    // The bounds of the canvas
    pub fn bounds(&self) -> p2d::bounding_volume::AABB {
        p2d::bounding_volume::AABB::new(
            na::point![f64::from(0.0), f64::from(0.0)],
            na::point![f64::from(self.width()), f64::from(self.height())],
        )
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::Canvas::from_instance(self);

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

        self.bind_property(
            "scalefactor",
            &appwindow.mainheader().canvasmenu().zoomreset_button(),
            "label",
        )
        .transform_to(|_, value| {
            let scalefactor = value.get::<f64>().unwrap();
            Some(format!("{:.0}%", scalefactor * 100.0).to_value())
        })
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

        // Mouse drawing
        priv_.mouse_drawing_gesture.connect_drag_begin(
            clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
                if let Some(event) = mouse_drawing_gesture.current_event() {
                    // Guard not to handle touch events that emulate a pointer
                    if event.is_pointer_emulated() {
                        return;
                    }

                    mouse_drawing_gesture.set_state(EventSequenceState::Claimed);

                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![0.0, 0.0]);

                    canvas.processing_draw_begin(&appwindow, &mut data_entries);
                }
            }),
        );

        priv_.mouse_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            if let Some(event) = mouse_drawing_gesture.current_event() {
                // Guard not to handle touch events that emulate a pointer
                if event.is_pointer_emulated() {
                    return;
                }

                if let Some(start_point) = mouse_drawing_gesture.start_point() {
                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![start_point.0, start_point.1]);

                    canvas.processing_draw_motion(&appwindow, &mut data_entries);
                }
            }
        }));

        priv_.mouse_drawing_gesture.connect_drag_end(
            clone!(@weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {
                if let Some(event) = mouse_drawing_gesture.current_event() {
                    // Guard not to handle touch events that emulate a pointer
                    if event.is_pointer_emulated() {
                        return;
                    }

                    if let Some(start_point) = mouse_drawing_gesture.start_point() {
                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![start_point.0, start_point.1]);

                    canvas.processing_draw_end(&appwindow, &mut data_entries);
                    }
                }
            }),
        );

        // Stylus Drawing
        priv_.stylus_drawing_gesture.connect_down(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            stylus_drawing_gesture.set_state(EventSequenceState::Claimed);
            if let Some(device_tool) = stylus_drawing_gesture.device_tool() {

                // Disable backlog, only allowed in motion signal handler
                let mut data_entries = input::retreive_stylus_inputdata(stylus_drawing_gesture, false, x, y);
                input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![0.0, 0.0]);


                match device_tool.tool_type() {
                    gdk::DeviceToolType::Pen => { },
                    gdk::DeviceToolType::Eraser => {
                    appwindow.downcast_ref::<RnoteAppWindow>().unwrap()
                        .change_action_state("tmperaser", &true.to_variant());
                    }
                    _ => { canvas.current_pen().set(PenStyle::Unkown) },
                }

                canvas.processing_draw_begin(&appwindow, &mut data_entries);
            }
        }));

        priv_.stylus_drawing_gesture.connect_motion(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            if stylus_drawing_gesture.device_tool().is_some() {
                // backlog doesn't provide time equidistant inputdata and makes line look worse, so its disabled for now
                let mut data_entries: VecDeque<InputData> = input::retreive_stylus_inputdata(stylus_drawing_gesture, false, x, y);
                input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![0.0, 0.0]);

                canvas.processing_draw_motion(&appwindow, &mut data_entries);
            }
        }));

        priv_.stylus_drawing_gesture.connect_up(
            clone!(@weak self as canvas, @weak appwindow => move |gesture_stylus,x,y| {
                let mut data_entries = input::retreive_stylus_inputdata(gesture_stylus, false, x, y);

                input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![0.0, 0.0]);
                canvas.processing_draw_end(&appwindow, &mut data_entries);
            }),
        );

        // Touch drawing
        priv_.touch_drawing_gesture.connect_drag_begin(
            clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
                touch_drawing_gesture.set_state(EventSequenceState::Claimed);

                let mut data_entries = input::retreive_pointer_inputdata(x, y);

                input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![0.0, 0.0]);
                canvas.processing_draw_begin(&appwindow, &mut data_entries);
            }),
        );

        priv_.touch_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_inputdata(x, y);
                input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![start_point.0, start_point.1]);

                canvas.processing_draw_motion(&appwindow, &mut data_entries);
            }
        }));

        priv_.touch_drawing_gesture.connect_drag_end(
            clone!(@weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
                if let Some(start_point) = touch_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_inputdata(x, y);
                input::map_inputdata(canvas.scalefactor(), &mut data_entries, na::vector![start_point.0, start_point.1]);

                canvas.processing_draw_end(&appwindow, &mut data_entries);
                }
            }),
        );
    }

    /// Zoom temporarily to a new scalefactor, not rescaling the contents while doing it.
    /// To scale the content and reset the zoom, use scale_to().
    pub fn zoom_temporarily_to(&self, temp_scalefactor: f64) {
        let priv_ = imp::Canvas::from_instance(self);

        // Only capture when texture_buffer is resetted (= None)
        if priv_.texture_buffer.borrow().is_none() {
            *priv_.texture_buffer.borrow_mut() = self.current_content_as_texture(na::vector![
                f64::from(self.width()),
                f64::from(self.height())
            ]);
        }
        self.set_temporary_zoom(temp_scalefactor / self.scalefactor());
    }

    /// Scales the canvas and its contents to a new scalefactor
    pub fn scale_to(&self, scalefactor: f64) {
        let priv_ = imp::Canvas::from_instance(self);

        /*         if let Some(texture_buffer) = &*priv_.texture_buffer.borrow() {
            texture_buffer.save_to_png(Path::new("./tests/canvas.png"));
        } */

        *priv_.texture_buffer.borrow_mut() = None;
        self.set_temporary_zoom(1.0);
        self.set_scalefactor(scalefactor);

        // regenerating bounds, hitboxes,..
        self.sheet()
            .strokes_state()
            .borrow_mut()
            .complete_all_strokes();

        self.regenerate_content(false, true);
    }

    /// Zooms temporarily and then scale the canvas and its contents to a new scalefactor after a given time.
    /// Repeated calls to this function reset the timeout.
    pub fn zoom_temporarily_then_scale_to_after_timeout(
        &self,
        scalefactor: f64,
        timeout_time: time::Duration,
    ) {
        let priv_ = imp::Canvas::from_instance(self);

        if let Some(zoom_timeout) = priv_.zoom_timeout.take() {
            glib::source::source_remove(zoom_timeout);
        }

        self.zoom_temporarily_to(scalefactor);

        priv_
            .zoom_timeout
            .borrow_mut()
            .replace(glib::source::timeout_add_local_once(
                timeout_time,
                clone!(@weak self as canvas => move || {
                    let priv_ = imp::Canvas::from_instance(&canvas);

                    canvas.scale_to(scalefactor);
                    priv_.zoom_timeout.borrow_mut().take();
                }),
            ));
    }

    /// regenerating the background rendernodes.
    /// use force_regenerate to force regeneration of the texture_cache of the background (for example when changing the background pattern)
    pub fn regenerate_background(&self, force_regenerate: bool, redraw: bool) {
        match self.sheet().background().borrow_mut().update_rendernode(
            self.scalefactor(),
            self.sheet().bounds(),
            force_regenerate,
        ) {
            Err(e) => {
                log::error!("failed to regenerate background, {}", e)
            }
            Ok(_) => {}
        }
        if redraw {
            self.queue_resize();
            self.queue_draw();
        }
    }

    /// regenerate the rendernodes of the canvas content. force_regenerate  regenerates all rendernodes from scratch
    pub fn regenerate_content(&self, force_regenerate: bool, redraw: bool) {
        self.sheet().strokes_state().borrow_mut().update_rendering();

        self.regenerate_background(force_regenerate, redraw);
    }

    /// Captures the current content of the canvas as a gdk::Texture
    pub fn current_content_as_texture(&self, size: na::Vector2<f64>) -> Option<gdk::Texture> {
        let snapshot = Snapshot::new();
        self.preview().snapshot(
            snapshot.dynamic_cast_ref::<gdk::Snapshot>().unwrap(),
            size[0],
            size[1],
        );

        if let Some(node) = snapshot.to_node() {
            render::rendernode_to_texture(self.upcast_ref::<Widget>(), &node, self.bounds())
                .unwrap_or_else(|e| {
                    log::error!("{}", e);
                    None
                })
        } else {
            None
        }
    }

    /// Process the beginning of a stroke drawing
    fn processing_draw_begin(
        &self,
        appwindow: &RnoteAppWindow,
        data_entries: &mut VecDeque<InputData>,
    ) {
        let priv_ = imp::Canvas::from_instance(self);

        let scalefactor = self.scalefactor();

        self.set_unsaved_changes(true);
        self.set_empty(false);
        self.sheet().strokes_state().borrow_mut().deselect();
        appwindow.selection_modifier().set_visible(false);

        match self.current_pen().get() {
            PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper => {
                self.set_cursor(Some(&self.motion_cursor()));

                let filter_bounds = p2d::bounding_volume::AABB::new(
                    na::point![
                        priv_.sheet.x() as f64 - Self::INPUT_OVERSHOOT,
                        priv_.sheet.y() as f64 - Self::INPUT_OVERSHOOT
                    ],
                    na::point![
                        (priv_.sheet.x() + priv_.sheet.width()) as f64 + Self::INPUT_OVERSHOOT,
                        (priv_.sheet.y() + priv_.sheet.height()) as f64 + Self::INPUT_OVERSHOOT
                    ],
                );
                input::filter_mapped_inputdata(filter_bounds, data_entries);

                if let Some(inputdata) = data_entries.pop_back() {
                    self.sheet().strokes_state().borrow_mut().new_stroke(
                        Element::new(inputdata),
                        self.current_pen().get(),
                        &self.pens().borrow(),
                    );

                    self.sheet()
                        .strokes_state()
                        .borrow_mut()
                        .update_rendering_newest_stroke();
                }
            }
            PenStyle::Eraser => {
                if let Some(inputdata) = data_entries.pop_back() {
                    self.set_cursor(gdk::Cursor::from_name("none", None).as_ref());
                    self.pens().borrow_mut().eraser.current_input = Some(inputdata);
                    self.pens().borrow_mut().eraser.set_shown(true);
                }
            }
            PenStyle::Selector => {
                if let Some(inputdata) = data_entries.pop_back() {
                    self.set_cursor(gdk::Cursor::from_name("cell", None).as_ref());

                    self.pens().borrow_mut().selector.new_path(inputdata);
                    self.pens().borrow_mut().selector.set_shown(true);

                    // update the rendernode of the current stroke
                    self.pens().borrow_mut().selector.update_rendernode(
                        scalefactor,
                        &self.sheet().strokes_state().borrow().renderer,
                    );
                }
            }
            PenStyle::Unkown => {}
        }

        self.queue_draw();
    }

    /// Process the motion of a strokes drawing
    fn processing_draw_motion(
        &self,
        appwindow: &RnoteAppWindow,
        data_entries: &mut VecDeque<InputData>,
    ) {
        let priv_ = imp::Canvas::from_instance(self);

        let scalefactor = self.scalefactor();

        match self.current_pen().get() {
            PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper => {
                let filter_bounds = p2d::bounding_volume::AABB::new(
                    na::point![
                        priv_.sheet.x() as f64 - Self::INPUT_OVERSHOOT,
                        priv_.sheet.y() as f64 - Self::INPUT_OVERSHOOT
                    ],
                    na::point![
                        (priv_.sheet.x() + priv_.sheet.width()) as f64 + Self::INPUT_OVERSHOOT,
                        (priv_.sheet.y() + priv_.sheet.height()) as f64 + Self::INPUT_OVERSHOOT
                    ],
                );
                input::filter_mapped_inputdata(filter_bounds, data_entries);

                for inputdata in data_entries {
                    self.sheet()
                        .strokes_state()
                        .borrow_mut()
                        .add_to_last_stroke(Element::new(inputdata.clone()), &self.pens().borrow());

                    self.queue_draw();
                }
            }
            PenStyle::Eraser => {
                let canvas_scroller_viewport_descaled =
                    if let Some(viewport) = appwindow.canvas_scroller_viewport() {
                        Some(geometry::aabb_scale(viewport, 1.0 / scalefactor))
                    } else {
                        None
                    };

                if let Some(inputdata) = data_entries.pop_back() {
                    self.pens().borrow_mut().eraser.current_input = Some(inputdata);

                    self.sheet()
                        .strokes_state()
                        .borrow_mut()
                        .trash_colliding_strokes(
                            &self.pens().borrow().eraser,
                            canvas_scroller_viewport_descaled,
                        );
                    if self.sheet().resize_endless() {
                        self.regenerate_background(false, false);
                    }
                    self.queue_draw();
                }
            }
            PenStyle::Selector => {
                for inputdata in data_entries {
                    self.pens()
                        .borrow_mut()
                        .selector
                        .push_elem(inputdata.clone());
                    self.pens().borrow_mut().selector.update_rendernode(
                        scalefactor,
                        &self.sheet().strokes_state().borrow().renderer,
                    );
                    self.queue_draw();
                }
            }
            PenStyle::Unkown => {}
        }
    }

    /// Process the end of a strokes drawing
    fn processing_draw_end(
        &self,
        appwindow: &RnoteAppWindow,
        _data_entries: &mut VecDeque<InputData>,
    ) {
        let scalefactor = self.scalefactor();

        self.set_cursor(Some(&self.cursor()));

        appwindow
            .downcast_ref::<RnoteAppWindow>()
            .unwrap()
            .change_action_state("tmperaser", &false.to_variant());

        // complete the last stroke
        let last_key = self.sheet().strokes_state().borrow().last_stroke_key();
        if let Some(last_key) = last_key {
            self.sheet()
                .strokes_state()
                .borrow_mut()
                .complete_stroke(last_key);
        }

        match self.current_pen().get() {
            PenStyle::Selector => {
                let canvas_scroller_viewport_descaled =
                    if let Some(viewport) = appwindow.canvas_scroller_viewport() {
                        Some(geometry::aabb_scale(viewport, 1.0 / scalefactor))
                    } else {
                        None
                    };

                self.sheet()
                    .strokes_state()
                    .borrow_mut()
                    .update_selection_for_selector(
                        &self.pens().borrow().selector,
                        canvas_scroller_viewport_descaled,
                    );

                // Show the selection modifier if selection bounds are some
                appwindow.selection_modifier().set_visible(
                    self.sheet()
                        .strokes_state()
                        .borrow()
                        .selection_bounds
                        .is_some(),
                );
            }
            PenStyle::Marker
            | PenStyle::Brush
            | PenStyle::Shaper
            | PenStyle::Eraser
            | PenStyle::Unkown => {}
        }

        self.pens().borrow_mut().eraser.set_shown(false);
        self.pens().borrow_mut().selector.set_shown(false);
        self.pens().borrow_mut().selector.clear_path();

        if self.sheet().resize_endless() {
            self.regenerate_background(false, false);
        }
        self.queue_resize();
        self.queue_draw();
    }
}

/// fmodule for visual debugging
pub mod debug {
    use gtk4::{gdk, graphene, gsk, Snapshot};

    pub const COLOR_POS: gdk::RGBA = gdk::RGBA {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
    pub const COLOR_POS_ALT: gdk::RGBA = gdk::RGBA {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
        alpha: 1.0,
    };
    pub const COLOR_STROKE_HITBOX: gdk::RGBA = gdk::RGBA {
        red: 0.0,
        green: 0.8,
        blue: 0.2,
        alpha: 0.7,
    };
    pub const COLOR_STROKE_BOUNDS: gdk::RGBA = gdk::RGBA {
        red: 0.0,
        green: 0.8,
        blue: 0.8,
        alpha: 1.0,
    };
    pub const COLOR_SELECTOR_BOUNDS: gdk::RGBA = gdk::RGBA {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
    pub const COLOR_SHEET_BOUNDS: gdk::RGBA = gdk::RGBA {
        red: 0.8,
        green: 0.0,
        blue: 0.8,
        alpha: 1.0,
    };

    pub fn draw_bounds(
        bounds: p2d::bounding_volume::AABB,
        color: gdk::RGBA,
        scalefactor: f64,
        snapshot: &Snapshot,
    ) {
        let bounds = graphene::Rect::new(
            bounds.mins[0] as f32,
            bounds.mins[1] as f32,
            (bounds.maxs[0] - bounds.mins[0]) as f32,
            (bounds.maxs[1] - bounds.mins[1]) as f32,
        );

        let border_width = 1.5;
        let rounded_rect = gsk::RoundedRect::new(
            bounds.scale(scalefactor as f32, scalefactor as f32),
            graphene::Size::zero(),
            graphene::Size::zero(),
            graphene::Size::zero(),
            graphene::Size::zero(),
        );

        snapshot.append_border(
            &rounded_rect,
            &[border_width, border_width, border_width, border_width],
            &[color, color, color, color],
        )
    }

    pub fn draw_pos(
        pos: na::Vector2<f64>,
        color: gdk::RGBA,
        scalefactor: f64,
        snapshot: &Snapshot,
    ) {
        snapshot.append_color(
            &color,
            &graphene::Rect::new(
                (scalefactor * pos[0] - 1.0) as f32,
                (scalefactor * pos[1] - 1.0) as f32,
                2.0,
                2.0,
            ),
        );
    }
}
