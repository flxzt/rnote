use adw::{
    prelude::MessageDialogExtManual,
    traits::{ActionRowExt, PreferencesGroupExt},
};
use cairo::glib::{self, clone};
use gettextrs::gettext;
use gtk4::{prelude::FileExt, traits::ToggleButtonExt};
use gtk4::{
    subclass::prelude::ObjectSubclassIsExt, traits::GtkWindowExt, Builder, FileDialog, ToggleButton,
};
use std::{ffi::OsStr, fs::read_dir, path::PathBuf};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::{appwindow::RnAppWindow, config, env::recovery_dir};
use rnote_engine::fileformats::recovery_metadata::RecoveryMetadata;

#[derive(Clone, Debug)]
pub(crate) enum RnRecoveryAction {
    Discard,
    SaveAs(PathBuf),
    Keep,
    Open,
}

pub(crate) async fn dialog_recover_documents(appwindow: &RnAppWindow) {
    let files = get_files();
    if files.is_empty() {
        log::debug!("No recovery files found");
        // return;
    }
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/recovery.ui").as_str(),
    );
    let mut rows = Vec::new();
    let dialog: adw::MessageDialog = builder.object("dialog_recover_documents").unwrap();
    let recover_documents_group: adw::PreferencesGroup =
        builder.object("recover_documents_group").unwrap();
    dialog.set_transient_for(Some(appwindow));
    appwindow
        .imp()
        .recovery_actions
        .replace([(); 4].map(|_| RnRecoveryAction::Discard).to_vec());
    for (i, metadata) in files.iter().enumerate() {
        // let recovery_row: RnRecoveryRow = RnRecoveryRow::new();
        // recovery_row.init(appwindow, metadata.clone());
        let row: adw::ActionRow = adw::ActionRow::builder()
            .title(metadata.title().unwrap_or_else(|| String::from("Unsaved")))
            .subtitle(format_unix_timestamp(metadata.last_changed()))
            .subtitle_lines(2)
            .build();
        let open_button = ToggleButton::builder()
            .icon_name("tab-new-filled-symbolic")
            .tooltip_text("Recover document in new tab")
            .build();
        let save_as_button = ToggleButton::builder()
            .icon_name("doc-save-symbolic")
            .tooltip_text("Save file to selected path")
            .group(&open_button)
            .build();
        let keep_button = ToggleButton::builder()
            .icon_name("workspacelistentryicon-clock-symbolic")
            .tooltip_text("Ask me again next session")
            .group(&open_button)
            .build();
        let discard_button = ToggleButton::builder()
            .icon_name("trash-empty")
            .tooltip_text("Discard document")
            .active(true)
            .group(&open_button)
            .build();
        discard_button.connect_toggled(clone!(@weak appwindow => move |button| {
            if button.is_active() {
                appwindow.set_recovery_action(i, RnRecoveryAction::Discard)
            }
        }));
        open_button.connect_toggled(clone!(@weak appwindow => move |button| {
            if button.is_active(){
                appwindow.set_recovery_action(i, RnRecoveryAction::Open)
            }
        }));
        save_as_button.connect_toggled(clone!(@weak appwindow => move |button| {
            if !button.is_active(){
                return;
            }
            glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                let filedialog = FileDialog::builder()
                    .title("Save recovered file as...")
                    .accept_label(gettext("Save"))
                    .modal(true)
                    .build();

                match filedialog.save_future(Some(&appwindow)).await {
                    Ok(f) => {
                        let path = f.path().unwrap();
                        // if path.extension().ne(Some("rnote")){
                        //     path.set_extension()
                        // }
                        appwindow.set_recovery_action(i, RnRecoveryAction::SaveAs(path))
                    }
                    Err(e) => {
                        log::error!("Failed to get save path for revovery file: {e}")
                    }
                }
            }));
        }));
        keep_button.connect_toggled(clone!(@weak appwindow => move |button| {
            if button.is_active(){
                appwindow.set_recovery_action(i, RnRecoveryAction::Keep)
            }
        }));
        // recover_document_button.connect_clicked();
        // save_as_button.connect_clicked();
        // discard_button.connect_clicked(clone!(@weak appwindow => move |button|{
        //      b

        // }));
        row.add_suffix(&open_button);
        row.add_suffix(&save_as_button);
        row.add_suffix(&discard_button);
        recover_documents_group.add(&row);
        rows.push(row);
    }
    dialog.choose_future().await;
}

fn get_files() -> Vec<RecoveryMetadata> {
    let mut recovery_files = Vec::new();
    let recovery_ext: &OsStr = OsStr::new("json");
    for file in read_dir(recovery_dir().expect("Failed to get recovery dir"))
        .expect("failed to read recovery dir")
    {
        let file = file.expect("Failed to get DirEntry");
        if file.path().extension().ne(&Some(recovery_ext)) {
            continue;
        }
        let metadata =
            RecoveryMetadata::load_from_path(&file.path()).expect("Failed to load recovery file");
        recovery_files.push(metadata);
    }
    recovery_files
}

fn format_unix_timestamp(unix: i64) -> String {
    // Shows occuring errors in timesptamp label field instead of crashing
    match OffsetDateTime::from_unix_timestamp(unix) {
        Err(e) => {
            log::error!("Failed to get time from unix time: {e}");
            String::from("Error getting time")
        }
        Ok(ts) => ts.format(&Rfc2822).unwrap_or_else(|e| {
            log::error!("Failed to format time: {e}");
            String::from("Error formatting time")
        }),
    }
}

// pub(crate) async fn discard(appwindow: &RnAppWindow, i: usize) /*-> gio::SimpleAction*/
// {
//     let action_discard_file = gio::SimpleAction::new("discard", None);
//     action_discard_file.connect_activate(
//         clone!(@weak appwindow => move |_action_discard_file, _| {
//             let medata = appwindow.imp().recovered_documents.borrow().get(i);
//             if metadata.is_some() && imp.meta.borrow().is_some() {
//                 // Unwrapping should be safe here since the condition makes sure they're not None
//                 let meta = imp.meta.replace(None).unwrap();
//                 let meta_path = imp.meta_path.replace(None).unwrap();

//                 if let Err(e) = remove_file(meta.recovery_file_path()){
//                     log::error!("Failed to remove recovery file {}: {e}", meta.recovery_file_path().display())
//                 };
//                 if let Err(e) = remove_file(meta_path.path().unwrap()){
//                     log::error!("Failed to remove recovery file {}: {e}", meta_path)
//                 };
//             }
//         }),
//     );

//     // action_discard_file
// }
