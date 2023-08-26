// Imports
use crate::{
    config, RnOverlays, RnSettingsPanel, RnWorkspaceBrowser, {dialogs, RnMainHeader},
};
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk4::{
    gdk, gio, glib, glib::clone, Align, ArrowType, Box, Button, CompositeTemplate, CornerType,
    CssProvider, GestureDrag, Grid, PackType, PadActionType, PadController, PositionType,
    PropagationPhase,
};
use once_cell::sync::Lazy;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Debug, CompositeTemplate)]
#[template(resource = "/com/github/flxzt/rnote/ui/appwindow.ui")]
pub(crate) struct RnAppWindow {
    pub(crate) app_settings: gio::Settings,
    pub(crate) drawing_pad_controller: RefCell<Option<PadController>>,
    pub(crate) autosave_source_id: RefCell<Option<glib::SourceId>>,
    pub(crate) periodic_configsave_source_id: RefCell<Option<glib::SourceId>>,

    pub(crate) autosave: Cell<bool>,
    pub(crate) autosave_interval_secs: Cell<u32>,
    pub(crate) righthanded: Cell<bool>,
    pub(crate) block_pinch_zoom: Cell<bool>,
    pub(crate) touch_drawing: Cell<bool>,
    pub(crate) focus_mode: Cell<bool>,

    #[template_child]
    pub(crate) main_grid: TemplateChild<Grid>,
    #[template_child]
    pub(crate) overlays: TemplateChild<RnOverlays>,
    #[template_child]
    pub(crate) tabbar: TemplateChild<adw::TabBar>,
    #[template_child]
    pub(crate) settings_panel: TemplateChild<RnSettingsPanel>,
    #[template_child]
    pub(crate) flap: TemplateChild<adw::Flap>,
    #[template_child]
    pub(crate) flap_box: TemplateChild<gtk4::Box>,
    #[template_child]
    pub(crate) flap_header: TemplateChild<adw::HeaderBar>,
    #[template_child]
    pub(crate) flap_resizer: TemplateChild<gtk4::Box>,
    #[template_child]
    pub(crate) flap_resizer_box: TemplateChild<gtk4::Box>,
    #[template_child]
    pub(crate) flap_close_button: TemplateChild<Button>,
    #[template_child]
    pub(crate) flap_stack: TemplateChild<adw::ViewStack>,
    #[template_child]
    pub(crate) workspacebrowser: TemplateChild<RnWorkspaceBrowser>,
    #[template_child]
    pub(crate) flap_menus_box: TemplateChild<Box>,
    #[template_child]
    pub(crate) mainheader: TemplateChild<RnMainHeader>,
}

