pub mod canvaslayout;
pub mod input;

mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::{Arc, RwLock};

    use super::canvaslayout::CanvasLayout;
    use crate::canvas::ExpandMode;
    use crate::config;
    use crate::selectionmodifier::SelectionModifier;
    use rnote_engine::compose::geometry::AABBHelpers;
    use rnote_engine::pens::{PenStyle, Pens};
    use rnote_engine::render::Renderer;
    use rnote_engine::sheet::Sheet;
    use rnote_engine::strokesstate::render_comp::visual_debug;

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
                "zoom" => {
                    let zoom: f64 = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.")
                        .clamp(super::Canvas::ZOOM_MIN, super::Canvas::ZOOM_MAX);
                    self.zoom.replace(zoom);

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

                    obj.return_to_origin_page();
                    obj.resize_sheet_to_fit_strokes();
                    obj.update_background_rendernode(false);
                    obj.regenerate_content(false, true)
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
            let adj_values = widget.adj_values();

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
                -adj_values[0] as f32,
                -adj_values[1] as f32,
            ));

            // From here in scaled sheet coordinate space
            snapshot.scale(temporary_zoom as f32, temporary_zoom as f32);

            self.draw_shadow(
                widget
                    .sheet()
                    .borrow()
                    .bounds()
                    .scale(na::Vector2::from_element(zoom)),
                f64::from(super::Canvas::SHADOW_WIDTH) * zoom,
                snapshot,
            );

            // Clip sheet and stroke drawing to sheet bounds
            snapshot.push_clip(
                &widget
                    .sheet()
                    .borrow()
                    .bounds()
                    .scale(na::Vector2::from_element(zoom))
                    .to_graphene_rect(),
            );

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

            if let Err(e) = widget.pens().borrow().draw(
                snapshot,
                &*widget.sheet().borrow(),
                Some(widget.viewport_in_sheet_coords()),
                widget.zoom(),
                widget.renderer(),
            ) {
                log::debug!("pens draw() failed in canvas snapshot() with Err {}", e);
            };

            if self.visual_debug.get() {
                self.draw_debug(widget, snapshot);
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
                bounds.to_graphene_rect(),
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
        fn draw_debug(&self, widget: &super::Canvas, snapshot: &Snapshot) {
            let zoom = widget.zoom();
            let pen_shown = self.pens.borrow().pen_shown();

            if pen_shown {
                let current_pen_style = self.pens.borrow().style_w_override();

                match current_pen_style {
                    PenStyle::EraserStyle => {
                        if let Some(current_input) = self.pens.borrow().eraser.current_input {
                            visual_debug::draw_pos(
                                current_input.pos(),
                                visual_debug::COLOR_POS_ALT,
                                zoom,
                                snapshot,
                            );
                        }
                    }
                    PenStyle::SelectorStyle => {
                        if let Some(bounds) = self.pens.borrow().selector.gen_bounds() {
                            visual_debug::draw_bounds(
                                bounds,
                                visual_debug::COLOR_SELECTOR_BOUNDS,
                                zoom,
                                snapshot,
                            );
                        }
                    }
                    PenStyle::BrushStyle | PenStyle::ShaperStyle | PenStyle::ToolsStyle => {}
                }
            }

            visual_debug::draw_bounds(
                self.sheet.borrow().bounds(),
                visual_debug::COLOR_SHEET_BOUNDS,
                zoom,
                snapshot,
            );

            let viewport = widget.viewport_in_sheet_coords().tightened(1.0);
            visual_debug::draw_bounds(viewport, visual_debug::COLOR_STROKE_BOUNDS, zoom, snapshot);

            self.sheet.borrow().strokes_state.draw_debug(zoom, snapshot);
        }
    }
}

