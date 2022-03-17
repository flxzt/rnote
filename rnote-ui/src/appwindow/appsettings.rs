use std::path::PathBuf;

use crate::app::RnoteApp;
use crate::appwindow::RnoteAppWindow;
use crate::canvas::ExpandMode;
use rnote_engine::compose::color::Color;
use rnote_engine::pens::Pens;
use rnote_engine::sheet::background::Background;
use rnote_engine::sheet::format::Format;

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

        // righthanded
        self.app_settings()
            .bind("righthanded", self, "righthanded")
            .build();

        // pen sounds
        self.app_settings()
            .bind("pen-sounds", self, "pen-sounds")
            .build();

        // touch drawing
        self.app_settings()
            .bind("touch-drawing", &self.canvas(), "touch-drawing")
            .build();

        // expand mode
        self.app_settings()
            .bind("expand-mode", &self.canvas(), "expand-mode")
            .mapping(move |settings_value, _type_| {
                let value = settings_value.get::<String>().unwrap();
                match value.as_str() {
                    "fixed-size" => Some(ExpandMode::FixedSize.to_value()),
                    "endless-vertical" => Some(ExpandMode::EndlessVertical.to_value()),
                    "infinite" => Some(ExpandMode::Infinite.to_value()),
                    _ => {
                        log::error!(
                            "mapping expand-mode to setting failed, invalid str {}",
                            value.as_str()
                        );
                        None
                    }
                }
            })
            .set_mapping(
                move |value, _type_| match value.get::<ExpandMode>().unwrap() {
                    ExpandMode::FixedSize => Some(String::from("fixed-size").to_variant()),
                    ExpandMode::EndlessVertical => {
                        Some(String::from("endless-vertical").to_variant())
                    }
                    ExpandMode::Infinite => Some(String::from("infinite").to_variant()),
                },
            )
            .build();

        // format borders
        self.app_settings()
            .bind("format-borders", &self.canvas(), "format-borders")
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

        // lock resize aspectratio
        self.app_settings()
            .bind(
                "resize-lock-aspectratio",
                &self.canvas().selection_modifier(),
                "resize-lock-aspectratio",
            )
            .build();
    }

    /// load settings that are not bound in setup_settings. Setting changes through gsettings / dconf might not be applied until app restarts
    pub fn load_settings(&self) -> Result<(), anyhow::Error> {
        let _app = self.application().unwrap().downcast::<RnoteApp>().unwrap();

        // appwindow
        self.load_window_size();

        // colorscheme
        // Set the buttons, as the style manager colorscheme property may not be changed from the binding
        match self.app_settings().string("color-scheme").as_str() {
            "default" => self.mainheader().appmenu().default_theme_toggle().set_active(true),
            "force-light" => self.mainheader().appmenu().light_theme_toggle().set_active(true),
            "force-dark" => self.mainheader().appmenu().dark_theme_toggle().set_active(true),
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
            // Load format
            if let Ok(loaded_format) =
                serde_json::from_str::<Format>(self.app_settings().string("sheet-format").as_str())
            {
                self.canvas().sheet().borrow_mut().format = loaded_format;
            }
        }

        {
            // Load background
            if let Ok(loaded_background) = serde_json::from_str::<Background>(
                self.app_settings().string("sheet-background").as_str(),
            ) {
                self.canvas().sheet().borrow_mut().background = loaded_background;
            }
        }

        {
            // Load pens
            if let Ok(loaded_pens) =
                serde_json::from_str::<Pens>(self.app_settings().string("pens").as_str())
            {
                *self.canvas().pens().borrow_mut() = loaded_pens;
            }
        }

        // refresh the UI
        adw::prelude::ActionGroupExt::activate_action(self, "refresh-ui-for-sheet", None);
        Ok(())
    }

    /// Save all state that is not bound in setup_settings
    pub fn save_to_settings(&self) -> Result<(), anyhow::Error> {
        self.save_window_size()?;

        {
            // Brush page
            let colors = self
                .penssidebar()
                .brush_page()
                .colorpicker()
                .fetch_all_colors()
                .into_iter()
                .map(|color| color.to_u32())
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
                .map(|color| color.to_u32())
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
                .map(|color| color.to_u32())
                .collect::<Vec<u32>>();
            let fills = (fills[0], fills[1]);
            self.app_settings()
                .set_value("shaperpage-fills", &fills.to_variant())?;

            // Save format
            let format_string = serde_json::to_string(&self.canvas().sheet().borrow().format)?;
            self.app_settings()
                .set_string("sheet-format", format_string.as_str())?;

            //println!("format:\n{}", format_string);

            // Save background
            let background_string =
                serde_json::to_string(&self.canvas().sheet().borrow().background)?;
            self.app_settings()
                .set_string("sheet-background", background_string.as_str())?;

            //println!("background:\n{}", background_string);

            // Save pens
            let pens_string = serde_json::to_string(&*self.canvas().pens().borrow())?;
            self.app_settings()
                .set_string("pens", pens_string.as_str())?;

            //println!("pens:\n{}", pens_string);
        }

        Ok(())
    }
}