impl Default for RnAppWindow {
    fn default() -> Self {
        Self {
            app_settings: gio::Settings::new(config::APP_ID),
            drawing_pad_controller: RefCell::new(None),
            autosave_source_id: RefCell::new(None),
            periodic_configsave_source_id: RefCell::new(None),

            autosave: Cell::new(true),
            autosave_interval_secs: Cell::new(super::RnAppWindow::AUTOSAVE_INTERVAL_DEFAULT),
            righthanded: Cell::new(true),
            block_pinch_zoom: Cell::new(false),
            touch_drawing: Cell::new(false),
            focus_mode: Cell::new(false),

            main_grid: TemplateChild::<Grid>::default(),
            overlays: TemplateChild::<RnOverlays>::default(),
            tabbar: TemplateChild::<adw::TabBar>::default(),
            settings_panel: TemplateChild::<RnSettingsPanel>::default(),
            flap: TemplateChild::<adw::Flap>::default(),
            flap_box: TemplateChild::<gtk4::Box>::default(),
            flap_header: TemplateChild::<adw::HeaderBar>::default(),
            flap_resizer: TemplateChild::<gtk4::Box>::default(),
            flap_resizer_box: TemplateChild::<gtk4::Box>::default(),
            flap_close_button: TemplateChild::<Button>::default(),
            flap_stack: TemplateChild::<adw::ViewStack>::default(),
            workspacebrowser: TemplateChild::<RnWorkspaceBrowser>::default(),
            flap_menus_box: TemplateChild::<Box>::default(),
            mainheader: TemplateChild::<RnMainHeader>::default(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RnAppWindow {
    const NAME: &'static str = "RnAppWindow";
    type Type = super::RnAppWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for RnAppWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        let _windowsettings = obj.settings();

        if config::PROFILE == "devel" {
            obj.add_css_class("devel");
        }

        // Load the application css
        let css = CssProvider::new();
        css.load_from_resource((String::from(config::APP_IDPATH) + "ui/style.css").as_str());

        let display = gdk::Display::default().unwrap();
        gtk4::style_context_add_provider_for_display(
            &display,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        self.setup_tabbar();
        self.setup_flap();
        self.setup_input();
    }

    fn dispose(&self) {
        self.dispose_template();
        while let Some(child) = self.obj().first_child() {
            child.unparent();
        }
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecBoolean::builder("autosave")
                    .default_value(false)
                    .build(),
                glib::ParamSpecUInt::builder("autosave-interval-secs")
                    .minimum(5)
                    .maximum(u32::MAX)
                    .default_value(super::RnAppWindow::AUTOSAVE_INTERVAL_DEFAULT)
                    .build(),
                glib::ParamSpecBoolean::builder("righthanded")
                    .default_value(false)
                    .build(),
                glib::ParamSpecBoolean::builder("block-pinch-zoom")
                    .default_value(false)
                    .build(),
                glib::ParamSpecBoolean::builder("touch-drawing")
                    .default_value(false)
                    .build(),
                glib::ParamSpecBoolean::builder("focus-mode")
                    .default_value(false)
                    .build(),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "autosave" => self.autosave.get().to_value(),
            "autosave-interval-secs" => self.autosave_interval_secs.get().to_value(),
            "righthanded" => self.righthanded.get().to_value(),
            "block-pinch-zoom" => self.block_pinch_zoom.get().to_value(),
            "touch-drawing" => self.touch_drawing.get().to_value(),
            "focus-mode" => self.focus_mode.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "autosave" => {
                let autosave = value
                    .get::<bool>()
                    .expect("The value needs to be of type `bool`");

                self.autosave.replace(autosave);

                if autosave {
                    self.update_autosave_handler();
                } else if let Some(autosave_source_id) = self.autosave_source_id.borrow_mut().take()
                {
                    autosave_source_id.remove();
                }
            }
            "autosave-interval-secs" => {
                let autosave_interval_secs = value
                    .get::<u32>()
                    .expect("The value needs to be of type `u32`");

                self.autosave_interval_secs.replace(autosave_interval_secs);

                if self.autosave.get() {
                    self.update_autosave_handler();
                }
            }
            "righthanded" => {
                let righthanded = value
                    .get::<bool>()
                    .expect("The value needs to be of type `bool`");

                self.righthanded.replace(righthanded);

                self.handle_righthanded_property(righthanded);
            }
            "block-pinch-zoom" => {
                let block_pinch_zoom: bool =
                    value.get().expect("The value needs to be of type `bool`");
                self.block_pinch_zoom.replace(block_pinch_zoom);
            }
            "touch-drawing" => {
                let touch_drawing: bool =
                    value.get().expect("The value needs to be of type `bool`");
                self.touch_drawing.replace(touch_drawing);
            }
            "focus-mode" => {
                let focus_mode: bool = value.get().expect("The value needs to be of type `bool`");
                self.focus_mode.replace(focus_mode);

                self.overlays.pens_toggles_box().set_visible(!focus_mode);
                self.overlays.colorpicker().set_visible(!focus_mode);
                self.overlays.sidebar_box().set_visible(!focus_mode);
            }
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for RnAppWindow {}

impl WindowImpl for RnAppWindow {
    // Save window state right before the window will be closed
    fn close_request(&self) -> glib::Propagation {
        let obj = self.obj().to_owned();

        // Save current doc
        if obj.tabs_any_unsaved_changes() {
            glib::MainContext::default().spawn_local(clone!(@weak obj as appwindow => async move {
                dialogs::dialog_close_window(&obj).await;
            }));
        } else {
            obj.close_force();
        }

        // Inhibit (Overwrite) the default handler. This handler is then responsible for destroying the window.
        glib::Propagation::Stop
    }
}

impl ApplicationWindowImpl for RnAppWindow {}
impl AdwWindowImpl for RnAppWindow {}
impl AdwApplicationWindowImpl for RnAppWindow {}

impl RnAppWindow {
    fn update_autosave_handler(&self) {
        let obj = self.obj();

        if let Some(removed_id) = self.autosave_source_id.borrow_mut().replace(glib::source::timeout_add_seconds_local(self.autosave_interval_secs.get(),
                clone!(@weak obj as appwindow => @default-return glib::ControlFlow::Break, move || {
                    let canvas = appwindow.active_tab_wrapper().canvas();

                    if let Some(output_file) = canvas.output_file() {
                        glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            if let Err(e) = canvas.save_document_to_file(&output_file).await {
                                canvas.set_output_file(None);

                                log::error!("saving document failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed"));
                            }
                        }
                    ));
                }

                glib::ControlFlow::Continue
            }))) {
                removed_id.remove();
            }
    }

    fn setup_tabbar(&self) {
        self.tabbar.set_view(Some(&self.overlays.tabview()));
    }

    // Setting up the sidebar flap
    fn setup_flap(&self) {
        let obj = self.obj();
        let flap = self.flap.get();
        let flap_box = self.flap_box.get();
        let flap_resizer = self.flap_resizer.get();
        let flap_resizer_box = self.flap_resizer_box.get();
        let workspace_headerbar = self.flap_header.get();
        let left_flapreveal_toggle = obj.mainheader().left_flapreveal_toggle();
        let right_flapreveal_toggle = obj.mainheader().right_flapreveal_toggle();

        flap.set_locked(true);
        flap.set_fold_policy(adw::FlapFoldPolicy::Auto);

        let expanded_revealed = Rc::new(Cell::new(flap.reveals_flap()));

        left_flapreveal_toggle
            .bind_property("active", &flap, "reveal-flap")
            .sync_create()
            .bidirectional()
            .build();
        right_flapreveal_toggle
            .bind_property("active", &flap, "reveal-flap")
            .sync_create()
            .bidirectional()
            .build();

        left_flapreveal_toggle.connect_toggled(
            clone!(@weak flap, @strong expanded_revealed => move |flapreveal_toggle| {
                flap.set_reveal_flap(flapreveal_toggle.is_active());
                if !flap.is_folded() {
                    expanded_revealed.set(flapreveal_toggle.is_active());
                }
            }),
        );

        right_flapreveal_toggle.connect_toggled(
            clone!(@weak flap, @strong expanded_revealed => move |flapreveal_toggle| {
                flap.set_reveal_flap(flapreveal_toggle.is_active());
                if !flap.is_folded() {
                    expanded_revealed.set(flapreveal_toggle.is_active());
                }
            }),
        );

        self.flap
                .connect_folded_notify(clone!(@weak obj as appwindow, @strong expanded_revealed, @weak left_flapreveal_toggle, @weak right_flapreveal_toggle, @weak workspace_headerbar => move |flap| {
                    if appwindow.mainheader().appmenu().parent().is_some() {
                        appwindow.mainheader().appmenu().unparent();
                    }

                    if flap.reveals_flap() && !flap.is_folded() {
                        // Set visible before appending, to avoid allocation glitch
                        appwindow.flap_menus_box().set_visible(true);
                        appwindow.flap_close_button().set_visible(false);
                        appwindow.flap_menus_box().append(&appwindow.mainheader().appmenu());
                    } else {
                        appwindow.flap_menus_box().set_visible(false);
                        appwindow.flap_close_button().set_visible(true);
                        appwindow.mainheader().menus_box().append(&appwindow.mainheader().appmenu());
                    }

                    if flap.is_folded() {
                        left_flapreveal_toggle.set_active(false);
                        right_flapreveal_toggle.set_active(false);
                    } else if expanded_revealed.get() || flap.reveals_flap() {
                        expanded_revealed.set(true);
                        left_flapreveal_toggle.set_active(true);
                        right_flapreveal_toggle.set_active(true);
                    }

                    if flap.flap_position() == PackType::Start {
                        workspace_headerbar.set_show_start_title_buttons(flap.reveals_flap());
                        workspace_headerbar.set_show_end_title_buttons(false);
                    } else if flap.flap_position() == PackType::End {
                        workspace_headerbar.set_show_start_title_buttons(false);
                        workspace_headerbar.set_show_end_title_buttons(flap.reveals_flap());
                    }
                }));

        self.flap.connect_reveal_flap_notify(
            clone!(@weak workspace_headerbar, @weak obj as appwindow => move |flap| {
                if appwindow.mainheader().appmenu().parent().is_some() {
                    appwindow.mainheader().appmenu().unparent();
                }

                if flap.reveals_flap() && !flap.is_folded() {
                    appwindow.flap_menus_box().set_visible(true);
                    appwindow.flap_close_button().set_visible(false);
                    appwindow.flap_menus_box().append(&appwindow.mainheader().appmenu());
                } else {
                    appwindow.flap_menus_box().set_visible(false);
                    appwindow.flap_close_button().set_visible(true);
                    appwindow.mainheader().menus_box().append(&appwindow.mainheader().appmenu());
                }

                if flap.flap_position() == PackType::Start {
                    workspace_headerbar.set_show_start_title_buttons(flap.reveals_flap());
                    workspace_headerbar.set_show_end_title_buttons(false);
                } else if flap.flap_position() == PackType::End {
                    workspace_headerbar.set_show_start_title_buttons(false);
                    workspace_headerbar.set_show_end_title_buttons(flap.reveals_flap());
                }
            }),
        );

        self.flap.connect_flap_position_notify(clone!(
            @weak flap_resizer_box,
            @weak flap_resizer,
            @weak flap_box,
            @weak workspace_headerbar,
            @strong expanded_revealed,
            @weak obj as appwindow => move |flap| {
            if flap.flap_position() == PackType::Start {
                workspace_headerbar.set_show_start_title_buttons(flap.reveals_flap());
                workspace_headerbar.set_show_end_title_buttons(false);

                flap_resizer_box.reorder_child_after(&flap_resizer, Some(&flap_box));

                appwindow.flap_header().remove(&appwindow.flap_close_button());
                appwindow.flap_header().pack_end(&appwindow.flap_close_button());
                appwindow.flap_close_button().set_icon_name("left-symbolic");
            } else if flap.flap_position() == PackType::End {
                workspace_headerbar.set_show_start_title_buttons(false);
                workspace_headerbar.set_show_end_title_buttons(flap.reveals_flap());

                flap_resizer_box.reorder_child_after(&flap_box, Some(&flap_resizer));

                appwindow.flap_header().remove(&appwindow.flap_close_button());
                appwindow.flap_header().pack_start(&appwindow.flap_close_button());
                appwindow.flap_close_button().set_icon_name("right-symbolic");
            }
        }));

        // Resizing the flap contents
        let resizer_drag_gesture = GestureDrag::builder()
            .name("resizer_drag_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.flap_resizer
            .add_controller(resizer_drag_gesture.clone());

        // hack to stop resizing when it is switching from non-folded to folded or vice versa (else gtk crashes)
        let prev_folded = Rc::new(Cell::new(self.flap.get().is_folded()));

        resizer_drag_gesture.connect_drag_begin(clone!(@strong prev_folded, @weak flap, @weak flap_box => move |_resizer_drag_gesture, _x , _y| {
                    prev_folded.set(flap.is_folded());
            }));

        resizer_drag_gesture.connect_drag_update(clone!(
            @strong prev_folded,
            @weak flap,
            @weak flap_box,
            @weak left_flapreveal_toggle,
            @weak right_flapreveal_toggle,
            @weak obj as appwindow => move |_resizer_drag_gesture, x , _y| {
                if flap.is_folded() == prev_folded.get() {
                    // Set BEFORE new width request
                    prev_folded.set(flap.is_folded());

                    let new_width = if flap.flap_position() == PackType::Start {
                        flap_box.width() + x.ceil() as i32
                    } else {
                        flap_box.width() - x.floor() as i32
                    };

                    if new_width > 0 && new_width < appwindow.mainheader().width() - super::RnAppWindow::FLAP_FOLDED_RESIZE_MARGIN as i32 {
                        flap_box.set_width_request(new_width);
                    }
                } else if flap.is_folded() {
                    left_flapreveal_toggle.set_active(true);
                    right_flapreveal_toggle.set_active(true);
                }
            }));

        self.flap_resizer.set_cursor(
            gdk::Cursor::from_name(
                "col-resize",
                gdk::Cursor::from_name("default", None).as_ref(),
            )
            .as_ref(),
        );

        self.flap_close_button.get().connect_clicked(
            clone!(@weak obj as appwindow => move |_flap_close_button| {
                if appwindow.flap().reveals_flap() && appwindow.flap().is_folded() {
                    appwindow.flap().set_reveal_flap(false);
                }
            }),
        );
    }

    fn setup_input(&self) {
        let obj = self.obj();
        let drawing_pad_controller = PadController::new(&*obj, None);

        drawing_pad_controller.set_action(
            PadActionType::Button,
            0,
            -1,
            &gettext("Button 1"),
            "drawing-pad-pressed-button-0",
        );
        drawing_pad_controller.set_action(
            PadActionType::Button,
            1,
            -1,
            &gettext("Button 2"),
            "drawing-pad-pressed-button-1",
        );
        drawing_pad_controller.set_action(
            PadActionType::Button,
            2,
            -1,
            &gettext("Button 3"),
            "drawing-pad-pressed-button-2",
        );
        drawing_pad_controller.set_action(
            PadActionType::Button,
            3,
            -1,
            &gettext("Button 4"),
            "drawing-pad-pressed-button-3",
        );

        obj.add_controller(drawing_pad_controller.clone());
        self.drawing_pad_controller
            .replace(Some(drawing_pad_controller));
    }

    fn handle_righthanded_property(&self, righthanded: bool) {
        let obj = self.obj();

        if righthanded {
            obj.flap().set_flap_position(PackType::Start);
            obj.mainheader().left_flapreveal_toggle().set_visible(true);
            obj.mainheader()
                .right_flapreveal_toggle()
                .set_visible(false);
            obj.mainheader()
                .appmenu()
                .righthanded_toggle()
                .set_active(true);

            obj.workspacebrowser()
                .grid()
                .remove(&obj.workspacebrowser().workspacesbar());
            obj.workspacebrowser()
                .grid()
                .remove(&obj.workspacebrowser().corner_filler());
            obj.workspacebrowser()
                .grid()
                .remove(&obj.workspacebrowser().dir_box());
            obj.workspacebrowser()
                .grid()
                .remove(&obj.workspacebrowser().files_scroller());
            obj.workspacebrowser().grid().attach(
                &obj.workspacebrowser().corner_filler(),
                0,
                0,
                1,
                1,
            );
            obj.workspacebrowser().grid().attach(
                &obj.workspacebrowser().workspacesbar(),
                0,
                1,
                1,
                1,
            );
            obj.workspacebrowser()
                .grid()
                .attach(&obj.workspacebrowser().dir_box(), 2, 0, 1, 1);
            obj.workspacebrowser().grid().attach(
                &obj.workspacebrowser().files_scroller(),
                2,
                1,
                1,
                1,
            );
            obj.workspacebrowser()
                .files_scroller()
                .set_window_placement(CornerType::TopRight);
            obj.workspacebrowser()
                .workspacesbar()
                .workspaces_scroller()
                .set_window_placement(CornerType::TopRight);

            obj.settings_panel()
                .settings_scroller()
                .set_window_placement(CornerType::TopRight);

            obj.overlays().sidebar_box().set_halign(Align::Start);
            obj.overlays()
                .sidebar_scroller()
                .set_window_placement(CornerType::TopRight);
            obj.overlays()
                .penssidebar()
                .brush_page()
                .brushconfig_menubutton()
                .set_direction(ArrowType::Right);
            obj.overlays()
                .penssidebar()
                .brush_page()
                .brushstyle_menubutton()
                .set_direction(ArrowType::Right);
            obj.overlays()
                .penssidebar()
                .brush_page()
                .stroke_width_picker()
                .set_position(PositionType::Left);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .shaperstyle_menubutton()
                .set_direction(ArrowType::Right);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .shapeconfig_menubutton()
                .set_direction(ArrowType::Right);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .shapebuildertype_menubutton()
                .set_direction(ArrowType::Right);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .constraint_menubutton()
                .set_direction(ArrowType::Right);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .stroke_width_picker()
                .set_position(PositionType::Left);
            obj.overlays()
                .penssidebar()
                .typewriter_page()
                .emojichooser_menubutton()
                .set_direction(ArrowType::Right);
            obj.overlays()
                .penssidebar()
                .eraser_page()
                .stroke_width_picker()
                .set_position(PositionType::Left);
        } else {
            obj.flap().set_flap_position(PackType::End);
            obj.mainheader().left_flapreveal_toggle().set_visible(false);
            obj.mainheader().right_flapreveal_toggle().set_visible(true);
            obj.mainheader()
                .appmenu()
                .lefthanded_toggle()
                .set_active(true);

            obj.workspacebrowser()
                .grid()
                .remove(&obj.workspacebrowser().files_scroller());
            obj.workspacebrowser()
                .grid()
                .remove(&obj.workspacebrowser().dir_box());
            obj.workspacebrowser()
                .grid()
                .remove(&obj.workspacebrowser().corner_filler());
            obj.workspacebrowser()
                .grid()
                .remove(&obj.workspacebrowser().workspacesbar());
            obj.workspacebrowser()
                .grid()
                .attach(&obj.workspacebrowser().dir_box(), 0, 0, 1, 1);
            obj.workspacebrowser().grid().attach(
                &obj.workspacebrowser().files_scroller(),
                0,
                1,
                1,
                1,
            );
            obj.workspacebrowser().grid().attach(
                &obj.workspacebrowser().corner_filler(),
                2,
                0,
                1,
                1,
            );
            obj.workspacebrowser().grid().attach(
                &obj.workspacebrowser().workspacesbar(),
                2,
                1,
                1,
                1,
            );
            obj.workspacebrowser()
                .files_scroller()
                .set_window_placement(CornerType::TopLeft);
            obj.workspacebrowser()
                .workspacesbar()
                .workspaces_scroller()
                .set_window_placement(CornerType::TopLeft);

            obj.settings_panel()
                .settings_scroller()
                .set_window_placement(CornerType::TopLeft);

            obj.overlays().sidebar_box().set_halign(Align::End);
            obj.overlays()
                .sidebar_scroller()
                .set_window_placement(CornerType::TopLeft);
            obj.overlays()
                .penssidebar()
                .brush_page()
                .brushconfig_menubutton()
                .set_direction(ArrowType::Left);
            obj.overlays()
                .penssidebar()
                .brush_page()
                .brushstyle_menubutton()
                .set_direction(ArrowType::Left);
            obj.overlays()
                .penssidebar()
                .brush_page()
                .stroke_width_picker()
                .set_position(PositionType::Right);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .shaperstyle_menubutton()
                .set_direction(ArrowType::Left);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .shapeconfig_menubutton()
                .set_direction(ArrowType::Left);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .shapebuildertype_menubutton()
                .set_direction(ArrowType::Left);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .constraint_menubutton()
                .set_direction(ArrowType::Left);
            obj.overlays()
                .penssidebar()
                .shaper_page()
                .stroke_width_picker()
                .set_position(PositionType::Right);
            obj.overlays()
                .penssidebar()
                .typewriter_page()
                .emojichooser_menubutton()
                .set_direction(ArrowType::Left);
            obj.overlays()
                .penssidebar()
                .eraser_page()
                .stroke_width_picker()
                .set_position(PositionType::Right);
        }
    }
}
