pub mod canvaslayout;

mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::canvaslayout::CanvasLayout;
    use super::debug;
    use crate::pens::{PenStyle, Pens};
    use crate::sheet::Sheet;
    use crate::ui::selectionmodifier::SelectionModifier;
    use crate::{config, geometry};

    use gtk4::{
        gdk, glib, graphene, gsk, prelude::*, subclass::prelude::*, GestureDrag, GestureStylus,
        PropagationPhase, Snapshot, Widget,
    };
    use gtk4::{AccessibleRole, Adjustment, Scrollable, ScrollablePolicy};

    use once_cell::sync::Lazy;
    use p2d::bounding_volume::BoundingVolume;

    #[derive(Debug)]
    pub struct Canvas {
        pub pens: Rc<RefCell<Pens>>,
        pub current_pen: Rc<Cell<PenStyle>>,
        pub sheet: Sheet,
        pub sheet_margin: Cell<f64>,
        pub zoom: Cell<f64>,
        pub temporary_zoom: Cell<f64>,
        pub pdf_import_width: Cell<f64>,
        pub pdf_import_as_vector: Cell<bool>,
        pub visual_debug: Cell<bool>,
        pub touch_drawing: Cell<bool>,
        pub unsaved_changes: Cell<bool>,
        pub empty: Cell<bool>,
        pub cursor: gdk::Cursor,
        pub motion_cursor: gdk::Cursor,
        pub stylus_drawing_gesture: GestureStylus,
        pub mouse_drawing_gesture: GestureDrag,
        pub touch_drawing_gesture: GestureDrag,
        pub zoom_timeout_id: RefCell<Option<glib::SourceId>>,

        pub hadjustment: RefCell<Option<Adjustment>>,
        pub hadjustment_signal: RefCell<Option<glib::SignalHandlerId>>,
        pub vadjustment: RefCell<Option<Adjustment>>,
        pub vadjustment_signal: RefCell<Option<glib::SignalHandlerId>>,
        pub hscroll_policy: Cell<ScrollablePolicy>,
        pub vscroll_policy: Cell<ScrollablePolicy>,

        pub selection_modifier: SelectionModifier,
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
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            // Gesture grouping
            mouse_drawing_gesture.group_with(&stylus_drawing_gesture);
            touch_drawing_gesture.group_with(&stylus_drawing_gesture);

            Self {
                pens: Rc::new(RefCell::new(Pens::default())),
                current_pen: Rc::new(Cell::new(PenStyle::default())),
                sheet: Sheet::default(),
                sheet_margin: Cell::new(super::Canvas::SHEET_MARGIN_DEFAULT),
                zoom: Cell::new(super::Canvas::ZOOM_DEFAULT),
                temporary_zoom: Cell::new(1.0),
                pdf_import_width: Cell::new(super::Canvas::PDF_IMPORT_WIDTH_DEFAULT),
                pdf_import_as_vector: Cell::new(true),
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
                zoom_timeout_id: RefCell::new(None),

                hadjustment: RefCell::new(None),
                hadjustment_signal: RefCell::new(None),
                vadjustment: RefCell::new(None),
                vadjustment_signal: RefCell::new(None),
                hscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                vscroll_policy: Cell::new(ScrollablePolicy::Minimum),

                selection_modifier: SelectionModifier::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Canvas {
        const NAME: &'static str = "Canvas";
        type Type = super::Canvas;
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

    impl ObjectImpl for Canvas {
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
                let interface: glib::object::InterfaceRef<Scrollable> =
                    glib::Interface::from_type(Scrollable::static_type()).unwrap();

                // The temporary zoom factor (multiplied on top of zoom!) which is used when doing zoom gestures ( to avoid strokes redrawing )
                vec![
                    // The margin of the sheet in px when zoom = 1.0
                    glib::ParamSpec::new_double(
                        "sheet-margin",
                        "sheet-margin",
                        "sheet-margin",
                        f64::MIN,
                        f64::MAX,
                        super::Canvas::SHEET_MARGIN_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    // The zoom of the canvas in relation to the sheet
                    glib::ParamSpec::new_double(
                        "zoom",
                        "zoom",
                        "zoom",
                        f64::MIN,
                        f64::MAX,
                        super::Canvas::ZOOM_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    // The temporary zoom (on top of the normal zoom)
                    glib::ParamSpec::new_double(
                        "temporary-zoom",
                        "temporary-zoom",
                        "temporary-zoom",
                        f64::MIN,
                        f64::MAX,
                        1.0,
                        glib::ParamFlags::READWRITE,
                    ),
                    // import PDFs with with in percentage to sheet width
                    glib::ParamSpec::new_double(
                        "pdf-import-width",
                        "pdf-import-width",
                        "pdf-import-width",
                        1.0,
                        100.0,
                        super::Canvas::PDF_IMPORT_WIDTH_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    // import PDFs as vector images ( if false = as bitmap images )
                    glib::ParamSpec::new_boolean(
                        "pdf-import-as-vector",
                        "pdf-import-as-vector",
                        "pdf-import-as-vector",
                        true,
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
                    // Scrollable properties
                    glib::ParamSpec::new_override(
                        "hscroll-policy",
                        &interface.find_property("hscroll-policy").unwrap(),
                    ),
                    glib::ParamSpec::new_override(
                        "vscroll-policy",
                        &interface.find_property("vscroll-policy").unwrap(),
                    ),
                    glib::ParamSpec::new_override(
                        "hadjustment",
                        &interface.find_property("hadjustment").unwrap(),
                    ),
                    glib::ParamSpec::new_override(
                        "vadjustment",
                        &interface.find_property("vadjustment").unwrap(),
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "sheet-margin" => self.sheet_margin.get().to_value(),
                "zoom" => self.zoom.get().to_value(),
                "temporary-zoom" => self.temporary_zoom.get().to_value(),
                "pdf-import-width" => self.pdf_import_width.get().to_value(),
                "pdf-import-as-vector" => self.pdf_import_as_vector.get().to_value(),
                "visual-debug" => self.visual_debug.get().to_value(),
                "touch-drawing" => self.touch_drawing.get().to_value(),
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
                "empty" => self.empty.get().to_value(),
                "hadjustment" => self.hadjustment.borrow().to_value(),
                "vadjustment" => self.vadjustment.borrow().to_value(),
                "hscroll-policy" => self.hscroll_policy.get().to_value(),
                "vscroll-policy" => self.vscroll_policy.get().to_value(),
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
                "sheet-margin" => {
                    let sheet_margin = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.");

                    self.sheet_margin.replace(sheet_margin);
                }
                "zoom" => {
                    let zoom: f64 = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.")
                        .clamp(super::Canvas::ZOOM_MIN, super::Canvas::ZOOM_MAX);
                    self.zoom.replace(zoom);

                    self.sheet.strokes_state().borrow_mut().zoom = zoom;

                    obj.queue_resize();
                    obj.queue_draw();
                }
                "temporary-zoom" => {
                    let temporary_zoom = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.")
                        .clamp(
                            super::Canvas::ZOOM_MIN / self.zoom.get(),
                            super::Canvas::ZOOM_MAX / self.zoom.get(),
                        );

                    self.temporary_zoom.replace(temporary_zoom);

                    obj.queue_resize();
                    obj.queue_draw();
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
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Canvas {
        // request_mode(), measure(), allocate() overrides happen in the CanvasLayout LayoutManager

        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk4::Snapshot) {
            let temporary_zoom = widget.temporary_zoom();
            let zoom = widget.zoom();
            let hadj = widget.hadjustment().unwrap();
            let vadj = widget.vadjustment().unwrap();
            let sheet_margin = self.sheet_margin.get();

            // Clip everything outside the current view
            snapshot.push_clip(&graphene::Rect::new(
                0.0,
                0.0,
                widget.width() as f32,
                widget.height() as f32,
            ));

            snapshot.save();

            snapshot.translate(&graphene::Point::new(
                (-hadj.value()) as f32,
                (-vadj.value()) as f32,
            ));

            snapshot.scale(temporary_zoom as f32, temporary_zoom as f32);

            // From here in scaled sheet coordinate space
            snapshot.translate(&graphene::Point::new(
                (sheet_margin * zoom) as f32,
                (sheet_margin * zoom) as f32,
            ));

            self.draw_shadow(
                geometry::aabb_scale(widget.sheet().bounds(), zoom),
                sheet_margin * zoom,
                snapshot,
            );

            // Clip sheet and stroke drawing to sheet bounds
            snapshot.push_clip(&geometry::aabb_to_graphene_rect(geometry::aabb_scale(
                widget.sheet().bounds(),
                zoom,
            )));

            self.sheet.draw(zoom, snapshot);

            self.sheet
                .strokes_state()
                .borrow()
                .draw_strokes(snapshot, Some(widget.viewport_in_sheet_coords()));

            snapshot.pop();

            self.sheet
                .strokes_state()
                .borrow()
                .draw_selection(zoom, snapshot);

            self.pens
                .borrow()
                .draw(self.current_pen.get(), snapshot, zoom);

            if self.visual_debug.get() {
                self.draw_debug(widget, snapshot, zoom);
            }

            // Draw the children
            snapshot.restore();

            widget.snapshot_child(&self.selection_modifier, snapshot);

            snapshot.pop();
        }
    }

    impl ScrollableImpl for Canvas {}

    impl Canvas {
        pub const SHADOW_WIDTH: f64 = 30.0;

        pub fn draw_shadow(
            &self,
            bounds: p2d::bounding_volume::AABB,
            width: f64,
            snapshot: &Snapshot,
        ) {
            let shadow_color = gdk::RGBA {
                red: 0.1,
                green: 0.1,
                blue: 0.1,
                alpha: 0.3,
            };
            let corner_radius = graphene::Size::new(width as f32 / 4.0, width as f32 / 4.0);

            let rounded_rect = gsk::RoundedRect::new(
                geometry::aabb_to_graphene_rect(bounds),
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
                (width / 2.0) as f32,
                (width / 2.0) as f32,
            );
        }

        // Draw bounds, positions, .. for visual debugging purposes
        fn draw_debug(&self, widget: &super::Canvas, snapshot: &Snapshot, zoom: f64) {
            match self.current_pen.get() {
                PenStyle::Eraser => {
                    if self.pens.borrow().eraser.shown() {
                        if let Some(ref current_input) = self.pens.borrow().eraser.current_input {
                            debug::draw_pos(
                                current_input.pos(),
                                debug::COLOR_POS_ALT,
                                zoom,
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
                                zoom,
                                snapshot,
                            );
                        }
                    }
                }
                PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper | PenStyle::Unkown => {}
            }

            debug::draw_bounds(
                self.sheet.bounds(),
                debug::COLOR_SHEET_BOUNDS,
                zoom,
                snapshot,
            );

            let viewport = widget.viewport_in_sheet_coords().tightened(1.0);
            debug::draw_bounds(viewport, debug::COLOR_STROKE_BOUNDS, zoom, snapshot);

            self.sheet
                .strokes_state()
                .borrow()
                .draw_debug(zoom, snapshot);
        }
    }
}

use crate::audioplayer::RnoteAudioPlayer;
use crate::strokes::strokestyle::{Element, InputData};
use crate::strokes::StrokeKey;
use crate::ui::selectionmodifier::SelectionModifier;
use crate::{
    app::RnoteApp, pens::PenStyle, pens::Pens, render, sheet::Sheet, ui::appwindow::RnoteAppWindow,
};
use crate::{geometry, input};

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time;

use gtk4::{gdk, glib, glib::clone, prelude::*, subclass::prelude::*};
use gtk4::{
    gio, Adjustment, DropTarget, EventSequenceState, PropagationPhase, Scrollable, Snapshot, Widget,
};
use p2d::bounding_volume::BoundingVolume;

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<imp::Canvas>)
        @extends gtk4::Widget,
        @implements Scrollable;
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    pub const ZOOM_MIN: f64 = 0.1;
    pub const ZOOM_MAX: f64 = 8.0;
    pub const ZOOM_DEFAULT: f64 = 1.0;
    /// The zoom amount when activating the zoom-in / zoom-out action
    pub const ZOOM_ACTION_DELTA: f64 = 0.1;
    pub const ZOOM_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(300);
    pub const INPUT_OVERSHOOT: f64 = 30.0;
    pub const SHEET_MARGIN_DEFAULT: f64 = 30.0;
    // The default width of imported PDF's in percentage to the sheet width
    pub const PDF_IMPORT_WIDTH_DEFAULT: f64 = 50.0;

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

    pub fn sheet_margin(&self) -> f64 {
        self.property("sheet-margin").unwrap().get::<f64>().unwrap()
    }

    pub fn set_sheet_margin(&self, sheet_margin: f64) {
        self.set_property("sheet-margin", sheet_margin.to_value())
            .unwrap();
    }

    pub fn zoom(&self) -> f64 {
        self.property("zoom").unwrap().get::<f64>().unwrap()
    }

    fn set_zoom(&self, zoom: f64) {
        self.set_property("zoom", zoom.to_value()).unwrap();
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

    pub fn pdf_import_width(&self) -> f64 {
        self.property("pdf-import-width")
            .unwrap()
            .get::<f64>()
            .unwrap()
    }

    pub fn set_pdf_import_width(&self, pdf_import_width: f64) {
        self.set_property("pdf-import-width", pdf_import_width.to_value())
            .unwrap();
    }

    pub fn pdf_import_as_vector(&self) -> bool {
        self.property("pdf-import-as-vector")
            .unwrap()
            .get::<bool>()
            .unwrap()
    }

    pub fn set_pdf_import_as_vector(&self, as_vector: bool) {
        self.set_property("pdf-import-as-vector", as_vector.to_value())
            .unwrap();
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

    fn set_hadjustment(&self, adj: Option<Adjustment>) {
        let self_ = imp::Canvas::from_instance(self);
        if let Some(signal_id) = self_.hadjustment_signal.borrow_mut().take() {
            let old_adj = self_.hadjustment.borrow().as_ref().unwrap().clone();
            old_adj.disconnect(signal_id);
        }

        if let Some(ref adjustment) = adj {
            let signal_id =
                adjustment.connect_value_changed(clone!(@weak self as canvas => move |_| {
                    canvas.regenerate_content(false, false);

                    canvas.queue_allocate();
                    canvas.queue_draw();
                }));
            self_.hadjustment_signal.replace(Some(signal_id));
        }
        self_.hadjustment.replace(adj);
    }

    fn set_vadjustment(&self, adj: Option<Adjustment>) {
        let self_ = imp::Canvas::from_instance(self);
        if let Some(signal_id) = self_.vadjustment_signal.borrow_mut().take() {
            let old_adj = self_.vadjustment.borrow().as_ref().unwrap().clone();
            old_adj.disconnect(signal_id);
        }

        if let Some(ref adjustment) = adj {
            let signal_id =
                adjustment.connect_value_changed(clone!(@weak self as canvas => move |_| {
                    canvas.regenerate_content(false, false);

                    canvas.queue_allocate();
                    canvas.queue_draw();
                }));
            self_.vadjustment_signal.replace(Some(signal_id));
        }
        self_.vadjustment.replace(adj);
    }

    pub fn selection_modifier(&self) -> SelectionModifier {
        imp::Canvas::from_instance(self).selection_modifier.clone()
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
            "zoom",
            &appwindow.mainheader().canvasmenu().zoomreset_button(),
            "label",
        )
        .transform_to(|_, value| {
            let zoom = value.get::<f64>().unwrap();
            Some(format!("{:.0}%", zoom * 100.0).to_value())
        })
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

        let current_stroke_key = Rc::new(Cell::new(None));

        // Stylus Drawing
        priv_.stylus_drawing_gesture.connect_down(clone!(@strong current_stroke_key, @weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            if let Some(device_tool) = stylus_drawing_gesture.device_tool() {
                stylus_drawing_gesture.set_state(EventSequenceState::Claimed);

                // Disable backlog, only allowed in motion signal handler
                let mut data_entries = input::retreive_stylus_inputdata(stylus_drawing_gesture, false, x, y);

                input::map_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]));

                match device_tool.tool_type() {
                    gdk::DeviceToolType::Pen => { },
                    gdk::DeviceToolType::Eraser => {
                    appwindow.downcast_ref::<RnoteAppWindow>().unwrap()
                        .change_action_state("tmperaser", &true.to_variant());
                    }
                    _ => { canvas.current_pen().set(PenStyle::Unkown) },
                }

                current_stroke_key.set(canvas.processing_draw_begin(&appwindow, &mut data_entries));
            }
        }));

        priv_.stylus_drawing_gesture.connect_motion(clone!(@strong current_stroke_key, @weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            // backlog doesn't provide time equidistant inputdata and makes line look worse, so its disabled for now
            let mut data_entries: VecDeque<InputData> = input::retreive_stylus_inputdata(stylus_drawing_gesture, false, x, y);
            input::map_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]));
            canvas.processing_draw_motion(&appwindow, current_stroke_key.get(), &mut data_entries);
        }));

        priv_.stylus_drawing_gesture.connect_up(
            clone!(@strong current_stroke_key, @weak self as canvas, @weak appwindow => move |gesture_stylus,x,y| {
                let mut data_entries = input::retreive_stylus_inputdata(gesture_stylus, false, x, y);

                input::map_inputdata(canvas.zoom(), &mut data_entries, na::vector![0.0, 0.0]);
                canvas.processing_draw_end(&appwindow, &mut data_entries);

                current_stroke_key.set(None);
            }),
        );

        // Mouse drawing
        priv_.mouse_drawing_gesture.connect_drag_begin(
            clone!(@strong current_stroke_key, @weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
                if let Some(event) = mouse_drawing_gesture.current_event() {
                    // Guard not to handle touch events that emulate a pointer
                    if event.is_pointer_emulated() {
                        return;
                    }

                    mouse_drawing_gesture.set_state(EventSequenceState::Claimed);

                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::map_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]));
                    current_stroke_key.set(canvas.processing_draw_begin(&appwindow, &mut data_entries));
                }
            }),
        );

        priv_.mouse_drawing_gesture.connect_drag_update(clone!(@strong current_stroke_key, @weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            if let Some(event) = mouse_drawing_gesture.current_event() {
                // Guard not to handle touch events that emulate a pointer
                if event.is_pointer_emulated() {
                    return;
                }

                if let Some(start_point) = mouse_drawing_gesture.start_point() {
                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::map_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]));
                    canvas.processing_draw_motion(&appwindow, current_stroke_key.get(), &mut data_entries);
                }
            }
        }));

        priv_.mouse_drawing_gesture.connect_drag_end(
            clone!(@strong current_stroke_key, @weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {

                if let Some(event) = mouse_drawing_gesture.current_event() {
                    // Guard not to handle touch events that emulate a pointer
                    if event.is_pointer_emulated() {
                        return;
                    }

                    if let Some(start_point) = mouse_drawing_gesture.start_point() {
                        let mut data_entries = input::retreive_pointer_inputdata(x, y);
                        input::map_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]));
                        canvas.processing_draw_end(&appwindow, &mut data_entries);
                    }

                    current_stroke_key.set(None);
                }
            }),
        );

        // Touch drawing
        priv_.touch_drawing_gesture.connect_drag_begin(
            clone!(@strong current_stroke_key, @weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
                touch_drawing_gesture.set_state(EventSequenceState::Claimed);

                let mut data_entries = input::retreive_pointer_inputdata(x, y);
                input::map_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]));

                current_stroke_key.set(canvas.processing_draw_begin(&appwindow, &mut data_entries));
            }),
        );

        priv_.touch_drawing_gesture.connect_drag_update(clone!(@strong current_stroke_key, @weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_inputdata(x, y);
                input::map_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]));
                canvas.processing_draw_motion(&appwindow, current_stroke_key.get(), &mut data_entries);
            }
        }));

        priv_.touch_drawing_gesture.connect_drag_end(
            clone!(@strong current_stroke_key, @weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
                if let Some(start_point) = touch_drawing_gesture.start_point() {
                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::map_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]));
                    canvas.processing_draw_end(&appwindow, &mut data_entries);
                }

                current_stroke_key.set(None);
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

        drop_target.connect_drop(clone!(@weak appwindow => @default-return false, move |_drop_target, value, x, y| {
            let pos = appwindow.canvas().transform_canvas_coords_to_sheet_coords(na::vector![x, y]);

            if let Ok(file) = value.get::<gio::File>() {
                appwindow.open_file_w_dialogs(&file, Some(pos))
            }
            true
        }));
    }

    pub fn total_zoom(&self) -> f64 {
        self.zoom() * self.temporary_zoom()
    }

    /// The widget bounds in its coordinate space. Is the bounds of the view, not the bounds of the scaled sheet!
    pub fn bounds(&self) -> p2d::bounding_volume::AABB {
        p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![f64::from(self.width()), f64::from(self.height())],
        )
    }

    /// The bounds of the drawn content, meaning the bounds of the scaled sheet + margin
    pub fn content_bounds(&self) -> p2d::bounding_volume::AABB {
        let zoom = self.zoom();

        self.sheet_bounds_in_canvas_coords()
            .loosened(self.sheet_margin() * zoom)
    }

    /// The bounds of the sheet in the coordinate space of the canvas
    pub fn sheet_bounds_in_canvas_coords(&self) -> p2d::bounding_volume::AABB {
        let zoom = self.zoom();
        let sheet_margin = self.sheet_margin();

        p2d::bounding_volume::AABB::new(
            na::point![sheet_margin * zoom, sheet_margin * zoom],
            na::point![
                (sheet_margin + f64::from(self.sheet().width())) * zoom,
                (sheet_margin + f64::from(self.sheet().height())) * zoom
            ],
        )
    }

    /// transforming coordinates of canvas coordinate space into sheet coordinate space
    pub fn transform_canvas_coords_to_sheet_coords(
        &self,
        canvas_coords: na::Vector2<f64>,
    ) -> na::Vector2<f64> {
        let total_zoom = self.total_zoom();

        (canvas_coords
            + na::vector![
                self.hadjustment().unwrap().value(),
                self.vadjustment().unwrap().value()
            ])
            / total_zoom
            - na::vector![self.sheet_margin(), self.sheet_margin()]
    }

    /// transforming coordinates of canvas coordinate space into sheet coordinate space
    pub fn transform_sheet_coords_to_canvas_coords(
        &self,
        sheet_coords: na::Vector2<f64>,
    ) -> na::Vector2<f64> {
        let total_zoom = self.total_zoom();

        sheet_coords
            + na::vector![
                self.sheet_margin() * total_zoom,
                self.sheet_margin() * total_zoom
            ]
            - na::vector![
                self.hadjustment().unwrap().value(),
                self.vadjustment().unwrap().value()
            ]
    }

    /// The view of the parent scroller onto the Canvas
    pub fn viewport(&self) -> p2d::bounding_volume::AABB {
        let parent = self.parent().unwrap();
        let (parent_width, parent_height) = (f64::from(parent.width()), f64::from(parent.height()));
        let (parent_offset_x, parent_offset_y) = self
            .translate_coordinates(&parent, 0.0, 0.0)
            .unwrap_or((0.0, 0.0));

        let (x, y) = (
            self.hadjustment().unwrap().value() - parent_offset_x,
            self.vadjustment().unwrap().value() - parent_offset_y,
        );

        p2d::bounding_volume::AABB::new(
            na::point![x, y],
            na::point![x + parent_width, y + parent_height],
        )
    }

    /// The viewport transformed to match the coordinate space of the sheet
    pub fn viewport_in_sheet_coords(&self) -> p2d::bounding_volume::AABB {
        let mut viewport = self.viewport();
        let total_zoom = self.total_zoom();
        let sheet_margin = self.sheet_margin();

        viewport = geometry::aabb_translate(
            geometry::aabb_scale(viewport, 1.0 / total_zoom),
            -na::vector![sheet_margin, sheet_margin],
        );

        viewport
    }

    /// The point parameter has the coordinate space of the sheet!
    pub fn center_around_coord_on_sheet(&self, coord: na::Vector2<f64>) {
        let (parent_width, parent_height) = (
            f64::from(self.parent().unwrap().width()),
            f64::from(self.parent().unwrap().height()),
        );
        let total_zoom = self.total_zoom();
        let sheet_margin = self.sheet_margin();

        let (canvas_width, canvas_height) = (
            f64::from(self.sheet().width()) * total_zoom,
            f64::from(self.sheet().height()) * total_zoom,
        );

        if canvas_width > parent_width {
            self.hadjustment()
                .unwrap()
                .set_value(((sheet_margin + coord[0]) * total_zoom) - parent_width * 0.5);
        }
        if canvas_height > parent_height {
            self.vadjustment()
                .unwrap()
                .set_value(((sheet_margin + coord[1]) * total_zoom) - parent_height * 0.5);
        }
    }

    /// Zoom temporarily to a new zoom, not regenerating the contents while doing it.
    /// To zoom and regenerate the content and reset the temporary zoom, use zoom_to().
    pub fn zoom_temporarily_to(&self, temp_zoom: f64) {
        self.set_temporary_zoom(temp_zoom / self.zoom());
    }

    /// zooms and regenerates the canvas and its contents to a new zoom
    pub fn zoom_to(&self, zoom: f64) {
        self.set_temporary_zoom(1.0);
        self.set_zoom(zoom);

        self.sheet()
            .strokes_state()
            .borrow_mut()
            .reset_regenerate_flag_all_strokes();

        // update rendernodes to new zoom until threaded regeneration is finished
        self.update_content_rendernodes();

        self.regenerate_background(false);
        self.regenerate_content(false, true);
    }

    /// Zooms temporarily and then scale the canvas and its contents to a new zoom after a given time.
    /// Repeated calls to this function reset the timeout.
    pub fn zoom_temporarily_then_scale_to_after_timeout(
        &self,
        zoom: f64,
        timeout_time: time::Duration,
    ) {
        let priv_ = imp::Canvas::from_instance(self);

        if let Some(zoom_timeout) = priv_.zoom_timeout_id.take() {
            glib::source::source_remove(zoom_timeout);
        }

        self.zoom_temporarily_to(zoom);

        priv_
            .zoom_timeout_id
            .borrow_mut()
            .replace(glib::source::timeout_add_local_once(
                timeout_time,
                clone!(@weak self as canvas => move || {
                    let priv_ = imp::Canvas::from_instance(&canvas);

                    canvas.zoom_to(zoom);

                    // Removing the timeout id
                    let mut zoom_timeout_id = priv_.zoom_timeout_id.borrow_mut();
                    if let Some(zoom_timeout_id) = zoom_timeout_id.take() {
                        glib::source::source_remove(zoom_timeout_id);
                    }
                }),
            ));
    }

    /// Update rendernodes of the background. Used when sheet size, but not zoom changed
    pub fn update_background_rendernode(&self) {
        self.sheet()
        .background()
        .borrow_mut()
        .update_rendernode(self.zoom(), self.sheet().bounds()).unwrap_or_else(|e| {
            log::error!("failed to update rendernode for background in update_background_rendernode() with Err {}", e);
        });

        self.queue_resize();
        self.queue_draw();
    }

    /// Update rendernodes of the background. Used when sheet size, but not zoom changed
    pub fn update_content_rendernodes(&self) {
        self.sheet()
            .strokes_state()
            .borrow_mut()
            .update_rendernodes_current_zoom(self.zoom());

        self.queue_resize();
        self.queue_draw();
    }

    /// regenerating the background image and rendernode.
    /// use for example when changing the background pattern or zoom
    pub fn regenerate_background(&self, redraw: bool) {
        if let Err(e) = self
            .sheet()
            .background()
            .borrow_mut()
            .regenerate_background(
                &self
                    .sheet()
                    .strokes_state()
                    .borrow()
                    .renderer
                    .read()
                    .unwrap(),
                self.zoom(),
                self.sheet().bounds(),
            )
        {
            log::error!("failed to regenerate background, {}", e)
        };

        if redraw {
            self.queue_resize();
            self.queue_draw();
        }
    }

    /// regenerate the rendernodes of the canvas content. force_regenerate regenerate all images and rendernodes from scratch. redraw: queue canvas redrawing
    pub fn regenerate_content(&self, force_regenerate: bool, redraw: bool) {
        self.sheet()
            .strokes_state()
            .borrow_mut()
            .regenerate_strokes_current_view_threaded(
                Some(self.viewport_in_sheet_coords()),
                force_regenerate,
            );

        if redraw {
            self.queue_resize();
            self.queue_draw();
        }
    }

    /// Captures the current view of the canvas as a gdk::Texture
    pub fn current_view_as_texture(&self) -> Option<gdk::Texture> {
        let priv_ = imp::Canvas::from_instance(self);
        let snapshot = Snapshot::new();

        self.selection_modifier().set_visible(false);

        priv_.snapshot(self, &snapshot);

        let texture = if let Some(node) = snapshot.to_node() {
            let texture = render::rendernode_to_texture(self.upcast_ref::<Widget>(), &node, None)
                .unwrap_or_else(|e| {
                    log::error!("rendernode_to_texture() in current_content_as_texture() failed with Err {}", e);
                    None
                });
            texture
        } else {
            None
        };

        self.selection_modifier().set_visible(true);

        texture
    }

    /// Process the beginning of a stroke drawing
    fn processing_draw_begin(
        &self,
        appwindow: &RnoteAppWindow,
        data_entries: &mut VecDeque<InputData>,
    ) -> Option<StrokeKey> {
        let priv_ = imp::Canvas::from_instance(self);
        let mut stroke_key = None;

        let zoom = self.zoom();

        appwindow.audioplayer().borrow().play_pen_sound_begin(
            RnoteAudioPlayer::PLAY_TIMEOUT_TIME,
            self.current_pen().get(),
        );

        self.set_unsaved_changes(true);
        self.set_empty(false);
        self.sheet()
            .strokes_state()
            .borrow_mut()
            .deselect_all_strokes();
        self.selection_modifier().set_visible(false);

        match self.current_pen().get() {
            PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper => {
                self.set_cursor(Some(&self.motion_cursor()));

                let filter_bounds = p2d::bounding_volume::AABB::new(
                    na::point![-Self::INPUT_OVERSHOOT, -Self::INPUT_OVERSHOOT],
                    na::point![
                        (priv_.sheet.width()) as f64 + Self::INPUT_OVERSHOOT,
                        (priv_.sheet.height()) as f64 + Self::INPUT_OVERSHOOT
                    ],
                );
                input::filter_mapped_inputdata(filter_bounds, data_entries);

                if let Some(inputdata) = data_entries.pop_back() {
                    stroke_key = self.sheet().strokes_state().borrow_mut().new_stroke(
                        Element::new(inputdata),
                        self.current_pen().get(),
                        &self.pens().borrow(),
                    );

                    self.sheet()
                        .strokes_state()
                        .borrow_mut()
                        .regenerate_rendering_newest_stroke_threaded();
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
                        zoom,
                        &self
                            .sheet()
                            .strokes_state()
                            .borrow()
                            .renderer
                            .read()
                            .unwrap(),
                    );
                }
            }
            PenStyle::Unkown => {}
        }

        self.queue_resize();
        self.queue_draw();

        stroke_key
    }

    /// Process the motion of a strokes drawing
    fn processing_draw_motion(
        &self,
        appwindow: &RnoteAppWindow,
        current_stroke_key: Option<StrokeKey>,
        data_entries: &mut VecDeque<InputData>,
    ) {
        let priv_ = imp::Canvas::from_instance(self);

        let zoom = self.zoom();

        appwindow.audioplayer().borrow().play_pen_sound_motion(
            RnoteAudioPlayer::PLAY_TIMEOUT_TIME,
            self.current_pen().get(),
        );

        match self.current_pen().get() {
            PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper => {
                let filter_bounds = p2d::bounding_volume::AABB::new(
                    na::point![-Self::INPUT_OVERSHOOT, -Self::INPUT_OVERSHOOT],
                    na::point![
                        (priv_.sheet.width()) as f64 + Self::INPUT_OVERSHOOT,
                        (priv_.sheet.height()) as f64 + Self::INPUT_OVERSHOOT
                    ],
                );
                input::filter_mapped_inputdata(filter_bounds, data_entries);

                if let Some(current_stroke_key) = current_stroke_key {
                    for inputdata in data_entries {
                        self.sheet()
                            .strokes_state()
                            .borrow_mut()
                            .add_to_stroke(current_stroke_key, Element::new(*inputdata));
                    }
                }
            }
            PenStyle::Eraser => {
                if let Some(inputdata) = data_entries.pop_back() {
                    self.pens().borrow_mut().eraser.current_input = Some(inputdata);

                    self.sheet()
                        .strokes_state()
                        .borrow_mut()
                        .trash_colliding_strokes(
                            &self.pens().borrow().eraser,
                            Some(self.viewport_in_sheet_coords()),
                        );
                    if self.sheet().resize_endless() {
                        self.update_background_rendernode();
                    }
                }
            }
            PenStyle::Selector => {
                for inputdata in data_entries {
                    self.pens()
                        .borrow_mut()
                        .selector
                        .add_elem_to_path(*inputdata);
                    self.pens().borrow_mut().selector.update_rendernode(
                        zoom,
                        &self
                            .sheet()
                            .strokes_state()
                            .borrow()
                            .renderer
                            .read()
                            .unwrap(),
                    );
                }
            }
            PenStyle::Unkown => {}
        }

        self.queue_resize();
        self.queue_draw();
    }

    /// Process the end of a strokes drawing
    fn processing_draw_end(
        &self,
        appwindow: &RnoteAppWindow,
        _data_entries: &mut VecDeque<InputData>,
    ) {
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
                .update_stroke_geometry(last_key);

            self.sheet()
                .strokes_state()
                .borrow_mut()
                .regenerate_rendering_for_stroke_threaded(last_key);
        }

        match self.current_pen().get() {
            PenStyle::Selector => {
                self.sheet()
                    .strokes_state()
                    .borrow_mut()
                    .update_selection_for_selector(
                        &self.pens().borrow().selector,
                        Some(self.viewport_in_sheet_coords()),
                    );

                // Show the selection modifier if selection bounds are some
                self.selection_modifier().set_visible(
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
            self.update_background_rendernode();
        }

        self.queue_resize();
        self.queue_draw();
    }
}

