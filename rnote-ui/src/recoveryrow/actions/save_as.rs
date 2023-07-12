use gettextrs::gettext;
use gtk4::{prelude::FileExt, subclass::prelude::ObjectSubclassIsExt, FileDialog, FileFilter};

use crate::{appwindow::RnAppWindow, recoveryrow::RnRecoveryRow};

pub(crate) async fn save_as(recoveryrow: &RnRecoveryRow, appwindow: &RnAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_suffix("rnote");
    filter.set_name(Some(&gettext(".rnote")));

    let filedialog = FileDialog::builder()
        .title("Save recovered file as...")
        .accept_label(gettext("Save"))
        .modal(true)
        .default_filter(&filter)
        .build();
    match filedialog.save_future(Some(appwindow)).await {
        Ok(f) => {
            std::fs::copy(
                recoveryrow
                    .imp()
                    .meta
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .recovery_file_path(),
                f.path().unwrap(),
            )
            .unwrap();
        }
        Err(e) => {
            log::error!("Failed to save revovery file as: {e}")
        }
    }
}
