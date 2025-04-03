// Imports
use crate::appwindow::RnAppWindow;
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{gdk, glib, glib::clone};
use tracing::error;

impl RnAppWindow {
    /// Setup settings binds.
    pub(crate) fn setup_settings_binds(&self) -> anyhow::Result<()> {
        let app = self.app();
        let app_settings = app
            .app_settings()
            .ok_or_else(|| anyhow::anyhow!("Settings schema not found."))?;

        app.style_manager().connect_color_scheme_notify(clone!(
            #[weak]
            app_settings,
            move |style_manager| {
                let color_scheme = match style_manager.color_scheme() {
                    adw::ColorScheme::Default => String::from("default"),
                    adw::ColorScheme::ForceLight => String::from("force-light"),
                    adw::ColorScheme::ForceDark => String::from("force-dark"),
                    _ => String::from("default"),
                };

                if let Err(e) = app_settings.set_string("color-scheme", &color_scheme) {
                    error!("Failed to set setting `color-scheme`, Err: {e:?}");
                }
            }
        ));

        app_settings
            .bind("sidebar-show", &self.split_view(), "show-sidebar")
            .get_no_changes()
            .build();

        // autosave
        app_settings
            .bind("autosave", self, "autosave")
            .get_no_changes()
            .build();

        // autosave interval secs
        app_settings
            .bind("autosave-interval-secs", self, "autosave-interval-secs")
            .get_no_changes()
            .build();

        // righthanded
        app_settings
            .bind("righthanded", self, "righthanded")
            .get_no_changes()
            .build();

        // block pinch zoom
        app_settings
            .bind("block-pinch-zoom", self, "block-pinch-zoom")
            .get_no_changes()
            .build();

        // touch drawing
        app_settings
            .bind("touch-drawing", self, "touch-drawing")
            .get_no_changes()
            .build();

        // respect borders
        app_settings
            .bind("respect-borders", self, "respect-borders")
            .get_no_changes()
            .build();

        // show scrollbars
        app_settings
            .bind(
                "show-scrollbars",
                &self
                    .sidebar()
                    .settings_panel()
                    .general_show_scrollbars_row(),
                "active",
            )
            .get_no_changes()
            .build();

        // inertial scrolling
        app_settings
            .bind(
                "inertial-scrolling",
                &self
                    .sidebar()
                    .settings_panel()
                    .general_inertial_scrolling_row(),
                "active",
            )
            .get_no_changes()
            .build();

        // regular cursor
        app_settings
            .bind(
                "regular-cursor",
                &self
                    .sidebar()
                    .settings_panel()
                    .general_regular_cursor_picker(),
                "picked",
            )
            .get_no_changes()
            .build();

        // drawing cursor
        app_settings
            .bind(
                "drawing-cursor",
                &self
                    .sidebar()
                    .settings_panel()
                    .general_drawing_cursor_picker(),
                "picked",
            )
            .get_no_changes()
            .build();

        // show drawing cursor
        app_settings
            .bind(
                "show-drawing-cursor",
                &self
                    .sidebar()
                    .settings_panel()
                    .general_show_drawing_cursor_row(),
                "active",
            )
            .get_no_changes()
            .build();

        // colorpicker palette
        let gdk_color_mapping = |var: &glib::Variant, _: glib::Type| {
            let (red, green, blue, alpha) = var.get::<(f64, f64, f64, f64)>()?;
            Some(gdk::RGBA::new(red as f32, green as f32, blue as f32, alpha as f32).to_value())
        };
        let gdk_color_set_mapping = |val: &glib::Value, _: glib::VariantType| {
            let color = val.get::<gdk::RGBA>().ok()?;
            Some(
                (
                    color.red() as f64,
                    color.green() as f64,
                    color.blue() as f64,
                    color.alpha() as f64,
                )
                    .to_variant(),
            )
        };

        app_settings
            .bind(
                "active-stroke-color",
                &self.overlays().colorpicker(),
                "stroke-color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "active-fill-color",
                &self.overlays().colorpicker(),
                "fill-color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-1",
                &self.overlays().colorpicker().setter_1(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-2",
                &self.overlays().colorpicker().setter_2(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-3",
                &self.overlays().colorpicker().setter_3(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-4",
                &self.overlays().colorpicker().setter_4(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-5",
                &self.overlays().colorpicker().setter_5(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-6",
                &self.overlays().colorpicker().setter_6(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-7",
                &self.overlays().colorpicker().setter_7(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-8",
                &self.overlays().colorpicker().setter_8(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "colorpicker-color-9",
                &self.overlays().colorpicker().setter_9(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();

        // brush stroke widths
        app_settings
            .bind(
                "brush-width-1",
                &self
                    .overlays()
                    .penssidebar()
                    .brush_page()
                    .stroke_width_picker()
                    .setter_1(),
                "stroke-width",
            )
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "brush-width-2",
                &self
                    .overlays()
                    .penssidebar()
                    .brush_page()
                    .stroke_width_picker()
                    .setter_2(),
                "stroke-width",
            )
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "brush-width-3",
                &self
                    .overlays()
                    .penssidebar()
                    .brush_page()
                    .stroke_width_picker()
                    .setter_3(),
                "stroke-width",
            )
            .get_no_changes()
            .build();

        // shaper stroke widths
        app_settings
            .bind(
                "shaper-width-1",
                &self
                    .overlays()
                    .penssidebar()
                    .shaper_page()
                    .stroke_width_picker()
                    .setter_1(),
                "stroke-width",
            )
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "shaper-width-2",
                &self
                    .overlays()
                    .penssidebar()
                    .shaper_page()
                    .stroke_width_picker()
                    .setter_2(),
                "stroke-width",
            )
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "shaper-width-3",
                &self
                    .overlays()
                    .penssidebar()
                    .shaper_page()
                    .stroke_width_picker()
                    .setter_3(),
                "stroke-width",
            )
            .get_no_changes()
            .build();

        // eraser widths
        app_settings
            .bind(
                "eraser-width-1",
                &self
                    .overlays()
                    .penssidebar()
                    .eraser_page()
                    .stroke_width_picker()
                    .setter_1(),
                "stroke-width",
            )
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "eraser-width-2",
                &self
                    .overlays()
                    .penssidebar()
                    .eraser_page()
                    .stroke_width_picker()
                    .setter_2(),
                "stroke-width",
            )
            .get_no_changes()
            .build();
        app_settings
            .bind(
                "eraser-width-3",
                &self
                    .overlays()
                    .penssidebar()
                    .eraser_page()
                    .stroke_width_picker()
                    .setter_3(),
                "stroke-width",
            )
            .get_no_changes()
            .build();

        Ok(())
    }

    /// Load settings that are not bound as binds.
    ///
    /// Settings changes through gsettings / dconf might not be applied until the app restarts.
    pub(crate) fn load_settings(&self) -> anyhow::Result<()> {
        let app = self.app();
        let app_settings = app
            .app_settings()
            .ok_or_else(|| anyhow::anyhow!("Settings schema not found."))?;

        // appwindow
        {
            let window_width = app_settings.int("window-width");
            let window_height = app_settings.int("window-height");
            let is_maximized = app_settings.boolean("is-maximized");

            if is_maximized {
                self.maximize();
            } else {
                self.set_default_size(window_width, window_height);
            }

            // set the color-scheme through the action
            let color_scheme = app_settings.string("color-scheme");
            self.app()
                .activate_action("color-scheme", Some(&color_scheme.to_variant()));
        }

        {
            // Workspaces bar
            self.sidebar()
                .workspacebrowser()
                .workspacesbar()
                .load_from_settings(&app_settings);
        }

        Ok(())
    }

    /// Save settings that are not bound as binds.
    pub(crate) fn save_to_settings(&self) -> anyhow::Result<()> {
        let app = self.app();
        let app_settings = app
            .app_settings()
            .ok_or_else(|| anyhow::anyhow!("Settings schema not found."))?;

        {
            // Appwindow
            app_settings.set_boolean("is-maximized", self.is_maximized())?;
            if !self.is_maximized() {
                app_settings.set_int("window-width", self.width())?;
                app_settings.set_int("window-height", self.height())?;
            }
        }

        {
            // Save engine config of the current active tab
            if let Some(canvas) = self.active_tab_canvas() {
                canvas.save_engine_config(&app_settings)?;
            }
        }

        {
            // Workspaces list
            self.sidebar()
                .workspacebrowser()
                .workspacesbar()
                .save_to_settings(&app_settings);
        }

        Ok(())
    }

    pub(crate) fn setup_periodic_save(&self) -> anyhow::Result<()> {
        let app = self.app();
        let app_settings = app
            .app_settings()
            .ok_or_else(|| anyhow::anyhow!("Settings schema not found."))?;

        if let Some(removed_id) = self
            .imp()
            .periodic_configsave_source_id
            .borrow_mut()
            .replace(glib::source::timeout_add_seconds_local(
                Self::PERIODIC_CONFIGSAVE_INTERVAL,
                clone!(
                    #[weak]
                    app_settings,
                    #[weak(rename_to=appwindow)]
                    self,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return glib::ControlFlow::Continue;
                        };
                        if let Err(e) = canvas.save_engine_config(&app_settings) {
                            error!(
                                "Saving engine config in periodic save task failed , Err: {e:?}"
                            );
                        }

                        glib::ControlFlow::Continue
                    }
                ),
            ))
        {
            removed_id.remove();
        }

        Ok(())
    }
}
