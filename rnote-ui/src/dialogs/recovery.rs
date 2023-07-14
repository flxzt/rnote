use adw::{
    prelude::MessageDialogExtManual,
    traits::{ActionRowExt, MessageDialogExt, PreferencesGroupExt},
};
use cairo::glib::{self, clone, Cast};
use gettextrs::gettext;
use gtk4::{
    gdk::Display,
    gio,
    prelude::{DisplayExt, FileExt},
    subclass::prelude::ObjectSubclassIsExt,
    traits::GtkWindowExt,
    traits::ToggleButtonExt,
    Builder, FileDialog, ToggleButton,
};
use std::{
    ffi::OsStr,
    fs::remove_file,
    path::{Path, PathBuf},
};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::{appwindow::RnAppWindow, canvaswrapper::RnCanvasWrapper, config, env::recovery_dir};
use rnote_engine::RnRecoveryMetadata;

#[derive(Clone, Debug, Default)]
pub(crate) enum RnRecoveryAction {
    Discard,
    SaveAs(PathBuf),
    ShowLater,
    #[default]
    Open,
    CleanInvalid,
}

pub(crate) async fn dialog_recovery_info(appwindow: &RnAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/recovery.ui").as_str(),
    );
    let dialog: adw::MessageDialog = builder.object("dialog_recovery_info").unwrap();
    dialog.set_transient_for(Some(appwindow));
    dialog.set_modal(true);
    let canvas = appwindow.active_tab().canvas();
    let info = {
        let recovery = appwindow.recovery();
        let autosave = appwindow.autosave();
        let unsaved_changes_recovery = canvas.unsaved_changes_recovery();
        let unsaved_changes = canvas.unsaved_changes();
        let recovery_metadata = canvas.imp().recovery_metadata.borrow();
        let recovery_paused = canvas.recovery_paused();
        let created = recovery_metadata
            .as_ref()
            .map(|m| format_unix_timestamp(m.crated()));
        let last_changed = recovery_metadata
            .as_ref()
            .map(|m| format_unix_timestamp(m.last_changed()));
        format!(
            "recovery: {recovery}
autosave: {autosave}
unsaved_changed: {unsaved_changes}
unsaved_changes_recovery: {unsaved_changes_recovery}
metadata: {recovery_metadata:#?}
recovery_paused: {recovery_paused}
created: {created:?}
last_changed: {last_changed:?}",
        )
    };
    dialog.set_body(&info);
    match dialog.choose_future().await.as_str() {
        "copy" => Display::default().unwrap().clipboard().set_text(&info),
        "ok" => (),
        c => unimplemented!("{c}"),
    };
}

pub(crate) async fn dialog_recover_documents(appwindow: &RnAppWindow) {
    let metadata_found = find_metadata();
    if metadata_found.is_empty() {
        log::debug!("No recovery files found");
        return;
    }
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/recovery.ui").as_str(),
    );
    let mut rows = Vec::new();
    let dialog: adw::MessageDialog = builder.object("dialog_recover_documents").unwrap();
    let recover_documents_group: adw::PreferencesGroup =
        builder.object("recover_documents_group").unwrap();
    dialog.set_transient_for(Some(appwindow));
    appwindow.imp().recovery_actions.replace(Some(vec![
        RnRecoveryAction::default();
        metadata_found.len()
    ]));
    for (i, metadata) in metadata_found.iter().enumerate() {
        // let recovery_row: RnRecoveryRow = RnRecoveryRow::new();
        // recovery_row.init(appwindow, metadata.clone());
        let mut valid = true;
        let subtitle = match metadata.recovery_file_path() {
            // detect default value => missing path
            p if p == PathBuf::from("/") => {
                valid = false;
                "ERROR: No recovery_file specified".to_string()
            }
            p if !p.exists() => {
                valid = false;
                "ERROR: recovery_file does not exist".to_string()
            }
            _ => format!(
                "Created: {}\nLast Changed: {}",
                format_unix_timestamp(metadata.crated()),
                format_unix_timestamp(metadata.last_changed())
            ),
        };
        let row: adw::ActionRow = adw::ActionRow::builder()
            .title(metadata.title().unwrap_or_else(|| String::from("Unsaved")))
            .subtitle(subtitle)
            .subtitle_lines(2)
            .build();
        let open_button = ToggleButton::builder()
            .icon_name("tab-new-filled-symbolic")
            .tooltip_text("Recover document in new tab")
            .active(true)
            .build();
        let save_as_button = ToggleButton::builder()
            .icon_name("doc-save-symbolic")
            .tooltip_text("Save file to selected path")
            .group(&open_button)
            .build();
        let show_later_button = ToggleButton::builder()
            .icon_name("workspacelistentryicon-clock-symbolic")
            .tooltip_text("Show option again next launch")
            .group(&open_button)
            .build();
        let discard_button = ToggleButton::builder()
            .icon_name("trash-empty")
            .tooltip_text("Discard document")
            .group(&open_button)
            .build();
        if valid {
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
                            let mut path = f.path().unwrap();
                            if path.extension() != Some(OsStr::new("rnote")){
                                path.set_extension("rnote");
                            }
                            appwindow.set_recovery_action(i, RnRecoveryAction::SaveAs(path))
                        }
                        Err(e) => {
                            log::error!("Failed to get save path for revovery file: {e}")
                        }
                    }
                }));
            }));
            show_later_button.connect_toggled(clone!(@weak appwindow => move |button| {
                if button.is_active(){
                    appwindow.set_recovery_action(i, RnRecoveryAction::ShowLater)
                }
            }));
            row.add_suffix(&open_button);
            row.add_suffix(&save_as_button);
            row.add_suffix(&show_later_button);
            row.add_suffix(&discard_button);
        } else {
            appwindow.set_recovery_action(i, RnRecoveryAction::CleanInvalid)
        }

        recover_documents_group.add(&row);
        rows.push(row);
    }
    let choice = dialog.choose_future().await;
    let mut actions = appwindow.imp().recovery_actions.replace(None).unwrap();
    assert_eq!(metadata_found.len(), actions.len());
    match choice.as_str() {
        // CleanInvalid will always be executed
        "discard_all" => {
            for action in &mut actions {
                match action {
                    RnRecoveryAction::CleanInvalid => (),
                    _ => *action = RnRecoveryAction::Discard,
                }
            }
        }
        "show_later" => {
            for action in &mut actions {
                match action {
                    RnRecoveryAction::CleanInvalid => (),
                    _ => *action = RnRecoveryAction::ShowLater,
                }
            }
        }

        "apply" => (),
        c => unimplemented!("unknown choice {}", c),
    };
    for (i, meta) in metadata_found.into_iter().enumerate() {
        match &actions[i] {
            RnRecoveryAction::Discard => discard(meta),
            RnRecoveryAction::ShowLater => (),
            RnRecoveryAction::Open => open(appwindow, meta),
            RnRecoveryAction::SaveAs(target) => {
                save_as(&meta, target);
                discard(meta)
            }
            RnRecoveryAction::CleanInvalid => {
                if let Err(e) = remove_file(&meta.metadata_path()) {
                    log::error!("Failedro delete {}, {e}", meta.metadata_path().display())
                }
            }
        }
    }
}

