pub mod canvaslayout;

mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::{Arc, RwLock};

    use super::canvaslayout::CanvasLayout;
    use super::debug;
    use crate::compose::geometry;
    use crate::config;
    use crate::pens::{PenStyle, Pens};
    use crate::render::Renderer;
    use crate::sheet::Sheet;
    use crate::ui::canvas::ExpandMode;
    use crate::ui::selectionmodifier::SelectionModifier;

    use gtk4::{
        gdk, glib, graphene, gsk, prelude::*, subclass::prelude::*, GestureDrag, GestureStylus,
        PropagationPhase, Snapshot, Widget,
    };
    use gtk4::{AccessibleRole, Adjustment, Scrollable, ScrollablePolicy};

    use once_cell::sync::Lazy;
    use p2d::bounding_volume::{BoundingVolume, AABB};

    #[derive(Debug)]
    pub struct Canvas {
        pub renderer: Arc<RwLock<Renderer>>,

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

        pub pens: Rc<RefCell<Pens>>,
        pub pen_shown: Cell<bool>,
        pub sheet: Rc<RefCell<Sheet>>,
        pub zoom: Cell<f64>,
        pub temporary_zoom: Cell<f64>,
        pub visual_debug: Cell<bool>,
        pub unsaved_changes: Cell<bool>,
        pub empty: Cell<bool>,

        // State that is saved in settings
        pub touch_drawing: Cell<bool>,
        pub expand_mode: Cell<ExpandMode>,
        pub endless_sheet: Cell<bool>,
        pub format_borders: Cell<bool>,
        pub pdf_import_width: Cell<f64>,
        pub pdf_import_as_vector: Cell<bool>,
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
                renderer: Arc::new(RwLock::new(Renderer::default())),
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

                pens: Rc::new(RefCell::new(Pens::default())),
                pen_shown: Cell::new(false),
                sheet: Rc::new(RefCell::new(Sheet::default())),

                zoom: Cell::new(super::Canvas::ZOOM_DEFAULT),
                temporary_zoom: Cell::new(1.0),
                visual_debug: Cell::new(false),
                unsaved_changes: Cell::new(false),
                empty: Cell::new(true),

                touch_drawing: Cell::new(false),
                expand_mode: Cell::new(ExpandMode::default()),
                endless_sheet: Cell::new(true),
                format_borders: Cell::new(true),
                pdf_import_width: Cell::new(super::Canvas::PDF_IMPORT_WIDTH_DEFAULT),
                pdf_import_as_vector: Cell::new(true),
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
                vec![
                    glib::ParamSpecBoolean::new(
                        "pen-shown",
                        "pen-shown",
                        "pen-shown",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    // The zoom of the canvas in relation to the sheet
                    glib::ParamSpecDouble::new(
                        "zoom",
                        "zoom",
                        "zoom",
                        f64::MIN,
                        f64::MAX,
                        super::Canvas::ZOOM_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    // The temporary zoom (on top of the normal zoom)
                    glib::ParamSpecDouble::new(
                        "temporary-zoom",
                        "temporary-zoom",
                        "temporary-zoom",
                        f64::MIN,
                        f64::MAX,
                        1.0,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Visual debugging, which shows bounding boxes, hitboxes, ... (enable in developer action menu)
                    glib::ParamSpecBoolean::new(
                        "visual-debug",
                        "visual-debug",
                        "visual-debug",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
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
                    // expand mode
                    glib::ParamSpecEnum::new(
                        "expand-mode",
                        "expand-mode",
                        "expand-mode",
                        ExpandMode::static_type(),
                        ExpandMode::default() as i32,
                        glib::ParamFlags::READWRITE,
                    ),
                    // format borders
                    glib::ParamSpecBoolean::new(
                        "format-borders",
                        "format-borders",
                        "format-borders",
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
                        super::Canvas::PDF_IMPORT_WIDTH_DEFAULT,
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
                "pen-shown" => self.pen_shown.get().to_value(),
                "zoom" => self.zoom.get().to_value(),
                "temporary-zoom" => self.temporary_zoom.get().to_value(),
                "visual-debug" => self.visual_debug.get().to_value(),
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
                "empty" => self.empty.get().to_value(),
                "hadjustment" => self.hadjustment.borrow().to_value(),
                "vadjustment" => self.vadjustment.borrow().to_value(),
                "hscroll-policy" => self.hscroll_policy.get().to_value(),
                "vscroll-policy" => self.vscroll_policy.get().to_value(),
                "expand-mode" => self.expand_mode.get().to_value(),
                "format-borders" => self.format_borders.get().to_value(),
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
            match pspec.name() {
                "pen-shown" => {
                    let pen_shown = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`.");

                    self.pen_shown.replace(pen_shown);
                }
                "zoom" => {
                    let zoom: f64 = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.")
                        .clamp(super::Canvas::ZOOM_MIN, super::Canvas::ZOOM_MAX);
                    self.zoom.replace(zoom);
                    obj.update_size_autoexpand();

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
                    obj.update_size_autoexpand();

                    obj.queue_resize();
                    obj.queue_draw();
                }
                "visual-debug" => {
                    let visual_debug: bool =
                        value.get().expect("The value needs to be of type `bool`.");
                    self.visual_debug.replace(visual_debug);

                    obj.queue_draw();
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
                "expand-mode" => {
                    let expand_mode = value
                        .get::<ExpandMode>()
                        .expect("The value needs to be of type `ExpandMode`.");

                    self.expand_mode.replace(expand_mode);
                    obj.resize_sheet_to_fit_strokes();
                    obj.return_to_origin_page();
                }
                "format-borders" => {
                    let format_borders = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`.");

                    self.format_borders.replace(format_borders);
                    obj.queue_draw();
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

    impl WidgetImpl for Canvas {
        // request_mode(), measure(), allocate() overrides happen in the CanvasLayout LayoutManager

        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk4::Snapshot) {
            let temporary_zoom = widget.temporary_zoom();
            let zoom = widget.zoom();
            let hadj = widget.hadjustment().unwrap();
            let vadj = widget.vadjustment().unwrap();

            let (clip_x, clip_y, clip_width, clip_height) = if let Some(parent) = widget.parent() {
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
            // Clip everything outside the current view
            snapshot.push_clip(&graphene::Rect::new(
                clip_x,
                clip_y,
                clip_width,
                clip_height,
            ));

            snapshot.save();

            snapshot.translate(&graphene::Point::new(
                (-hadj.value()) as f32,
                (-vadj.value()) as f32,
            ));

            // From here in scaled sheet coordinate space
            snapshot.scale(temporary_zoom as f32, temporary_zoom as f32);

            self.draw_shadow(
                geometry::aabb_scale(widget.sheet().borrow().bounds(), zoom),
                f64::from(super::Canvas::SHADOW_WIDTH) * zoom,
                snapshot,
            );

            // Clip sheet and stroke drawing to sheet bounds
            snapshot.push_clip(&geometry::aabb_to_graphene_rect(geometry::aabb_scale(
                widget.sheet().borrow().bounds(),
                zoom,
            )));

            self.sheet
                .borrow()
                .draw(zoom, snapshot, widget.format_borders());

            self.sheet
                .borrow()
                .strokes_state
                .draw_strokes(snapshot, Some(widget.viewport_in_sheet_coords()));

            snapshot.pop();

            self.sheet
                .borrow()
                .strokes_state
                .draw_selection(zoom, snapshot);

            if self.pen_shown.get() {
                if let Err(e) = self.pens.borrow().current_pen.draw(widget, snapshot) {
                    log::debug!("pens draw() failed in canvas snapshot() with Err {}", e);
                };
            }

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
        pub fn draw_shadow(&self, bounds: AABB, width: f64, snapshot: &Snapshot) {
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
                &super::Canvas::SHADOW_COLOR.to_gdk(),
                0.0,
                0.0,
                (width / 2.0) as f32,
                (width / 2.0) as f32,
            );
        }

        // Draw bounds, positions, .. for visual debugging purposes
        fn draw_debug(&self, widget: &super::Canvas, snapshot: &Snapshot, zoom: f64) {
            if self.pen_shown.get() {
                let current_pen = self.pens.borrow().current_pen;

                match current_pen {
                    PenStyle::EraserStyle => {
                        if let Some(current_input) = self.pens.borrow().eraser.current_input {
                            debug::draw_pos(
                                current_input.pos(),
                                debug::COLOR_POS_ALT,
                                zoom,
                                snapshot,
                            );
                        }
                    }
                    PenStyle::SelectorStyle => {
                        if let Some(bounds) = self.pens.borrow().selector.gen_bounds() {
                            debug::draw_bounds(
                                bounds,
                                debug::COLOR_SELECTOR_BOUNDS,
                                zoom,
                                snapshot,
                            );
                        }
                    }
                    PenStyle::MarkerStyle
                    | PenStyle::BrushStyle
                    | PenStyle::ShaperStyle
                    | PenStyle::ToolsStyle => {}
                }
            }

            debug::draw_bounds(
                self.sheet.borrow().bounds(),
                debug::COLOR_SHEET_BOUNDS,
                zoom,
                snapshot,
            );

            let viewport = widget.viewport_in_sheet_coords().tightened(1.0);
            debug::draw_bounds(viewport, debug::COLOR_STROKE_BOUNDS, zoom, snapshot);

            self.sheet.borrow().strokes_state.draw_debug(zoom, snapshot);
        }
    }
}

use crate::compose::color::Color;
use crate::compose::geometry;
use crate::input;
use crate::render::Renderer;
use crate::strokes::strokestyle::InputData;
use crate::ui::selectionmodifier::SelectionModifier;
use crate::{app::RnoteApp, pens::Pens, render, sheet::Sheet, ui::appwindow::RnoteAppWindow};

use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time;

use gtk4::{gdk, glib, glib::clone, prelude::*, subclass::prelude::*};
use gtk4::{gio, Adjustment, DropTarget, EventSequenceState, PropagationPhase, Snapshot, Widget};
use p2d::bounding_volume::{BoundingVolume, AABB};

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, glib::Enum, FromPrimitive, ToPrimitive,
)]
#[enum_type(name = "ExpandMode")]
#[repr(i32)]
pub enum ExpandMode {
    #[enum_value(name = "FixedSize", nick = "fixed-size")]
    FixedSize = 0,
    #[enum_value(name = "EndlessVertical", nick = "endless-vertical")]
    EndlessVertical = 1,
    #[enum_value(name = "Infinite", nick = "infinite")]
    Infinite = 2,
}

impl Default for ExpandMode {
    fn default() -> Self {
        Self::FixedSize
    }
}

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<imp::Canvas>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Scrollable;
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    pub const ZOOM_MIN: f64 = 0.2;
    pub const ZOOM_MAX: f64 = 8.0;
    pub const ZOOM_DEFAULT: f64 = 1.0;
    pub const SHADOW_WIDTH: f64 = 30.0;
    pub const SHADOW_COLOR: Color = Color {
        r: 0.1,
        g: 0.1,
        b: 0.1,
        a: 0.3,
    };

    /// The zoom amount when activating the zoom-in / zoom-out action
    pub const ZOOM_ACTION_DELTA: f64 = 0.1;
    pub const ZOOM_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(300);
    // The default width of imported PDF's in percentage to the sheet width
    pub const PDF_IMPORT_WIDTH_DEFAULT: f64 = 50.0;

    pub fn new() -> Self {
        let canvas: Canvas = glib::Object::new(&[]).expect("Failed to create Canvas");

        canvas
    }

    pub fn renderer(&self) -> Arc<RwLock<Renderer>> {
        Arc::clone(&self.imp().renderer)
    }

    pub fn cursor(&self) -> gdk::Cursor {
        self.imp().cursor.clone()
    }

    pub fn motion_cursor(&self) -> gdk::Cursor {
        self.imp().motion_cursor.clone()
    }

    /// Only change the pens state in actions to avoid nested mutable borrows!
    pub fn pens(&self) -> Rc<RefCell<Pens>> {
        self.imp().pens.clone()
    }

    pub fn pen_shown(&self) -> bool {
        self.property::<bool>("pen-shown")
    }

    pub fn set_pen_shown(&self, pen_shown: bool) {
        self.set_property("pen-shown", pen_shown);
    }

    /// Only change the sheet state in actions to avoid nested mutable borrows!
    pub fn sheet(&self) -> Rc<RefCell<Sheet>> {
        Rc::clone(&imp::Canvas::from_instance(self).sheet)
    }

    pub fn expand_mode(&self) -> ExpandMode {
        self.property::<ExpandMode>("expand-mode")
    }

    pub fn set_expand_mode(&self, expand_mode: ExpandMode) {
        self.set_property("expand-mode", expand_mode.to_value());
    }

    pub fn format_borders(&self) -> bool {
        self.property::<bool>("format-borders")
    }

    pub fn set_format_borders(&self, format_borders: bool) {
        self.set_property("format-borders", format_borders.to_value());
    }

    pub fn zoom(&self) -> f64 {
        self.property::<f64>("zoom")
    }

    fn set_zoom(&self, zoom: f64) {
        self.set_property("zoom", zoom.to_value());
    }

    pub fn temporary_zoom(&self) -> f64 {
        self.property::<f64>("temporary-zoom")
    }

    fn set_temporary_zoom(&self, temporary_zoom: f64) {
        self.set_property("temporary-zoom", temporary_zoom.to_value());
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

    pub fn visual_debug(&self) -> bool {
        self.property::<bool>("visual-debug")
    }

    pub fn set_visual_debug(&self, visual_debug: bool) {
        self.set_property("visual-debug", visual_debug.to_value());
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
                     canvas.update_size_infinite_mode();

                    canvas.update_background_rendernode(false);
                    canvas.regenerate_content(false, true);
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
                     canvas.update_size_infinite_mode();

                    canvas.update_background_rendernode(false);
                    canvas.regenerate_content(false, true);
                }));
            self_.vadjustment_signal.replace(Some(signal_id));
        }
        self_.vadjustment.replace(adj);
    }

    pub fn selection_modifier(&self) -> SelectionModifier {
        imp::Canvas::from_instance(self).selection_modifier.clone()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.hadjustment()
            .unwrap()
            .connect_value_changed(clone!(@weak appwindow => move |_hadj| {
                if appwindow.canvas().expand_mode() == ExpandMode::Infinite {
                    let viewport = appwindow.canvas().viewport_in_sheet_coords();
                    let center_bounds = geometry::aabb_expand(
                        AABB::new(
                            na::point![0.0, 0.0],
                            na::point![appwindow.canvas().sheet().borrow().format.width, appwindow.canvas().sheet().borrow().format.width]),
                        // Expand to a few format sizes around the center
                        na::vector![
                            2.0 * appwindow.canvas().sheet().borrow().format.width,
                            2.0 * appwindow.canvas().sheet().borrow().format.height]);

                    if !viewport.intersects(&center_bounds) {
                        appwindow.canvas().show_return_to_center_toast(&appwindow)
                    } else {
                        appwindow.canvas().dismiss_return_to_center_toast();
                    }
                }
            }));

        self.vadjustment()
            .unwrap()
            .connect_value_changed(clone!(@weak appwindow => move |_hadj| {
                if appwindow.canvas().expand_mode() == ExpandMode::Infinite {
                    let viewport = appwindow.canvas().viewport_in_sheet_coords();
                    let center_bounds = geometry::aabb_expand(
                        AABB::new(
                            na::point![0.0, 0.0],
                            na::point![appwindow.canvas().sheet().borrow().format.width, appwindow.canvas().sheet().borrow().format.width]),
                        // Expand to a few format sizes around the center
                        na::vector![
                            2.0 * appwindow.canvas().sheet().borrow().format.width,
                            2.0 * appwindow.canvas().sheet().borrow().format.height]);

                    if !viewport.intersects(&center_bounds) {
                        appwindow.canvas().show_return_to_center_toast(&appwindow)
                    } else {
                        appwindow.canvas().dismiss_return_to_center_toast();
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

        self.bind_property(
            "expand-mode",
            &appwindow.mainheader().pageedit_revealer(),
            "reveal-child",
        )
        .transform_to(move |_, value| match value.get::<ExpandMode>().unwrap() {
            ExpandMode::FixedSize => Some(true.to_value()),
            ExpandMode::EndlessVertical | ExpandMode::Infinite => Some(false.to_value()),
        })
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
        .build();

        // Stylus Drawing
        self.imp().stylus_drawing_gesture.connect_down(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            if let Some(device_tool) = stylus_drawing_gesture.device_tool() {
                stylus_drawing_gesture.set_state(EventSequenceState::Claimed);

                // Disable backlog, only allowed in motion signal handler
                let mut data_entries = input::retreive_stylus_inputdata(stylus_drawing_gesture, false, x, y);
                input::transform_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]));

                match device_tool.tool_type() {
                    gdk::DeviceToolType::Pen => { },
                    gdk::DeviceToolType::Eraser => {

                    gtk4::prelude::ActionGroupExt::activate_action(
                        &appwindow,
                        "tmperaser",
                        Some(&true.to_variant())
                    );
                    }
                    _ => { return; },
                }

                input::process_peninput_start(&appwindow, data_entries);
            }
        }));

        self.imp().stylus_drawing_gesture.connect_motion(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            // backlog doesn't provide time equidistant inputdata and makes line look worse, so its disabled for now
            let mut data_entries: VecDeque<InputData> = input::retreive_stylus_inputdata(stylus_drawing_gesture, false, x, y);
            input::transform_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]));
            input::process_peninput_motion(&appwindow, data_entries);
        }));

        self.imp().stylus_drawing_gesture.connect_up(
            clone!(@weak self as canvas, @weak appwindow => move |gesture_stylus,x,y| {
                let mut data_entries = input::retreive_stylus_inputdata(gesture_stylus, false, x, y);
                input::transform_inputdata(canvas.zoom(), &mut data_entries, na::vector![0.0, 0.0]);

                input::process_peninput_end(&appwindow, data_entries);
            }),
        );

        // Mouse drawing
        self.imp().mouse_drawing_gesture.connect_drag_begin(
            clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
                if let Some(event) = mouse_drawing_gesture.current_event() {
                    // Guard not to handle touch events
                    let event_type = event.event_type();
                    if event.is_pointer_emulated()
                        || event_type == gdk::EventType::TouchBegin
                        || event_type == gdk::EventType::TouchUpdate
                        ||event_type == gdk::EventType::TouchEnd
                        || event_type == gdk::EventType::TouchCancel {
                        return;
                    }
                    mouse_drawing_gesture.set_state(EventSequenceState::Claimed);

                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::transform_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]));

                    input::process_peninput_start(&appwindow, data_entries);
                }
            }),
        );

        self.imp().mouse_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            if let Some(event) = mouse_drawing_gesture.current_event() {
                // Guard not to handle touch events
                let event_type = event.event_type();
                if event.is_pointer_emulated()
                    || event_type == gdk::EventType::TouchBegin
                    || event_type == gdk::EventType::TouchUpdate
                    ||event_type == gdk::EventType::TouchEnd
                    || event_type == gdk::EventType::TouchCancel {
                    return;
                }

                if let Some(start_point) = mouse_drawing_gesture.start_point() {
                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::transform_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]));

                    input::process_peninput_motion(&appwindow, data_entries);
                }
            }
        }));

        self.imp().mouse_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {
            if let Some(event) = mouse_drawing_gesture.current_event() {
                // Guard not to handle touch events
                let event_type = event.event_type();
                if event.is_pointer_emulated()
                    || event_type == gdk::EventType::TouchBegin
                    || event_type == gdk::EventType::TouchUpdate
                    ||event_type == gdk::EventType::TouchEnd
                    || event_type == gdk::EventType::TouchCancel {
                    return;
                }

                if let Some(start_point) = mouse_drawing_gesture.start_point() {
                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::transform_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]));

                    input::process_peninput_end(&appwindow, data_entries);
                }
            }
        }));

        // Touch drawing
        self.imp().touch_drawing_gesture.connect_drag_begin(
            clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
                touch_drawing_gesture.set_state(EventSequenceState::Claimed);

                let mut data_entries = input::retreive_pointer_inputdata(x, y);
                input::transform_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]));

                input::process_peninput_start(&appwindow, data_entries);
            }),
        );

        self.imp().touch_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_inputdata(x, y);
                input::transform_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]));

                input::process_peninput_motion(&appwindow, data_entries);
            }
        }));

        self.imp().touch_drawing_gesture.connect_drag_end(
            clone!(@weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
                if let Some(start_point) = touch_drawing_gesture.start_point() {
                    let mut data_entries = input::retreive_pointer_inputdata(x, y);
                    input::transform_inputdata(canvas.zoom(), &mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]));
                    input::process_peninput_end(&appwindow, data_entries);
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
    pub fn bounds(&self) -> AABB {
        AABB::new(
            na::point![0.0, 0.0],
            na::point![f64::from(self.width()), f64::from(self.height())],
        )
    }

    /// The bounds of the sheet in the coordinate space of the canvas
    pub fn sheet_bounds_in_canvas_coords(&self) -> AABB {
        geometry::aabb_scale(self.sheet().borrow().bounds(), self.total_zoom())
    }

    /// transforming a AABB in canvas coordinate space into sheet coordinate space
    pub fn transform_canvas_aabb_to_sheet(&self, aabb: AABB) -> AABB {
        let mins = na::Point2::from(self.transform_canvas_coords_to_sheet_coords(aabb.mins.coords));
        let maxs = na::Point2::from(self.transform_canvas_coords_to_sheet_coords(aabb.maxs.coords));
        AABB::new(mins, maxs)
    }

    /// transforming coordinates in canvas coordinate space into sheet coordinate space
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
    }

    /// transforming a AABB in sheet coordinate space into canvas coordinate space
    pub fn transform_sheet_aabb_to_canvas(&self, aabb: AABB) -> AABB {
        let mins = na::Point2::from(self.transform_sheet_coords_to_canvas_coords(aabb.mins.coords));
        let maxs = na::Point2::from(self.transform_sheet_coords_to_canvas_coords(aabb.maxs.coords));
        AABB::new(mins, maxs)
    }

    /// transforming coordinates in sheet coordinate space into canvas coordinate space
    pub fn transform_sheet_coords_to_canvas_coords(
        &self,
        sheet_coords: na::Vector2<f64>,
    ) -> na::Vector2<f64> {
        let total_zoom = self.total_zoom();

        sheet_coords * total_zoom
            - na::vector![
                self.hadjustment().unwrap().value(),
                self.vadjustment().unwrap().value()
            ]
    }

    /// The view of the parent scroller onto the Canvas
    pub fn viewport(&self) -> AABB {
        let parent = self.parent().unwrap();
        let (parent_width, parent_height) = (f64::from(parent.width()), f64::from(parent.height()));
        let (parent_offset_x, parent_offset_y) = self
            .translate_coordinates(&parent, 0.0, 0.0)
            .unwrap_or((0.0, 0.0));

        let (x, y) = (
            self.hadjustment().unwrap().value() - parent_offset_x,
            self.vadjustment().unwrap().value() - parent_offset_y,
        );

        AABB::new(
            na::point![x, y],
            na::point![x + parent_width, y + parent_height],
        )
    }

    /// The viewport transformed to match the coordinate space of the sheet
    pub fn viewport_in_sheet_coords(&self) -> AABB {
        let viewport = self.viewport();
        let total_zoom = self.total_zoom();

        geometry::aabb_scale(viewport, 1.0 / total_zoom)
    }

    /// Called when sheet should resize to the format
    pub fn resize_sheet_to_fit_strokes(&self) {
        match self.expand_mode() {
            ExpandMode::FixedSize => {
                self.update_size_fixed_size_mode();
            }
            ExpandMode::EndlessVertical => {
                self.update_size_endless_vertical_mode();
            }
            ExpandMode::Infinite => {
                self.update_size_infinite_mode();
            }
        }
    }

    /// resize the sheet when in autoexpanding expand modes. called e.g. when finishing a new stroke
    pub fn update_size_autoexpand(&self) {
        match self.expand_mode() {
            ExpandMode::FixedSize => {
                // Does not auto update size in fixed size mode, use update_size_to_strokes for it.
            }
            ExpandMode::EndlessVertical => {
                self.update_size_endless_vertical_mode();
            }
            ExpandMode::Infinite => {
                self.update_size_infinite_mode();
            }
        }
    }

    /// updates size in fixed size mode to fit all strokes and will resize to full pages
    pub fn update_size_fixed_size_mode(&self) {
        if self.expand_mode() == ExpandMode::FixedSize {
            let format_height = self.sheet().borrow().format.height;

            let new_width = self.sheet().borrow().format.width;
            // +1.0 because then 'fraction'.ceil() is at least 1
            let new_height = (f64::from(self.sheet().borrow().strokes_state.calc_height() + 1.0)
                / f64::from(format_height))
            .ceil()
                * format_height;

            self.sheet().borrow_mut().x = 0.0;
            self.sheet().borrow_mut().y = 0.0;
            self.sheet().borrow_mut().width = new_width;
            self.sheet().borrow_mut().height = new_height;

            self.update_background_rendernode(true);
        }
    }

    /// updates size in endless vertical mode to fit all strokes
    pub fn update_size_endless_vertical_mode(&self) {
        if self.expand_mode() == ExpandMode::EndlessVertical {
            let padding_bottom = self.sheet().borrow().format.height;
            let new_height = self.sheet().borrow().strokes_state.calc_height() + padding_bottom;
            let new_width = self.sheet().borrow().format.width;

            self.sheet().borrow_mut().x = 0.0;
            self.sheet().borrow_mut().y = 0.0;
            self.sheet().borrow_mut().width = new_width;
            self.sheet().borrow_mut().height = new_height;

            self.update_background_rendernode(true);
        }
    }

    /// Called when the sheet should be resized to fit all strokes and the viewport in infinite expand mode
    pub fn update_size_infinite_mode(&self) {
        if self.expand_mode() == ExpandMode::Infinite {
            let padding_horizontal = self.sheet().borrow().format.width;
            let padding_vertical = self.sheet().borrow().format.height;

            let mut new_bounds = self
                .sheet()
                .borrow()
                .bounds()
                .merged(&self.viewport_in_sheet_coords());

            let keys = self.sheet().borrow().strokes_state.keys_sorted_chrono();

            // Expand by stroke bounds
            if let Some(stroke_bounds) =
                self.sheet()
                    .borrow()
                    .strokes_state
                    .gen_bounds(&keys)
                    .map(|stroke_bounds| {
                        geometry::aabb_expand(
                            stroke_bounds,
                            na::vector![padding_horizontal, padding_vertical],
                        )
                    })
            {
                new_bounds.merge(&stroke_bounds)
            }

            self.sheet().borrow_mut().x = new_bounds.mins[0];
            self.sheet().borrow_mut().y = new_bounds.mins[1];
            self.sheet().borrow_mut().width = new_bounds.extents()[0];
            self.sheet().borrow_mut().height = new_bounds.extents()[1];

            self.update_background_rendernode(true);
        }
    }

    // Resizing the sheet by merging with new desired adjustment values. Will expand to fit the strokes if the desired adj values are not large enough
    pub fn resize_sheet_infinite_mode_new_adjs(&self, desired_adjs: na::Vector2<f64>) {
        if self.expand_mode() == ExpandMode::Infinite {
            let total_zoom = self.total_zoom();
            let scroller_width = f64::from(self.parent().unwrap().width());
            let scroller_height = f64::from(self.parent().unwrap().height());

            let new_x = self
                .sheet()
                .borrow_mut()
                .x
                .min(desired_adjs[0] / total_zoom);
            let new_y = self
                .sheet()
                .borrow_mut()
                .y
                .min(desired_adjs[1] / total_zoom);

            let new_width = self
                .sheet()
                .borrow()
                .width
                .max((desired_adjs[0] + scroller_width) / total_zoom - self.sheet().borrow().x);
            let new_height =
                self.sheet().borrow().height.max(
                    (desired_adjs[1] + scroller_height) / total_zoom - self.sheet().borrow().y,
                );

            self.sheet().borrow_mut().x = new_x;
            self.sheet().borrow_mut().y = new_y;
            self.sheet().borrow_mut().width = new_width;
            self.sheet().borrow_mut().height = new_height;

            // updating size to fit strokes
            self.update_size_infinite_mode();
        }
    }

    /// The point parameter has the coordinate space of the sheet!
    pub fn center_around_coord_on_sheet(&self, coord: na::Vector2<f64>) {
        let (parent_width, parent_height) = (
            f64::from(self.parent().unwrap().width()),
            f64::from(self.parent().unwrap().height()),
        );
        let total_zoom = self.total_zoom();

        let new_adjs = na::vector![
            ((coord[0]) * total_zoom) - parent_width * 0.5,
            ((coord[1]) * total_zoom) - parent_height * 0.5
        ];
        self.resize_sheet_infinite_mode_new_adjs(new_adjs);

        self.hadjustment().unwrap().set_value(new_adjs[0]);
        self.vadjustment().unwrap().set_value(new_adjs[1]);
        self.queue_resize();
    }

    /// Centering the view to the first page
    pub fn return_to_origin_page(&self) {
        let total_zoom = self.total_zoom();
        let center = na::vector![
            self.sheet().borrow().format.width / 2.0,
            (f64::from(self.parent().unwrap().height()) / 2.0 - Self::SHADOW_WIDTH) / total_zoom
        ];
        self.center_around_coord_on_sheet(center);
    }

    /// Zoom temporarily to a new zoom, not regenerating the contents while doing it.
    /// To zoom and regenerate the content and reset the temporary zoom, use zoom_to().
    pub fn zoom_temporarily_to(&self, temp_zoom: f64) {
        self.set_temporary_zoom(temp_zoom / self.zoom());

        self.update_size_infinite_mode();
    }

    /// zooms and regenerates the canvas and its contents to a new zoom
    pub fn zoom_to(&self, zoom: f64) {
        // Remove the timeout if existss
        if let Some(zoom_timeout_id) = self.imp().zoom_timeout_id.take() {
            zoom_timeout_id.remove();
        }
        self.set_temporary_zoom(1.0);
        self.set_zoom(zoom);

        self.sheet()
            .borrow_mut()
            .strokes_state
            .reset_regenerate_flag_all_strokes();

        self.update_size_infinite_mode();

        // update rendernodes to new zoom until threaded regeneration is finished
        self.update_content_rendernodes(false);

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
        if let Some(zoom_timeout_id) = self.imp().zoom_timeout_id.take() {
            zoom_timeout_id.remove();
        }

        self.zoom_temporarily_to(zoom);

        self.imp()
            .zoom_timeout_id
            .borrow_mut()
            .replace(glib::source::timeout_add_local_once(
                timeout_time,
                clone!(@weak self as canvas => move || {

                    canvas.zoom_to(zoom);

                    // Removing the timeout id
                    let mut zoom_timeout_id = canvas.imp().zoom_timeout_id.borrow_mut();
                    if let Some(zoom_timeout_id) = zoom_timeout_id.take() {
                        zoom_timeout_id.remove();
                    }
                }),
            ));
    }

    /// Update rendernodes of the background. Used when the background itself did not change, but for example the format
    pub fn update_background_rendernode(&self, redraw: bool) {
        let sheet_bounds = self.sheet().borrow().bounds();

        self.sheet()
        .borrow_mut()
        .background
        .update_rendernode(self.zoom(), sheet_bounds, Some(self.viewport_in_sheet_coords()) ).unwrap_or_else(|e| {
            log::error!("failed to update rendernode for background in update_background_rendernode() with Err {}", e);
        });

        if redraw {
            self.queue_resize();
            self.queue_draw();
        }
    }

    /// Update rendernodes of the background. Used when sheet size, but not zoom changed
    pub fn update_content_rendernodes(&self, redraw: bool) {
        self.sheet()
            .borrow_mut()
            .strokes_state
            .update_rendernodes_current_zoom(self.zoom());

        if redraw {
            self.queue_resize();
            self.queue_draw();
        }
    }

    /// regenerating the background image and rendernode.
    /// use for example when changing the background pattern or zoom
    pub fn regenerate_background(&self, redraw: bool) {
        let background_bounds = self.sheet().borrow().bounds();
        let total_zoom = self.total_zoom();

        if let Err(e) = self.sheet().borrow_mut().background.regenerate_background(
            total_zoom,
            background_bounds,
            Some(self.viewport_in_sheet_coords()),
            self.renderer(),
        ) {
            log::error!("failed to regenerate background, {}", e)
        };

        if redraw {
            self.queue_resize();
            self.queue_draw();
        }
    }

    /// regenerate the rendernodes of the canvas content. force_regenerate regenerate all images and rendernodes from scratch. redraw: queue canvas redrawing
    pub fn regenerate_content(&self, force_regenerate: bool, redraw: bool) {
        let viewport = self.viewport_in_sheet_coords();

        self.sheet()
            .borrow_mut()
            .strokes_state
            .regenerate_rendering_current_view_threaded(
                Some(viewport),
                force_regenerate,
                self.renderer(),
                self.zoom(),
            );

        if redraw {
            self.queue_resize();
            self.queue_draw();
        }
    }

    /// Captures the current view of the canvas as a gdk::Texture
    pub fn current_view_as_texture(&self) -> Option<gdk::Texture> {
        let snapshot = Snapshot::new();

        self.selection_modifier().set_visible(false);

        self.imp().snapshot(self, &snapshot);

        let rendernode = snapshot.to_node()?;
        let texture = render::rendernode_to_texture(self.upcast_ref::<Widget>(), &rendernode, None)
            .unwrap_or_else(|e| {
                log::error!(
                    "rendernode_to_texture() in current_content_as_texture() failed with Err {}",
                    e
                );
                None
            })?;

        self.selection_modifier().update_state(self);

        Some(texture)
    }

    pub fn show_return_to_center_toast(&self, appwindow: &RnoteAppWindow) {
        let return_to_center_toast_is_some = self.imp().return_to_center_toast.borrow().is_some();

        if !return_to_center_toast_is_some {
            let return_to_center_toast = adw::Toast::builder()
                .title("Return to origin")
                .timeout(0)
                .button_label("Return")
                .priority(adw::ToastPriority::High)
                .action_name("win.return-origin-page")
                .build();

            return_to_center_toast.connect_dismissed(
                clone!(@weak self as canvas => move |_toast| {
                    canvas.imp().return_to_center_toast.borrow_mut().take();
                }),
            );

            appwindow.toast_overlay().add_toast(&return_to_center_toast);
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

/// module for visual debugging
pub mod debug {
    use gtk4::{graphene, gsk, Snapshot};
    use p2d::bounding_volume::AABB;

    use crate::compose::color::Color;
    use crate::compose::geometry;

    pub const COLOR_POS: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const COLOR_POS_ALT: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const COLOR_STROKE_HITBOX: Color = Color {
        r: 0.0,
        g: 0.8,
        b: 0.2,
        a: 0.5,
    };
    pub const COLOR_STROKE_BOUNDS: Color = Color {
        r: 0.0,
        g: 0.8,
        b: 0.8,
        a: 1.0,
    };
    pub const COLOR_STROKE_REGENERATE_FLAG: Color = Color {
        r: 0.9,
        g: 0.0,
        b: 0.8,
        a: 0.3,
    };
    pub const COLOR_SELECTOR_BOUNDS: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.8,
        a: 1.0,
    };
    pub const COLOR_SHEET_BOUNDS: Color = Color {
        r: 0.8,
        g: 0.0,
        b: 0.8,
        a: 1.0,
    };

    pub fn draw_bounds(bounds: AABB, color: Color, zoom: f64, snapshot: &Snapshot) {
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
            &[
                color.to_gdk(),
                color.to_gdk(),
                color.to_gdk(),
                color.to_gdk(),
            ],
        )
    }

    pub fn draw_pos(pos: na::Vector2<f64>, color: Color, zoom: f64, snapshot: &Snapshot) {
        snapshot.append_color(
            &color.to_gdk(),
            &graphene::Rect::new(
                (zoom * pos[0] - 1.0) as f32,
                (zoom * pos[1] - 1.0) as f32,
                2.0,
                2.0,
            ),
        );
    }

    pub fn draw_fill(rect: AABB, color: Color, zoom: f64, snapshot: &Snapshot) {
        snapshot.append_color(
            &color.to_gdk(),
            &geometry::aabb_to_graphene_rect(geometry::aabb_scale(rect, zoom)),
        );
    }
}