use crate::selectionmodifier::SelectionModifier;
use crate::{app::RnoteApp, appwindow::RnoteAppWindow};
use futures::StreamExt;
use rnote_engine::compose::color::Color;
use rnote_engine::compose::geometry::AABBHelpers;
use rnote_engine::pens::Pens;
use rnote_engine::render::{self, Renderer};
use rnote_engine::sheet::Sheet;
use rnote_engine::strokes::inputdata::InputData;

use gettextrs::gettext;
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

        if let Some(ref hadjustment) = adj {
            let signal_id = hadjustment.connect_value_changed(
                clone!(@weak self as canvas => move |_hadjustment| {
                    canvas.update_background_rendernode(false);
                    canvas.regenerate_content(false, true);
                }),
            );
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

        if let Some(ref vadjustment) = adj {
            let signal_id = vadjustment.connect_value_changed(
                clone!(@weak self as canvas => move |_vadjustment| {
                    canvas.update_background_rendernode(false);
                    canvas.regenerate_content(false, true);
                }),
            );
            self_.vadjustment_signal.replace(Some(signal_id));
        }
        self_.vadjustment.replace(adj);
    }

    pub fn selection_modifier(&self) -> SelectionModifier {
        imp::Canvas::from_instance(self).selection_modifier.clone()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        // receive strokes_state tasks
        let main_cx = glib::MainContext::default();

        main_cx.spawn_local(clone!(@strong self as canvas, @strong appwindow => async move {
            let mut task_rx = canvas.sheet().borrow_mut().strokes_state.tasks_rx.take().unwrap();

            loop {
                if let Some(task) = task_rx.next().await {
                    let surface_flags = canvas.sheet().borrow_mut().strokes_state.process_received_task(task, canvas.zoom(), canvas.renderer());
                    appwindow.handle_surface_flags(surface_flags);
                }
            }
        }));

        self.hadjustment()
            .unwrap()
            .connect_value_changed(clone!(@weak appwindow => move |_hadj| {
                if appwindow.canvas().expand_mode() == ExpandMode::Infinite {
                    let viewport = appwindow.canvas().viewport_in_sheet_coords();
                    let origin_page_bounds =
                        AABB::new(
                            na::point![0.0, 0.0],
                            na::point![appwindow.canvas().sheet().borrow().format.width, appwindow.canvas().sheet().borrow().format.width]).expand(
                        // Expand to a few format sizes around the origin page
                        na::vector![
                            2.0 * appwindow.canvas().sheet().borrow().format.width,
                            2.0 * appwindow.canvas().sheet().borrow().format.height]);

                    if !viewport.intersects(&origin_page_bounds) {
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
                    let origin_page_bounds =
                        AABB::new(
                            na::point![0.0, 0.0],
                            na::point![appwindow.canvas().sheet().borrow().format.width, appwindow.canvas().sheet().borrow().format.width]).expand(
                        // Expand to a few format sizes around the origin page
                        na::vector![
                            2.0 * appwindow.canvas().sheet().borrow().format.width,
                            2.0 * appwindow.canvas().sheet().borrow().format.height]);

                    if !viewport.intersects(&origin_page_bounds) {
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
            log::trace!("stylus_drawing_gesture down");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            // filter out invalid stylus input
            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }

            let mut data_entries = input::retreive_stylus_inputdata(stylus_drawing_gesture, x, y);
            input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]), canvas.zoom());

            let shortcut_key = input::retreive_stylus_shortcut_key(&stylus_drawing_gesture);
            stylus_drawing_gesture.set_state(EventSequenceState::Claimed);

            input::process_pen_down(data_entries, &appwindow, shortcut_key);
        }));

        self.imp().stylus_drawing_gesture.connect_motion(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture, x, y| {
            log::trace!("stylus_drawing_gesture motion");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            // filter out invalid stylus input
            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }

            let mut data_entries: VecDeque<InputData> = input::retreive_stylus_inputdata(stylus_drawing_gesture, x, y);
            input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]), canvas.zoom());
            input::process_pen_motion(data_entries, &appwindow);
        }));

        self.imp().stylus_drawing_gesture.connect_up(clone!(@weak self as canvas, @weak appwindow => move |stylus_drawing_gesture,x,y| {
            log::trace!("stylus_drawing_gesture up");
            //input::debug_stylus_gesture(&stylus_drawing_gesture);

            // filter out invalid stylus input
            if input::filter_stylus_input(&stylus_drawing_gesture) { return; }

            let mut data_entries = input::retreive_stylus_inputdata(stylus_drawing_gesture, x, y);
            input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]), canvas.zoom());
            input::process_pen_up(data_entries, &appwindow);
        }));

        // Mouse drawing
        self.imp().mouse_drawing_gesture.connect_drag_begin(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            log::trace!("mouse_drawing_gesture begin");
            //input::debug_drag_gesture(&mouse_drawing_gesture);
            // filter out invalid point input
            if input::filter_mouse_input(mouse_drawing_gesture) { return; }
            mouse_drawing_gesture.set_state(EventSequenceState::Claimed);

            let shortcut_key = input::retreive_mouse_shortcut_key(&mouse_drawing_gesture);
            let mut data_entries = input::retreive_pointer_inputdata(mouse_drawing_gesture, x, y);
            input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]), canvas.zoom());
            input::process_pen_down(data_entries, &appwindow, shortcut_key);
        }));

        self.imp().mouse_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |mouse_drawing_gesture, x, y| {
            log::trace!("mouse_drawing_gesture motion");
            // filter out invalid point input
            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_inputdata(mouse_drawing_gesture, x, y);
                input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]), canvas.zoom());
                input::process_pen_motion(data_entries, &appwindow);
            }
        }));

        self.imp().mouse_drawing_gesture.connect_drag_end(clone!(@weak self as canvas @weak appwindow => move |mouse_drawing_gesture, x, y| {
            log::trace!("mouse_drawing_gesture end");
            // filter out invalid point input
            if input::filter_mouse_input(mouse_drawing_gesture) { return; }

            if let Some(start_point) = mouse_drawing_gesture.start_point() {
                let mut data_entries = input::retreive_pointer_inputdata(mouse_drawing_gesture, x, y);
                input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]), canvas.zoom());
                input::process_pen_up(data_entries, &appwindow);
            }
        }));

        // Touch drawing
        self.imp().touch_drawing_gesture.connect_drag_begin(
            clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
                log::trace!("touch_drawing_gesture begin");
                // filter out invalid stylus input
                if input::filter_touch_input(touch_drawing_gesture) { return; }
                touch_drawing_gesture.set_state(EventSequenceState::Claimed);

                let mut data_entries = input::retreive_pointer_inputdata(touch_drawing_gesture, x, y);
                input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![0.0, 0.0]), canvas.zoom());
                input::process_pen_down(data_entries, &appwindow, None);
            }),
        );

        self.imp().touch_drawing_gesture.connect_drag_update(clone!(@weak self as canvas, @weak appwindow => move |touch_drawing_gesture, x, y| {
            if let Some(start_point) = touch_drawing_gesture.start_point() {
                log::trace!("touch_drawing_gesture motion");
                // filter out invalid stylus input
                if input::filter_touch_input(touch_drawing_gesture) { return; }

                let mut data_entries = input::retreive_pointer_inputdata(touch_drawing_gesture, x, y);
                input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]), canvas.zoom());
                input::process_pen_motion(data_entries, &appwindow);
            }
        }));

        self.imp().touch_drawing_gesture.connect_drag_end(
            clone!(@weak self as canvas @weak appwindow => move |touch_drawing_gesture, x, y| {
                if let Some(start_point) = touch_drawing_gesture.start_point() {
                    log::trace!("touch_drawing_gesture end");
                    // filter out invalid stylus input
                    if input::filter_touch_input(touch_drawing_gesture) { return; }

                    let mut data_entries = input::retreive_pointer_inputdata(touch_drawing_gesture, x, y);
                    input::transform_inputdata(&mut data_entries, canvas.transform_canvas_coords_to_sheet_coords(na::vector![start_point.0, start_point.1]), canvas.zoom());
                    input::process_pen_up(data_entries, &appwindow);
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
        self.sheet()
            .borrow()
            .bounds()
            .scale(na::Vector2::from_element(self.total_zoom()))
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

        (canvas_coords + self.adj_values()) / total_zoom
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

        sheet_coords * total_zoom - self.adj_values()
    }

    /// The view of the parent scroller onto the Canvas
    pub fn viewport(&self) -> AABB {
        let parent = self.parent().unwrap();
        let parent_size = na::vector![f64::from(parent.width()), f64::from(parent.height())];
        let parent_offset = self
            .translate_coordinates(&parent, 0.0, 0.0)
            .map(|parent_offset| na::vector![parent_offset.0, parent_offset.1])
            .unwrap();

        let offset = self.adj_values() - parent_offset;

        AABB::new(
            na::Point2::from(offset),
            na::Point2::from(offset + parent_size),
        )
    }

    /// The viewport transformed to match the coordinate space of the sheet
    pub fn viewport_in_sheet_coords(&self) -> AABB {
        let viewport = self.viewport();
        let total_zoom = self.total_zoom();

        viewport.scale(na::Vector2::from_element(1.0 / total_zoom))
    }

    pub fn adj_values(&self) -> na::Vector2<f64> {
        na::vector![
            self.hadjustment().unwrap().value(),
            self.vadjustment().unwrap().value()
        ]
    }

    pub fn update_adj_values(&self, new_values: na::Vector2<f64>) {
        match self.expand_mode() {
            ExpandMode::Infinite => {
                let new_viewport = self
                    .viewport_in_sheet_coords()
                    .translate(new_values - self.adj_values());

                self.sheet()
                    .borrow_mut()
                    .expand_sheet_mode_infinite_for_viewport(new_viewport);

                self.update_adj_config(na::vector![
                    f64::from(self.allocation().width()),
                    f64::from(self.allocation().height())
                ]);
            }
            _ => {}
        }

        self.hadjustment().unwrap().set_value(new_values[0]);
        self.vadjustment().unwrap().set_value(new_values[1]);

        self.queue_resize();
    }

    pub fn update_adj_config(&self, size_request: na::Vector2<f64>) {
        let total_zoom = self.total_zoom();

        let hadj = self.hadjustment().unwrap();

        let (h_lower, h_upper) = match self.expand_mode() {
            ExpandMode::FixedSize | ExpandMode::EndlessVertical => (
                (self.sheet().borrow().x - Canvas::SHADOW_WIDTH) * total_zoom,
                (self.sheet().borrow().x + self.sheet().borrow().width + Canvas::SHADOW_WIDTH)
                    * total_zoom,
            ),
            ExpandMode::Infinite => (
                self.sheet().borrow().x * total_zoom,
                (self.sheet().borrow().x + self.sheet().borrow().width) * total_zoom,
            ),
        };

        let vadj = self.vadjustment().unwrap();

        let (v_lower, v_upper) = match self.expand_mode() {
            ExpandMode::FixedSize | ExpandMode::EndlessVertical => (
                (self.sheet().borrow().y - Canvas::SHADOW_WIDTH) * total_zoom,
                (self.sheet().borrow().y + self.sheet().borrow().height + Canvas::SHADOW_WIDTH)
                    * total_zoom,
            ),
            ExpandMode::Infinite => (
                self.sheet().borrow().y * total_zoom,
                (self.sheet().borrow().y + self.sheet().borrow().height) * total_zoom,
            ),
        };

        hadj.configure(
            hadj.value(),
            h_lower,
            h_upper,
            0.1 * size_request[0],
            0.9 * size_request[0],
            size_request[0],
        );

        vadj.configure(
            vadj.value(),
            v_lower,
            v_upper,
            0.1 * size_request[1],
            0.9 * size_request[1],
            size_request[1],
        );
    }

    /// Called when sheet should resize to the format and to fit all strokes
    pub fn resize_sheet_to_fit_strokes(&self) {
        match self.expand_mode() {
            ExpandMode::FixedSize => {
                self.sheet().borrow_mut().resize_sheet_mode_fixed_size();
            }
            ExpandMode::EndlessVertical => {
                self.sheet()
                    .borrow_mut()
                    .resize_sheet_mode_endless_vertical();
            }
            ExpandMode::Infinite => {
                self.sheet()
                    .borrow_mut()
                    .resize_sheet_mode_infinite_to_fit_strokes();
                self.sheet()
                    .borrow_mut()
                    .expand_sheet_mode_infinite_for_viewport(self.viewport_in_sheet_coords());
            }
        }

        self.queue_resize();
    }

    /// resize the sheet when in autoexpanding expand modes. called e.g. when finishing a new stroke
    pub fn resize_sheet_autoexpand(&self) {
        match self.expand_mode() {
            ExpandMode::FixedSize => {
                // Does not resize in fixed size mode, use resize_sheet_to_fit_strokes() for it.
            }
            ExpandMode::EndlessVertical => {
                self.sheet()
                    .borrow_mut()
                    .resize_sheet_mode_endless_vertical();
            }
            ExpandMode::Infinite => {
                self.sheet()
                    .borrow_mut()
                    .resize_sheet_mode_infinite_to_fit_strokes();
                self.sheet()
                    .borrow_mut()
                    .expand_sheet_mode_infinite_for_viewport(self.viewport_in_sheet_coords());
            }
        }
        self.queue_resize();
    }

    /// Centers the view around a coord on the sheet. The coord parameter has the coordinate space of the sheet!
    pub fn center_around_coord_on_sheet(&self, coord: na::Vector2<f64>) {
        let (parent_width, parent_height) = (
            f64::from(self.parent().unwrap().width()),
            f64::from(self.parent().unwrap().height()),
        );
        let total_zoom = self.total_zoom();

        let new_adj_values = na::vector![
            ((coord[0]) * total_zoom) - parent_width * 0.5,
            ((coord[1]) * total_zoom) - parent_height * 0.5
        ];

        self.update_adj_values(new_adj_values);
    }

    /// Centering the view to the first page
    pub fn return_to_origin_page(&self) {
        let total_zoom = self.total_zoom();

        let new_adj_values = na::vector![
            ((self.sheet().borrow().format.width / 2.0) * total_zoom)
                - f64::from(self.parent().unwrap().width()) * 0.5,
            -Self::SHADOW_WIDTH * total_zoom
        ];

        self.update_adj_values(new_adj_values);
    }

    /// Zoom temporarily to a new zoom, not regenerating the contents while doing it.
    /// To zoom and regenerate the content and reset the temporary zoom, use zoom_to().
    pub fn zoom_temporarily_to(&self, temp_zoom: f64) {
        self.set_temporary_zoom(temp_zoom / self.zoom());

        self.update_background_rendernode(true);
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

        self.resize_sheet_autoexpand();

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
        }
    }

    /// regenerate the rendernodes of the canvas content. force_regenerate regenerate all images and rendernodes from scratch. redraw: queue canvas redrawing
    pub fn regenerate_content(&self, force_regenerate: bool, redraw: bool) {
        self.sheet()
            .borrow_mut()
            .strokes_state
            .regenerate_rendering_current_view_threaded(
                Some(self.viewport_in_sheet_coords()),
                force_regenerate,
                self.renderer(),
                self.zoom(),
            );

        if redraw {
            self.queue_resize();
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

        Some(texture)
    }

    pub fn show_return_to_center_toast(&self, appwindow: &RnoteAppWindow) {
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
