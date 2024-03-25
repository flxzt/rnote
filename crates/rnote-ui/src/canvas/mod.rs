// Modules
mod canvaslayout;
pub(crate) mod imexport;
mod input;
mod widgetflagsboxed;

// Re-exports
pub(crate) use canvaslayout::RnCanvasLayout;
pub(crate) use widgetflagsboxed::WidgetFlagsBoxed;

// Imports
use crate::{config, RnAppWindow};
use futures::StreamExt;
use gettextrs::gettext;
use gtk4::{
    gdk, gio, glib, glib::clone, graphene, prelude::*, subclass::prelude::*, Adjustment,
    DropTarget, EventControllerKey, EventControllerLegacy, IMMulticontext, PropagationPhase,
    Scrollable, ScrollablePolicy, Widget,
};
use notify_debouncer_full::notify::{self, Watcher};
use once_cell::sync::Lazy;
use p2d::bounding_volume::Aabb;
use rnote_compose::ext::AabbExt;
use rnote_compose::penevent::PenState;
use rnote_engine::ext::GraphenePointExt;
use rnote_engine::ext::GrapheneRectExt;
use rnote_engine::Camera;
use rnote_engine::{Engine, WidgetFlags};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Default)]
struct Connections {
    hadjustment: Option<glib::SignalHandlerId>,
    vadjustment: Option<glib::SignalHandlerId>,
    tab_page_output_file: Option<glib::Binding>,
    tab_page_unsaved_changes: Option<glib::Binding>,
    appwindow_output_file: Option<glib::SignalHandlerId>,
    appwindow_scalefactor: Option<glib::SignalHandlerId>,
    appwindow_unsaved_changes: Option<glib::SignalHandlerId>,
    appwindow_touch_drawing: Option<glib::Binding>,
    appwindow_show_drawing_cursor: Option<glib::Binding>,
    appwindow_regular_cursor: Option<glib::Binding>,
    appwindow_drawing_cursor: Option<glib::Binding>,
    appwindow_drop_target: Option<glib::SignalHandlerId>,
    appwindow_handle_widget_flags: Option<glib::SignalHandlerId>,
}

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnCanvas {
        pub(super) connections: RefCell<Connections>,
        pub(crate) hadjustment: RefCell<Option<Adjustment>>,
        pub(crate) vadjustment: RefCell<Option<Adjustment>>,
        pub(crate) hscroll_policy: Cell<ScrollablePolicy>,
        pub(crate) vscroll_policy: Cell<ScrollablePolicy>,
        pub(crate) regular_cursor_icon_name: RefCell<String>,
        pub(crate) regular_cursor: RefCell<gdk::Cursor>,
        pub(crate) drawing_cursor_icon_name: RefCell<String>,
        pub(crate) drawing_cursor: RefCell<gdk::Cursor>,
        pub(crate) invisible_cursor: RefCell<gdk::Cursor>,
        pub(crate) pointer_controller: EventControllerLegacy,
        pub(crate) key_controller: EventControllerKey,
        pub(crate) key_controller_im_context: IMMulticontext,
        pub(crate) drop_target: DropTarget,
        pub(crate) drawing_cursor_enabled: Cell<bool>,

        pub(crate) engine: RefCell<Engine>,
        pub(crate) engine_task_handler_handle: RefCell<Option<glib::JoinHandle<()>>>,

        // dnd status
        pub(crate) dnd_status: Cell<bool>,
        pub(crate) dnd_respect_borders: Cell<bool>,

        pub(crate) output_file: RefCell<Option<gio::File>>,
        pub(crate) output_file_watcher_task: RefCell<Option<glib::JoinHandle<()>>>,
        pub(crate) output_file_modified_toast_singleton: RefCell<Option<adw::Toast>>,
        pub(crate) output_file_expect_write: Cell<bool>,
        pub(crate) save_in_progress: Cell<bool>,
        pub(crate) unsaved_changes: Cell<bool>,
        pub(crate) empty: Cell<bool>,
        pub(crate) touch_drawing: Cell<bool>,
        pub(crate) show_drawing_cursor: Cell<bool>,

        pub(crate) last_export_dir: RefCell<Option<gio::File>>,
    }

    impl Default for RnCanvas {
        fn default() -> Self {
            let pointer_controller = EventControllerLegacy::builder()
                .name("pointer_controller")
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

            // the order here is important: first files, then text
            drop_target.set_types(&[gio::File::static_type(), glib::types::Type::STRING]);

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

            let invisible_cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH)
                        + "icons/scalable/actions/cursor-invisible.svg")
                        .as_str(),
                ),
                32,
                32,
                gdk::Cursor::from_name("default", None).as_ref(),
            );

            let engine = Engine::default();

            Self {
                connections: RefCell::new(Connections::default()),

                hadjustment: RefCell::new(None),
                vadjustment: RefCell::new(None),
                hscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                vscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                regular_cursor: RefCell::new(regular_cursor),
                regular_cursor_icon_name: RefCell::new(regular_cursor_icon_name),
                drawing_cursor: RefCell::new(drawing_cursor),
                drawing_cursor_icon_name: RefCell::new(drawing_cursor_icon_name),
                invisible_cursor: RefCell::new(invisible_cursor),
                pointer_controller,
                key_controller,
                key_controller_im_context,
                drop_target,
                drawing_cursor_enabled: Cell::new(false),

                engine: RefCell::new(engine),
                engine_task_handler_handle: RefCell::new(None),

                dnd_status: Cell::new(false),
                dnd_respect_borders: Cell::new(false),

                output_file: RefCell::new(None),
                output_file_watcher_task: RefCell::new(None),
                // is automatically updated whenever the output file changes.
                output_file_modified_toast_singleton: RefCell::new(None),
                output_file_expect_write: Cell::new(false),
                save_in_progress: Cell::new(false),
                unsaved_changes: Cell::new(false),
                empty: Cell::new(true),
                touch_drawing: Cell::new(false),
                show_drawing_cursor: Cell::new(false),

                last_export_dir: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnCanvas {
        const NAME: &'static str = "RnCanvas";
        type Type = super::RnCanvas;
        type ParentType = Widget;
        type Interfaces = (Scrollable,);

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<RnCanvasLayout>();
        }

        fn new() -> Self {
            Self::default()
        }
    }

    impl ObjectImpl for RnCanvas {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.set_hexpand(false);
            obj.set_vexpand(false);
            // keyboard focus needed for typewriter
            obj.set_can_focus(true);
            obj.set_focusable(true);

            obj.set_cursor(Some(&*self.regular_cursor.borrow()));

            obj.add_controller(self.pointer_controller.clone());
            obj.add_controller(self.key_controller.clone());
            obj.add_controller(self.drop_target.clone());

            // receive and handle engine tasks
            let engine_task_handler_handle = glib::spawn_future_local(
                clone!(@weak obj as canvas => async move {
                    let Some(mut task_rx) = canvas.engine_mut().take_engine_tasks_rx() else {
                        tracing::error!("Installing the engine task handler failed, taken tasks_rx is None.");
                        return;
                    };

                    loop {
                        if let Some(task) = task_rx.recv().await {
                            let (widget_flags, quit) = canvas.engine_mut().handle_engine_task(task);
                            canvas.emit_handle_widget_flags(widget_flags);

                            if quit {
                                break;
                            }
                        }
                    }
                }),
            );

            *self.engine_task_handler_handle.borrow_mut() = Some(engine_task_handler_handle);

            self.setup_input();
        }

        fn dispose(&self) {
            self.obj().disconnect_connections();
            self.obj().abort_engine_task_handler();

            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    // this is nullable, so it can be used to represent Option<gio::File>
                    glib::ParamSpecObject::builder::<gio::File>("output-file").build(),
                    glib::ParamSpecBoolean::builder("unsaved-changes")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("empty")
                        .default_value(true)
                        .build(),
                    glib::ParamSpecBoolean::builder("touch-drawing")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("show-drawing-cursor")
                        .default_value(true)
                        .build(),
                    glib::ParamSpecString::builder("regular-cursor")
                        .default_value(Some("cursor-dot-medium"))
                        .build(),
                    glib::ParamSpecString::builder("drawing-cursor")
                        .default_value(Some("cursor-dot-small"))
                        .build(),
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
                "show-drawing-cursor" => self.show_drawing_cursor.get().to_value(),
                "regular-cursor" => self.regular_cursor_icon_name.borrow().to_value(),
                "drawing-cursor" => self.drawing_cursor_icon_name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "output-file" => {
                    let output_file = value
                        .get::<Option<gio::File>>()
                        .expect("The value needs to be of type `Option<gio::File>`");
                    self.output_file.replace(output_file);
                }
                "unsaved-changes" => {
                    let unsaved_changes: bool =
                        value.get().expect("The value needs to be of type `bool`");
                    self.unsaved_changes.replace(unsaved_changes);
                }
                "empty" => {
                    let empty: bool = value.get().expect("The value needs to be of type `bool`");
                    self.empty.replace(empty);
                    if empty {
                        obj.set_unsaved_changes(false);
                    }
                }
                "hadjustment" => {
                    let hadj = value.get().unwrap();
                    self.set_hadjustment_prop(hadj);
                }
                "hscroll-policy" => {
                    let hscroll_policy = value.get().unwrap();
                    self.hscroll_policy.replace(hscroll_policy);
                }
                "vadjustment" => {
                    let vadj = value.get().unwrap();
                    self.set_vadjustment_prop(vadj);
                }
                "vscroll-policy" => {
                    let vscroll_policy = value.get().unwrap();
                    self.vscroll_policy.replace(vscroll_policy);
                }
                "touch-drawing" => {
                    let touch_drawing: bool =
                        value.get().expect("The value needs to be of type `bool`");
                    self.touch_drawing.replace(touch_drawing);
                }
                "show-drawing-cursor" => {
                    let show_drawing_cursor: bool =
                        value.get().expect("The value needs to be of type `bool`");
                    self.show_drawing_cursor.replace(show_drawing_cursor);

                    if self.drawing_cursor_enabled.get() {
                        if show_drawing_cursor {
                            obj.set_cursor(Some(&*self.drawing_cursor.borrow()));
                        } else {
                            obj.set_cursor(Some(&*self.invisible_cursor.borrow()));
                        }
                    } else {
                        obj.set_cursor(Some(&*self.regular_cursor.borrow()));
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

                    obj.set_cursor(Some(&*self.regular_cursor.borrow()));
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
                vec![glib::subclass::Signal::builder("handle-widget-flags")
                    .param_types([WidgetFlagsBoxed::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for RnCanvas {
        // request_mode(), measure(), allocate() overrides happen in the CanvasLayout LayoutManager

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let obj = self.obj();

            if let Err(e) = || -> anyhow::Result<()> {
                let clip_bounds = if let Some(parent) = obj.parent() {
                    Aabb::new_positive(
                        parent
                            .compute_point(&*obj, &graphene::Point::zero())
                            .unwrap()
                            .to_na_point(),
                        na::point![f64::from(parent.width()), f64::from(parent.height())],
                    )
                } else {
                    obj.bounds()
                };
                // push the clip
                snapshot.push_clip(&graphene::Rect::from_p2d_aabb(clip_bounds));

                // Draw the entire engine
                self.engine
                    .borrow()
                    .draw_to_gtk_snapshot(snapshot, obj.bounds())?;

                // pop the clip
                snapshot.pop();
                Ok(())
            }() {
                tracing::error!("Snapshot canvas failed , Err: {e:?}");
            }
        }
    }

    impl ScrollableImpl for RnCanvas {}

    impl RnCanvas {
        fn setup_input(&self) {
            let obj = self.obj();

            // Pointer controller
            let pen_state = Cell::new(PenState::Up);
            self.pointer_controller.connect_event(clone!(@strong pen_state, @weak obj as canvas => @default-return glib::Propagation::Proceed, move |_, event| {
                let (propagation, new_state) = super::input::handle_pointer_controller_event(&canvas, event, pen_state.get());
                pen_state.set(new_state);
                propagation
            }));

            // For unicode text the input is committed from the IM context, and won't trigger the key_pressed signal
            self.key_controller_im_context.connect_commit(
                clone!(@weak obj as canvas => move |_cx, text| {
                    super::input::handle_imcontext_text_commit(&canvas, text);
                }),
            );

            // Key controller
            self.key_controller.connect_key_pressed(clone!(@weak obj as canvas => @default-return glib::Propagation::Proceed, move |_, key, _raw, modifier| {
                super::input::handle_key_controller_key_pressed(&canvas, key, modifier)
            }));

            self.key_controller.connect_key_released(
                clone!(@weak obj as canvas => move |_key_controller, key, _raw, modifier| {
                    super::input::handle_key_controller_key_released(&canvas, key, modifier)
                }),
            );
        }

        fn set_hadjustment_prop(&self, hadj: Option<Adjustment>) {
            let obj = self.obj();

            let hadj_value = self
                .hadjustment
                .borrow()
                .as_ref()
                .map(|adj| adj.value())
                .unwrap_or(-Camera::OVERSHOOT_HORIZONTAL);
            let vadj_value = self
                .vadjustment
                .borrow()
                .as_ref()
                .map(|adj| adj.value())
                .unwrap_or(-Camera::OVERSHOOT_VERTICAL);
            let widget_size = obj.widget_size();
            let offset_mins_maxs = obj.engine_ref().camera_offset_mins_maxs();

            if let Some(signal_id) = self.connections.borrow_mut().hadjustment.take() {
                let old_adj = self.hadjustment.borrow().as_ref().unwrap().clone();
                old_adj.disconnect(signal_id);
            }

            if let Some(ref hadj) = hadj {
                let signal_id =
                    hadj.connect_value_changed(clone!(@weak obj as canvas => move |_| {
                        // this triggers a canvaslayout allocate() call,
                        // where the camera and content rendering is updated based on some conditions
                        canvas.queue_resize();
                    }));

                self.connections.borrow_mut().hadjustment.replace(signal_id);
            }
            self.hadjustment.replace(hadj);

            obj.configure_adjustments(
                widget_size,
                offset_mins_maxs,
                na::vector![hadj_value, vadj_value],
            );
        }

        fn set_vadjustment_prop(&self, vadj: Option<Adjustment>) {
            let obj = self.obj();

            let hadj_value = self
                .hadjustment
                .borrow()
                .as_ref()
                .map(|adj| adj.value())
                .unwrap_or(-Camera::OVERSHOOT_HORIZONTAL);
            let vadj_value = self
                .vadjustment
                .borrow()
                .as_ref()
                .map(|adj| adj.value())
                .unwrap_or(-Camera::OVERSHOOT_VERTICAL);
            let widget_size = obj.widget_size();
            let offset_mins_maxs = obj.engine_ref().camera_offset_mins_maxs();

            if let Some(signal_id) = self.connections.borrow_mut().vadjustment.take() {
                let old_adj = self.vadjustment.borrow().as_ref().unwrap().clone();
                old_adj.disconnect(signal_id);
            }

            if let Some(ref vadj) = vadj {
                let signal_id =
                    vadj.connect_value_changed(clone!(@weak obj as canvas => move |_| {
                        // this triggers a canvaslayout allocate() call,
                        // where the camera and content rendering is updated based on some conditions
                        canvas.queue_resize();
                    }));

                self.connections.borrow_mut().vadjustment.replace(signal_id);
            }
            self.vadjustment.replace(vadj);

            obj.configure_adjustments(
                widget_size,
                offset_mins_maxs,
                na::vector![hadj_value, vadj_value],
            );
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnCanvas(ObjectSubclass<imp::RnCanvas>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Scrollable;
}

impl Default for RnCanvas {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) static OUTPUT_FILE_NEW_TITLE: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| gettext("New Document"));
pub(crate) static OUTPUT_FILE_NEW_SUBTITLE: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| gettext("Draft"));

impl RnCanvas {
    // Sets the canvas zoom scroll step in % for one unit of the event controller delta
    pub(crate) const ZOOM_SCROLL_STEP: f64 = 0.1;

    pub(crate) fn new() -> Self {
        glib::Object::new()
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
    pub(crate) fn set_output_file(&self, output_file: Option<gio::File>) {
        self.set_property("output-file", output_file.to_value());
    }

    #[allow(unused)]
    pub(crate) fn output_file_expect_write(&self) -> bool {
        self.imp().output_file_expect_write.get()
    }

    #[allow(unused)]
    pub(crate) fn set_output_file_expect_write(&self, expect_write: bool) {
        self.imp().output_file_expect_write.set(expect_write);
    }

    #[allow(unused)]
    pub(crate) fn save_in_progress(&self) -> bool {
        self.imp().save_in_progress.get()
    }

    #[allow(unused)]
    pub(crate) fn set_save_in_progress(&self, save_in_progress: bool) {
        self.imp().save_in_progress.set(save_in_progress);
    }

    #[allow(unused)]
    pub(crate) fn unsaved_changes(&self) -> bool {
        self.property::<bool>("unsaved-changes")
    }

    #[allow(unused)]
    pub(crate) fn set_unsaved_changes(&self, unsaved_changes: bool) {
        if self.imp().unsaved_changes.get() != unsaved_changes {
            self.set_property("unsaved-changes", unsaved_changes.to_value());
        }
    }

    #[allow(unused)]
    pub(crate) fn empty(&self) -> bool {
        self.property::<bool>("empty")
    }

    #[allow(unused)]
    pub(crate) fn set_empty(&self, empty: bool) {
        if self.imp().empty.get() != empty {
            self.set_property("empty", empty.to_value());
        }
    }

    #[allow(unused)]
    pub(crate) fn touch_drawing(&self) -> bool {
        self.property::<bool>("touch-drawing")
    }

    #[allow(unused)]
    pub(crate) fn set_touch_drawing(&self, touch_drawing: bool) {
        if self.imp().touch_drawing.get() != touch_drawing {
            self.set_property("touch-drawing", touch_drawing.to_value());
        }
    }

    #[allow(unused)]
    pub(crate) fn show_drawing_cursor(&self) -> bool {
        self.property::<bool>("show-drawing-cursor")
    }

    #[allow(unused)]
    pub(crate) fn set_show_drawing_cursor(&self, show_drawing_cursor: bool) {
        if self.imp().show_drawing_cursor.get() != show_drawing_cursor {
            self.set_property("show-drawing-cursor", show_drawing_cursor.to_value());
        }
    }

    #[allow(unused)]
    pub(super) fn emit_handle_widget_flags(&self, widget_flags: WidgetFlags) {
        self.emit_by_name::<()>(
            "handle-widget-flags",
            &[&WidgetFlagsBoxed::from(widget_flags)],
        );
    }

    pub(crate) fn last_export_dir(&self) -> Option<gio::File> {
        self.imp().last_export_dir.borrow().clone()
    }

    pub(crate) fn set_last_export_dir(&self, dir: Option<gio::File>) {
        self.imp().last_export_dir.replace(dir);
    }

    pub(crate) fn canvas_layout_manager(&self) -> RnCanvasLayout {
        self.layout_manager()
            .and_downcast::<RnCanvasLayout>()
            .unwrap()
    }

    pub(crate) fn configure_adjustments(
        &self,
        widget_size: na::Vector2<f64>,
        offset_mins_maxs: (na::Vector2<f64>, na::Vector2<f64>),
        offset: na::Vector2<f64>,
    ) {
        let (offset_mins, offset_maxs) = offset_mins_maxs;

        if let Some(hadj) = self.hadjustment() {
            hadj.configure(
                // This gets clamped to the lower and upper values
                offset[0],
                offset_mins[0],
                offset_maxs[0],
                0.1 * widget_size[0],
                0.9 * widget_size[0],
                widget_size[0],
            )
        };

        if let Some(vadj) = self.vadjustment() {
            vadj.configure(
                // This gets clamped to the lower and upper values
                offset[1],
                offset_mins[1],
                offset_maxs[1],
                0.1 * widget_size[1],
                0.9 * widget_size[1],
                widget_size[1],
            );
        }

        self.queue_resize();
    }

    pub(crate) fn widget_size(&self) -> na::Vector2<f64> {
        na::vector![self.width() as f64, self.height() as f64]
    }

    /// Immutable borrow of the engine.
    pub(crate) fn engine_ref(&self) -> Ref<Engine> {
        self.imp().engine.borrow()
    }

    /// Mutable borrow of the engine.
    pub(crate) fn engine_mut(&self) -> RefMut<Engine> {
        self.imp().engine.borrow_mut()
    }

    /// Abort the engine task handler.
    ///
    /// Because the installed engine task handler holds a reference to the canvas,
    /// this MUST be called when the widget is removed from the widget tree,
    /// it's instance should be destroyed and it's memory should be freed.
    pub(crate) fn abort_engine_task_handler(&self) {
        if let Some(h) = self.imp().engine_task_handler_handle.take() {
            h.abort();
        }
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

    pub(crate) fn save_engine_config(&self, settings: &gio::Settings) -> anyhow::Result<()> {
        let engine_config = self.engine_ref().export_engine_config_as_json()?;
        Ok(settings.set_string("engine-config", engine_config.as_str())?)
    }

    pub(crate) fn load_engine_config_from_settings(
        &self,
        settings: &gio::Settings,
    ) -> anyhow::Result<()> {
        // load engine config
        let engine_config = settings.string("engine-config");
        let widget_flags = match self
            .engine_mut()
            .import_engine_config_from_json(&engine_config, crate::env::pkg_data_dir().ok())
        {
            Err(e) => {
                if engine_config.is_empty() {
                    // On first app startup the engine config is empty, so we don't log an error
                    tracing::debug!("Did not load `engine-config` from settings, was empty");
                } else {
                    return Err(e);
                }
                None
            }
            Ok(widget_flags) => Some(widget_flags),
        };

        // Avoiding already borrowed
        if let Some(widget_flags) = widget_flags {
            self.emit_handle_widget_flags(widget_flags);
        }
        Ok(())
    }

    /// Switches between the regular and the drawing cursor
    pub(crate) fn enable_drawing_cursor(&self, drawing_cursor: bool) {
        if drawing_cursor == self.imp().drawing_cursor_enabled.get() {
            return;
        };
        self.imp().drawing_cursor_enabled.set(drawing_cursor);

        if drawing_cursor {
            if self.imp().show_drawing_cursor.get() {
                self.set_cursor(Some(&*self.imp().drawing_cursor.borrow()));
            } else {
                self.set_cursor(Some(&*self.imp().invisible_cursor.borrow()));
            }
        } else {
            self.set_cursor(Some(&*self.imp().regular_cursor.borrow()));
        }
    }

    /// The document title for display. Can be used to get a string as the basename of the existing / a new save file.
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

    pub(crate) fn clear_output_file_watcher(&self) {
        if let Some(handle) = self.imp().output_file_watcher_task.take() {
            handle.abort();
        }
    }

    pub(crate) fn dismiss_output_file_modified_toast(&self) {
        if let Some(output_file_modified_toast) =
            self.imp().output_file_modified_toast_singleton.take()
        {
            output_file_modified_toast.dismiss();
        }
    }

    pub(crate) fn create_output_file_watcher(&self, file: &gio::File, appwindow: &RnAppWindow) {
        let dispatch_toast_reload_modified_file = |appwindow: &RnAppWindow, canvas: &RnCanvas| {
            canvas.set_unsaved_changes(true);

            appwindow.overlays().dispatch_toast_w_button_singleton(
                        &gettext("Opened file was modified on disk"),
                        &gettext("Reload"),
                        clone!(@weak canvas, @weak appwindow => move |_reload_toast| {
                            glib::spawn_future_local(clone!(@weak appwindow => async move {
                                appwindow.overlays().progressbar_start_pulsing();

                                if let Err(e) = canvas.reload_from_disk().await {
                                    tracing::error!("Failed to reload current output file, Err: {e:?}");
                                    appwindow.overlays().dispatch_toast_error(&gettext("Reloading .rnote file from disk failed"));
                                    appwindow.overlays().progressbar_abort();
                                } else {
                                    appwindow.overlays().progressbar_finish();
                                }
                            }));
                        }),
                        0,
                    &mut canvas.imp().output_file_modified_toast_singleton.borrow_mut());
        };

        let event_handler = move |appwindow: &RnAppWindow,
                                  canvas: &RnCanvas,
                                  event: notify_debouncer_full::DebouncedEvent,
                                  file_path: &Path| {
            use notify::event::{AccessKind, AccessMode, ModifyKind, RenameMode};
            use notify::EventKind;

            tracing::trace!("file parent directory watcher - received event: {event:?}");

            match event.kind {
                EventKind::Create(_create_kind) => {}
                EventKind::Modify(ModifyKind::Data(_data_change)) => {
                    let Some(event_path) = event.paths.first() else {
                        return;
                    };
                    if !crate::utils::paths_abs_eq(file_path, event_path).unwrap_or(false) {
                        return;
                    }
                    if canvas.output_file_expect_write() {
                        // While writing is in progress, multiple modify events might occur.
                        return;
                    }
                    dispatch_toast_reload_modified_file(appwindow, canvas);
                }
                EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                    let (Some(from_path), Some(to_path)) =
                        (event.paths.first(), event.paths.get(1))
                    else {
                        return;
                    };
                    if !crate::utils::paths_abs_eq(file_path, from_path).unwrap_or(false) {
                        return;
                    }
                    // Only when the new path is known we can update the output file
                    canvas.set_output_file(Some(gio::File::for_path(to_path)));
                }
                EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    let Some(event_path) = event.paths.first() else {
                        return;
                    };
                    if !crate::utils::paths_abs_eq(file_path, event_path).unwrap_or(false) {
                        return;
                    }
                    dispatch_toast_reload_modified_file(appwindow, canvas);
                }
                EventKind::Modify(ModifyKind::Name(_)) => {
                    let Some(event_path) = event.paths.first() else {
                        return;
                    };
                    if !crate::utils::paths_abs_eq(file_path, event_path).unwrap_or(false) {
                        return;
                    }
                    canvas.set_unsaved_changes(true);
                    canvas.set_output_file(None);
                    appwindow.overlays().dispatch_toast_text(
                        &gettext("Opened file was renamed or moved."),
                        crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                    );
                }
                EventKind::Remove(_remove_kind) => {
                    let Some(event_path) = event.paths.first() else {
                        return;
                    };
                    if !crate::utils::paths_abs_eq(file_path, event_path).unwrap_or(false) {
                        return;
                    }
                    canvas.set_unsaved_changes(true);
                    canvas.set_output_file(None);
                    appwindow.overlays().dispatch_toast_text(
                        &gettext("Opened file was removed."),
                        crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                    );
                }
                EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                    let Some(event_path) = event.paths.first() else {
                        return;
                    };
                    if !crate::utils::paths_abs_eq(file_path, event_path).unwrap_or(false) {
                        return;
                    }
                    if canvas.output_file_expect_write() {
                        // Own file writing has finished
                        canvas.set_output_file_expect_write(false);
                    }
                }
                _ => {}
            }
        };

        let new_watcher_task = glib::spawn_future_local(
            glib::clone!(@strong file, @weak self as canvas, @weak appwindow => async move {
                let (tx, mut rx) = futures::channel::mpsc::unbounded();
                let Some(file_path) = file.path() else {
                    tracing::warn!("Can't create watcher for file that has no path");
                    return;
                };
                let Some(parent_path) = file_path.parent() else {
                    tracing::warn!("Can't create watcher for file that has no parent directory");
                    return;
                };

                let mut debouncer = match notify_debouncer_full::new_debouncer(Duration::from_millis(1000), None, move |res| {
                    if let Err(e) = tx.unbounded_send(res) {
                        tracing::error!("File watcher reported change, but failed to send it through channel. Err: {e:?}");
                    }
                }) {
                    Ok(w) => {
                        w
                    },
                    Err(e) => {
                        tracing::error!("Failed to create file watcher, Err: {e:?}");
                        return;
                    }
                };
                if let Err(e) = debouncer.watcher().watch(parent_path, notify::RecursiveMode::NonRecursive) {
                    tracing::error!("Failed to start watching directory '{}', Err: {e:?}", parent_path.display());
                }
                debouncer.cache().add_root(parent_path, notify::RecursiveMode::NonRecursive);
                while let Some(res) = rx.next().await {
                    match res {
                        Ok(events) => {
                            for event in events {
                                event_handler(&appwindow, &canvas, event, &file_path);
                            }
                        }
                        Err(e) => tracing::error!("File watcher sent error message, Err: {e:?}"),
                    }
                }
            }),
        );

        if let Some(old_watcher_task) = self
            .imp()
            .output_file_watcher_task
            .borrow_mut()
            .replace(new_watcher_task)
        {
            old_watcher_task.abort();
        }
    }

    /// Replaces and installs a new file monitor when there is an output file present
    fn reinstall_output_file_watcher(&self, appwindow: &RnAppWindow) {
        if let Some(output_file) = self.output_file() {
            self.create_output_file_watcher(&output_file, appwindow);
        } else {
            self.clear_output_file_watcher();
        }
    }

    /// Initializes for the given appwindow. Usually `init()` is only called once, but since this widget can be moved between appwindows through tabs,
    /// this function also disconnects and replaces all existing old connections
    pub(crate) fn init_reconnect(&self, appwindow: &RnAppWindow) {
        // initialize file watcher, needed when the tab is moved to another window and is re-initializing
        self.reinstall_output_file_watcher(appwindow);

        let appwindow_output_file = self.connect_notify_local(
            Some("output-file"),
            clone!(@weak appwindow => move |canvas, _pspec| {
                if let Some(output_file) = canvas.output_file(){
                    canvas.create_output_file_watcher(&output_file, &appwindow);
                } else {
                    canvas.clear_output_file_watcher();
                    canvas.dismiss_output_file_modified_toast();
                }

                appwindow.refresh_titles(&appwindow.active_tab_wrapper());
            }),
        );

        // set scale factor initially
        let _ = self
            .engine_mut()
            .set_scale_factor(self.scale_factor() as f64);
        // and connect
        let appwindow_scalefactor =
            self.connect_notify_local(Some("scale-factor"), move |canvas, _pspec| {
                let widget_flags = canvas
                    .engine_mut()
                    .set_scale_factor(canvas.scale_factor() as f64);
                canvas.emit_handle_widget_flags(widget_flags);
            });

        // Update titles when there are changes
        let appwindow_unsaved_changes = self.connect_notify_local(
            Some("unsaved-changes"),
            clone!(@weak appwindow => move |_canvas, _pspec| {
                appwindow.refresh_titles(&appwindow.active_tab_wrapper());
            }),
        );

        // one per-appwindow property for touch-drawing
        let appwindow_touch_drawing = appwindow
            .bind_property("touch-drawing", self, "touch-drawing")
            .sync_create()
            .build();

        // bind cursors
        let appwindow_regular_cursor = appwindow
            .sidebar()
            .settings_panel()
            .general_regular_cursor_picker()
            .bind_property("picked", self, "regular-cursor")
            .transform_to(|_, v: Option<String>| v)
            .sync_create()
            .build();

        let appwindow_drawing_cursor = appwindow
            .sidebar()
            .settings_panel()
            .general_drawing_cursor_picker()
            .bind_property("picked", self, "drawing-cursor")
            .transform_to(|_, v: Option<String>| v)
            .sync_create()
            .build();

        // bind show-drawing-cursor
        let appwindow_show_drawing_cursor = appwindow
            .sidebar()
            .settings_panel()
            .general_show_drawing_cursor_row()
            .bind_property("active", self, "show-drawing-cursor")
            .sync_create()
            .build();

        // Drop Target
        // change the global state for dnd
        self.imp().drop_target.connect_enter(
            clone!(@weak self as canvas => @default-return gdk::DragAction::COPY, move |_,_,_| {
                    canvas.imp().dnd_status.set(true);
                    canvas.imp().dnd_respect_borders.set(false);
                    gdk::DragAction::COPY
            }),
        );

        self.imp().drop_target.connect_leave(
            clone!(@weak self as canvas => @default-return (), move |_| {
                    canvas.imp().dnd_status.set(false); // set the status to false
            }),
        );
        let appwindow_drop_target = self.imp().drop_target.connect_drop(
            clone!(@weak self as canvas, @weak appwindow => @default-return false, move |_, value, x, y| {
                let pos = (canvas.engine_ref().camera.transform().inverse() *
                    na::point![x,y]).coords;
                let mut accept_drop = false;

                // should we respect borders ?
                let respect_border = canvas.imp().dnd_respect_borders.get();

                if value.is::<gio::File>() {
                    // In some scenarios, get() can fail with `UnexpectedNone` even though is() returned true, e.g. when dealing with trashed files.
                    match value.get::<gio::File>() {
                        Ok(file) => {
                            glib::spawn_future_local(clone!(@weak appwindow => async move {
                                appwindow.open_file_w_dialogs(file, Some(pos), true,respect_border).await;
                            }));
                            accept_drop = true;
                        },
                        Err(e) => {
                            tracing::error!("Failed to get dropped in file, Err: {e:?}");
                            appwindow.overlays().dispatch_toast_error(&gettext("Inserting file failed"));
                        },
                    };
                } else if value.is::<String>() {
                    match canvas.load_in_text(value.get::<String>().unwrap(), Some(pos)) {
                        Ok(_) => {
                            accept_drop = true;
                        },
                        Err(e) => {
                            tracing::error!("Failed to insert dropped in text, Err: {e:?}");
                            appwindow.overlays().dispatch_toast_error(&gettext("Inserting text failed"));
                        }
                    };
                }

                accept_drop
            }),
        );

        // handle widget flags
        let appwindow_handle_widget_flags = self.connect_local(
            "handle-widget-flags",
            false,
            clone!(@weak self as canvas, @weak appwindow => @default-return None, move |args| {
                // first argument is the widget, second is widget flags
                let widget_flags = args[1].get::<WidgetFlagsBoxed>().unwrap().inner();

                appwindow.handle_widget_flags(widget_flags, &canvas);
                None
            }),
        );

        // Replace connections
        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections
            .appwindow_output_file
            .replace(appwindow_output_file)
        {
            self.disconnect(old);
        }
        if let Some(old) = connections
            .appwindow_scalefactor
            .replace(appwindow_scalefactor)
        {
            self.disconnect(old);
        }
        if let Some(old) = connections
            .appwindow_unsaved_changes
            .replace(appwindow_unsaved_changes)
        {
            self.disconnect(old);
        }
        if let Some(old) = connections
            .appwindow_touch_drawing
            .replace(appwindow_touch_drawing)
        {
            old.unbind();
        }
        if let Some(old) = connections
            .appwindow_show_drawing_cursor
            .replace(appwindow_show_drawing_cursor)
        {
            old.unbind();
        }
        if let Some(old) = connections
            .appwindow_regular_cursor
            .replace(appwindow_regular_cursor)
        {
            old.unbind();
        }
        if let Some(old) = connections
            .appwindow_drawing_cursor
            .replace(appwindow_drawing_cursor)
        {
            old.unbind();
        }
        if let Some(old) = connections
            .appwindow_drop_target
            .replace(appwindow_drop_target)
        {
            self.imp().drop_target.disconnect(old);
        }
        if let Some(old) = connections
            .appwindow_handle_widget_flags
            .replace(appwindow_handle_widget_flags)
        {
            self.disconnect(old);
        }
    }

    /// Disconnect all connections with references to external objects
    /// to prepare moving the widget to another appwindow or closing it,
    /// when it is inside a tab page.
    pub(crate) fn disconnect_connections(&self) {
        self.clear_output_file_watcher();

        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections.appwindow_output_file.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_scalefactor.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_unsaved_changes.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_touch_drawing.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_show_drawing_cursor.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_regular_cursor.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_drawing_cursor.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_drop_target.take() {
            self.imp().drop_target.disconnect(old);
        }
        if let Some(old) = connections.appwindow_handle_widget_flags.take() {
            self.disconnect(old);
        }

        // tab page connections
        if let Some(old) = connections.tab_page_output_file.take() {
            old.unbind();
        }
        if let Some(old) = connections.tab_page_unsaved_changes.take() {
            old.unbind();
        }
    }

    /// When the widget is the child of a tab page, we want to connect their titles, icons, ..
    ///
    /// disconnects existing connections to old tab pages.
    pub(crate) fn connect_to_tab_page(&self, page: &adw::TabPage) {
        // update the tab title whenever the canvas output file changes
        let tab_page_output_file = self
            .bind_property("output-file", page, "title")
            .sync_create()
            .transform_to(|b, _output_file: Option<gio::File>| {
                Some(
                    b.source()?
                        .downcast::<RnCanvas>()
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

        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections
            .tab_page_output_file
            .replace(tab_page_output_file)
        {
            old.unbind();
        }
        if let Some(old) = connections
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
}
