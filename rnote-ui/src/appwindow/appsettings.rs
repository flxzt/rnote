use std::path::PathBuf;

use crate::appwindow::RnoteAppWindow;
use crate::config;
use gtk4::gdk;
use rnote_compose::Color;

use adw::prelude::*;
use rnote_engine::utils::GdkRGBAHelpers;

impl RnoteAppWindow {
    /// Settings binds
    pub(crate) fn setup_settings_binds(&self) {
        let app = self.app();

        // Color scheme
        self.app_settings()
            .bind("color-scheme", &app.style_manager(), "color-scheme")
            .mapping(|variant, _| {
                let value = variant.get::<String>().unwrap();

                match value.as_str() {
                    "default" => Some(adw::ColorScheme::Default.to_value()),
                    "force-light" => Some(adw::ColorScheme::ForceLight.to_value()),
                    "force-dark" => Some(adw::ColorScheme::ForceDark.to_value()),
                    _ => {
                        log::error!(
                            "mapping color-scheme to setting failed, invalid str {}",
                            value.as_str()
                        );
                        None
                    }
                }
            })
            .set_mapping(|value, _| match value.get::<adw::ColorScheme>().unwrap() {
                adw::ColorScheme::Default => Some(String::from("default").to_variant()),
                adw::ColorScheme::ForceLight => Some(String::from("force-light").to_variant()),
                adw::ColorScheme::ForceDark => Some(String::from("force-dark").to_variant()),
                _ => None,
            })
            .build();

        // autosave
        self.app_settings()
            .bind("autosave", self, "autosave")
            .build();

        // autosave interval secs
        self.app_settings()
            .bind("autosave-interval-secs", self, "autosave-interval-secs")
            .build();

        // righthanded
        self.app_settings()
            .bind("righthanded", self, "righthanded")
            .build();

        // touch drawing
        self.app_settings()
            .bind("touch-drawing", self, "touch-drawing")
            .build();

        // permanently hide canvas scrollbars
        self.app_settings()
            .bind(
                "show-scrollbars",
                &self.settings_panel().general_show_scrollbars_switch(),
                "active",
            )
            .build();

        // regular cursor
        self.app_settings()
            .bind(
                "regular-cursor",
                &self.settings_panel().general_regular_cursor_picker(),
                "picked",
            )
            .build();

        // drawing cursor
        self.app_settings()
            .bind(
                "drawing-cursor",
                &self.settings_panel().general_drawing_cursor_picker(),
                "picked",
            )
            .build();

        // Active stroke color
        self.app_settings()
            .bind(
                "active-stroke-color",
                &self.overlays().colorpicker(),
                "stroke-color",
            )
            .mapping(|val, _| {
                Some(
                    gdk::RGBA::from_compose_color(Color::from(val.get::<u32>().unwrap()))
                        .to_value(),
                )
            })
            .set_mapping(|val, _| {
                Some(u32::from(val.get::<gdk::RGBA>().unwrap().into_compose_color()).to_variant())
            })
            .build();

        // Active fill color
        self.app_settings()
            .bind(
                "active-fill-color",
                &self.overlays().colorpicker(),
                "fill-color",
            )
            .mapping(|val, _| {
                Some(
                    gdk::RGBA::from_compose_color(Color::from(val.get::<u32>().unwrap()))
                        .to_value(),
                )
            })
            .set_mapping(|val, _| {
                Some(u32::from(val.get::<gdk::RGBA>().unwrap().into_compose_color()).to_variant())
            })
            .build();

        // brush stroke widths
        self.app_settings()
            .bind(
                "brush-width-1",
                &self
                    .penssidebar()
                    .brush_page()
                    .stroke_width_picker()
                    .setter_1(),
                "stroke-width",
            )
            .build();
        self.app_settings()
            .bind(
                "brush-width-2",
                &self
                    .penssidebar()
                    .brush_page()
                    .stroke_width_picker()
                    .setter_2(),
                "stroke-width",
            )
            .build();
        self.app_settings()
            .bind(
                "brush-width-3",
                &self
                    .penssidebar()
                    .brush_page()
                    .stroke_width_picker()
                    .setter_3(),
                "stroke-width",
            )
            .build();

        // shaper stroke widths
        self.app_settings()
            .bind(
                "shaper-width-1",
                &self
                    .penssidebar()
                    .shaper_page()
                    .stroke_width_picker()
                    .setter_1(),
                "stroke-width",
            )
            .build();
        self.app_settings()
            .bind(
                "shaper-width-2",
                &self
                    .penssidebar()
                    .shaper_page()
                    .stroke_width_picker()
                    .setter_2(),
                "stroke-width",
            )
            .build();
        self.app_settings()
            .bind(
                "shaper-width-3",
                &self
                    .penssidebar()
                    .shaper_page()
                    .stroke_width_picker()
                    .setter_3(),
                "stroke-width",
            )
            .build();
    }

