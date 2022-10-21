use crate::app::RnoteApp;
use adw::prelude::*;
use gtk4::{gio, glib, glib::clone};

impl RnoteApp {
    pub fn setup_actions(&self) {
        let action_quit = gio::SimpleAction::new("quit", None);
        self.add_action(&action_quit);

        let action_color_scheme = gio::SimpleAction::new_stateful(
            "color-scheme",
            Some(&glib::VariantType::new("s").unwrap()),
            &String::from("system").to_variant(),
        );
        self.add_action(&action_color_scheme);
        /*
        action_color_scheme.connect_activate(clone!(@weak self as app => move |action_color_scheme, target| {
                let color_scheme = target.unwrap().str().unwrap();

                match color_scheme {
                    "system" => app.style_manager().set_color_scheme(adw::ColorScheme::Default),
                    "light" => app.style_manager().set_color_scheme(adw::ColorScheme::ForceLight),
                    "dark" => app.style_manager().set_color_scheme(adw::ColorScheme::ForceDark),
                    _ => {}
                }

                action_color_scheme.set_state(&color_scheme.to_variant());
        })); */

        action_color_scheme
            .bind_property("state", &self.style_manager(), "color-scheme")
            .transform_to(move |_, val| {
                match val
                    .get::<glib::Variant>()
                    .unwrap()
                    .get::<String>()
                    .unwrap()
                    .as_str()
                {
                    "default" => Some(adw::ColorScheme::Default.to_value()),
                    "force-light" => Some(adw::ColorScheme::ForceLight.to_value()),
                    "force-dark" => Some(adw::ColorScheme::ForceDark.to_value()),
                    _ => None,
                }
            })
            .transform_from(move |_, val| match val.get::<adw::ColorScheme>().unwrap() {
                adw::ColorScheme::Default => Some(String::from("default").to_variant().to_value()),
                adw::ColorScheme::ForceLight => Some(String::from("force-light").to_variant().to_value()),
                adw::ColorScheme::ForceDark => Some(String::from("force-dark").to_variant().to_value()),
                _ => None,
            })
            .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
            .build();

        // Quit App
        action_quit.connect_activate(clone!(@weak self as app => move |_, _| {
            // Request closing all windows. They then get the chance to display a save dialog on unsaved changes
            for appwindow in app.windows() {
                appwindow.close();
            }

            if app.windows().is_empty() {
                app.quit();
            }
        }));
    }

    // Accelerators / Keyboard Shortcuts
    pub fn setup_action_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Ctrl>q"]);
    }
}
