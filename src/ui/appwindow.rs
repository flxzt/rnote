mod imp {
    use std::cell::RefCell;
    use std::{cell::Cell, rc::Rc};

    use adw::{prelude::*, subclass::prelude::*};
    use gtk4::{
        gdk, gio, glib, glib::clone, subclass::prelude::*, Box, CompositeTemplate, CssProvider,
        FileChooserNative, Grid, Inhibit, PackType, ScrolledWindow, StyleContext, ToggleButton,
    };
    use gtk4::{GestureDrag, PropagationPhase, Revealer, Separator};

    use crate::ui::appsettings;
    use crate::{
        app::RnoteApp, config, ui::canvas::Canvas, ui::develactions::DevelActions, ui::dialogs,
        ui::mainheader::MainHeader, ui::penssidebar::PensSideBar, ui::settingspanel::SettingsPanel,
        ui::workspacebrowser::WorkspaceBrowser,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/appwindow.ui")]
    pub struct RnoteAppWindow {
        pub settings: gio::Settings,
        pub filechoosernative: Rc<RefCell<Option<FileChooserNative>>>,
        #[template_child]
        pub main_grid: TemplateChild<Grid>,
        #[template_child]
        pub devel_actions_revealer: TemplateChild<Revealer>,
        #[template_child]
        pub devel_actions: TemplateChild<DevelActions>,
        #[template_child]
        pub canvas_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub canvas: TemplateChild<Canvas>,
        #[template_child]
        pub settings_panel: TemplateChild<SettingsPanel>,
        #[template_child]
        pub sidebar_grid: TemplateChild<Grid>,
        #[template_child]
        pub sidebar_sep: TemplateChild<Separator>,
        #[template_child]
        pub flap: TemplateChild<adw::Flap>,
        #[template_child]
        pub flap_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub flap_header: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub flap_resizer: TemplateChild<gtk4::Box>,
        #[template_child]
        pub flap_resizer_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub workspacebrowser: TemplateChild<WorkspaceBrowser>,
        #[template_child]
        pub flapreveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub flap_menus_box: TemplateChild<Box>,
        #[template_child]
        pub mainheader: TemplateChild<MainHeader>,
        #[template_child]
        pub penssidebar: TemplateChild<PensSideBar>,
    }

    impl Default for RnoteAppWindow {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(config::APP_ID),
                filechoosernative: Rc::new(RefCell::new(None)),
                main_grid: TemplateChild::<Grid>::default(),
                devel_actions_revealer: TemplateChild::<Revealer>::default(),
                devel_actions: TemplateChild::<DevelActions>::default(),
                canvas_scroller: TemplateChild::<ScrolledWindow>::default(),
                canvas: TemplateChild::<Canvas>::default(),
                settings_panel: TemplateChild::<SettingsPanel>::default(),
                sidebar_grid: TemplateChild::<Grid>::default(),
                sidebar_sep: TemplateChild::<Separator>::default(),
                flap: TemplateChild::<adw::Flap>::default(),
                flap_box: TemplateChild::<gtk4::Box>::default(),
                flap_header: TemplateChild::<adw::HeaderBar>::default(),
                flap_resizer: TemplateChild::<gtk4::Box>::default(),
                flap_resizer_box: TemplateChild::<gtk4::Box>::default(),
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let _windowsettings = obj.settings();

            // Load the application css
            let css = CssProvider::new();
            css.load_from_resource((String::from(config::APP_IDPATH) + "ui/custom.css").as_str());

            let display = gdk::Display::default().unwrap();
            StyleContext::add_provider_for_display(
                &display,
                &css,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            self.setup_flap(obj);

            // Load latest window state
            obj.load_window_size();
        }
    }

    impl WidgetImpl for RnoteAppWindow {}

    impl WindowImpl for RnoteAppWindow {
        // Save window state right before the window will be closed
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            if let Err(err) = obj.save_window_size() {
                log::error!("Failed to save window state, {}", &err);
            }

            if let Err(err) = appsettings::save_state_to_settings(&obj) {
                log::error!("Failed to save app state, {}", &err);
            }

            // Save current sheet
            if obj
                .application()
                .unwrap()
                .downcast::<RnoteApp>()
                .unwrap()
                .unsaved_changes()
            {
                dialogs::dialog_quit_save(obj);
            } else {
                obj.destroy();
            }
            // Inhibit (Overwrite) the default handler. This handler is then responsible for destoying the window.
            Inhibit(true)
        }
    }

    impl ApplicationWindowImpl for RnoteAppWindow {}
    impl AdwWindowImpl for RnoteAppWindow {}
    impl AdwApplicationWindowImpl for RnoteAppWindow {}

    impl RnoteAppWindow {
        // Setting up the sidebar flap
        fn setup_flap(&self, obj: &super::RnoteAppWindow) {
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
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
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
                .connect_folded_notify(clone!(@strong expanded_revealed, @weak flapreveal_toggle, @weak workspace_headerbar => move |flap| {
                    if flap.is_folded() {
                        flapreveal_toggle.set_active(false);
                    } else {
                        if flap.flap_position() == PackType::End {
                            workspace_headerbar.set_show_end_title_buttons(flap.reveals_flap());
                        }
                        if expanded_revealed.get() || flap.reveals_flap() {
                            expanded_revealed.set(true);
                            flapreveal_toggle.set_active(true);
                        }
                    }
                }));

            self.flap
                .connect_reveal_flap_notify(clone!(@weak workspace_headerbar => move |flap| {
                    if !flap.is_folded() && flap.flap_position() == PackType::End {
                        workspace_headerbar.set_show_end_title_buttons(flap.reveals_flap());
                    } else {
                        workspace_headerbar.set_show_end_title_buttons(false);
                    }
                }));

            self.flap.connect_flap_position_notify(
                clone!(@weak workspace_headerbar, @strong expanded_revealed => move |flap| {
                    if !flap.is_folded() && flap.flap_position() == PackType::End {
                        workspace_headerbar.set_show_end_title_buttons(expanded_revealed.get());
                    } else {
                        workspace_headerbar.set_show_end_title_buttons(false);
                    }
                }),
            );

            // Resizing the flap contents
            let resizer_drag_gesture = GestureDrag::builder()
                .name("resizer_drag_gesture")
                .propagation_phase(PropagationPhase::Capture)
                .build();
            self.flap_resizer.add_controller(&resizer_drag_gesture);

            // Dirty hack to stop resizing when it is switching from non-folded to folded or vice versa (else gtk crashes)
            let prev_folded = Rc::new(Cell::new(flap.is_folded()));

            resizer_drag_gesture.connect_drag_begin(clone!(@strong prev_folded, @weak flap, @weak flap_box => move |_resizer_drag_gesture, _x , _y| {
                    prev_folded.set(flap.is_folded());
            }));

            resizer_drag_gesture.connect_drag_update(clone!(@weak obj, @strong prev_folded, @weak flap, @weak flap_box, @weak flapreveal_toggle => move |_resizer_drag_gesture, x , _y| {
                if flap.is_folded() == prev_folded.get() {
                    // Set BEFORE new width request
                    prev_folded.set(flap.is_folded());

                    let new_width = if flap.flap_position() == PackType::Start {
                        flap_box.width() + x.ceil() as i32
                    } else {
                        flap_box.width() - x.floor() as i32
                    };
                    if new_width > 0 && new_width < obj.mainheader().width() - 64 {
                        flap_box.set_width_request(new_width);
                    }
                } else {
                    if flap.is_folded() {
                        flapreveal_toggle.set_active(true);
                    }
                }
            }));

            self.flap_resizer.set_cursor(
                gdk::Cursor::from_name(
                    "col-resize",
                    gdk::Cursor::from_name("default", None).as_ref(),
                )
                .as_ref(),
            );

            self.flap.get().connect_flap_position_notify(
                clone!(@weak flap_resizer_box, @weak flap_resizer, @weak flap_box => move |flap| {
                    if flap.flap_position() == PackType::Start {
                            flap_resizer_box.reorder_child_after(&flap_resizer, Some(&flap_box));
                    } else {
                            flap_resizer_box.reorder_child_after(&flap_box, Some(&flap_resizer));
                    }
                }),
            );
        }
    }
}

