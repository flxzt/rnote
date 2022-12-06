use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use fs_extra::dir::{CopyOptions, TransitProcessResult};
use fs_extra::{copy_items_with_progress, TransitProcess};
use gtk4::prelude::FileExt;
use gtk4::{gio, glib, glib::clone};
use regex::Regex;

use crate::workspacebrowser::FileRow;
use crate::RnoteAppWindow;

///                                 - Look for `.dup` pattern
///                                 |   - Look for `.dup1`/`.dup123`/`.dup1234`/...
///                                 |   |        - Look for the text after the `.dup<num>` part
///                                 |   |       |       - At the end of the word (here: file-path)
///                                 |  \d*      |       $
///                                 |           |
///                               \.dup   (?P<rest>(.*))
const DUP_REGEX_PATTERN: &str = r"\.dup\d*(?P<rest>(.*))$";
const DUPLICATE_SUFFIX: &str = ".dup";
const DOT: &str = ".";

/// Creates a new `duplicate` action
pub(crate) fn duplicate(filerow: &FileRow, appwindow: &RnoteAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("duplicate", None);

    action.connect_activate(
        clone!(@weak filerow as filerow, @weak appwindow => move |_action_duplicate_file, _| {
            let process_evaluator = create_process_evaluator(appwindow.clone());

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

            appwindow.canvas_wrapper().finish_progressbar();
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

        appwindow
            .canvas_wrapper()
            .progressbar()
            .set_fraction(status);
        TransitProcessResult::ContinueOrAbort
    }
}

fn duplicate_file(source_path: PathBuf) {
    if let Some(destination) = get_destination_path(&source_path) {
        let source = source_path.into_boxed_path();

        log::debug!("Duplicate source: {}", source.display());
        log::debug!("Duplicate destination: {}", destination.display());
        if let Err(e) = std::fs::copy(source, destination) {
            log::error!("Couldn't duplicate file, Err: {e:?}");
        }
    } else {
        log::warn!("Destination-file for duplication not found.");
    }
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
        if let Err(e) =
            copy_items_with_progress(&[source], destination, &options, process_evaluator)
        {
            log::error!("Couldn't copy items, Err: {e:?}");
        }
    }
}

/// returns a suitable destination path from the given source path
/// by adding `.dup` as often as needed to the source-path
fn get_destination_path(source_path: &PathBuf) -> Option<PathBuf> {
    let mut duplicate_index = 0;
    let mut destination_path = source_path.clone();
    let adjusted_source_path = remove_dup_word(source_path);

    if let Some(source_stem) = adjusted_source_path.file_stem() {
        loop {
            let destination_filename = generate_duplicate_filename(
                source_stem,
                adjusted_source_path.extension(),
                duplicate_index,
            );
            destination_path.set_file_name(destination_filename);

            if !destination_path.exists() {
                return Some(destination_path);
            }

            log::debug!("File '{}' already exists.", destination_path.display());
            duplicate_index += 1;
        }
    } else {
        log::debug!("No source stem for '{}'.", source_path.display());
    }

    None
}

/// Creates the duplicate-filename by the given information about the source.
///
/// ## Example
/// "test.txt" => "test.dup.txt" => "test.dup1.txt"
fn generate_duplicate_filename(
    source_stem: &OsStr,
    source_extension: Option<&OsStr>,
    duplicate_index: i32,
) -> OsString {
    let mut duplicate_filename = OsString::new();

    duplicate_filename.push(source_stem);
    duplicate_filename.push(DUPLICATE_SUFFIX);

    if duplicate_index > 0 {
        duplicate_filename.push(duplicate_index.to_string());
    }

    if let Some(extension) = source_extension {
        duplicate_filename.push(DOT);
        duplicate_filename.push(extension);
    }

    duplicate_filename
}

fn remove_dup_word(source_stem: impl AsRef<Path>) -> PathBuf {
    let source_stem = source_stem.as_ref().to_string_lossy().to_string();

    let re = Regex::new(DUP_REGEX_PATTERN).unwrap();

    let removed_dup_suffix = re.replace(&source_stem, "$rest").to_string();
    PathBuf::from(removed_dup_suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_dup_suffix() {
        // test on filename without ".dup" in name
        let normal = PathBuf::from("normal_file.txt");
        let normal_expected = normal.clone();
        assert_eq!(normal_expected, remove_dup_word(&normal));

        // test with ".dup" name
        let normal_dup = PathBuf::from("normal_file.dup.txt");
        let normal_dup_expected = PathBuf::from("normal_file.txt");
        assert_eq!(normal_dup_expected, remove_dup_word(&normal_dup));

        // test with ".dup1" which means, that a duplicated file has been duplicated
        let normal_dup1 = PathBuf::from("normal_file.dup1.txt");
        let normal_dup1_expected = PathBuf::from("normal_file.txt");
        assert_eq!(normal_dup1_expected, remove_dup_word(&normal_dup1));
    }
}
