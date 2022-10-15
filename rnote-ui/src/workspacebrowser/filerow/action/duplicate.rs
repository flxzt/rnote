use std::path::PathBuf;

use fs_extra::dir::{CopyOptions, TransitProcessResult};
use fs_extra::{copy_items_with_progress, TransitProcess};
use gtk4::prelude::FileExt;
use gtk4::{gio, glib, glib::clone};

use crate::workspacebrowser::FileRow;
use crate::RnoteAppWindow;

const DUPLICATE_SUFFIX: &str = ".dup";

pub fn duplicate(filerow: &FileRow, appwindow: &RnoteAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("duplicate", None);

    action.connect_activate(
        clone!(@weak filerow as filerow, @weak appwindow => move |_action_duplicate_file, _| {
            let process_evaluator = create_process_evaluator(appwindow);

            if let Some(current_file) = filerow.current_file() {
                if let Some(current_path) = current_file.path() {
                    let source_path = current_path.clone().into_boxed_path();

                    if source_path.is_dir() {
                        duplicate_dir(current_path, process_evaluator);
                    } else if source_path.is_file() {
                        duplicate_file(current_path);
                    }
                }
            }
        }),
    );

    action
}

/// returns the progress handler for
/// [copy_items_with_progress](https://docs.rs/fs_extra/1.2.0/fs_extra/fn.copy_items_with_progress.html)
fn create_process_evaluator(
    appwindow: RnoteAppWindow,
) -> impl Fn(TransitProcess) -> TransitProcessResult {
    move |process: TransitProcess| -> TransitProcessResult {
        let status = {
            let status = process.copied_bytes / process.total_bytes;
            status as f64
        };

        appwindow.canvas_progressbar().set_fraction(status);
        TransitProcessResult::ContinueOrAbort
    }
}

fn duplicate_file(source_path: PathBuf) {
    if let Some(destination) = get_destination_path(&source_path) {
        let source = source_path.into_boxed_path();

        log::debug!("Duplicate source: {}", source.display());
        log::debug!("Duplicate destination: {}", destination.display());
        if let Err(err) = std::fs::copy(source, destination) {
            log::error!("Couldn't duplicate file: {}", err);
        }
    }
    log::info!("Destination-file for duplication not found.");
}

fn duplicate_dir<F>(source_path: PathBuf, process_evaluator: F)
where
    F: Fn(TransitProcess) -> TransitProcessResult,
{
    if let Some(destination) = get_destination_path(&source_path) {
        let source = source_path.into_boxed_path();
        let options = CopyOptions {
            copy_inside: true,
            ..CopyOptions::default()
        };

        log::debug!("Duplicate source: {}", source.display());
        log::debug!("Duplicate destination: {}", destination.display());
        if let Err(err) =
            copy_items_with_progress(&[source], destination, &options, process_evaluator)
        {
            log::error!("Couldn't copy items: {}", err);
        }
    }
}

/// returns a suitable destination path from the given source path
/// by adding `.dup` as often as needed to the source-path
fn get_destination_path(source_path: &PathBuf) -> Option<PathBuf> {
    if let Some(destination_file_name) = source_path.file_name() {
        let mut destination_file_name = {
            let mut file_name = destination_file_name.to_os_string();
            file_name.push(DUPLICATE_SUFFIX);
            file_name
        };

        let mut destination_path = {
            let mut path = source_path.clone().to_path_buf();
            path.set_file_name(destination_file_name.clone());
            path
        };

        while destination_path.exists() {
            log::debug!("Destination: {} exists.", destination_path.display());
            destination_file_name.push(DUPLICATE_SUFFIX);
            destination_path.set_file_name(destination_file_name.clone());
        }

        log::debug!("Destination path: {}", destination_path.display());
        Some(destination_path)
    } else {
        None
    }
}