    /// load settings at start that are not bound in setup_settings. Setting changes through gsettings / dconf might not be applied until app restarts
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

            self.flap_box()
                .set_width_request(self.app_settings().int("flap-width"));
        }

        // color scheme
        // Set the action menu, as the style manager colorscheme property may not be changed from the binding at startup when opening a second window (FIXME: why?)
        let color_scheme = self.app_settings().string("color-scheme");
        self.app()
            .activate_action("color-scheme", Some(&color_scheme.to_variant()));

        {
            // Workspaces bar
            self.workspacebrowser()
                .workspacesbar()
                .load_from_settings(&self.app_settings());
        }

        {
            // Colorpicker
            let colors = self
                .app_settings()
                .get::<(u32, u32, u32, u32, u32, u32, u32, u32)>("colorpicker-palette");
            let colors = [
                colors.0, colors.1, colors.2, colors.3, colors.4, colors.5, colors.6, colors.7,
            ]
            .into_iter()
            .map(Color::from)
            .collect::<Vec<Color>>();
            self.overlays().colorpicker().load_colors(&colors);
        }

        {
            let canvas = self.active_tab().canvas();
            // load engine config
            let engine_config = self.app_settings().string("engine-config");
            let widget_flags = match canvas
                .engine()
                .borrow_mut()
                .load_engine_config(&engine_config, Some(PathBuf::from(config::PKGDATADIR)))
            {
                Err(e) => {
                    // On first app startup the engine config is empty, so we don't log an error
                    if engine_config.is_empty() {
                        log::debug!("did not load `engine-config` from settings, was empty");
                    } else {
                        log::error!("failed to load `engine-config` from settings, Err: {e:?}");
                    }
                    None
                }
                Ok(widget_flags) => Some(widget_flags),
            };

            // Avoiding already borrowed
            if let Some(widget_flags) = widget_flags {
                self.handle_widget_flags(widget_flags, &canvas);
            }
        }
    }

    /// Save all settings at shutdown that are not bound in setup_settings
    pub(crate) fn save_to_settings(&self) -> anyhow::Result<()> {
        {
            // Appwindow
            self.app_settings().set_int("window-width", self.width())?;
            self.app_settings()
                .set_int("window-height", self.height())?;
            self.app_settings()
                .set_boolean("is-maximized", self.is_maximized())?;

            self.app_settings()
                .set_int("flap-width", self.flap_box().width())?;
        }

        {
            // Colorpicker
            let colors = self
                .overlays()
                .colorpicker()
                .fetch_all_colors()
                .into_iter()
                .map(|color| color.into())
                .collect::<Vec<u32>>();
            let colors = (
                colors[0], colors[1], colors[2], colors[3], colors[4], colors[5], colors[6],
                colors[7],
            );
            self.app_settings()
                .set_value("colorpicker-palette", &colors.to_variant())?;
        }

        {
            // Save engine config
            self.save_engine_config_active_tab()?;
        }

        Ok(())
    }
}
