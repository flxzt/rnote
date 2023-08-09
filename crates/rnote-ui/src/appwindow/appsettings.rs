// Imports
use crate::appwindow::RnAppWindow;
use adw::prelude::*;
use gtk4::{gdk, glib, glib::clone};

impl RnAppWindow {
    /// Setup settings binds.
    pub(crate) fn setup_settings_binds(&self) {
        let app = self.app();

        app.style_manager().connect_color_scheme_notify(
            clone!(@weak app, @weak self as appwindow => move |style_manager| {
                let color_scheme = match style_manager.color_scheme() {
                    adw::ColorScheme::Default => String::from("default"),
                    adw::ColorScheme::ForceLight => String::from("force-light"),
                    adw::ColorScheme::ForceDark => String::from("force-dark"),
                    _ => String::from("default"),
                };

                if let Err(e) = appwindow.app_settings()
                    .set_string("color-scheme", &color_scheme) {
                        log::error!("failed to set setting `color-scheme`, Err: {e:?}");
                    }
            }),
        );

        self.app_settings()
            .bind("sidebar-show", &self.split_view(), "show-sidebar")
            .get_no_changes()
            .build();

        // autosave
        self.app_settings()
            .bind("autosave", self, "autosave")
            .get_no_changes()
            .build();

        // autosave interval secs
        self.app_settings()
            .bind("autosave-interval-secs", self, "autosave-interval-secs")
            .get_no_changes()
            .build();

        // righthanded
        self.app_settings()
            .bind("righthanded", self, "righthanded")
            .get_no_changes()
            .build();

        // block pinch zoom
        self.app_settings()
            .bind("block-pinch-zoom", self, "block-pinch-zoom")
            .get_no_changes()
            .build();

        // touch drawing
        self.app_settings()
            .bind("touch-drawing", self, "touch-drawing")
            .get_no_changes()
            .build();

        // show scrollbars
        self.app_settings()
            .bind(
                "show-scrollbars",
                &self
                    .sidebar()
                    .settings_panel()
                    .general_show_scrollbars_switch(),
                "active",
            )
            .get_no_changes()
            .build();

        // inertial scrolling
        self.app_settings()
            .bind(
                "inertial-scrolling",
                &self
                    .sidebar()
                    .settings_panel()
                    .general_inertial_scrolling_switch(),
                "active",
            )
            .get_no_changes()
            .build();

        // regular cursor
        self.app_settings()
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
        self.app_settings()
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
        self.app_settings()
            .bind(
                "show-drawing-cursor",
                &self
                    .sidebar()
                    .settings_panel()
                    .general_show_drawing_cursor_switch(),
                "active",
            )
            .get_no_changes()
            .build();

        // colorpicker palette
        let gdk_color_mapping = |var: &glib::Variant, _: glib::Type| {
            let color = var.get::<(f64, f64, f64, f64)>()?;
            Some(
                gdk::RGBA::new(
                    color.0 as f32,
                    color.1 as f32,
                    color.2 as f32,
                    color.3 as f32,
                )
                .to_value(),
            )
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

        self.app_settings()
            .bind(
                "active-stroke-color",
                &self.overlays().colorpicker(),
                "stroke-color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "active-fill-color",
                &self.overlays().colorpicker(),
                "fill-color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "colorpicker-color-1",
                &self.overlays().colorpicker().setter_1(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "colorpicker-color-2",
                &self.overlays().colorpicker().setter_2(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "colorpicker-color-3",
                &self.overlays().colorpicker().setter_3(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "colorpicker-color-4",
                &self.overlays().colorpicker().setter_4(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "colorpicker-color-5",
                &self.overlays().colorpicker().setter_5(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "colorpicker-color-6",
                &self.overlays().colorpicker().setter_6(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "colorpicker-color-7",
                &self.overlays().colorpicker().setter_7(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();
        self.app_settings()
            .bind(
                "colorpicker-color-8",
                &self.overlays().colorpicker().setter_8(),
                "color",
            )
            .mapping(gdk_color_mapping)
            .set_mapping(gdk_color_set_mapping)
            .get_no_changes()
            .build();

        // brush stroke widths
        self.app_settings()
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
        self.app_settings()
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
        self.app_settings()
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
        self.app_settings()
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
        self.app_settings()
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
        self.app_settings()
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
        self.app_settings()
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
        self.app_settings()
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
        self.app_settings()
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
    }

    /// Load settings that are not bound as binds.
    ///
    /// Settings changes through gsettings / dconf might not be applied until the app restarts.
    pub(crate) fn load_settings(&self) {
        let _app = self.app();

        // appwindow
        {
            let window_width = self.app_settings().int("window-width");
            let window_height = self.app_settings().int("window-height");
            let is_maximized = self.app_settings().boolean("is-maximized");

            self.set_default_size(window_width, window_height);
            if is_maximized {
                self.maximize();
            }

            // set the color-scheme through the action
            let color_scheme = self.app_settings().string("color-scheme");
            self.app()
                .activate_action("color-scheme", Some(&color_scheme.to_variant()));
        }

        {
            // Workspaces bar
            self.sidebar()
                .workspacebrowser()
                .workspacesbar()
                .load_from_settings(&self.app_settings());
        }
    }

    /// Save settings that are not bound as binds.
    pub(crate) fn save_to_settings(&self) -> anyhow::Result<()> {
        let _app = self.app();

        {
            // Appwindow
            self.app_settings().set_int("window-width", self.width())?;
            self.app_settings()
                .set_int("window-height", self.height())?;
            self.app_settings()
                .set_boolean("is-maximized", self.is_maximized())?;
        }

        {
            // Save engine config of the last active tab
            self.active_tab_wrapper()
                .canvas()
                .save_engine_config(&self.app_settings())?;
        }

        Ok(())
    }
}
