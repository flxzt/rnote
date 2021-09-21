mod imp {
    use std::{cell::Cell, rc::Rc};

    use adw::{prelude::*, subclass::prelude::*};
    use gtk4::{
        gdk, gio, glib, glib::clone, subclass::prelude::*, Box, Button, CompositeTemplate,
        CssProvider, Entry, Grid, InfoBar, Inhibit, Label, Overlay, PackType, ScrolledWindow,
        StyleContext, ToggleButton,
    };

    use crate::{
        config, ui::canvas::Canvas, ui::mainheader::MainHeader, ui::penssidebar::PensSideBar,
        ui::selectionmodifier::SelectionModifier, ui::workspacebrowser::WorkspaceBrowser,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/felixzwettler/rnote/ui/appwindow.ui")]
    pub struct RnoteAppWindow {
        pub settings: gio::Settings,
        #[template_child]
        pub main_grid: TemplateChild<Grid>,
        #[template_child]
        pub infobar: TemplateChild<InfoBar>,
        #[template_child]
        pub infobar_label: TemplateChild<Label>,
        #[template_child]
        pub canvas_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub canvas: TemplateChild<Canvas>,
        #[template_child]
        pub canvas_overlay: TemplateChild<Overlay>,
        #[template_child]
        pub selection_modifier: TemplateChild<SelectionModifier>,
        #[template_child]
        pub sidebar_grid: TemplateChild<Grid>,
        #[template_child]
        pub flap: TemplateChild<adw::Flap>,
        #[template_child]
        pub open_workspace_button: TemplateChild<Button>,
        #[template_child]
        pub workspace_pathup_button: TemplateChild<Button>,
        #[template_child]
        pub workspace_grid: TemplateChild<Grid>,
        #[template_child]
        pub workspace_headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub workspace_pathentry: TemplateChild<Entry>,
        #[template_child]
        pub workspacebrowser: TemplateChild<WorkspaceBrowser>,
        #[template_child]
        pub flapreveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub flaphide_button: TemplateChild<Button>,
        #[template_child]
        pub flaphide_box: TemplateChild<Box>,
        #[template_child]
        pub workspace_controlbox: TemplateChild<Box>,
        #[template_child]
        pub menus_box: TemplateChild<Box>,
        #[template_child]
        pub mainheader: TemplateChild<MainHeader>,
        #[template_child]
        pub penssidebar: TemplateChild<PensSideBar>,
    }

    impl Default for RnoteAppWindow {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(config::APP_ID),
                main_grid: TemplateChild::<Grid>::default(),
                infobar: TemplateChild::<InfoBar>::default(),
                infobar_label: TemplateChild::<Label>::default(),
                canvas_scroller: TemplateChild::<ScrolledWindow>::default(),
                canvas: TemplateChild::<Canvas>::default(),
                canvas_overlay: TemplateChild::<Overlay>::default(),
                selection_modifier: TemplateChild::<SelectionModifier>::default(),
                sidebar_grid: TemplateChild::<Grid>::default(),
                flap: TemplateChild::<adw::Flap>::default(),
                open_workspace_button: TemplateChild::<Button>::default(),
                workspace_pathup_button: TemplateChild::<Button>::default(),
                workspace_grid: TemplateChild::<Grid>::default(),
                workspace_headerbar: TemplateChild::<adw::HeaderBar>::default(),
                workspace_pathentry: TemplateChild::<Entry>::default(),
                workspacebrowser: TemplateChild::<WorkspaceBrowser>::default(),
                flapreveal_toggle: TemplateChild::<ToggleButton>::default(),
                flaphide_button: TemplateChild::<Button>::default(),
                flaphide_box: TemplateChild::<Box>::default(),
                workspace_controlbox: TemplateChild::<Box>::default(),
                menus_box: TemplateChild::<Box>::default(),
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

            let flap = self.flap.get();
            let workspace_headerbar = self.workspace_headerbar.get();
            let flapreveal_toggle = self.flapreveal_toggle.get();

            let _windowsettings = obj.settings();
            //windowsettings.set_gtk_application_prefer_dark_theme(true);

            flap.set_locked(true);
            flap.set_fold_policy(adw::FlapFoldPolicy::Auto);

            let css = CssProvider::new();
            css.load_from_resource((String::from(config::APP_IDPATH) + "ui/custom.css").as_str());

            let display = gdk::Display::default().unwrap();
            StyleContext::add_provider_for_display(
                &display,
                &css,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            let expanded_revealed = Rc::new(Cell::new(flap.reveals_flap()));

            self.flaphide_button.connect_clicked(
                clone!(@weak flap, @weak flapreveal_toggle => move |_flaphide_button| {
                    flapreveal_toggle.set_active(false);
                }),
            );

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

            self.flap
                .bind_property("folded", &self.flaphide_button.get(), "visible")
                .flags(glib::BindingFlags::DEFAULT)
                .build()
                .unwrap();

            self.flap.connect_flap_position_notify(
                clone!(@weak workspace_headerbar, @strong expanded_revealed => move |flap| {
                    if !flap.is_folded() && flap.flap_position() == PackType::End {
                        workspace_headerbar.set_show_end_title_buttons(expanded_revealed.get());
                    } else {
                        workspace_headerbar.set_show_end_title_buttons(false);
                    }
                }),
            );

            self.open_workspace_button.get().connect_clicked(
                clone!(@weak obj => move |_open_workspace_button| {
                    obj.application().unwrap().activate_action("open-workspace", None);
                }),
            );

            self.workspace_pathup_button.get().connect_clicked(
                clone!(@weak obj => move |_workspace_pathup_button| {
                        if let Some(current_path) = obj.workspacebrowser().primary_path() {
                            if let Some(parent_path) = current_path.parent() {
                                obj.workspacebrowser().set_primary_path(parent_path);
                            }
                        }
                }),
            );

            // Load latest window state
            obj.load_window_size();
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnoteAppWindow {}

    impl WindowImpl for RnoteAppWindow {
        // Save window state right before the window will be closed
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            if let Err(err) = obj.save_window_size() {
                log::error!("Failed to save window state, {}", &err);
            }
            // Do not inhibit the default handler
            Inhibit(false)
        }
    }

    impl ApplicationWindowImpl for RnoteAppWindow {}
    impl AdwWindowImpl for RnoteAppWindow {}
    impl AdwApplicationWindowImpl for RnoteAppWindow {}
}

use std::{boxed, error::Error, path::PathBuf};

use gtk4::{
    gdk, gio, glib, glib::clone, prelude::*, subclass::prelude::*, Application, Box, Button, Entry,
    Grid, InfoBar, Label, Overlay, ResponseType, ScrolledWindow,
};

use crate::{
    app::RnoteApp,
    strokes::{bitmapimage::BitmapImage, vectorimage::VectorImage, StrokeStyle},
    ui::canvas::Canvas,
    ui::mainheader::MainHeader,
    ui::penssidebar::PensSideBar,
    ui::{actions, selectionmodifier::SelectionModifier, workspacebrowser::WorkspaceBrowser},
    utils,
};

glib::wrapper! {
    pub struct RnoteAppWindow(ObjectSubclass<imp::RnoteAppWindow>)
        @extends gtk4::Widget, gtk4::Window, adw::Window, gtk4::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl RnoteAppWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create `RnoteAppWindow`.")
    }

    pub fn app_settings(&self) -> &gio::Settings {
        &imp::RnoteAppWindow::from_instance(self).settings
    }

    pub fn main_grid(&self) -> Grid {
        imp::RnoteAppWindow::from_instance(self).main_grid.get()
    }

    pub fn infobar(&self) -> InfoBar {
        imp::RnoteAppWindow::from_instance(self).infobar.get()
    }

    pub fn infobar_label(&self) -> Label {
        imp::RnoteAppWindow::from_instance(self).infobar_label.get()
    }

    pub fn canvas_scroller(&self) -> ScrolledWindow {
        imp::RnoteAppWindow::from_instance(self)
            .canvas_scroller
            .get()
    }

    pub fn canvas_overlay(&self) -> Overlay {
        imp::RnoteAppWindow::from_instance(self)
            .canvas_overlay
            .get()
    }

    pub fn selection_modifier(&self) -> SelectionModifier {
        imp::RnoteAppWindow::from_instance(self)
            .selection_modifier
            .get()
    }

    pub fn sidebar_grid(&self) -> Grid {
        imp::RnoteAppWindow::from_instance(self).sidebar_grid.get()
    }

    pub fn canvas(&self) -> Canvas {
        imp::RnoteAppWindow::from_instance(self).canvas.get()
    }

    pub fn workspace_grid(&self) -> Grid {
        imp::RnoteAppWindow::from_instance(self)
            .workspace_grid
            .get()
    }

    pub fn workspace_headerbar(&self) -> adw::HeaderBar {
        imp::RnoteAppWindow::from_instance(self)
            .workspace_headerbar
            .get()
    }

    pub fn workspacebrowser(&self) -> WorkspaceBrowser {
        imp::RnoteAppWindow::from_instance(self)
            .workspacebrowser
            .get()
    }

    pub fn flap(&self) -> adw::Flap {
        imp::RnoteAppWindow::from_instance(self).flap.get()
    }

    pub fn flaphide_button(&self) -> Button {
        imp::RnoteAppWindow::from_instance(self)
            .flaphide_button
            .get()
    }

    pub fn flaphide_box(&self) -> Box {
        imp::RnoteAppWindow::from_instance(self).flaphide_box.get()
    }

    pub fn workspace_controlbox(&self) -> Box {
        imp::RnoteAppWindow::from_instance(self)
            .workspace_controlbox
            .get()
    }

    pub fn workspace_pathentry(&self) -> Entry {
        imp::RnoteAppWindow::from_instance(self)
            .workspace_pathentry
            .get()
    }

    pub fn menus_box(&self) -> Box {
        imp::RnoteAppWindow::from_instance(self).menus_box.get()
    }

    pub fn mainheader(&self) -> MainHeader {
        imp::RnoteAppWindow::from_instance(self).mainheader.get()
    }

    pub fn penssidebar(&self) -> PensSideBar {
        imp::RnoteAppWindow::from_instance(self).penssidebar.get()
    }

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = &imp::RnoteAppWindow::from_instance(self).settings;

        let size = self.default_size();

        settings.set_int("window-width", size.0)?;
        settings.set_int("window-height", size.1)?;
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
        priv_.penssidebar.get().brush_templatechooser().init(self);
        priv_.canvas.get().sheet().selection().init(self);
        priv_.selection_modifier.get().init(self);

        // Loading in input file
        if let Some(input_file) = self
            .application()
            .unwrap()
            .downcast::<RnoteApp>()
            .unwrap()
            .input_file()
            .borrow()
            .to_owned()
        {
            if let Err(e) = self.load_in_file(&input_file) {
                log::error!("failed to load in input file, {}", e);
            }
        }

        self.workspace_headerbar().connect_show_end_title_buttons_notify(clone!(@weak self as appwindow => move |_files_headerbar| {
            if appwindow.workspace_headerbar().shows_end_title_buttons() {
                appwindow.mainheader().menus_box().remove(&appwindow.mainheader().canvasmenu());
                appwindow.mainheader().menus_box().remove(&appwindow.mainheader().appmenu());
                appwindow.menus_box().append(&appwindow.mainheader().canvasmenu());
                appwindow.menus_box().append(&appwindow.mainheader().appmenu());
            } else {
                appwindow.menus_box().remove(&appwindow.mainheader().canvasmenu());
                appwindow.menus_box().remove(&appwindow.mainheader().appmenu());
                appwindow.mainheader().menus_box().append(&appwindow.mainheader().canvasmenu());
                appwindow.mainheader().menus_box().append(&appwindow.mainheader().appmenu());
            }
        }));

        self.infobar().connect_response(
            clone!(@weak self as appwindow => move |infobar, response| {
                match response {
                    ResponseType::Close => {
                        infobar.set_revealed(false);
                    },
                    _ => {}
                }
            }),
        );

        // This dictates the overlay children position and size
        self.canvas_overlay().connect_get_child_position(
            clone!(@weak self as appwindow => @default-return None, move |_canvas_overlay, widget| {
                 match widget.widget_name().as_str() {
                     "selection_modifier" => {
                        let selectionmodifier = widget.clone().downcast::<SelectionModifier>().unwrap();
                        let scalefactor = selectionmodifier.property("scalefactor").unwrap().get::<f64>().unwrap();

                         //Some(gdk::Rectangle {x: bounds.x().round() as i32, y: bounds.y().round() as i32, width: bounds.width().round() as i32, height: bounds.height().round() as i32})
                        if let Some(bounds) = &*appwindow.canvas().sheet().selection().bounds().borrow() {
                            let translate_node_size = ((bounds.maxs[0] - bounds.mins[0]).min( bounds.maxs[1] - bounds.mins[1] ) * scalefactor).round() as i32 - 2 * SelectionModifier::TRANSLATE_NODE_MARGIN;

                            appwindow.selection_modifier().translate_node().image().set_pixel_size(
                                translate_node_size.clamp(SelectionModifier::TRANSLATE_NODE_SIZE_MIN,
                                    SelectionModifier::TRANSLATE_NODE_SIZE_MAX
                            ));

                            Some(gdk::Rectangle {
                                x: (bounds.mins[0] * scalefactor).round() as i32 - SelectionModifier::RESIZE_NODE_SIZE,
                                y:  (bounds.mins[1] * scalefactor).round() as i32 - SelectionModifier::RESIZE_NODE_SIZE,
                                width: ((bounds.maxs[0] -  bounds.mins[0]) * scalefactor).round() as i32 + 2 * SelectionModifier::RESIZE_NODE_SIZE,
                                height: ((bounds.maxs[1] - bounds.mins[1]) * scalefactor).round() as i32 + 2 * SelectionModifier::RESIZE_NODE_SIZE,
                            })
                        } else { None }
                    },
                    _ => { None }
                }
            }),
        );

        // actions and settings AFTER widget callback declarations
        actions::setup_actions(self);
        actions::setup_accels(self);
        self.setup_settings();
    }

    // ### Settings are setup only at startup. Setting changes through gsettings / dconf might not be applied until app restarts
    fn setup_settings(&self) {
        let _priv_ = imp::RnoteAppWindow::from_instance(self);

        // overwriting theme so users can choose dark / light in appmenu
        self.settings().set_gtk_theme_name(Some("Adwaita"));

        // Workspace directory
        self.workspacebrowser().set_primary_path(&PathBuf::from(
            self.app_settings().string("workspace-dir").as_str(),
        ));

        // prefer dark / light theme
        self.app_settings()
            .bind(
                "prefer-dark-theme",
                &self.settings(),
                "gtk-application-prefer-dark-theme",
            )
            .flags(gio::SettingsBindFlags::DEFAULT)
            .build();

        self.app_settings()
            .bind(
                "prefer-dark-theme",
                &self.mainheader().appmenu().dark_theme_toggle(),
                "active",
            )
            .flags(gio::SettingsBindFlags::DEFAULT)
            .build();

        // Ui for right / left handed writers
        // Why isnt this working?
        //self.application().unwrap().change_action_state("righthanded", &self.app_settings().boolean("righthanded").to_variant());
        self.application()
            .unwrap()
            .activate_action("righthanded", None);
        self.application()
            .unwrap()
            .activate_action("righthanded", None);

        // Mouse drawing
        self.app_settings()
            .bind("mouse-drawing", &self.canvas(), "mouse-drawing")
            .flags(gio::SettingsBindFlags::DEFAULT)
            .build();

        // Sheet format
        self.canvas().sheet().change_format(
            self.app_settings()
                .value("sheet-format")
                .get::<(i32, i32, i32)>()
                .unwrap(),
        );

        // Format borders
        self.canvas()
            .sheet()
            .set_format_borders(self.app_settings().boolean("format-borders"));

        // Autoexpand height
        let autoexpand_height = self.app_settings().boolean("autoexpand-height");
        self.canvas()
            .sheet()
            .set_autoexpand_height(autoexpand_height);
        self.mainheader()
            .pageedit_revealer()
            .set_reveal_child(!autoexpand_height);

        // Visual Debugging
        self.app_settings()
            .bind("visual-debug", &self.canvas(), "visual-debug")
            .flags(gio::SettingsBindFlags::DEFAULT)
            .build();

        self.app_settings()
            .bind(
                "devel",
                &self
                    .penssidebar()
                    .brush_templatechooser()
                    .predefined_template_experimental_listboxrow(),
                "visible",
            )
            .flags(gio::SettingsBindFlags::DEFAULT)
            .build();
    }

    pub fn load_in_file(&self, file: &gio::File) -> Result<(), boxed::Box<dyn Error>> {
        match utils::FileType::lookup_file_type(&file) {
            utils::FileType::Rnote => {
                self.canvas().sheet().open_sheet(&file)?;

                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );
                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().selection().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );

                self.canvas().queue_resize();
                self.canvas().queue_draw();

                Ok(())
            }
            utils::FileType::Svg => {
                let pos = if let Some(vadjustment) = self.canvas_scroller().vadjustment() {
                    na::vector![
                        VectorImage::OFFSET_X_DEFAULT,
                        vadjustment.value() + VectorImage::OFFSET_Y_DEFAULT
                    ]
                } else {
                    na::vector![VectorImage::OFFSET_X_DEFAULT, VectorImage::OFFSET_Y_DEFAULT]
                };
                self.canvas().sheet().import_file_as_svg(pos, &file)?;

                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );
                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().selection().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );

                self.canvas()
                    .sheet()
                    .selection()
                    .emit_by_name("redraw", &[])
                    .unwrap();
                self.canvas().queue_draw();

                self.mainheader().selector_toggle().set_active(true);
                Ok(())
            }
            utils::FileType::BitmapImage => {
                let pos = if let Some(vadjustment) = self.canvas_scroller().vadjustment() {
                    na::vector![
                        BitmapImage::OFFSET_X_DEFAULT,
                        vadjustment.value() + BitmapImage::OFFSET_Y_DEFAULT
                    ]
                } else {
                    na::vector![BitmapImage::OFFSET_X_DEFAULT, BitmapImage::OFFSET_Y_DEFAULT]
                };
                self.canvas()
                    .sheet()
                    .import_file_as_bitmapimage(pos, &file)?;

                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );
                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().selection().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );

                self.canvas()
                    .sheet()
                    .selection()
                    .emit_by_name("redraw", &[])
                    .unwrap();
                self.canvas().queue_draw();

                self.mainheader().selector_toggle().set_active(true);

                Ok(())
            }
            utils::FileType::Folder | utils::FileType::Unknown => {
                log::warn!("tried to open unsupported file type.");
                Ok(())
            }
        }
    }
}
