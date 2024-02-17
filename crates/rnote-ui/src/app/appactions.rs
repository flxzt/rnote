// Imports
use crate::RnApp;
use adw::prelude::*;
use gtk4::{gio, glib, glib::clone};

impl RnApp {
    pub(crate) fn setup_actions(&self) {
        let action_quit = gio::SimpleAction::new("quit", None);
        self.add_action(&action_quit);

        let action_color_scheme = gio::SimpleAction::new_stateful(
            "color-scheme",
            Some(&glib::VariantType::new("s").unwrap()),
            &String::from("system").to_variant(),
        );
        self.add_action(&action_color_scheme);

        let action_new_window = gio::SimpleAction::new("new-window", None);
        self.add_action(&action_new_window);

        action_color_scheme
            .bind_property("state", &self.style_manager(), "color-scheme")
            .transform_to(move |_, val: glib::Variant| {
                match val.get::<String>().unwrap().as_str() {
                    "default" => Some(adw::ColorScheme::Default),
                    "force-light" => Some(adw::ColorScheme::ForceLight),
                    "force-dark" => Some(adw::ColorScheme::ForceDark),
                    _ => None,
                }
            })
            .transform_from(move |_, val: adw::ColorScheme| match val {
                adw::ColorScheme::Default => Some(String::from("default").to_value()),
                adw::ColorScheme::ForceLight => Some(String::from("force-light").to_value()),
                adw::ColorScheme::ForceDark => Some(String::from("force-dark").to_value()),
                _ => None,
            })
            .sync_create()
            .bidirectional()
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

        action_new_window.connect_activate(clone!(@weak self as app => move |_, _| {
            app.new_appwindow_init_show();
        }));
    }

    // Accelerators / Keyboard Shortcuts
    pub(crate) fn setup_action_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Ctrl>q"]);
        self.set_accels_for_action("app.new-window", &["<Ctrl>n"]);
    }
}
