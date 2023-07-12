// Imports
use crate::recoveryrow::RnRecoveryRow;
use cairo::glib::{self, clone};
use gtk4::{gio, prelude::FileExt, subclass::prelude::ObjectSubclassIsExt};
use std::fs::remove_file;

pub(crate) async fn discard(recoveryrow: &RnRecoveryRow) /*-> gio::SimpleAction*/
{
    let action_discard_file = gio::SimpleAction::new("discard", None);
    action_discard_file.connect_activate(
        clone!(@weak recoveryrow => move |_action_discard_file, _| {
            let imp = recoveryrow.imp();
            if imp.meta_path.borrow().is_some() && imp.meta.borrow().is_some() {
                // Unwrapping should be safe here since the condition makes sure they're not None
                let meta = imp.meta.replace(None).unwrap();
                let meta_path = imp.meta_path.replace(None).unwrap();

                if let Err(e) = remove_file(meta.recovery_file_path()){
                    log::error!("Failed to remove recovery file {}: {e}", meta.recovery_file_path().display())
                };
                if let Err(e) = remove_file(meta_path.path().unwrap()){
                    log::error!("Failed to remove recovery file {}: {e}", meta_path)
                };
            }
        }),
    );

    // action_discard_file
}
