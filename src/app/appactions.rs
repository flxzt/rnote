use crate::app::RnoteApp;
use adw::prelude::*;
use gtk4::{gio, glib, glib::clone};

/* Actions follow this principle:
without any state: the activation triggers the callback
with boolean state: They have no parameter, and a boolean state. activating the action inverts its state.
    A state change can also be directly requested with change_action_state( somebool ).
for other stateful actions: They have the same values as their state as their parameters. Activating the action with a parameter is equivalent to changing its state directly
*/
impl RnoteApp {
    pub fn setup_actions(&self) {
        // Actions Definitions
        let action_quit = gio::SimpleAction::new("quit", None);
        self.add_action(&action_quit);
        let action_color_scheme =
            gio::PropertyAction::new("color-scheme", &self.style_manager(), "color-scheme");
        self.add_action(&action_color_scheme);

        // Quit App
        action_quit.connect_activate(clone!(@weak self as app => move |_, _| {
            app.quit();
        }));
    }

    // ### Accelerators / Keyboard Shortcuts
    pub fn setup_action_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Ctrl>q"]);
    }
}
