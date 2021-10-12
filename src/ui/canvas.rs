mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::debug;
    use crate::config;
    use crate::pens::{PenStyle, Pens};
    use crate::strokes::{render, StrokeStyle};
    use crate::{sheet::Sheet, strokes};

    use gtk4::Widget;
    use gtk4::{
        gdk, glib, graphene, gsk, prelude::*, subclass::prelude::*, GestureDrag, GestureStylus,
        Orientation, PropagationPhase, SizeRequestMode, Snapshot, WidgetPaintable,
    };

    use once_cell::sync::Lazy;

    #[derive(Debug)]
    pub struct Canvas {
        pub pens: Rc<RefCell<Pens>>,         // accessed via pens()
        pub current_pen: Rc<Cell<PenStyle>>, // accessed via current_pen()
        pub sheet: Sheet,                    // is a GObject
        pub scalefactor: Cell<f64>,          // is a property
        pub visual_debug: Cell<bool>,        // is a property
        pub touch_drawing: Cell<bool>,       // is a property
        pub unsaved_changes: Cell<bool>,     // is a property
        pub cursor: gdk::Cursor,             // is a property
        pub stylus_drawing_gesture: GestureStylus,
        pub mouse_drawing_gesture: GestureDrag,
        pub touch_drawing_gesture: GestureDrag,
        pub renderer: Rc<RefCell<render::Renderer>>,
        pub preview: WidgetPaintable,
    }

    impl Default for Canvas {
        fn default() -> Self {
            let stylus_drawing_gesture = GestureStylus::builder()
                .name("stylus_drawing_gesture")
                .n_points(2)
                .propagation_phase(PropagationPhase::Target)
                .build();

            // mouse gesture handlers have a guard to not handle emulated pointer events ( e.g. coming from touch input )
            // matching different input methods with gdk4::InputSource or gdk4::DeviceToolType did NOT WORK unfortunately, IDK why
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
                visual_debug: Cell::new(false),
                touch_drawing: Cell::new(false),
                unsaved_changes: Cell::new(false),
                cursor: gdk::Cursor::from_texture(
                    &gdk::Texture::from_resource(
                        (String::from(config::APP_IDPATH)
                            + "icons/scalable/actions/canvas-cursor-symbolic.svg")
                            .as_str(),
                    ),
                    8,
                    8,
                    gdk::Cursor::from_name("default", None).as_ref(),
                ),
                stylus_drawing_gesture,
                mouse_drawing_gesture,
                touch_drawing_gesture,
                renderer: Rc::new(RefCell::new(render::Renderer::default())),
                preview: WidgetPaintable::new::<Widget>(None),
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
            obj.set_cursor(Some(&self.cursor));

            obj.add_controller(&self.stylus_drawing_gesture);
            obj.add_controller(&self.mouse_drawing_gesture);
            obj.add_controller(&self.touch_drawing_gesture);

            obj.preview().set_widget(Some(obj));
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_double(
                        // Name
                        "scalefactor",
                        // Nickname
                        "scalefactor",
                        // Short description
                        "scalefactor",
                        // Minimum value
                        f64::MIN,
                        // Maximum value
                        f64::MAX,
                        // Default value
                        super::Canvas::SCALE_DEFAULT,
                        // The property can be read and written to
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "visual-debug",
                        "visual-debug",
                        "visual-debug",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "touch-drawing",
                        "touch-drawing",
                        "touch-drawing",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "unsaved-changes",
                        "unsaved-changes",
                        "unsaved-changes",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "scalefactor" => {
                    let scalefactor: f64 = value
                        .get::<f64>()
                        .expect("The value needs to be of type `i32`.")
                        .clamp(super::Canvas::SCALE_MIN, super::Canvas::SCALE_MAX);
                    self.scalefactor.replace(scalefactor);

                    StrokeStyle::update_all_rendernodes(
                        &mut *obj.sheet().strokes().borrow_mut(),
                        scalefactor,
                        &*obj.renderer().borrow(),
                    );
                    StrokeStyle::update_all_rendernodes(
                        &mut *obj.sheet().selection().strokes().borrow_mut(),
                        scalefactor,
                        &*obj.renderer().borrow(),
                    );

                    obj.queue_draw();
                    obj.queue_resize();
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "scalefactor" => self.scalefactor.get().to_value(),
                "visual-debug" => self.visual_debug.get().to_value(),
                "touch-drawing" => self.touch_drawing.get().to_value(),
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
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
            widget: &Self::Type,
            orientation: Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            if orientation == Orientation::Vertical {
                let minimum_size = (f64::from(widget.sheet().height()) * self.scalefactor.get()
                    + f64::from(widget.sheet().y()))
                .round() as i32;
                let natural_size = minimum_size;

                (minimum_size, natural_size, -1, -1)
            } else {
                let minimum_size = (f64::from(widget.sheet().width()) * self.scalefactor.get()
                    + f64::from(widget.sheet().x()))
                .round() as i32;
                let natural_size = minimum_size;

                (minimum_size, natural_size, -1, -1)
            }
        }

        fn snapshot(&self, _widget: &Self::Type, snapshot: &gtk4::Snapshot) {
            let scalefactor = self.scalefactor.get();

            let sheet_bounds_scaled = graphene::Rect::new(
                self.sheet.x() as f32 * scalefactor as f32,
                self.sheet.y() as f32 * scalefactor as f32,
                self.sheet.width() as f32 * scalefactor as f32,
                self.sheet.height() as f32 * scalefactor as f32,
            );

            self.draw_shadow(
                &sheet_bounds_scaled,
                Self::SHADOW_WIDTH * scalefactor,
                snapshot,
            );

            self.sheet.draw(scalefactor, snapshot);

            self.sheet.selection().draw(scalefactor, snapshot);

            self.pens
                .borrow()
                .draw_pens(self.current_pen.get(), snapshot, scalefactor);

            self.draw_debug(snapshot);
        }
    }

    impl Canvas {
        pub const SHADOW_WIDTH: f64 = 15.0;

        pub fn draw_shadow(&self, bounds: &graphene::Rect, width: f64, snapshot: &Snapshot) {
            let shadow_color = gdk::RGBA {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.5,
            };
            let corner_radius = graphene::Size::new(width as f32, width as f32);

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

        fn draw_debug(&self, snapshot: &Snapshot) {
            if self.visual_debug.get() {
                let scalefactor = self.scalefactor.get();

                match self.current_pen.get() {
                    PenStyle::Eraser => {
                        if self.pens.borrow().eraser.shown() {
                            debug::draw_pos(
                                self.pens.borrow().eraser.current_input.pos(),
                                debug::COLOR_POS_ALT,
                                scalefactor,
                                snapshot,
                            );
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

                for stroke in self.sheet.strokes().borrow().iter() {
                    match stroke {
                        strokes::StrokeStyle::MarkerStroke(markerstroke) => {
                            for element in markerstroke.elements.iter() {
                                debug::draw_pos(
                                    element.inputdata().pos(),
                                    debug::COLOR_POS,
                                    scalefactor,
                                    snapshot,
                                )
                            }
                            for &hitbox_elem in markerstroke.hitbox.iter() {
                                debug::draw_bounds(
                                    hitbox_elem,
                                    debug::COLOR_STROKE_HITBOX,
                                    scalefactor,
                                    snapshot,
                                );
                            }
                            debug::draw_bounds(
                                markerstroke.bounds,
                                debug::COLOR_STROKE_BOUNDS,
                                scalefactor,
                                snapshot,
                            );
                        }
                        strokes::StrokeStyle::BrushStroke(brushstroke) => {
                            for element in brushstroke.elements.iter() {
                                debug::draw_pos(
                                    element.inputdata().pos(),
                                    debug::COLOR_POS,
                                    scalefactor,
                                    snapshot,
                                )
                            }
                            for &hitbox_elem in brushstroke.hitbox.iter() {
                                debug::draw_bounds(
                                    hitbox_elem,
                                    debug::COLOR_STROKE_HITBOX,
                                    scalefactor,
                                    snapshot,
                                );
                            }
                            debug::draw_bounds(
                                brushstroke.bounds,
                                debug::COLOR_STROKE_BOUNDS,
                                scalefactor,
                                snapshot,
                            );
                        }
                        strokes::StrokeStyle::ShapeStroke(shapestroke) => {
                            debug::draw_bounds(
                                shapestroke.bounds,
                                debug::COLOR_STROKE_BOUNDS,
                                scalefactor,
                                snapshot,
                            );
                        }
                        strokes::StrokeStyle::VectorImage(vectorimage) => {
                            debug::draw_bounds(
                                vectorimage.bounds,
                                debug::COLOR_STROKE_BOUNDS,
                                scalefactor,
                                snapshot,
                            );
                        }
                        strokes::StrokeStyle::BitmapImage(bitmapimage) => {
                            debug::draw_bounds(
                                bitmapimage.bounds,
                                debug::COLOR_STROKE_BOUNDS,
                                scalefactor,
                                snapshot,
                            );
                        }
                    }
                }
            }
        }
    }
}

use crate::strokes::{render, Element, StrokeStyle};
use crate::utils;
use crate::{
    app::RnoteApp, pens::PenStyle, pens::Pens, sheet::Sheet, strokes::InputData,
    strokes::StrokeBehaviour, ui::appwindow::RnoteAppWindow,
};

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use gtk4::EventSequenceState;
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, GestureStylus, WidgetPaintable,
};

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
    pub const SCALE_MAX: f64 = 10.0;
    pub const SCALE_DEFAULT: f64 = 1.0;
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

    pub fn sheet(&self) -> Sheet {
        imp::Canvas::from_instance(self).sheet.clone()
    }

    pub fn scalefactor(&self) -> f64 {
        self.property("scalefactor").unwrap().get::<f64>().unwrap()
    }

    pub fn set_scalefactor(&self, scalefactor: f64) {
        match self.set_property("scalefactor", scalefactor.to_value()) {
            Ok(_) => {}
            Err(e) => {
                log::error!("failed to set property `scalefactor` of `Canvas`, {}", e)
            }
        }
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

    pub fn renderer(&self) -> Rc<RefCell<render::Renderer>> {
        imp::Canvas::from_instance(self).renderer.clone()
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

        self.bind_property(
            "scalefactor",
            &appwindow.selection_modifier(),
            "scalefactor",
        )
        .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
        .build();

        self.sheet()
            .selection()
            .connect_local(
                "redraw",
                false,
                clone!(@weak self as obj => @default-return None, move |_| {
                    let scalefactor = obj.property("scalefactor").unwrap().get::<f64>().unwrap();

                    StrokeStyle::update_all_rendernodes(
                        &mut *obj.sheet().selection().strokes().borrow_mut(),
                        scalefactor,
                        &*obj.renderer().borrow(),
                    );

                    obj.queue_draw();
                    None
                }),
            )
            .unwrap();

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
                // Guard not to handle touch events that emulate a pointer
                if let Some(event) = mouse_drawing_gesture.current_event() {
                    if event.is_pointer_emulated() {
                        return;
                    }
                }
                mouse_drawing_gesture.set_state(EventSequenceState::Claimed);

                let data_entries = Self::retreive_pointer_inputdata(x, y);
                let data_entries = canvas.map_inputdata(data_entries, na::vector![0.0, 0.0]);

                canvas.processing_draw_begin(&appwindow, data_entries);
            }),
        );

        priv_.mouse_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            // Guard not to handle touch events that emulate a pointer
            if let Some(event) = mouse_drawing_gesture.current_event() {
                if event.is_pointer_emulated() {
                    return;
                }
            }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let data_entries = Self::retreive_pointer_inputdata(x, y);
                let data_entries = canvas.map_inputdata(data_entries, na::vector![start_point.0, start_point.1]);

                canvas.processing_draw_motion(&appwindow, data_entries);
            }
        }));

        priv_.mouse_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {
            // Guard not to handle touch events that emulate a pointer
            if let Some(event) = mouse_drawing_gesture.current_event() {
                if event.is_pointer_emulated() {
                    return;
                }
            }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
            let data_entries = Self::retreive_pointer_inputdata(x, y);
            let data_entries = canvas.map_inputdata(data_entries, na::vector![start_point.0, start_point.1]);

            canvas.processing_draw_end(&appwindow, data_entries);
            }
        }));

        // Stylus Drawing
        priv_.stylus_drawing_gesture.connect_down(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            stylus_drawing_gesture.set_state(EventSequenceState::Claimed);
            if let Some(device_tool) = stylus_drawing_gesture.device_tool() {

                // Disable backlog, only allowed in motion signal handler
                let data_entries = Canvas::retreive_stylus_inputdata(stylus_drawing_gesture, false, x, y);
                let data_entries = canvas.map_inputdata(data_entries, na::vector![0.0, 0.0]);

                canvas.pens().borrow_mut().selector.clear_path();

                match device_tool.tool_type() {
                    gdk::DeviceToolType::Pen => { },
                    gdk::DeviceToolType::Eraser => {
                    appwindow
                        .application()
                        .unwrap()
                        .change_action_state("tmperaser", &true.to_variant());
                    }
                    _ => { canvas.current_pen().set(PenStyle::Unkown) },
                }

                canvas.processing_draw_begin(&appwindow, data_entries);
            }
        }));

        priv_.stylus_drawing_gesture.connect_motion(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            if stylus_drawing_gesture.device_tool().is_some() {
                // backlog doesn't provide time equidistant inputdata and makes line look worse, so its disabled for now
                let data_entries: VecDeque<InputData> = Canvas::retreive_stylus_inputdata(stylus_drawing_gesture, false, x, y);
                let data_entries = canvas.map_inputdata(data_entries, na::vector![0.0, 0.0]);

                canvas.processing_draw_motion(&appwindow, data_entries);
            }
        }));

        priv_.stylus_drawing_gesture.connect_up(
            clone!(@weak self as canvas, @weak appwindow => move |gesture_stylus,x,y| {
                let data_entries = Canvas::retreive_stylus_inputdata(gesture_stylus, false, x, y);
                let data_entries = canvas.map_inputdata(data_entries, na::vector![0.0, 0.0]);

                canvas.processing_draw_end(&appwindow, data_entries);
            }),
        );

        // Touch drawing
        priv_.touch_drawing_gesture.connect_drag_begin(
            clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
                touch_drawing_gesture.set_state(EventSequenceState::Claimed);
                let data_entries = Self::retreive_pointer_inputdata(x, y);
                let data_entries = canvas.map_inputdata(data_entries, na::vector![0.0, 0.0]);

                canvas.processing_draw_begin(&appwindow, data_entries);
            }),
        );

        priv_.touch_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                let data_entries = Self::retreive_pointer_inputdata(x, y);
                let data_entries = canvas.map_inputdata(data_entries, na::vector![start_point.0, start_point.1]);

                canvas.processing_draw_motion(&appwindow, data_entries);
            }
        }));

        priv_.touch_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
            let data_entries = Self::retreive_pointer_inputdata(x, y);
            let data_entries = canvas.map_inputdata(data_entries, na::vector![start_point.0, start_point.1]);

            canvas.processing_draw_end(&appwindow, data_entries);
            }
        }));
    }

    fn processing_draw_begin(
        &self,
        appwindow: &RnoteAppWindow,
        mut data_entries: VecDeque<InputData>,
    ) {
        self.set_unsaved_changes(true);

        if !self.sheet().selection().strokes().borrow().is_empty() {
            let mut strokes = self.sheet().selection().remove_strokes();
            self.sheet().strokes().borrow_mut().append(&mut strokes);
        }

        match self.current_pen().get() {
            PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper => {
                let mut data_entries = self.filter_inputdata(data_entries);

                if let Some(inputdata) = data_entries.pop_front() {
                    self.set_cursor(gdk::Cursor::from_name("cell", None).as_ref());

                    self.sheet().elements_trash().borrow_mut().clear();

                    StrokeStyle::new_stroke(
                        &mut *self.sheet().strokes().borrow_mut(),
                        Element::new(inputdata),
                        self.current_pen().get(),
                        &self.pens().borrow(),
                    );
                    if self.sheet().resize() {
                        self.queue_resize();
                    }
                    if let Some(stroke) = &mut self.sheet().strokes().borrow_mut().last_mut() {
                        stroke.update_rendernode(self.scalefactor(), &*self.renderer().borrow());
                    }
                }
            }
            PenStyle::Eraser => {
                if let Some(inputdata) = data_entries.pop_back() {
                    self.set_cursor(gdk::Cursor::from_name("none", None).as_ref());
                    self.pens().borrow_mut().eraser.current_input = inputdata;
                    self.pens().borrow_mut().eraser.set_shown(true);
                }
            }
            PenStyle::Selector => {
                if let Some(inputdata) = data_entries.pop_front() {
                    self.set_cursor(gdk::Cursor::from_name("cell", None).as_ref());

                    self.pens().borrow_mut().selector.set_shown(true);
                    self.pens().borrow_mut().selector.new_path(inputdata);
                    self.pens()
                        .borrow_mut()
                        .selector
                        .update_rendernode(self.scalefactor(), &*self.renderer().borrow());
                }

                self.processing_draw_motion(appwindow, data_entries);
            }
            PenStyle::Unkown => {}
        }

        self.queue_draw();
    }

    fn processing_draw_motion(
        &self,
        appwindow: &RnoteAppWindow,
        data_entries: VecDeque<InputData>,
    ) {
        self.set_unsaved_changes(true);

        match self.current_pen().get() {
            PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper => {
                let data_entries = self.filter_inputdata(data_entries);
                for inputdata in data_entries {
                    StrokeStyle::add_to_last_stroke(
                        &mut *self.sheet().strokes().borrow_mut(),
                        Element::new(inputdata),
                        &self.pens().borrow(),
                    );

                    if self.sheet().resize() {
                        self.queue_resize();
                    }

                    if let Some(stroke) = &mut self.sheet().strokes().borrow_mut().last_mut() {
                        stroke.update_rendernode(self.scalefactor(), &*self.renderer().borrow());
                    }
                }
            }
            PenStyle::Eraser => {
                let scalefactor = self.scalefactor();
                let canvas_scroller_viewport_scaled =
                    if let Some(viewport) = appwindow.canvas_scroller_viewport() {
                        Some(utils::aabb_scale(viewport, 1.0 / scalefactor))
                    } else {
                        None
                    };

                for inputdata in data_entries {
                    self.pens().borrow_mut().eraser.current_input = inputdata;
                    if self.sheet().remove_colliding_strokes(
                        &self.pens().borrow().eraser,
                        canvas_scroller_viewport_scaled,
                    ) {
                        self.queue_resize();
                    }
                }
            }
            PenStyle::Selector => {
                for inputdata in data_entries {
                    self.pens()
                        .borrow_mut()
                        .selector
                        .push_elem(inputdata.clone());
                    self.pens()
                        .borrow_mut()
                        .selector
                        .update_rendernode(self.scalefactor(), &*self.renderer().borrow());
                }
            }
            PenStyle::Unkown => {}
        }

        self.queue_draw();
    }

    fn processing_draw_end(&self, appwindow: &RnoteAppWindow, _data_entries: VecDeque<InputData>) {
        self.set_unsaved_changes(true);
        self.set_cursor(Some(&self.cursor()));

        appwindow
            .application()
            .unwrap()
            .change_action_state("tmperaser", &false.to_variant());

        if let Some(stroke) = self.sheet().strokes().borrow_mut().last_mut() {
            stroke.complete_stroke();
        }

        let scalefactor = self.scalefactor();
        let canvas_scroller_viewport_scaled =
            if let Some(viewport) = appwindow.canvas_scroller_viewport() {
                Some(utils::aabb_scale(viewport, 1.0 / scalefactor))
            } else {
                None
            };

        match self.current_pen().get() {
            PenStyle::Selector => {
                self.sheet().selection().update_selection(
                    &self.pens().borrow().selector,
                    &mut self.sheet().strokes().borrow_mut(),
                    canvas_scroller_viewport_scaled,
                );

                self.pens().borrow_mut().selector.clear_path();
                self.pens()
                    .borrow_mut()
                    .selector
                    .update_rendernode(self.scalefactor(), &*self.renderer().borrow());
            }
            PenStyle::Marker
            | PenStyle::Brush
            | PenStyle::Shaper
            | PenStyle::Eraser
            | PenStyle::Unkown => {
                self.pens().borrow_mut().eraser.set_shown(false);
                self.pens().borrow_mut().selector.set_shown(false);
            }
        }

        self.queue_draw();
    }

    // Map Stylus input to the position on a sheet
    fn map_inputdata(
        &self,
        data_entries: VecDeque<InputData>,
        offset: na::Vector2<f64>,
    ) -> VecDeque<InputData> {
        let data_entries: VecDeque<InputData> = data_entries
            .iter()
            .map(|inputdata| {
                InputData::new(
                    (inputdata.pos() + offset).scale(1.0 / self.scalefactor()),
                    inputdata.pressure(),
                )
            })
            .collect();

        data_entries
    }

    // Clip inputdata to sheet
    fn filter_inputdata(&self, mut data_entries: VecDeque<InputData>) -> VecDeque<InputData> {
        let priv_ = imp::Canvas::from_instance(self);

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

        data_entries
            .retain(|data| filter_bounds.contains_local_point(&na::Point2::from(data.pos())));

        data_entries
    }

    fn retreive_pointer_inputdata(x: f64, y: f64) -> VecDeque<InputData> {
        let mut data_entries: VecDeque<InputData> = VecDeque::with_capacity(1);

        data_entries.push_back(InputData::new(
            na::vector![x, y],
            InputData::PRESSURE_DEFAULT,
        ));
        data_entries
    }

    // Retreives available input axes, defaults if not available. X and Y is already available from closure, and should not retreived from .axis() (because of gtk-rs weirdness)
    fn retreive_stylus_inputdata(
        gesture_stylus: &GestureStylus,
        with_backlog: bool,
        x: f64,
        y: f64,
    ) -> VecDeque<InputData> {
        let mut data_entries: VecDeque<InputData> = VecDeque::new();

        if with_backlog {
            if let Some(backlog) = gesture_stylus.backlog() {
                log::debug!("{}", backlog.len());

                for logentry in backlog {
                    let axes = logentry.axes();
                    let x = axes[1];
                    let y = axes[2];
                    let pressure = axes[5];
                    //println!("{:?}", axes);
                    data_entries.push_back(InputData::new(na::vector![x, y], pressure));
                }
            }
        }

        // Get newest data
        let pressure = if let Some(pressure) = gesture_stylus.axis(gdk::AxisUse::Pressure) {
            pressure
        } else {
            InputData::PRESSURE_DEFAULT
        };

        data_entries.push_back(InputData::new(na::vector![x, y], pressure));

        data_entries
    }
}

mod debug {
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
