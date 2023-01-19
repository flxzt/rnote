mod appsettings;
mod appwindowactions;

use std::path::Path;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk4::{
    gdk, gio, glib, glib::clone, Align, Application, ArrowType, Box, Button, CompositeTemplate,
    CornerType, CssProvider, FileChooserNative, GestureDrag, Grid, IconTheme, Inhibit, PackType,
    PropagationPhase, ScrolledWindow, Separator, StyleContext, ToggleButton,
};
use once_cell::sync::Lazy;

use crate::canvas::RnoteCanvas;
use crate::{
    config,
    penssidebar::PensSideBar,
    settingspanel::SettingsPanel,
    workspacebrowser::WorkspaceBrowser,
    RnoteApp, RnoteCanvasWrapper, RnoteOverlays,
    {dialogs, mainheader::MainHeader},
};
use rnote_engine::{engine::EngineTask, WidgetFlags};

mod imp {
    use gtk4::PositionType;

    use super::*;

    #[allow(missing_debug_implementations)]
    #[derive(CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/appwindow.ui")]
    pub(crate) struct RnoteAppWindow {
        pub(crate) app_settings: gio::Settings,
        pub(crate) filechoosernative: Rc<RefCell<Option<FileChooserNative>>>,
        pub(crate) autosave_source_id: RefCell<Option<glib::SourceId>>,
        pub(crate) periodic_configsave_source_id: RefCell<Option<glib::SourceId>>,

        pub(crate) autosave: Cell<bool>,
        pub(crate) autosave_interval_secs: Cell<u32>,
        pub(crate) righthanded: Cell<bool>,
        pub(crate) touch_drawing: Cell<bool>,