/// fmodule for visual debugging
pub mod debug {
    use gtk4::{gdk, graphene, gsk, Snapshot};

    use crate::geometry;

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

    pub const COLOR_STROKE_REGENERATE_FLAG: gdk::RGBA = gdk::RGBA {
        red: 0.9,
        green: 0.0,
        blue: 0.0,
        alpha: 0.3,
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
        zoom: f64,
        snapshot: &Snapshot,
    ) {
        let bounds = graphene::Rect::new(
            bounds.mins[0] as f32,
            bounds.mins[1] as f32,
            (bounds.extents()[0]) as f32,
            (bounds.extents()[1]) as f32,
        );

        let border_width = 1.5;
        let rounded_rect = gsk::RoundedRect::new(
            bounds.scale(zoom as f32, zoom as f32),
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

    pub fn draw_pos(pos: na::Vector2<f64>, color: gdk::RGBA, zoom: f64, snapshot: &Snapshot) {
        snapshot.append_color(
            &color,
            &graphene::Rect::new(
                (zoom * pos[0] - 1.0) as f32,
                (zoom * pos[1] - 1.0) as f32,
                2.0,
                2.0,
            ),
        );
    }

    pub fn draw_fill(
        rect: p2d::bounding_volume::AABB,
        color: gdk::RGBA,
        zoom: f64,
        snapshot: &Snapshot,
    ) {
        snapshot.append_color(
            &color,
            &geometry::aabb_to_graphene_rect(geometry::aabb_scale(rect, zoom)),
        );
    }
}
