// Imports
use crate::recoveryrow::RnRecoveryRow;
use cairo::glib::{self, clone};
use gtk4::{gio, subclass::prelude::ObjectSubclassIsExt};
use std::fs::remove_file;

pub(crate) fn discard(recoveryrow: &RnRecoveryRow) /*-> gio::SimpleAction*/
{
    let action_discard_file = gio::SimpleAction::new("discard", None);
    action_discard_file.connect_activate(
        clone!(@weak recoveryrow => move |_action_discard_file, _| {
            let imp = recoveryrow.imp();
            if imp.meta_path.borrow().is_some() && imp.meta.borrow().is_some() {
                let meta = imp.meta.replace(None).unwrap();
                let meta_path = imp.meta_path.replace(None).unwrap();

                // Unwrapping should be safe here since
                remove_file(meta.recovery_file_path());
                remove_file(meta_path);
            }
        }),
    );

    // action_discard_file
}