fn find_metadata() -> Vec<RnRecoveryMetadata> {
    let mut recovery_files = Vec::new();
    for file in recovery_dir()
        .expect("Failed to get recovery dir")
        .read_dir()
        .expect("failed to read recovery dir")
    {
        let Ok(file) = file else {
            log::error!("failed to get DirEntry");
            continue
        };
        // clean up .rnote files without metadata in the recovery dir
        // they are usally a result of broken recovery metadata
        if file.path().extension() == Some(OsStr::new("rnote")) {
            let mut json_path = file.path();
            json_path.set_extension("json");
            if !json_path.exists() {
                if let Err(e) = remove_file(&file.path()) {
                    log::error!("failed to remove {}, {e}", file.path().display())
                }
            }
            continue;
        } else if file.path().extension() != Some(OsStr::new("json")) {
            continue;
        }
        let metadata =
            RnRecoveryMetadata::load_from_path(&file.path()).expect("Failed to load recovery file");
        recovery_files.push(metadata);
    }
    recovery_files
}

fn format_unix_timestamp(unix: u64) -> String {
    // Shows occuring errors in timesptamp label field instead of crashing
    match OffsetDateTime::from_unix_timestamp(unix as i64) {
        Err(e) => {
            log::error!("Failed to get time from unix time: {e}");
            String::from("Error getting time")
        }
        Ok(ts) => {
            let local_offset = time::UtcOffset::current_local_offset().unwrap_or_else(|e| {
                log::error!("Failed to get get local time, defaulting to UTC: {e}");
                time::UtcOffset::UTC
            });
            ts.to_offset(local_offset)
                .format(&Rfc2822)
                .unwrap_or_else(|e| {
                    log::error!("Failed to format time: {e}");
                    String::from("Error formatting time")
                })
        }
    }
}

pub(crate) fn discard(meta: RnRecoveryMetadata) {
    meta.delete()
}
pub(crate) fn save_as(meta: &RnRecoveryMetadata, target: &Path) {
    if let Err(e) = std::fs::rename(meta.recovery_file_path(), target) {
        log::error!(
            "Failed to move recovered document from {} to {}, because {e}",
            meta.recovery_file_path().display(),
            target.display()
        )
    }
}

pub(crate) fn open(appwindow: &RnAppWindow, meta: RnRecoveryMetadata) {
    let file = gio::File::for_path(meta.recovery_file_path());
    let canvas = {
        // open a new tab for rnote files
        let new_tab = appwindow.new_tab();
        new_tab
            .child()
            .downcast::<RnCanvasWrapper>()
            .unwrap()
            .canvas()
    };

    glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
        appwindow.overlays().start_pulsing_progressbar();
        match file.load_bytes_future().await {
            Ok((bytes, _)) => {
                if let Err(e) = canvas.load_in_rnote_bytes(bytes.to_vec(), file.path(), Some(meta)).await {
                    log::error!("load_in_rnote_bytes() failed with Err: {e:?}");
                    appwindow.overlays().dispatch_toast_error(&gettext("Opening .rnote file from recovery failed"));
                }
            }
            Err(e) => log::error!("failed to load bytes, Err: {e:?}"),
        }
        appwindow.overlays().finish_progressbar();
    }));
}