        #[template_child]
        pub(crate) main_grid: TemplateChild<Grid>,
        #[template_child]
        pub(crate) overlays: TemplateChild<RnoteOverlays>,
        #[template_child]
        pub(crate) tabbar: TemplateChild<adw::TabBar>,
        #[template_child]
        pub(crate) settings_panel: TemplateChild<SettingsPanel>,
        #[template_child]
        pub(crate) sidebar_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) sidebar_box: TemplateChild<Grid>,
        #[template_child]
        pub(crate) sidebar_sep: TemplateChild<Separator>,
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
        pub(crate) workspacebrowser: TemplateChild<WorkspaceBrowser>,
        #[template_child]
        pub(crate) flapreveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) flap_menus_box: TemplateChild<Box>,
        #[template_child]
        pub(crate) mainheader: TemplateChild<MainHeader>,
        #[template_child]
        pub(crate) penssidebar: TemplateChild<PensSideBar>,
    }

    impl Default for RnoteAppWindow {
        fn default() -> Self {
            Self {
                app_settings: gio::Settings::new(config::APP_ID),
                filechoosernative: Rc::new(RefCell::new(None)),
                autosave_source_id: RefCell::new(None),
                periodic_configsave_source_id: RefCell::new(None),

                autosave: Cell::new(true),
                autosave_interval_secs: Cell::new(super::RnoteAppWindow::AUTOSAVE_INTERVAL_DEFAULT),
                righthanded: Cell::new(true),
                touch_drawing: Cell::new(false),

                main_grid: TemplateChild::<Grid>::default(),
                overlays: TemplateChild::<RnoteOverlays>::default(),
                tabbar: TemplateChild::<adw::TabBar>::default(),
                settings_panel: TemplateChild::<SettingsPanel>::default(),
                sidebar_scroller: TemplateChild::<ScrolledWindow>::default(),
                sidebar_box: TemplateChild::<Grid>::default(),
                sidebar_sep: TemplateChild::<Separator>::default(),
                flap: TemplateChild::<adw::Flap>::default(),
                flap_box: TemplateChild::<gtk4::Box>::default(),
                flap_header: TemplateChild::<adw::HeaderBar>::default(),
                flap_resizer: TemplateChild::<gtk4::Box>::default(),
                flap_resizer_box: TemplateChild::<gtk4::Box>::default(),
                flap_close_button: TemplateChild::<Button>::default(),
                flap_stack: TemplateChild::<adw::ViewStack>::default(),
                workspacebrowser: TemplateChild::<WorkspaceBrowser>::default(),
                flapreveal_toggle: TemplateChild::<ToggleButton>::default(),
                flap_menus_box: TemplateChild::<Box>::default(),
                mainheader: TemplateChild::<MainHeader>::default(),
                penssidebar: TemplateChild::<PensSideBar>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteAppWindow {
        const NAME: &'static str = "RnoteAppWindow";
        type Type = super::RnoteAppWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnoteAppWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();
            let _windowsettings = inst.settings();

            if config::PROFILE == "devel" {
                inst.add_css_class("devel");
            }

            // Load the application css
            let css = CssProvider::new();
            css.load_from_resource((String::from(config::APP_IDPATH) + "ui/style.css").as_str());

            let display = gdk::Display::default().unwrap();
            StyleContext::add_provider_for_display(
                &display,
                &css,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            self.setup_tabbar();
            self.setup_flap();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    // autosave
                    glib::ParamSpecBoolean::new(
                        "autosave",
                        "autosave",
                        "autosave",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    // autosave interval in secs
                    glib::ParamSpecUInt::new(
                        "autosave-interval-secs",
                        "autosave-interval-secs",
                        "autosave-interval-secs",
                        5,
                        u32::MAX,
                        super::RnoteAppWindow::AUTOSAVE_INTERVAL_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    // righthanded
                    glib::ParamSpecBoolean::new(
                        "righthanded",
                        "righthanded",
                        "righthanded",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    // Whether to enable touch drawing
                    glib::ParamSpecBoolean::new(
                        "touch-drawing",
                        "touch-drawing",
                        "touch-drawing",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "autosave" => self.autosave.get().to_value(),
                "autosave-interval-secs" => self.autosave_interval_secs.get().to_value(),
                "righthanded" => self.righthanded.get().to_value(),
                "touch-drawing" => self.touch_drawing.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "autosave" => {
                    let autosave = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`.");

                    self.autosave.replace(autosave);

                    if autosave {
                        self.update_autosave_handler();
                    } else if let Some(autosave_source_id) =
                        self.autosave_source_id.borrow_mut().take()
                    {
                        autosave_source_id.remove();
                    }
                }
                "autosave-interval-secs" => {
                    let autosave_interval_secs = value
                        .get::<u32>()
                        .expect("The value needs to be of type `u32`.");

                    self.autosave_interval_secs.replace(autosave_interval_secs);

                    if self.autosave.get() {
                        self.update_autosave_handler();
                    }
                }
                "righthanded" => {
                    let righthanded = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`.");

                    self.righthanded.replace(righthanded);

                    self.handle_righthanded_property(righthanded);
                }
                "touch-drawing" => {
                    let touch_drawing: bool =
                        value.get().expect("The value needs to be of type `bool`.");
                    self.touch_drawing.replace(touch_drawing);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnoteAppWindow {}

    impl WindowImpl for RnoteAppWindow {
        // Save window state right before the window will be closed
        fn close_request(&self) -> Inhibit {
            let inst = self.instance().to_owned();

            // Save current doc
            if inst.tabs_any_unsaved_changes() {
                glib::MainContext::default().spawn_local(
                    clone!(@weak inst as appwindow => async move {
                        dialogs::dialog_close_window(&inst).await;
                    }),
                );
            } else {
                inst.close_force();
            }

            // Inhibit (Overwrite) the default handler. This handler is then responsible for destoying the window.
            Inhibit(true)
        }
    }

    impl ApplicationWindowImpl for RnoteAppWindow {}
    impl AdwWindowImpl for RnoteAppWindow {}
    impl AdwApplicationWindowImpl for RnoteAppWindow {}

    impl RnoteAppWindow {
        fn update_autosave_handler(&self) {
            let inst = self.instance();

            if let Some(removed_id) = self.autosave_source_id.borrow_mut().replace(glib::source::timeout_add_seconds_local(self.autosave_interval_secs.get(),
                clone!(@weak inst as appwindow => @default-return glib::source::Continue(false), move || {
                    let canvas = appwindow.active_tab().canvas();

                    if let Some(output_file) = canvas.output_file() {
                        glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            if let Err(e) = canvas.save_document_to_file(&output_file).await {
                                canvas.set_output_file(None);

                                log::error!("saving document failed with error `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed."));
                            }
                        }
                    ));
                }

                glib::source::Continue(true)
            }))) {
                removed_id.remove();
            }
        }

        fn setup_tabbar(&self) {
            self.tabbar.set_view(Some(&self.overlays.tabview()));
        }

        // Setting up the sidebar flap
        fn setup_flap(&self) {
            let inst = self.instance();
            let flap = self.flap.get();
            let flap_box = self.flap_box.get();
            let flap_resizer = self.flap_resizer.get();
            let flap_resizer_box = self.flap_resizer_box.get();
            let workspace_headerbar = self.flap_header.get();
            let flapreveal_toggle = self.flapreveal_toggle.get();

            flap.set_locked(true);
            flap.set_fold_policy(adw::FlapFoldPolicy::Auto);

            let expanded_revealed = Rc::new(Cell::new(flap.reveals_flap()));

            self.flapreveal_toggle
                .bind_property("active", &flap, "reveal-flap")
                .sync_create()
                .bidirectional()
                .build();

            self.flapreveal_toggle.connect_toggled(
                clone!(@weak flap, @strong expanded_revealed => move |flapreveal_toggle| {
                    flap.set_reveal_flap(flapreveal_toggle.is_active());
                    if !flap.is_folded() {
                        expanded_revealed.set(flapreveal_toggle.is_active());
                    }
                }),
            );

            self.flap
                .connect_folded_notify(clone!(@weak inst as appwindow, @strong expanded_revealed, @weak flapreveal_toggle, @weak workspace_headerbar => move |flap| {
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
                        flapreveal_toggle.set_active(false);
                    } else if expanded_revealed.get() || flap.reveals_flap() {
                        expanded_revealed.set(true);
                        flapreveal_toggle.set_active(true);
                    }

                    if flap.flap_position() == PackType::Start {
                        workspace_headerbar.set_show_start_title_buttons(flap.reveals_flap());
                        workspace_headerbar.set_show_end_title_buttons(false);
                    } else if flap.flap_position() == PackType::End {
                        workspace_headerbar.set_show_start_title_buttons(false);
                        workspace_headerbar.set_show_end_title_buttons(flap.reveals_flap());
                    }
                }));

            self.flap
                .connect_reveal_flap_notify(clone!(@weak inst as appwindow, @weak workspace_headerbar => move |flap| {
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
                }));

            self.flap.connect_flap_position_notify(
                clone!(@weak flap_resizer_box, @weak flap_resizer, @weak flap_box, @weak workspace_headerbar, @strong expanded_revealed, @weak inst as appwindow => move |flap| {
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
                }),
            );

            // Resizing the flap contents
            let resizer_drag_gesture = GestureDrag::builder()
                .name("resizer_drag_gesture")
                .propagation_phase(PropagationPhase::Capture)
                .build();
            self.flap_resizer.add_controller(&resizer_drag_gesture);

            // hack to stop resizing when it is switching from non-folded to folded or vice versa (else gtk crashes)
            let prev_folded = Rc::new(Cell::new(self.flap.get().is_folded()));

            resizer_drag_gesture.connect_drag_begin(clone!(@strong prev_folded, @weak flap, @weak flap_box => move |_resizer_drag_gesture, _x , _y| {
                    prev_folded.set(flap.is_folded());
            }));

            resizer_drag_gesture.connect_drag_update(clone!(@weak inst as appwindow, @strong prev_folded, @weak flap, @weak flap_box, @weak flapreveal_toggle => move |_resizer_drag_gesture, x , _y| {
                if flap.is_folded() == prev_folded.get() {
                    // Set BEFORE new width request
                    prev_folded.set(flap.is_folded());

                    let new_width = if flap.flap_position() == PackType::Start {
                        flap_box.width() + x.ceil() as i32
                    } else {
                        flap_box.width() - x.floor() as i32
                    };

                    if new_width > 0 && new_width < appwindow.mainheader().width() - super::RnoteAppWindow::FLAP_FOLDED_RESIZE_MARGIN as i32 {
                        flap_box.set_width_request(new_width);
                    }
                } else if flap.is_folded() {
                    flapreveal_toggle.set_active(true);
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
                clone!(@weak inst as appwindow => move |_flap_close_button| {
                    if appwindow.flap().reveals_flap() && appwindow.flap().is_folded() {
                        appwindow.flap().set_reveal_flap(false);
                    }
                }),
            );
        }

        fn handle_righthanded_property(&self, righthanded: bool) {
            let inst = self.instance();

            if righthanded {
                inst.flap().set_flap_position(PackType::Start);
                inst.main_grid().remove(&inst.overlays());
                inst.main_grid().remove(&inst.sidebar_sep());
                inst.main_grid().remove(&inst.sidebar_box());
                inst.main_grid().attach(&inst.overlays(), 2, 3, 1, 1);
                inst.main_grid().attach(&inst.sidebar_sep(), 1, 3, 1, 1);
                inst.main_grid().attach(&inst.sidebar_box(), 0, 3, 1, 1);
                inst.mainheader().quickactions_box().set_halign(Align::End);
                inst.mainheader()
                    .appmenu()
                    .righthanded_toggle()
                    .set_active(true);
                inst.workspacebrowser()
                    .grid()
                    .remove(&inst.workspacebrowser().workspacesbar());
                inst.workspacebrowser()
                    .grid()
                    .remove(&inst.workspacebrowser().files_scroller());
                inst.workspacebrowser().grid().attach(
                    &inst.workspacebrowser().workspacesbar(),
                    0,
                    0,
                    1,
                    1,
                );
                inst.workspacebrowser().grid().attach(
                    &inst.workspacebrowser().files_scroller(),
                    2,
                    0,
                    1,
                    1,
                );
                inst.workspacebrowser()
                    .files_scroller()
                    .set_window_placement(CornerType::TopRight);
                inst.workspacebrowser()
                    .workspacesbar()
                    .workspaces_scroller()
                    .set_window_placement(CornerType::TopRight);

                inst.sidebar_scroller()
                    .set_window_placement(CornerType::TopRight);
                inst.settings_panel()
                    .settings_scroller()
                    .set_window_placement(CornerType::TopRight);
                inst.penssidebar()
                    .brush_page()
                    .brushconfig_menubutton()
                    .set_direction(ArrowType::Right);
                inst.penssidebar()
                    .brush_page()
                    .brushstyle_menubutton()
                    .set_direction(ArrowType::Right);
                inst.penssidebar()
                    .brush_page()
                    .stroke_width_picker()
                    .set_position(PositionType::Left);
                inst.penssidebar()
                    .shaper_page()
                    .shaperstyle_menubutton()
                    .set_direction(ArrowType::Right);
                inst.penssidebar()
                    .shaper_page()
                    .shapeconfig_menubutton()
                    .set_direction(ArrowType::Right);
                inst.penssidebar()
                    .shaper_page()
                    .shapebuildertype_menubutton()
                    .set_direction(ArrowType::Right);
                inst.penssidebar()
                    .shaper_page()
                    .constraint_menubutton()
                    .set_direction(ArrowType::Right);
                inst.penssidebar()
                    .shaper_page()
                    .stroke_width_picker()
                    .set_position(PositionType::Left);
            } else {
                inst.flap().set_flap_position(PackType::End);
                inst.main_grid().remove(&inst.overlays());
                inst.main_grid().remove(&inst.sidebar_sep());
                inst.main_grid().remove(&inst.sidebar_box());
                inst.main_grid().attach(&inst.overlays(), 0, 3, 1, 1);
                inst.main_grid().attach(&inst.sidebar_sep(), 1, 3, 1, 1);
                inst.main_grid().attach(&inst.sidebar_box(), 2, 3, 1, 1);
                inst.mainheader()
                    .quickactions_box()
                    .set_halign(Align::Start);
                inst.mainheader()
                    .appmenu()
                    .lefthanded_toggle()
                    .set_active(true);
                inst.workspacebrowser()
                    .grid()
                    .remove(&inst.workspacebrowser().files_scroller());
                inst.workspacebrowser()
                    .grid()
                    .remove(&inst.workspacebrowser().workspacesbar());
                inst.workspacebrowser().grid().attach(
                    &inst.workspacebrowser().files_scroller(),
                    0,
                    0,
                    1,
                    1,
                );
                inst.workspacebrowser().grid().attach(
                    &inst.workspacebrowser().workspacesbar(),
                    2,
                    0,
                    1,
                    1,
                );
                inst.workspacebrowser()
                    .files_scroller()
                    .set_window_placement(CornerType::TopLeft);
                inst.workspacebrowser()
                    .workspacesbar()
                    .workspaces_scroller()
                    .set_window_placement(CornerType::TopLeft);

                inst.sidebar_scroller()
                    .set_window_placement(CornerType::TopLeft);
                inst.settings_panel()
                    .settings_scroller()
                    .set_window_placement(CornerType::TopLeft);
                inst.penssidebar()
                    .brush_page()
                    .brushconfig_menubutton()
                    .set_direction(ArrowType::Left);
                inst.penssidebar()
                    .brush_page()
                    .brushstyle_menubutton()
                    .set_direction(ArrowType::Left);
                inst.penssidebar()
                    .brush_page()
                    .stroke_width_picker()
                    .set_position(PositionType::Right);
                inst.penssidebar()
                    .shaper_page()
                    .shaperstyle_menubutton()
                    .set_direction(ArrowType::Left);
                inst.penssidebar()
                    .shaper_page()
                    .shapeconfig_menubutton()
                    .set_direction(ArrowType::Left);
                inst.penssidebar()
                    .shaper_page()
                    .shapebuildertype_menubutton()
                    .set_direction(ArrowType::Left);
                inst.penssidebar()
                    .shaper_page()
                    .constraint_menubutton()
                    .set_direction(ArrowType::Left);
                inst.penssidebar()
                    .shaper_page()
                    .stroke_width_picker()
                    .set_position(PositionType::Right);
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnoteAppWindow(ObjectSubclass<imp::RnoteAppWindow>)
        @extends gtk4::Widget, gtk4::Window, adw::Window, gtk4::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl RnoteAppWindow {
    const AUTOSAVE_INTERVAL_DEFAULT: u32 = 30;
    const PERIODIC_CONFIGSAVE_INTERVAL: u32 = 10;

    const FLAP_FOLDED_RESIZE_MARGIN: u32 = 64;

    pub(crate) fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)])
    }

    #[allow(unused)]
    pub(crate) fn autosave(&self) -> bool {
        self.property::<bool>("autosave")
    }

    #[allow(unused)]
    pub(crate) fn set_autosave(&self, autosave: bool) {
        self.set_property("autosave", autosave.to_value());
    }

    #[allow(unused)]
    pub(crate) fn autosave_interval_secs(&self) -> u32 {
        self.property::<u32>("autosave-interval-secs")
    }

    #[allow(unused)]
    pub(crate) fn set_autosave_interval_secs(&self, autosave_interval_secs: u32) {
        self.set_property("autosave-interval-secs", autosave_interval_secs.to_value());
    }

    #[allow(unused)]
    pub(crate) fn righthanded(&self) -> bool {
        self.property::<bool>("righthanded")
    }

    #[allow(unused)]
    pub(crate) fn set_righthanded(&self, righthanded: bool) {
        self.set_property("righthanded", righthanded.to_value());
    }

    #[allow(unused)]
    pub(crate) fn touch_drawing(&self) -> bool {
        self.property::<bool>("touch-drawing")
    }

    #[allow(unused)]
    pub(crate) fn set_touch_drawing(&self, touch_drawing: bool) {
        self.set_property("touch-drawing", touch_drawing.to_value());
    }

    pub(crate) fn app(&self) -> RnoteApp {
        self.application().unwrap().downcast::<RnoteApp>().unwrap()
    }

    pub(crate) fn app_settings(&self) -> gio::Settings {
        self.imp().app_settings.clone()
    }

    pub(crate) fn filechoosernative(&self) -> Rc<RefCell<Option<FileChooserNative>>> {
        self.imp().filechoosernative.clone()
    }

    pub(crate) fn main_grid(&self) -> Grid {
        self.imp().main_grid.get()
    }

    pub(crate) fn overlays(&self) -> RnoteOverlays {
        self.imp().overlays.get()
    }

    pub(crate) fn settings_panel(&self) -> SettingsPanel {
        self.imp().settings_panel.get()
    }

    pub(crate) fn sidebar_scroller(&self) -> ScrolledWindow {
        self.imp().sidebar_scroller.get()
    }

    pub(crate) fn sidebar_box(&self) -> Grid {
        self.imp().sidebar_box.get()
    }

    pub(crate) fn sidebar_sep(&self) -> Separator {
        self.imp().sidebar_sep.get()
    }

    pub(crate) fn flap_box(&self) -> gtk4::Box {
        self.imp().flap_box.get()
    }

    pub(crate) fn flap_header(&self) -> adw::HeaderBar {
        self.imp().flap_header.get()
    }

    pub(crate) fn workspacebrowser(&self) -> WorkspaceBrowser {
        self.imp().workspacebrowser.get()
    }

    pub(crate) fn flap(&self) -> adw::Flap {
        self.imp().flap.get()
    }

    pub(crate) fn flap_menus_box(&self) -> Box {
        self.imp().flap_menus_box.get()
    }

    pub(crate) fn flap_close_button(&self) -> Button {
        self.imp().flap_close_button.get()
    }

    pub(crate) fn flap_stack(&self) -> adw::ViewStack {
        self.imp().flap_stack.get()
    }

    pub(crate) fn mainheader(&self) -> MainHeader {
        self.imp().mainheader.get()
    }

    pub(crate) fn penssidebar(&self) -> PensSideBar {
        self.imp().penssidebar.get()
    }

    // Must be called after application is associated with it else it fails
    pub(crate) fn init(&self) {
        let imp = self.imp();

        imp.overlays.get().init(self);
        imp.workspacebrowser.get().init(self);
        imp.settings_panel.get().init(self);
        imp.mainheader.get().init(self);
        imp.mainheader.get().canvasmenu().init(self);
        imp.mainheader.get().appmenu().init(self);
        imp.penssidebar.get().init(self);
        imp.penssidebar.get().brush_page().init(self);
        imp.penssidebar.get().shaper_page().init(self);
        imp.penssidebar.get().typewriter_page().init(self);
        imp.penssidebar.get().eraser_page().init(self);
        imp.penssidebar.get().selector_page().init(self);
        imp.penssidebar.get().tools_page().init(self);

        // A first canvas. Must! come before binding the settings
        self.new_tab();

        // add icon theme resource path because automatic lookup does not work in the devel build.
        let app_icon_theme = IconTheme::for_display(&self.display());
        app_icon_theme.add_resource_path((String::from(config::APP_IDPATH) + "icons").as_str());

        // actions and settings AFTER widget inits
        self.setup_actions();
        self.setup_action_accels();
        self.setup_settings_binds();

        // Load settings
        self.load_settings();

        // Periodically save engine config
        if let Some(removed_id) = self.imp().periodic_configsave_source_id.borrow_mut().replace(
            glib::source::timeout_add_seconds_local(
                Self::PERIODIC_CONFIGSAVE_INTERVAL, clone!(@weak self as appwindow => @default-return glib::source::Continue(false), move || {
                    if let Err(e) = appwindow.save_engine_config_active_tab() {
                        log::error!("saving engine config in periodic task failed with Err: {e:?}");
                    }

                    glib::source::Continue(true)
        }))) {
            removed_id.remove();
        }

        self.init_misc();
    }

    // Anything that needs to be done right before showing the appwindow
    pub(crate) fn init_misc(&self) {
        // Set undo / redo as not sensitive as default ( setting it in .ui file did not work for some reason )
        self.mainheader().undo_button().set_sensitive(false);
        self.mainheader().redo_button().set_sensitive(false);

        // rerender the canvas
        self.active_tab().canvas().regenerate_background_pattern();
        self.active_tab().canvas().update_engine_rendering();

        adw::prelude::ActionGroupExt::activate_action(self, "refresh-ui-from-engine", None);
    }

    /// Called to close the window
    pub(crate) fn close_force(&self) {
        // Saving all state
        if let Err(e) = self.save_to_settings() {
            log::error!("Failed to save appwindow to settings, with Err: {e:?}");
        }

        // Closing the state tasks channel receiver
        if let Err(e) = self
            .active_tab()
            .canvas()
            .engine()
            .borrow()
            .tasks_tx()
            .unbounded_send(EngineTask::Quit)
        {
            log::error!("failed to send StateTask::Quit on store tasks_tx, Err: {e:?}");
        }

        self.destroy();
    }

    // Returns true if the flags indicate that any loop that handles the flags should be quit. (usually an async event loop)
    pub(crate) fn handle_widget_flags(&self, widget_flags: WidgetFlags, canvas: &RnoteCanvas) {
        if widget_flags.redraw {
            canvas.queue_draw();
        }
        if widget_flags.resize {
            canvas.queue_resize();
        }
        if widget_flags.refresh_ui {
            adw::prelude::ActionGroupExt::activate_action(self, "refresh-ui-from-engine", None);
        }
        if widget_flags.store_modified {
            canvas.set_unsaved_changes(true);
            canvas.set_empty(false);
        }
        if widget_flags.update_view {
            let camera_offset = canvas.engine().borrow().camera.offset;
            // this updates the canvas adjustment values with the ones from the camera
            canvas.update_camera_offset(camera_offset);
        }
        if let Some(hide_undo) = widget_flags.hide_undo {
            self.mainheader().undo_button().set_sensitive(!hide_undo);
        }
        if let Some(hide_redo) = widget_flags.hide_redo {
            self.mainheader().redo_button().set_sensitive(!hide_redo);
        }
        if let Some(enable_text_preprocessing) = widget_flags.enable_text_preprocessing {
            canvas.set_text_preprocessing(enable_text_preprocessing);
        }
    }

    pub(crate) fn save_engine_config_active_tab(&self) -> anyhow::Result<()> {
        let engine_config = self
            .active_tab()
            .canvas()
            .engine()
            .borrow()
            .save_engine_config()?;
        self.app_settings()
            .set_string("engine-config", engine_config.as_str())?;

        Ok(())
    }

    /// Get the active (selected) tab page. If there is none (which should only be the case on appwindow startup), we create one
    pub(crate) fn active_tab_page(&self) -> adw::TabPage {
        // We always create a single page, if there is none initially
        self.imp()
            .overlays
            .tabview()
            .selected_page()
            .unwrap_or_else(|| self.new_tab())
    }

    /// Get the active (selected) tab page child. If there is none (which should only be the case on appwindow startup), we create one
    pub(crate) fn active_tab(&self) -> RnoteCanvasWrapper {
        self.active_tab_page()
            .child()
            .downcast::<RnoteCanvasWrapper>()
            .unwrap()
    }

    /// Creates a new tab and set it as selected
    pub(crate) fn new_tab(&self) -> adw::TabPage {
        let new_wrapper = RnoteCanvasWrapper::new();

        // The tab page connections are handled in page_attached, which is fired when the page is added to the tabview
        let page = self.overlays().tabview().append(&new_wrapper);
        self.overlays().tabview().set_selected_page(&page);

        page
    }

    pub(crate) fn tab_pages_snapshot(&self) -> Vec<adw::TabPage> {
        self.overlays()
            .tabview()
            .pages()
            .snapshot()
            .into_iter()
            .map(|o| o.downcast::<adw::TabPage>().unwrap())
            .collect()
    }

    pub(crate) fn tabs_any_unsaved_changes(&self) -> bool {
        self.overlays()
            .tabview()
            .pages()
            .snapshot()
            .iter()
            .map(|o| {
                o.downcast_ref::<adw::TabPage>()
                    .unwrap()
                    .child()
                    .downcast_ref::<RnoteCanvasWrapper>()
                    .unwrap()
                    .canvas()
            })
            .any(|c| c.unsaved_changes())
    }

    pub(crate) fn tabs_query_file_opened(
        &self,
        input_file_path: impl AsRef<Path>,
    ) -> Option<adw::TabPage> {
        self.overlays()
            .tabview()
            .pages()
            .snapshot()
            .into_iter()
            .filter_map(|o| {
                let tab_page = o.downcast::<adw::TabPage>().unwrap();
                Some((
                    tab_page.clone(),
                    tab_page
                        .child()
                        .downcast_ref::<RnoteCanvasWrapper>()
                        .unwrap()
                        .canvas()
                        .output_file()?
                        .path()?,
                ))
            })
            .find(|(_, output_file_path)| {
                same_file::is_same_file(output_file_path, input_file_path.as_ref()).unwrap_or(false)
            })
            .map(|(found, _)| found)
    }

    pub(crate) fn clear_rendering_inactive_tabs(&self) {
        for inactive_page in self
            .overlays()
            .tabview()
            .pages()
            .snapshot()
            .into_iter()
            .map(|o| o.downcast::<adw::TabPage>().unwrap())
            .filter(|p| !p.is_selected())
        {
            inactive_page
                .child()
                .downcast::<RnoteCanvasWrapper>()
                .unwrap()
                .canvas()
                .engine()
                .borrow_mut()
                .clear_rendering();
        }
    }

    pub(crate) fn refresh_titles_active_tab(&self) {
        let canvas = self.active_tab().canvas();

        // Titles
        let title = canvas.doc_title_display();
        let subtitle = canvas.doc_folderpath_display();

        self.set_title(Some(
            &(title.clone() + " - " + config::APP_NAME_CAPITALIZED),
        ));

        self.mainheader()
            .main_title_unsaved_indicator()
            .set_visible(canvas.unsaved_changes());
        if canvas.unsaved_changes() {
            self.mainheader()
                .main_title()
                .add_css_class("unsaved_changes");
        } else {
            self.mainheader()
                .main_title()
                .remove_css_class("unsaved_changes");
        }

        self.mainheader().main_title().set_title(&title);
        self.mainheader().main_title().set_subtitle(&subtitle);
    }

    /// Opens the file, with import dialogs when appropriate.
    ///
    /// When the file is a rnote save file, `rnote_file_new_tab` determines if a new tab is opened, or if it overwrites the current active one.
    pub(crate) fn open_file_w_dialogs(
        &self,
        input_file: gio::File,
        target_pos: Option<na::Vector2<f64>>,
        rnote_file_new_tab: bool,
    ) {
        match crate::utils::FileType::lookup_file_type(&input_file) {
            crate::utils::FileType::RnoteFile => {
                let Some(input_file_path) = input_file.path() else {
                    log::error!("could not open file: {input_file:?}, path returned None");
                    return;
                };

                // If the file is already opened in a tab, simply switch to it
                if let Some(page) = self.tabs_query_file_opened(input_file_path) {
                    self.overlays().tabview().set_selected_page(&page);
                } else {
                    let canvas = if rnote_file_new_tab {
                        // open a new tab for rnote files
                        let new_tab = self.new_tab();
                        new_tab
                            .child()
                            .downcast::<RnoteCanvasWrapper>()
                            .unwrap()
                            .canvas()
                    } else {
                        self.active_tab().canvas()
                    };

                    if let Err(e) = self.load_in_file(input_file, target_pos, &canvas) {
                        log::error!(
                        "failed to load in file with FileType::RnoteFile | FileType::XoppFile, {e:?}"
                    );
                    }
                }
            }
            crate::utils::FileType::VectorImageFile | crate::utils::FileType::BitmapImageFile => {
                if let Err(e) =
                    self.load_in_file(input_file, target_pos, &self.active_tab().canvas())
                {
                    log::error!("failed to load in file with FileType::VectorImageFile / FileType::BitmapImageFile / FileType::Pdf, {e:?}");
                }
            }
            crate::utils::FileType::XoppFile => {
                // open a new tab for xopp file import
                let new_tab = self.new_tab();
                let canvas = new_tab
                    .child()
                    .downcast::<RnoteCanvasWrapper>()
                    .unwrap()
                    .canvas();

                dialogs::import::dialog_import_xopp_w_prefs(self, &canvas, input_file);
            }
            crate::utils::FileType::PdfFile => {
                dialogs::import::dialog_import_pdf_w_prefs(
                    self,
                    &self.active_tab().canvas(),
                    input_file,
                    target_pos,
                );
            }
            crate::utils::FileType::Folder => {
                if let Some(dir) = input_file.path() {
                    self.workspacebrowser()
                        .workspacesbar()
                        .set_selected_workspace_dir(dir);
                }
            }
            crate::utils::FileType::Unsupported => {
                log::error!("tried to open unsupported file type.");
            }
        }
    }

    /// Loads in a file of any supported type into the engine of the given canvas.
    ///
    /// ! if the file is a rnote save file, it will overwrite the state in the active tab so there should be a user prompt to confirm before this is called
    pub(crate) fn load_in_file(
        &self,
        file: gio::File,
        target_pos: Option<na::Vector2<f64>>,
        canvas: &RnoteCanvas,
    ) -> anyhow::Result<()> {
        glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak self as appwindow => async move {
            appwindow.overlays().start_pulsing_progressbar();

            match crate::utils::FileType::lookup_file_type(&file) {
                crate::utils::FileType::RnoteFile => {
                    match file.load_bytes_future().await {
                        Ok((bytes, _)) => {
                            if let Err(e) = canvas.load_in_rnote_bytes(bytes.to_vec(), file.path()).await {
                                log::error!("load_in_rnote_bytes() failed with Err: {e:?}");
                                appwindow.overlays().dispatch_toast_error(&gettext("Opening .rnote file failed."));
                            }
                        }
                        Err(e) => log::error!("failed to load bytes, Err: {e:?}"),
                    }
                }
                crate::utils::FileType::VectorImageFile => {
                    match file.load_bytes_future().await {
                        Ok((bytes, _)) => {
                            if let Err(e) = canvas.load_in_vectorimage_bytes(bytes.to_vec(), target_pos).await {
                                log::error!("load_in_vectorimage_bytes() failed with Err: {e:?}");
                                appwindow.overlays().dispatch_toast_error(&gettext("Opening vector image file failed."));
                            }
                        }
                        Err(e) => log::error!("failed to load bytes, Err: {e:?}"),
                    }
                }
                crate::utils::FileType::BitmapImageFile => {
                    match file.load_bytes_future().await {
                        Ok((bytes, _)) => {
                            if let Err(e) = canvas.load_in_bitmapimage_bytes(bytes.to_vec(), target_pos).await {
                                log::error!("load_in_bitmapimage_bytes() failed with Err: {e:?}");
                                appwindow.overlays().dispatch_toast_error(&gettext("Opening bitmap image file failed."));
                            }
                        }
                        Err(e) => log::error!("failed to load bytes, Err: {e:?}"),
                    }
                }
                crate::utils::FileType::XoppFile => {
                    match file.load_bytes_future().await {
                        Ok((bytes, _)) => {
                            if let Err(e) = canvas.load_in_xopp_bytes(bytes.to_vec()).await {
                                log::error!("load_in_xopp_bytes() failed with Err: {e:?}");
                                appwindow.overlays().dispatch_toast_error(&gettext("Opening Xournal++ file failed."));
                            }
                        }
                        Err(e) => log::error!("failed to load bytes, Err: {e:?}"),
                    }
                }
                crate::utils::FileType::PdfFile => {
                    match file.load_bytes_future().await {
                        Ok((bytes, _)) => {
                            if let Err(e) = canvas.load_in_pdf_bytes(bytes.to_vec(), target_pos, None).await {
                                log::error!("load_in_pdf_bytes() failed with Err: {e:?}");
                                appwindow.overlays().dispatch_toast_error(&gettext("Opening PDF file failed."));
                            }
                        }
                        Err(e) => log::error!("failed to load bytes, Err: {e:?}"),
                    }
                }
                crate::utils::FileType::Folder => {
                    log::error!("tried to open a folder as a file.");
                    appwindow.overlays()
                        .dispatch_toast_error(&gettext("Error: Tried opening folder as file"));
                }
                crate::utils::FileType::Unsupported => {
                    log::error!("tried to open a unsupported file type.");
                    appwindow.overlays()
                        .dispatch_toast_error(&gettext("Failed to open file: Unsupported file type."));
                }
            }

            appwindow.overlays().finish_progressbar();
        }));

        Ok(())
    }
}
