use std::path;

use crate::ui::appwindow::RnoteAppWindow;
use crate::{app::RnoteApp, sheet::background::PatternStyle, utils};

use gtk4::{gio, glib, prelude::*};
use tuple_conv::RepeatedTuple;

pub fn save_state_to_settings(appwindow: &RnoteAppWindow) -> Result<(), glib::BoolError> {
    // Marker Colors
    let marker_colors: Vec<u32> = appwindow
        .penssidebar()
        .marker_page()
        .colorpicker()
        .fetch_all_colors()
        .iter()
        .map(|color| {
            let value = color.to_u32();
            value
        })
        .collect();
    if marker_colors.len() != 8 {
        log::error!(
            "Couldn't save marker colors. Vector length does not match settings tuple length"
        )
    } else {
        appwindow.app_settings().set_value(
            "marker-colors",
            &(
                marker_colors[0],
                marker_colors[1],
                marker_colors[2],
                marker_colors[3],
                marker_colors[4],
                marker_colors[5],
                marker_colors[6],
                marker_colors[7],
            )
                .to_variant(),
        )?;
    }

    // Brush Colors
    let brush_colors: Vec<u32> = appwindow
        .penssidebar()
        .brush_page()
        .colorpicker()
        .fetch_all_colors()
        .iter()
        .map(|color| {
            let value = color.to_u32();
            value
        })
        .collect();
    if brush_colors.len() != 8 {
        log::error!(
            "Couldn't save brush colors. Vector length does not match settings tuple length"
        )
    } else {
        appwindow.app_settings().set_value(
            "brush-colors",
            &(
                brush_colors[0],
                brush_colors[1],
                brush_colors[2],
                brush_colors[3],
                brush_colors[4],
                brush_colors[5],
                brush_colors[6],
                brush_colors[7],
            )
                .to_variant(),
        )?;
    }

    // Shaper stroke colors
    let shaper_stroke_colors: Vec<u32> = appwindow
        .penssidebar()
        .shaper_page()
        .stroke_colorpicker()
        .fetch_all_colors()
        .iter()
        .map(|color| {
            let value = color.to_u32();
            value
        })
        .collect();
    if shaper_stroke_colors.len() != 2 {
        log::error!(
                "Couldn't save shaper stroke colors. Vector length does not match settings tuple length"
            )
    } else {
        appwindow.app_settings().set_value(
            "shaper-stroke-colors",
            &(shaper_stroke_colors[0], shaper_stroke_colors[1]).to_variant(),
        )?;
    }

    // Shaper fill colors
    let shaper_fill_colors: Vec<u32> = appwindow
        .penssidebar()
        .shaper_page()
        .fill_colorpicker()
        .fetch_all_colors()
        .iter()
        .map(|color| {
            let value = color.to_u32();
            value
        })
        .collect();
    if shaper_fill_colors.len() != 2 {
        log::error!(
            "Couldn't save shaper fill colors. Vector length does not match settings tuple length"
        )
    } else {
        appwindow.app_settings().set_value(
            "shaper-fill-colors",
            &(shaper_fill_colors[0], shaper_fill_colors[1]).to_variant(),
        )?;
    }

    // Background Color
    appwindow
        .app_settings()
        .set_uint(
            "background-color",
            utils::Color::from(
                appwindow
                    .settings_panel()
                    .background_color_choosebutton()
                    .rgba(),
            )
            .to_u32(),
        )
        .unwrap();

    // Background pattern
    appwindow
        .app_settings()
        .set_string(
            "background-pattern",
            match appwindow.canvas().sheet().background().borrow().pattern() {
                PatternStyle::None => "none",
                PatternStyle::Lines => "lines",
                PatternStyle::Grid => "grid",
            },
        )
        .unwrap();

    // Background Pattern Color
    appwindow
        .app_settings()
        .set_uint(
            "background-pattern-color",
            utils::Color::from(
                appwindow
                    .settings_panel()
                    .background_pattern_color_choosebutton()
                    .rgba(),
            )
            .to_u32(),
        )
        .unwrap();

    // Background pattern size
    let pattern_size = appwindow
        .canvas()
        .sheet()
        .background()
        .borrow()
        .pattern_size();
    appwindow.app_settings().set_value(
        "background-pattern-size",
        &(
            pattern_size[0].round() as u32,
            pattern_size[1].round() as u32,
        )
            .to_variant(),
    )?;

    Ok(())
}