use std::{
    cell::{Cell, RefCell},
    path::Path,
    rc::Rc,
};

use adw::prelude::*;
use gtk4::{
    gdk, gio, glib, glib::clone, subclass::prelude::*, Application, Box, EventControllerScroll,
    EventControllerScrollFlags, EventSequenceState, FileChooserNative, GestureDrag, GestureZoom,
    Grid, Inhibit, PropagationPhase, Revealer, ScrolledWindow, Separator, ToggleButton,
};

use crate::{
    app::RnoteApp,
    strokes::{bitmapimage::BitmapImage, vectorimage::VectorImage},
    ui::canvas::Canvas,
    ui::develactions::DevelActions,
    ui::settingspanel::SettingsPanel,
    ui::{actions, workspacebrowser::WorkspaceBrowser},
    ui::{appsettings, penssidebar::PensSideBar},
    ui::{dialogs, mainheader::MainHeader},
    utils,
};

glib::wrapper! {
    pub struct RnoteAppWindow(ObjectSubclass<imp::RnoteAppWindow>)
        @extends gtk4::Widget, gtk4::Window, adw::Window, gtk4::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl RnoteAppWindow {
    pub const CANVAS_ZOOMGESTURE_THRESHOLD: f64 = 0.005; // Sets the delta threshold (eg. 0.01 = 1% ) when to update the canvas when doing a zoom gesture
    pub const CANVAS_ZOOM_SCROLL_STEP: f64 = 0.1; // Sets the canvas zoom scroll step in % for one unit of the event controller delta

    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create `RnoteAppWindow`.")
    }

    pub fn app_settings(&self) -> &gio::Settings {
        &imp::RnoteAppWindow::from_instance(self).settings
    }

    pub fn filechoosernative(&self) -> Rc<RefCell<Option<FileChooserNative>>> {
        imp::RnoteAppWindow::from_instance(self)
            .filechoosernative
            .clone()
    }

    pub fn main_grid(&self) -> Grid {
        imp::RnoteAppWindow::from_instance(self).main_grid.get()
    }

    pub fn devel_actions_revealer(&self) -> Revealer {
        imp::RnoteAppWindow::from_instance(self)
            .devel_actions_revealer
            .get()
    }

    pub fn devel_actions(&self) -> DevelActions {
        imp::RnoteAppWindow::from_instance(self).devel_actions.get()
    }

    pub fn canvas_scroller(&self) -> ScrolledWindow {
        imp::RnoteAppWindow::from_instance(self)
            .canvas_scroller
            .get()
    }

    pub fn canvas(&self) -> Canvas {
        imp::RnoteAppWindow::from_instance(self).canvas.get()
    }

    pub fn settings_panel(&self) -> SettingsPanel {
        imp::RnoteAppWindow::from_instance(self)
            .settings_panel
            .get()
    }

    pub fn sidebar_grid(&self) -> Grid {
        imp::RnoteAppWindow::from_instance(self).sidebar_grid.get()
    }

    pub fn sidebar_sep(&self) -> Separator {
        imp::RnoteAppWindow::from_instance(self).sidebar_sep.get()
    }

    pub fn flap_header(&self) -> adw::HeaderBar {
        imp::RnoteAppWindow::from_instance(self).flap_header.get()
    }

    pub fn workspacebrowser(&self) -> WorkspaceBrowser {
        imp::RnoteAppWindow::from_instance(self)
            .workspacebrowser
            .get()
    }

    pub fn flap(&self) -> adw::Flap {
        imp::RnoteAppWindow::from_instance(self).flap.get()
    }

    pub fn flapreveal_toggle(&self) -> ToggleButton {
        imp::RnoteAppWindow::from_instance(self)
            .flapreveal_toggle
            .get()
    }

    pub fn flap_menus_box(&self) -> Box {
        imp::RnoteAppWindow::from_instance(self)
            .flap_menus_box
            .get()
    }

    pub fn mainheader(&self) -> MainHeader {
        imp::RnoteAppWindow::from_instance(self).mainheader.get()
    }

    pub fn penssidebar(&self) -> PensSideBar {
        imp::RnoteAppWindow::from_instance(self).penssidebar.get()
    }

    pub fn set_color_scheme(&self, color_scheme: adw::ColorScheme) {
        self.application()
            .unwrap()
            .downcast::<RnoteApp>()
            .unwrap()
            .style_manager()
            .set_color_scheme(color_scheme);

        match color_scheme {
            adw::ColorScheme::Default => {
                self.app_settings()
                    .set_string("color-scheme", "default")
                    .unwrap();
                self.mainheader()
                    .appmenu()
                    .default_theme_toggle()
                    .set_active(true);
            }
            adw::ColorScheme::ForceLight => {
                self.app_settings()
                    .set_string("color-scheme", "force-light")
                    .unwrap();
                self.mainheader()
                    .appmenu()
                    .light_theme_toggle()
                    .set_active(true);
            }
            adw::ColorScheme::ForceDark => {
                self.app_settings()
                    .set_string("color-scheme", "force-dark")
                    .unwrap();
                self.mainheader()
                    .appmenu()
                    .dark_theme_toggle()
                    .set_active(true);
            }
            _ => {
                log::error!("unsupported color_scheme in set_color_scheme()");
            }
        }
    }

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = &imp::RnoteAppWindow::from_instance(self).settings;

        let mut width = self.width();
        let mut height = self.height();

        // Window would grow without subtracting this size. Why? I dont know
        width -= 122;
        height -= 122;

        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;
        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let width = self.app_settings().int("window-width");
        let height = self.app_settings().int("window-height");
        let is_maximized = self.app_settings().boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    // Must be called after application is associated with it else it fails
    pub fn init(&self) {
        let priv_ = imp::RnoteAppWindow::from_instance(self);

        priv_.workspacebrowser.get().init(self);
        priv_.canvas.get().init(self);
        priv_.mainheader.get().init(self);
        priv_.mainheader.get().canvasmenu().init(self);
        priv_.mainheader.get().appmenu().init(self);
        priv_.penssidebar.get().init(self);
        priv_.penssidebar.get().marker_page().init(self);
        priv_.penssidebar.get().brush_page().init(self);
        priv_
            .penssidebar
            .get()
            .brush_page()
            .templatechooser()
            .init(self);
        priv_.penssidebar.get().shaper_page().init(self);
        priv_.penssidebar.get().eraser_page().init(self);
        priv_.penssidebar.get().selector_page().init(self);
        priv_.settings_panel.get().init(self);
        priv_.devel_actions.get().init(self);
        priv_.canvas.get().sheet().format().init(self);
        priv_.canvas.get().selection_modifier().init(self);

        // Loading in input file
        if let Some(input_file) = self
            .application()
            .unwrap()
            .downcast::<RnoteApp>()
            .unwrap()
            .input_file()
            .to_owned()
        {
            if self
                .application()
                .unwrap()
                .downcast::<RnoteApp>()
                .unwrap()
                .unsaved_changes()
            {
                dialogs::dialog_open_overwrite(self);
            } else if let Err(e) = self.load_in_file(&input_file) {
                log::error!("failed to load in input file, {}", e);
            }
        }

        self.flap()
            .connect_reveal_flap_notify(clone!(@weak self as appwindow => move |flap| {
                if appwindow.mainheader().appmenu().parent().is_some() {
                    appwindow.mainheader().appmenu().unparent();
                }
                if flap.reveals_flap() && !flap.is_folded() {
                    appwindow.flap_menus_box().append(&appwindow.mainheader().appmenu());
                } else {
                    appwindow.mainheader().menus_box().append(&appwindow.mainheader().appmenu());
                }
            }));

        self.flap()
            .connect_folded_notify(clone!(@weak self as appwindow => move |flap| {
                if appwindow.mainheader().appmenu().parent().is_some() {
                    appwindow.mainheader().appmenu().unparent();
                }
                if flap.reveals_flap() && !flap.is_folded() {
                    appwindow.flap_menus_box().append(&appwindow.mainheader().appmenu());
                } else {
                    appwindow.mainheader().menus_box().append(&appwindow.mainheader().appmenu());
                }
            }));

        // zoom scrolling with <ctrl> + scroll
        let canvas_zoom_scroll_controller = EventControllerScroll::builder()
            .name("canvas_zoom_scroll_controller")
            .propagation_phase(PropagationPhase::Capture)
            .flags(EventControllerScrollFlags::VERTICAL)
            .build();

        canvas_zoom_scroll_controller.connect_scroll(clone!(@weak self as appwindow => @default-return Inhibit(false), move |zoom_scroll_controller, _dx, dy| {
            let total_zoom = appwindow.canvas().total_zoom();
            if zoom_scroll_controller.current_event_state() == gdk::ModifierType::CONTROL_MASK {
                let delta = dy * Self::CANVAS_ZOOM_SCROLL_STEP * total_zoom;
                let new_zoom = total_zoom - delta;

                // the sheet position BEFORE scaling
                let sheet_center_pos = na::vector![
                    ((appwindow.canvas().hadjustment().unwrap().value()
                        + f64::from(appwindow.canvas_scroller().width()) * 0.5
                        + appwindow.canvas().sheet_margin())
                        / total_zoom)
                        ,
                    ((appwindow.canvas().vadjustment().unwrap().value()
                        + f64::from(appwindow.canvas_scroller().height()) * 0.5
                        + appwindow.canvas().sheet_margin())
                        / total_zoom)
                ];

                appwindow.canvas().zoom_temporarily_then_scale_to_after_timeout(new_zoom, Canvas::ZOOM_TIMEOUT_TIME);

                // Reposition scroller center to the previous sheet position
                appwindow.canvas().center_around_coord_on_sheet(sheet_center_pos);
                // Stop event propagation
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        }));
        self.canvas_scroller()
            .add_controller(&canvas_zoom_scroll_controller);

        // Move Canvas with touch gesture
        let canvas_touch_drag_gesture = GestureDrag::builder()
            .name("canvas_touch_drag_gesture")
            .touch_only(true)
            .propagation_phase(PropagationPhase::Bubble)
            .build();

        let touch_drag_start_x = Rc::new(Cell::new(0.0));
        let touch_drag_start_y = Rc::new(Cell::new(0.0));

        canvas_touch_drag_gesture.connect_drag_begin(clone!(@strong touch_drag_start_x, @strong touch_drag_start_y, @weak self as appwindow => move |_canvas_touch_drag_gesture, _x, _y| {
            touch_drag_start_x.set(appwindow.canvas().hadjustment().unwrap().value());
            touch_drag_start_y.set(appwindow.canvas().vadjustment().unwrap().value());
        }));
        canvas_touch_drag_gesture.connect_drag_update(clone!(@strong touch_drag_start_x, @strong touch_drag_start_y, @weak self as appwindow => move |_canvas_touch_drag_gesture, x, y| {
            appwindow.canvas().hadjustment().unwrap().set_value(touch_drag_start_x.get() - x);
            appwindow.canvas().vadjustment().unwrap().set_value(touch_drag_start_y.get() - y);
        }));
        self.canvas_scroller()
            .add_controller(&canvas_touch_drag_gesture);

        // Move Canvas with middle mouse button
        let canvas_mouse_drag_gesture = GestureDrag::builder()
            .name("canvas_mouse_drag_gesture")
            .button(gdk::BUTTON_MIDDLE)
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.canvas_scroller()
            .add_controller(&canvas_mouse_drag_gesture);

        let mouse_drag_start_x = Rc::new(Cell::new(0.0));
        let mouse_drag_start_y = Rc::new(Cell::new(0.0));

        canvas_mouse_drag_gesture.connect_drag_begin(clone!(@strong mouse_drag_start_x, @strong mouse_drag_start_y, @weak self as appwindow => move |_canvas_mouse_drag_gesture, _x, _y| {
            mouse_drag_start_x.set(appwindow.canvas().hadjustment().unwrap().value());
            mouse_drag_start_y.set(appwindow.canvas().vadjustment().unwrap().value());
        }));
        canvas_mouse_drag_gesture.connect_drag_update(clone!(@strong mouse_drag_start_x, @strong mouse_drag_start_y, @weak self as appwindow => move |_canvas_mouse_drag_gesture, x, y| {
            appwindow.canvas().hadjustment().unwrap().set_value(mouse_drag_start_x.get() - x);
            appwindow.canvas().vadjustment().unwrap().set_value(mouse_drag_start_y.get() - y);
        }));

        // Canvas gesture zooming with preview and dragging
        let canvas_zoom_gesture = GestureZoom::builder()
            .name("canvas_zoom_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.canvas_scroller().add_controller(&canvas_zoom_gesture);

        let prev_zoom = Rc::new(Cell::new(1_f64));
        let scale_begin = Rc::new(Cell::new(1_f64));
        let new_zoom = Rc::new(Cell::new(self.canvas().zoom()));
        let zoomgesture_canvasscroller_start_pos = Rc::new(Cell::new((0.0, 0.0)));
        let zoomgesture_bbcenter_start: Rc<Cell<Option<(f64, f64)>>> = Rc::new(Cell::new(None));

        canvas_zoom_gesture.connect_begin(clone!(
            @strong scale_begin,
            @strong prev_zoom,
            @strong new_zoom,
            @strong zoomgesture_canvasscroller_start_pos,
            @strong zoomgesture_bbcenter_start,
            @weak self as appwindow => move |canvas_zoom_gesture, _eventsequence| {
                canvas_zoom_gesture.set_state(EventSequenceState::Claimed);

                scale_begin.set(appwindow.canvas().zoom());
                new_zoom.set(appwindow.canvas().zoom());

                prev_zoom.set(1.0);
                appwindow.canvas().zoom_temporarily_to(appwindow.canvas().zoom());

                zoomgesture_canvasscroller_start_pos.set(
                    (
                        appwindow.canvas().hadjustment().unwrap().value(),
                        appwindow.canvas().vadjustment().unwrap().value()
                    )
                );
                if let Some(bbcenter) = canvas_zoom_gesture.bounding_box_center() {
                    zoomgesture_bbcenter_start.set(Some(
                        bbcenter
                    ));
                }
        }));

        canvas_zoom_gesture.connect_scale_changed(
            clone!(@strong scale_begin, @strong new_zoom, @strong prev_zoom, @strong zoomgesture_canvasscroller_start_pos, @strong zoomgesture_bbcenter_start, @weak self as appwindow => move |canvas_zoom_gesture, zoom| {
                let new_zoom = if scale_begin.get() * zoom > Canvas::SCALE_MAX || scale_begin.get() * zoom < Canvas::SCALE_MIN {
                    prev_zoom.get()
                } else {
                    new_zoom.set(scale_begin.get() * zoom);
                    appwindow.canvas().zoom_temporarily_to(new_zoom.get());

                    prev_zoom.set(zoom);
                    zoom
                };

                if let Some(bbcenter) = canvas_zoom_gesture.bounding_box_center() {
                    if let Some(bbcenter_start) = zoomgesture_bbcenter_start.get() {
                        let bbcenter_delta = (
                            bbcenter.0 - bbcenter_start.0 * new_zoom,
                            bbcenter.1 - bbcenter_start.1 * new_zoom
                        );

                        appwindow.canvas().hadjustment().unwrap().set_value(
                            zoomgesture_canvasscroller_start_pos.get().0 * new_zoom - bbcenter_delta.0
                        );
                        appwindow.canvas().vadjustment().unwrap().set_value(
                            zoomgesture_canvasscroller_start_pos.get().1 * new_zoom - bbcenter_delta.1
                        );
                    } else {
                        // Setting the start position if connect_scale_start didn't set it
                        zoomgesture_bbcenter_start.set(Some(
                            bbcenter
                        ));
                    }
                }
            }),
        );

        canvas_zoom_gesture.connect_cancel(
            clone!(@strong scale_begin, @strong zoomgesture_bbcenter_start, @weak self as appwindow => move |canvas_zoom_gesture, _eventsequence| {
                canvas_zoom_gesture.set_state(EventSequenceState::Denied);

                zoomgesture_bbcenter_start.set(None);
            }),
        );

        canvas_zoom_gesture.connect_end(
            clone!(@strong scale_begin, @strong new_zoom, @strong zoomgesture_bbcenter_start, @weak self as appwindow => move |canvas_zoom_gesture, _eventsequence| {
                canvas_zoom_gesture.set_state(EventSequenceState::Denied);

                zoomgesture_bbcenter_start.set(None);
                appwindow.canvas().zoom_to(new_zoom.get());
            }),
        );

        // Gesture Grouping
        canvas_mouse_drag_gesture.group_with(&canvas_touch_drag_gesture);
        canvas_zoom_gesture.group_with(&canvas_touch_drag_gesture);

        // actions and settings AFTER widget callback declarations
        actions::setup_actions(self);
        actions::setup_accels(self);
        appsettings::load_settings(self);
    }

    /// Loads in a file of any supported type into the current sheet.
    pub fn load_in_file(&self, file: &gio::File) -> Result<(), anyhow::Error> {
        let app = self.application().unwrap().downcast::<RnoteApp>().unwrap();

        match utils::FileType::lookup_file_type(file) {
            utils::FileType::RnoteFile => {
                let (file_bytes, _) = file.load_bytes::<gio::Cancellable>(None)?;
                self.load_in_rnote_bytes(&file_bytes, file.path())?;
            }
            utils::FileType::VectorImageFile => {
                let (file_bytes, _) = file.load_bytes::<gio::Cancellable>(None)?;
                self.load_in_vectorimage_bytes(&file_bytes)?;
            }
            utils::FileType::BitmapImageFile => {
                let (file_bytes, _) = file.load_bytes::<gio::Cancellable>(None)?;
                self.load_in_bitmapimage_bytes(&file_bytes)?;
            }
            utils::FileType::Pdf => {
                let (file_bytes, _) = file.load_bytes::<gio::Cancellable>(None)?;
                self.load_in_pdf_bytes(&file_bytes)?;
            }
            utils::FileType::Folder => {
                log::warn!("tried to open folder as sheet.");
            }
            utils::FileType::UnknownFile => {
                log::warn!("tried to open a unsupported file type.");
                app.set_input_file(None);
            }
        }

        Ok(())
    }

    pub fn load_in_rnote_bytes<P>(&self, bytes: &[u8], path: Option<P>) -> Result<(), anyhow::Error>
    where
        P: AsRef<Path>,
    {
        let app = self.application().unwrap().downcast::<RnoteApp>().unwrap();
        self.canvas().sheet().open_sheet(bytes)?;

        // Loading the sheet properties into the format settings panel
        self.settings_panel().load_format(self.canvas().sheet());
        self.settings_panel().load_background(self.canvas().sheet());

        self.canvas().set_unsaved_changes(false);
        app.set_input_file(None);
        if let Some(path) = path {
            let file = gio::File::for_path(path);
            app.set_output_file(Some(&file), self);
        }

        self.canvas().set_unsaved_changes(false);
        self.canvas().set_empty(false);
        self.canvas().regenerate_content(true, true);

        Ok(())
    }

    pub fn load_in_vectorimage_bytes(&self, bytes: &[u8]) -> Result<(), anyhow::Error> {
        let app = self.application().unwrap().downcast::<RnoteApp>().unwrap();

        let pos = if let Some(vadjustment) = self.canvas().vadjustment() {
            na::vector![
                VectorImage::OFFSET_X_DEFAULT,
                vadjustment.value() + VectorImage::OFFSET_Y_DEFAULT
            ]
        } else {
            na::vector![VectorImage::OFFSET_X_DEFAULT, VectorImage::OFFSET_Y_DEFAULT]
        };
        self.canvas().sheet().import_bytes_as_svg(pos, bytes)?;

        self.canvas().set_unsaved_changes(true);
        self.mainheader().selector_toggle().set_active(true);
        app.set_input_file(None);

        self.canvas().set_unsaved_changes(true);
        self.canvas().set_empty(false);
        self.canvas().regenerate_content(true, true);
        self.canvas().selection_modifier().set_visible(true);

        Ok(())
    }

    pub fn load_in_bitmapimage_bytes(&self, bytes: &[u8]) -> Result<(), anyhow::Error> {
        let app = self.application().unwrap().downcast::<RnoteApp>().unwrap();

        let pos = if let Some(vadjustment) = self.canvas().vadjustment() {
            na::vector![
                BitmapImage::OFFSET_X_DEFAULT,
                vadjustment.value() + BitmapImage::OFFSET_Y_DEFAULT
            ]
        } else {
            na::vector![BitmapImage::OFFSET_X_DEFAULT, BitmapImage::OFFSET_Y_DEFAULT]
        };
        self.canvas()
            .sheet()
            .import_bytes_as_bitmapimage(pos, bytes)?;

        self.canvas().set_unsaved_changes(true);
        self.mainheader().selector_toggle().set_active(true);
        app.set_input_file(None);

        self.canvas().set_unsaved_changes(true);
        self.canvas().set_empty(false);
        self.canvas().regenerate_content(true, true);
        self.canvas().selection_modifier().set_visible(true);

        Ok(())
    }

    pub fn load_in_pdf_bytes(&self, bytes: &[u8]) -> Result<(), anyhow::Error> {
        let app = self.application().unwrap().downcast::<RnoteApp>().unwrap();

        let pos = if let Some(vadjustment) = self.canvas().vadjustment() {
            na::vector![
                BitmapImage::OFFSET_X_DEFAULT,
                vadjustment.value() + BitmapImage::OFFSET_Y_DEFAULT
            ]
        } else {
            na::vector![BitmapImage::OFFSET_X_DEFAULT, BitmapImage::OFFSET_Y_DEFAULT]
        };
        self.canvas()
            .sheet()
            .import_bytes_as_pdf_bitmap(pos, bytes)?;

        self.canvas().set_unsaved_changes(true);
        self.mainheader().selector_toggle().set_active(true);
        app.set_input_file(None);

        self.canvas().set_unsaved_changes(true);
        self.canvas().set_empty(false);
        self.canvas().regenerate_content(false, true);
        self.canvas().selection_modifier().set_visible(true);

        Ok(())
    }
}
