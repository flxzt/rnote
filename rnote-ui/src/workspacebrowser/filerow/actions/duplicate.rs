use std::path::{Path, PathBuf};

use gtk4::prelude::FileExt;
use gtk4::{gio, glib, glib::clone};

use crate::workspacebrowser::FileRow;

const DUPLICATE_SUFFIX: &str = ".dup";

impl FileRow {
    pub fn duplicate_action(&self) -> gio::SimpleAction {
        let action = gio::SimpleAction::new("duplicate-file", None);

        action.connect_activate(
            clone!(@weak self as filerow => move |_action_duplicate_file, _| {
                if let Some(current_file) = filerow.current_file() {
                    if let Some(current_path) = current_file.path() {
                        let path = current_path.clone().into_boxed_path();

                        if path.is_dir() {
                            duplicate_dir(current_path);
                        } else if path.is_file() {
                            duplicate_file(current_path);
                        }
                    }
                }
            }),
        );

        action
    }
}

fn duplicate_file(source_path: PathBuf) {
    if let Some(destination) = get_destination_path(&source_path) {
        let source = source_path.into_boxed_path();
        if let Err(err) = std::fs::copy(source, destination) {
            log::error!("Couldn't duplicate file: {}", err);
        }
    }
    log::info!("Destination-file for duplication not found.");
}

fn duplicate_dir(_source: PathBuf) {}

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
            destination_file_name.push(DUPLICATE_SUFFIX);
            destination_path.set_file_name(destination_file_name.clone());
        }

        Some(destination_path)
    } else {
        None
    }
}