// ### Settings are setup only at startup. Setting changes through gsettings / dconf might not be applied until app restarts
pub fn load_settings(appwindow: &RnoteAppWindow) {
    // overwriting theme so users can choose dark / light in appmenu
    //appwindow.settings().set_gtk_theme_name(Some("Adwaita"));

    // Workspace directory
    appwindow
        .workspacebrowser()
        .set_primary_path(&path::PathBuf::from(
            appwindow.app_settings().string("workspace-dir").as_str(),
        ));

    // color schemes
    match appwindow.app_settings().string("color-scheme").as_str() {
        "default" => appwindow.set_color_scheme(adw::ColorScheme::Default),
        "force-light" => appwindow.set_color_scheme(adw::ColorScheme::ForceLight),
        "force-dark" => appwindow.set_color_scheme(adw::ColorScheme::ForceDark),
        _ => {
            log::error!("failed to load setting color-scheme, unsupported string as key")
        }
    }

    // Marker colors
    let marker_colors = appwindow
        .app_settings()
        .value("marker-colors")
        .get::<(u32, u32, u32, u32, u32, u32, u32, u32)>()
        .unwrap();
    let marker_colors_vec: Vec<utils::Color> = marker_colors
        .to_vec()
        .iter()
        .map(|color_value| utils::Color::from(*color_value))
        .collect();
    appwindow
        .penssidebar()
        .marker_page()
        .colorpicker()
        .load_all_colors(&marker_colors_vec);

    // Brush colors
    let brush_colors = appwindow
        .app_settings()
        .value("brush-colors")
        .get::<(u32, u32, u32, u32, u32, u32, u32, u32)>()
        .unwrap();
    let brush_colors_vec: Vec<utils::Color> = brush_colors
        .to_vec()
        .iter()
        .map(|color_value| utils::Color::from(*color_value))
        .collect();
    appwindow
        .penssidebar()
        .brush_page()
        .colorpicker()
        .load_all_colors(&brush_colors_vec);

    // Shaper stroke colors
    let brush_colors = appwindow
        .app_settings()
        .value("shaper-stroke-colors")
        .get::<(u32, u32)>()
        .unwrap();
    let brush_colors_vec: Vec<utils::Color> = brush_colors
        .to_vec()
        .iter()
        .map(|color_value| utils::Color::from(*color_value))
        .collect();
    appwindow
        .penssidebar()
        .shaper_page()
        .stroke_colorpicker()
        .load_all_colors(&brush_colors_vec);

    // Shaper fill colors
    let brush_colors = appwindow
        .app_settings()
        .value("shaper-fill-colors")
        .get::<(u32, u32)>()
        .unwrap();
    let brush_colors_vec: Vec<utils::Color> = brush_colors
        .to_vec()
        .iter()
        .map(|color_value| utils::Color::from(*color_value))
        .collect();
    appwindow
        .penssidebar()
        .shaper_page()
        .fill_colorpicker()
        .load_all_colors(&brush_colors_vec);

    // Background color
    let background_color = utils::Color::from(appwindow.app_settings().uint("background-color"));
    appwindow
        .canvas()
        .sheet()
        .background()
        .borrow_mut()
        .set_color(background_color);

    // Background pattern
    match appwindow
        .app_settings()
        .string("background-pattern")
        .as_str()
    {
        "none" => {
            appwindow
                .canvas()
                .sheet()
                .background()
                .borrow_mut()
                .set_pattern(PatternStyle::None);
        }
        "lines" => appwindow
            .canvas()
            .sheet()
            .background()
            .borrow_mut()
            .set_pattern(PatternStyle::Lines),
        "grid" => appwindow
            .canvas()
            .sheet()
            .background()
            .borrow_mut()
            .set_pattern(PatternStyle::Grid),
        _ => {
            log::error!("failed to load setting color-scheme, unsupported string as key")
        }
    }

    // Background pattern color
    let background_pattern_color =
        utils::Color::from(appwindow.app_settings().uint("background-pattern-color"));
    appwindow
        .canvas()
        .sheet()
        .background()
        .borrow_mut()
        .set_pattern_color(background_pattern_color);

    // Background pattern size
    let background_pattern_size = appwindow
        .app_settings()
        .value("background-pattern-size")
        .get::<(u32, u32)>()
        .unwrap();
    appwindow
        .canvas()
        .sheet()
        .background()
        .borrow_mut()
        .set_pattern_size(na::vector![
            f64::from(background_pattern_size.0),
            f64::from(background_pattern_size.1)
        ]);

    // Ui for right / left handed writers
    appwindow.application().unwrap().change_action_state(
        "righthanded",
        &appwindow.app_settings().boolean("righthanded").to_variant(),
    );
    appwindow
        .application()
        .unwrap()
        .activate_action("righthanded", None);
    appwindow
        .application()
        .unwrap()
        .activate_action("righthanded", None);

    // Touch drawing
    appwindow
        .app_settings()
        .bind("touch-drawing", &appwindow.canvas(), "touch-drawing")
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();

    // Format borders
    appwindow
        .canvas()
        .set_format_borders(appwindow.app_settings().boolean("format-borders"));

    // Autoexpand height
    let autoexpand_height = appwindow.app_settings().boolean("autoexpand-height");
    appwindow
        .canvas()
        .sheet()
        .set_autoexpand_height(autoexpand_height);
    appwindow
        .mainheader()
        .pageedit_revealer()
        .set_reveal_child(!autoexpand_height);

    // Visual Debugging
    appwindow
        .app_settings()
        .bind("visual-debug", &appwindow.canvas(), "visual-debug")
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();

    // Developer mode
    appwindow
        .app_settings()
        .bind(
            "devel",
            &appwindow
                .penssidebar()
                .brush_page()
                .templatechooser()
                .predefined_template_experimental_listboxrow(),
            "visible",
        )
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();

    let action_devel_settings = appwindow
        .application()
        .unwrap()
        .downcast::<RnoteApp>()
        .unwrap()
        .lookup_action("devel-settings")
        .unwrap();
    action_devel_settings
        .downcast::<gio::SimpleAction>()
        .unwrap()
        .set_enabled(appwindow.app_settings().boolean("devel"));

    appwindow
        .devel_actions_revealer()
        .set_reveal_child(appwindow.app_settings().boolean("devel"));

    // Loading the sheet properties into the format settings panel
    appwindow
        .settings_panel()
        .load_format(appwindow.canvas().sheet());
    appwindow
        .settings_panel()
        .load_background(appwindow.canvas().sheet());
}
