use crate::app::RnoteApp;
use adw::prelude::*;
use gtk4::{gio, glib, glib::clone};

impl RnoteApp {
    pub fn setup_actions(&self) {
        // Actions Definitions
        let action_quit = gio::SimpleAction::new("quit", None);
        self.add_action(&action_quit);

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

    // ### Accelerators / Keyboard Shortcuts
    pub fn setup_action_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Ctrl>q"]);
    }
}
