use std::fs;
use std::path::PathBuf;

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

fn duplicate_file(source: PathBuf) {
    let mut duplicate_name = {
        let path = source.clone();
        path.push(DUPLICATE_SUFFIX);
        path
    };
}

fn duplicate_dir(_source: PathBuf) {}

fn get_duplicate_name(parent_dir: PathBuf, name: PathBuf) -> PathBuf {
    let path = name.into_boxed_path();

    name.push(DUPLICATE_SUFFIX);
    name
}
