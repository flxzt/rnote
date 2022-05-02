use std::path::PathBuf;

use crate::app::RnoteApp;
use crate::appwindow::RnoteAppWindow;
use rnote_compose::Color;

use adw::prelude::*;
use gtk4::gio;

impl RnoteAppWindow {
    /// Settings binds
    pub fn setup_settings(&self) {
        let app = self.application().unwrap().downcast::<RnoteApp>().unwrap();

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

        // Workspace directory
        self.app_settings()
            .bind(
                "workspace-dir",
                &self.workspacebrowser().primary_dirlist(),
                "file",
            )
            .mapping(|variant, _| {
                let path = PathBuf::from(variant.get::<String>().unwrap());
                Some(gio::File::for_path(&path).to_value())
            })
            .set_mapping(|value, _| {
                let file = value.get::<gio::File>().unwrap();

                file.path().map(|path| path.to_string_lossy().to_variant())
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
            .bind("touch-drawing", &self.canvas(), "touch-drawing")
            .build();

        // pdf import width
        self.app_settings()
            .bind("pdf-import-width", &self.canvas(), "pdf-import-width")
            .build();

        // pdf import as vector image
        self.app_settings()
            .bind(
                "pdf-import-as-vector",
                &self.canvas(),
                "pdf-import-as-vector",
            )
            .build();

        // Brush page
        self.app_settings()
            .bind(
                "brushpage-selected-color",
                &self.penssidebar().brush_page().colorpicker(),
                "selected",
            )
            .build();

        // Shaper page
        self.app_settings()
            .bind(
                "shaperpage-selected-color",
                &self.penssidebar().shaper_page().stroke_colorpicker(),
                "selected",
            )
            .build();
        self.app_settings()
            .bind(
                "shaperpage-selected-fill",
                &self.penssidebar().shaper_page().fill_colorpicker(),
                "selected",
            )
            .build();
    }

    /// load settings at start that are not bound in setup_settings. Setting changes through gsettings / dconf might not be applied until app restarts
    pub fn load_settings(&self) {
        let _app = self.application().unwrap().downcast::<RnoteApp>().unwrap();

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

        // colorscheme
        // Set the buttons, as the style manager colorscheme property may not be changed from the binding
        match self.app_settings().string("color-scheme").as_str() {
            "default" => self
                .mainheader()
                .appmenu()
                .default_theme_toggle()
                .set_active(true),
            "force-light" => self
                .mainheader()
                .appmenu()
                .light_theme_toggle()
                .set_active(true),
            "force-dark" => self
                .mainheader()
                .appmenu()
                .dark_theme_toggle()
                .set_active(true),
            _ => {}
        }

        {
            // Brush page
            let colors = self
                .app_settings()
                .get::<(u32, u32, u32, u32, u32, u32, u32, u32)>("brushpage-colors");
            let colors = [
                colors.0, colors.1, colors.2, colors.3, colors.4, colors.5, colors.6, colors.7,
            ]
            .into_iter()
            .map(|color| Color::from(color))
            .collect::<Vec<Color>>();
            self.penssidebar()
                .brush_page()
                .colorpicker()
                .load_colors(&colors);
        }

        {
            // Shaper page
            let colors = self.app_settings().get::<(u32, u32)>("shaperpage-colors");
            let colors = [colors.0, colors.1]
                .into_iter()
                .map(|color| Color::from(color))
                .collect::<Vec<Color>>();
            self.penssidebar()
                .shaper_page()
                .stroke_colorpicker()
                .load_colors(&colors);

            // Shaper page fills

            let fill_colors = self.app_settings().get::<(u32, u32)>("shaperpage-fills");
            let fill_colors = [fill_colors.0, fill_colors.1]
                .into_iter()
                .map(|color| Color::from(color))
                .collect::<Vec<Color>>();
            self.penssidebar()
                .shaper_page()
                .fill_colorpicker()
                .load_colors(&fill_colors);
        }

        {
            // load engine config
            let engine_config = self.app_settings().string("engine-config");
            match self
                .canvas()
                .engine()
                .borrow_mut()
                .load_engine_config(&engine_config)
            {
                Err(e) => {
                    // On first app startup the engine config is empty, so we don't log an error
                    if engine_config.is_empty() {
                        log::debug!("did not load `engine-config` from settings, was empty");
                    } else {
                        log::error!("failed to load `engine-config` from settings, Err {}", e);
                    }
                }
                Ok(()) => {}
            }
        }

        // refresh the UI
        adw::prelude::ActionGroupExt::activate_action(self, "refresh-ui-for-engine", None);
    }

    /// Save all settings at shutdown that are not bound in setup_settings
    pub fn save_to_settings(&self) -> anyhow::Result<()> {
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
            // Brush page
            let colors = self
                .penssidebar()
                .brush_page()
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
                .set_value("brushpage-colors", &colors.to_variant())?;
        }

        {
            // Shaper page colors
            let colors = self
                .penssidebar()
                .shaper_page()
                .stroke_colorpicker()
                .fetch_all_colors()
                .into_iter()
                .map(|color| color.into())
                .collect::<Vec<u32>>();
            let colors = (colors[0], colors[1]);
            self.app_settings()
                .set_value("shaperpage-colors", &colors.to_variant())?;

            // Shaper page fills
            let fills = self
                .penssidebar()
                .shaper_page()
                .fill_colorpicker()
                .fetch_all_colors()
                .into_iter()
                .map(|color| color.into())
                .collect::<Vec<u32>>();
            let fills = (fills[0], fills[1]);
            self.app_settings()
                .set_value("shaperpage-fills", &fills.to_variant())?;
        }

        {
            // Save engine config
            let engine_config = self.canvas().engine().borrow().save_engine_config()?;
            self.app_settings()
                .set_string("engine-config", engine_config.as_str())?;
        }

        Ok(())
    }
}
