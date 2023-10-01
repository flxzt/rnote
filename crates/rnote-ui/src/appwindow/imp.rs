// Imports
use crate::{config, dialogs, RnMainHeader, RnOverlays, RnSidebar};
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk4::{
    gdk, gio, glib, glib::clone, Align, ArrowType, CompositeTemplate, CornerType, CssProvider,
    PackType, PadActionType, PadController, PositionType,
};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

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
    pub(crate) main_header: TemplateChild<RnMainHeader>,
    #[template_child]
    pub(crate) split_view: TemplateChild<adw::OverlaySplitView>,
    #[template_child]
    pub(crate) sidebar: TemplateChild<RnSidebar>,
    #[template_child]
    pub(crate) tabbar: TemplateChild<adw::TabBar>,
    #[template_child]
    pub(crate) overlays: TemplateChild<RnOverlays>,
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

            main_header: TemplateChild::<RnMainHeader>::default(),
            split_view: TemplateChild::<adw::OverlaySplitView>::default(),
            sidebar: TemplateChild::<RnSidebar>::default(),
            tabbar: TemplateChild::<adw::TabBar>::default(),
            overlays: TemplateChild::<RnOverlays>::default(),
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

        self.setup_input();
        self.setup_split_view();
        self.setup_tabbar();
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

    fn setup_tabbar(&self) {
        self.tabbar.set_view(Some(&self.overlays.tabview()));
    }

    fn setup_split_view(&self) {
        let obj = self.obj();
        let split_view = self.split_view.get();
        let left_sidebar_reveal_toggle = obj.main_header().left_sidebar_reveal_toggle();
        let right_sidebar_reveal_toggle = obj.main_header().right_sidebar_reveal_toggle();

        left_sidebar_reveal_toggle
            .bind_property("active", &right_sidebar_reveal_toggle, "active")
            .sync_create()
            .bidirectional()
            .build();

        left_sidebar_reveal_toggle
            .bind_property("active", &split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();
        right_sidebar_reveal_toggle
            .bind_property("active", &split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();

        let update_widgets = move |split_view: &adw::OverlaySplitView,
                                   appwindow: &super::RnAppWindow| {
            let sidebar_position = split_view.sidebar_position();
            let sidebar_collapsed = split_view.is_collapsed();
            let sidebar_shown = split_view.shows_sidebar();

            let sidebar_appmenu_visibility = !sidebar_collapsed && sidebar_shown;
            let sidebar_left_close_button_visibility =
                (sidebar_position == PackType::End) && sidebar_collapsed && sidebar_shown;
            let sidebar_right_close_button_visibility =
                (sidebar_position == PackType::Start) && sidebar_collapsed && sidebar_shown;

            appwindow
                .main_header()
                .appmenu()
                .set_visible(!sidebar_appmenu_visibility);
            appwindow
                .sidebar()
                .appmenu()
                .set_visible(sidebar_appmenu_visibility);
            appwindow
                .sidebar()
                .left_close_button()
                .set_visible(sidebar_left_close_button_visibility);
            appwindow
                .sidebar()
                .right_close_button()
                .set_visible(sidebar_right_close_button_visibility);

            if sidebar_position == PackType::End {
                appwindow
                    .sidebar()
                    .left_close_button()
                    .set_icon_name("right-symbolic");
                appwindow
                    .sidebar()
                    .right_close_button()
                    .set_icon_name("right-symbolic");
            } else {
                appwindow
                    .sidebar()
                    .left_close_button()
                    .set_icon_name("left-symbolic");
                appwindow
                    .sidebar()
                    .right_close_button()
                    .set_icon_name("left-symbolic");
            }
        };

        let sidebar_expanded_shown = Rc::new(Cell::new(false));

        self.split_view.connect_show_sidebar_notify(
            clone!(@strong sidebar_expanded_shown, @weak obj as appwindow => move |split_view| {
                if !split_view.is_collapsed() {
                    sidebar_expanded_shown.set(split_view.shows_sidebar());
                }
                update_widgets(split_view, &appwindow);
            }),
        );

        self.split_view.connect_sidebar_position_notify(
            clone!(@weak obj as appwindow => move |split_view| {
                update_widgets(split_view, &appwindow);
            }),
        );

        self.split_view.connect_collapsed_notify(
            clone!(@strong sidebar_expanded_shown, @weak obj as appwindow => move |split_view| {
                if split_view.is_collapsed() {
                    // Always hide sidebar when transitioning from non-collapsed to collapsed.
                    split_view.set_show_sidebar(false);
                } else {
                    // show sidebar again when it was shown before it was collapsed
                    if sidebar_expanded_shown.get() {
                        split_view.set_show_sidebar(true);
                    }
                    // update the shown state for when the sidebar was toggled shown in the collapsed state
                    sidebar_expanded_shown.set(split_view.shows_sidebar());
                }
                update_widgets(split_view, &appwindow);
            }),
        );
    }

    fn handle_righthanded_property(&self, righthanded: bool) {
        let obj = self.obj();

        if righthanded {
            obj.split_view().set_sidebar_position(PackType::Start);
            obj.main_header()
                .left_sidebar_reveal_toggle()
                .set_visible(true);
            obj.main_header()
                .right_sidebar_reveal_toggle()
                .set_visible(false);

            obj.sidebar()
                .workspacebrowser()
                .grid()
                .remove(&obj.sidebar().workspacebrowser().workspacesbar());
            obj.sidebar()
                .workspacebrowser()
                .grid()
                .remove(&obj.sidebar().workspacebrowser().corner_filler());
            obj.sidebar()
                .workspacebrowser()
                .grid()
                .remove(&obj.sidebar().workspacebrowser().dir_box());
            obj.sidebar()
                .workspacebrowser()
                .grid()
                .remove(&obj.sidebar().workspacebrowser().files_scroller());
            obj.sidebar().workspacebrowser().grid().attach(
                &obj.sidebar().workspacebrowser().corner_filler(),
                0,
                0,
                1,
                1,
            );
            obj.sidebar().workspacebrowser().grid().attach(
                &obj.sidebar().workspacebrowser().workspacesbar(),
                0,
                1,
                1,
                1,
            );
            obj.sidebar().workspacebrowser().grid().attach(
                &obj.sidebar().workspacebrowser().dir_box(),
                2,
                0,
                1,
                1,
            );
            obj.sidebar().workspacebrowser().grid().attach(
                &obj.sidebar().workspacebrowser().files_scroller(),
                2,
                1,
                1,
                1,
            );
            obj.sidebar()
                .workspacebrowser()
                .files_scroller()
                .set_window_placement(CornerType::TopRight);
            obj.sidebar()
                .workspacebrowser()
                .workspacesbar()
                .workspaces_scroller()
                .set_window_placement(CornerType::TopRight);

            obj.sidebar()
                .settings_panel()
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
            obj.split_view().set_sidebar_position(PackType::End);
            obj.main_header()
                .left_sidebar_reveal_toggle()
                .set_visible(false);
            obj.main_header()
                .right_sidebar_reveal_toggle()
                .set_visible(true);

            obj.sidebar()
                .workspacebrowser()
                .grid()
                .remove(&obj.sidebar().workspacebrowser().files_scroller());
            obj.sidebar()
                .workspacebrowser()
                .grid()
                .remove(&obj.sidebar().workspacebrowser().dir_box());
            obj.sidebar()
                .workspacebrowser()
                .grid()
                .remove(&obj.sidebar().workspacebrowser().corner_filler());
            obj.sidebar()
                .workspacebrowser()
                .grid()
                .remove(&obj.sidebar().workspacebrowser().workspacesbar());
            obj.sidebar().workspacebrowser().grid().attach(
                &obj.sidebar().workspacebrowser().dir_box(),
                0,
                0,
                1,
                1,
            );
            obj.sidebar().workspacebrowser().grid().attach(
                &obj.sidebar().workspacebrowser().files_scroller(),
                0,
                1,
                1,
                1,
            );
            obj.sidebar().workspacebrowser().grid().attach(
                &obj.sidebar().workspacebrowser().corner_filler(),
                2,
                0,
                1,
                1,
            );
            obj.sidebar().workspacebrowser().grid().attach(
                &obj.sidebar().workspacebrowser().workspacesbar(),
                2,
                1,
                1,
                1,
            );
            obj.sidebar()
                .workspacebrowser()
                .files_scroller()
                .set_window_placement(CornerType::TopLeft);
            obj.sidebar()
                .workspacebrowser()
                .workspacesbar()
                .workspaces_scroller()
                .set_window_placement(CornerType::TopLeft);

            obj.sidebar()
                .settings_panel()
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
