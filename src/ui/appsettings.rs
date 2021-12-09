use std::path;

use crate::ui::appwindow::RnoteAppWindow;
use crate::{app::RnoteApp, render, sheet::background::PatternStyle, utils};

use gtk4::{gio, glib, glib::clone, prelude::*};
use tuple_conv::RepeatedTuple;

pub fn save_state_to_settings(appwindow: &RnoteAppWindow) -> Result<(), glib::BoolError> {
    // Marker
    appwindow.app_settings().set_double(
        "marker-width",
        appwindow.canvas().pens().borrow().marker.width(),
    )?;

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

    // Brush
    appwindow.app_settings().set_double(
        "brush-width",
        appwindow.canvas().pens().borrow().brush.width(),
    )?;

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

    // Shaper
    appwindow.app_settings().set_double(
        "shaper-width",
        appwindow.canvas().pens().borrow().shaper.width(),
    )?;

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

    // Eraser
    appwindow.app_settings().set_double(
        "eraser-width",
        appwindow.canvas().pens().borrow().eraser.width(),
    )?;

    // Format Size
    appwindow.app_settings().set_value(
        "format-size",
        &(
            appwindow.canvas().sheet().format().width() as u32,
            appwindow.canvas().sheet().format().height() as u32,
        )
            .to_variant(),
    )?;

    // Format DPI
    appwindow
        .app_settings()
        .set_double("format-dpi", appwindow.canvas().sheet().format().dpi())?;

    // Background Color
    appwindow
        .app_settings()
        .set_uint(
            "background-color",
            utils::Color::from(
                appwindow
                    .canvas()
                    .sheet()
                    .background()
                    .borrow()
                    .color()
                    .to_gdk(),
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
                PatternStyle::Dots => "dots",
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
    let app = appwindow
        .application()
        .unwrap()
        .downcast::<RnoteApp>()
        .unwrap();

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

    // renderer backend
    match appwindow.app_settings().string("renderer-backend").as_str() {
        "librsvg" => {
            appwindow
                .canvas()
                .sheet()
                .strokes_state()
                .borrow_mut()
                .renderer
                .backend = render::RendererBackend::Librsvg;
        }
        "resvg" => {
            appwindow
                .canvas()
                .sheet()
                .strokes_state()
                .borrow_mut()
                .renderer
                .backend = render::RendererBackend::Resvg;
        }
        _ => {
            log::error!("failed to load setting renderer-backend, unsupported string as key")
        }
    }

    // Marker
    let marker_width = appwindow.app_settings().double("marker-width");
    appwindow
        .penssidebar()
        .marker_page()
        .width_adj()
        .set_value(marker_width);
    appwindow
        .canvas()
        .pens()
        .borrow_mut()
        .marker
        .set_width(marker_width);

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

    // Brush
    let brush_width = appwindow.app_settings().double("brush-width");
    appwindow
        .penssidebar()
        .brush_page()
        .width_adj()
        .set_value(brush_width);
    appwindow
        .canvas()
        .pens()
        .borrow_mut()
        .brush
        .set_width(brush_width);

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

    // Shaper
    let shaper_width = appwindow.app_settings().double("shaper-width");
    appwindow
        .penssidebar()
        .shaper_page()
        .width_adj()
        .set_value(shaper_width);
    appwindow
        .canvas()
        .pens()
        .borrow_mut()
        .shaper
        .set_width(shaper_width);

    let shaper_colors = appwindow
        .app_settings()
        .value("shaper-stroke-colors")
        .get::<(u32, u32)>()
        .unwrap();
    let shaper_colors_vec: Vec<utils::Color> = shaper_colors
        .to_vec()
        .iter()
        .map(|color_value| utils::Color::from(*color_value))
        .collect();
    appwindow
        .penssidebar()
        .shaper_page()
        .stroke_colorpicker()
        .load_all_colors(&shaper_colors_vec);

    let shaper_fill = appwindow
        .app_settings()
        .value("shaper-fill-colors")
        .get::<(u32, u32)>()
        .unwrap();
    let shaper_fill_vec: Vec<utils::Color> = shaper_fill
        .to_vec()
        .iter()
        .map(|color_value| utils::Color::from(*color_value))
        .collect();
    appwindow
        .penssidebar()
        .shaper_page()
        .fill_colorpicker()
        .load_all_colors(&shaper_fill_vec);

    // Eraser
    let eraser_width = appwindow.app_settings().double("eraser-width");
    appwindow
        .penssidebar()
        .eraser_page()
        .width_adj()
        .set_value(eraser_width);
    appwindow
        .canvas()
        .pens()
        .borrow_mut()
        .eraser
        .set_width(eraser_width);

    // Format Size
    let format_size = appwindow
        .app_settings()
        .value("format-size")
        .get::<(u32, u32)>()
        .unwrap();
    appwindow
        .canvas()
        .sheet()
        .format()
        .set_width(format_size.0 as i32);
    appwindow
        .canvas()
        .sheet()
        .format()
        .set_height(format_size.1 as i32);

    // Format DPI
    appwindow
        .canvas()
        .sheet()
        .format()
        .set_dpi(appwindow.app_settings().double("format-dpi"));

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
        "dots" => appwindow
            .canvas()
            .sheet()
            .background()
            .borrow_mut()
            .set_pattern(PatternStyle::Dots),
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

    // Touch drawing
    appwindow
        .app_settings()
        .bind("touch-drawing", &appwindow.canvas(), "touch-drawing")
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();
    appwindow.app_settings().connect_changed(
        Some("touch-drawing"),
        clone!(@weak appwindow => move |appsettings, _key_str| {
                    let touch_drawing = appsettings.boolean("touch-drawing");

                    appwindow.canvas_scroller().set_overlay_scrolling(!touch_drawing);

                    // Changing PolicyType to Automatic is not behaving as expected, not sure why
        /*             if touch_drawing {
                        appwindow.canvas_scroller().set_policy(PolicyType::Always, PolicyType::Always);
                    } else {
                        appwindow.canvas_scroller().set_policy(PolicyType::Automatic, PolicyType::Automatic);
                    } */
                }),
    );

    // Format borders
    appwindow
        .canvas()
        .sheet()
        .set_format_borders(appwindow.app_settings().boolean("format-borders"));

    // endless sheet
    let endless_sheet = appwindow.app_settings().boolean("endless-sheet");
    appwindow.canvas().sheet().set_endless_sheet(endless_sheet);
    appwindow
        .mainheader()
        .pageedit_revealer()
        .set_reveal_child(!endless_sheet);

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

    let action_devel_settings = app.lookup_action("devel-settings").unwrap();
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
