// Imports
use crate::workspacebrowser::RnFileRow;
use crate::RnAppWindow;
use fs_extra::dir::{CopyOptions, TransitProcessResult};
use fs_extra::TransitProcess;
use gtk4::prelude::FileExt;
use gtk4::{gio, glib, glib::clone};
use once_cell::sync::Lazy;
use regex::Regex;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The regex used to search for duplicated files
/// ```text
/// - Look for delimiter
/// |         - Look for `<delim>1`/`<delim>2`/`<delim>123`/...
/// |         |        - Look for the rest after the `<delim><num>` part
/// |         |       |       - At the end of the file name
/// |         |       |       |
/// |        \d*      |       $
/// |                 |
/// DELIM       (?P<rest>(.*))
/// ```
static DUP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&(crate::utils::FILE_DUP_SUFFIX_DELIM_REGEX.to_string() + r"\d*(?P<rest>(.*))$"))
        .unwrap()
});

/// Create a new `duplicate` action.
pub(crate) fn duplicate(filerow: &RnFileRow, appwindow: &RnAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("duplicate", None);

    action.connect_activate(
        clone!(@weak filerow as filerow, @weak appwindow => move |_action_duplicate_file, _| {
            if let Some(current_file) = filerow.current_file() {
                if let Some(current_path) = current_file.path() {
                    if current_path.is_file() {
                        if let Err(e) = duplicate_file(&current_path) {
                            tracing::error!("Duplicating file for path `{current_path:?}` failed, Err: {e:?}");
                        }
                    } else if current_path.is_dir() {
                        let progress_handler = creat_dup_dir_progress_handler(appwindow.clone());

                        if let Err(e) = duplicate_dir(&current_path, progress_handler) {
                            tracing::error!("Duplicating directory for path `{current_path:?}` failed, Err: {e:?}");
                        }
                    }
                }
            } else {
                tracing::warn!("Could not duplicate file, current file is None.");
            }

            appwindow.overlays().progressbar_finish();
        }),
    );

    action
}

/// Returns the progress handler for
/// [copy_items_with_progress](https://docs.rs/fs_extra/1.2.0/fs_extra/fn.copy_items_with_progress.html).
fn creat_dup_dir_progress_handler(
    appwindow: RnAppWindow,
) -> impl Fn(TransitProcess) -> TransitProcessResult {
    move |process: TransitProcess| -> TransitProcessResult {
        let status = {
            let status = process.copied_bytes / process.total_bytes;
            status as f64
        };
        appwindow.overlays().progressbar().set_fraction(status);
        TransitProcessResult::ContinueOrAbort
    }
}

fn duplicate_file(source: impl AsRef<Path>) -> anyhow::Result<()> {
    let destination = generate_destination_path(&source)?;
    std::fs::copy(source, destination)?;
    Ok(())
}

fn duplicate_dir<F>(source: impl AsRef<Path>, progress_handler: F) -> anyhow::Result<()>
where
    F: Fn(TransitProcess) -> TransitProcessResult,
{
    let destination = generate_destination_path(&source)?;
    let options = CopyOptions {
        copy_inside: true,
        ..CopyOptions::default()
    };
    fs_extra::copy_items_with_progress(
        &[source.as_ref()],
        destination,
        &options,
        progress_handler,
    )?;
    Ok(())
}

/// returns a suitable not-already-existing destination path from the given source path
/// by adding or replacing `<delim><num>` to the source-path, where `<num>` is incremented as often as needed.
fn generate_destination_path(source: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let mut duplicate_index = 1;
    let mut destination_path = source.as_ref().to_owned();
    let adjusted_source_path = remove_dup_suffix(source);

    let Some(source_stem) = adjusted_source_path.file_stem() else {
        return Err(anyhow::anyhow!(
            "file of source path '{adjusted_source_path:?}' does not have a file stem."
        ));
    };
    loop {
        destination_path.set_file_name(generate_duplicate_filename(
            source_stem,
            adjusted_source_path.extension(),
            duplicate_index,
        ));

        if !destination_path.exists() {
            return Ok(destination_path);
        }

        tracing::debug!(
            "File '{destination_path:?}' already exists. Incrementing duplication index.",
        );
        duplicate_index += 1;
    }
}

/// Creates the duplicate-filename by the given information about the source.
///
/// For example:
/// "test.txt" => "test-1.txt" => "test-2.txt"
fn generate_duplicate_filename(
    source_stem: &OsStr,
    source_extension: Option<&OsStr>,
    duplicate_index: i32,
) -> OsString {
    let mut duplicate_filename = OsString::from(source_stem);
    duplicate_filename.push(crate::utils::FILE_DUP_SUFFIX_DELIM);
    duplicate_filename.push(duplicate_index.to_string());
    if let Some(extension) = source_extension {
        duplicate_filename.push(OsString::from("."));
        duplicate_filename.push(extension);
    }
    duplicate_filename
}

/// Recursively removes found file-name suffixes that match with the above regex from the given file path.
fn remove_dup_suffix(source: impl AsRef<Path>) -> PathBuf {
    let mut removed = source.as_ref().to_string_lossy().to_string();
    loop {
        let new = DUP_REGEX.replace(&removed, "$rest").to_string();
        if removed == new {
            break;
        }
        removed = new;
    }
    PathBuf::from(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_dup_suffix() {
        let suf = crate::utils::FILE_DUP_SUFFIX_DELIM;
        {
            let source = PathBuf::from("normal_file.txt");
            let expected = source.clone();
            assert_eq!(expected, remove_dup_suffix(&source));
        }

        {
            let source = PathBuf::from(String::from("normal_file") + suf + "1.txt");
            let expected = PathBuf::from("normal_file.txt");
            assert_eq!(expected, remove_dup_suffix(&source));
        }

        {
            let source = PathBuf::from(String::from("normal_file") + suf + "2.txt");
            let expected = PathBuf::from("normal_file.txt");
            assert_eq!(expected, remove_dup_suffix(&source));
        }

        {
            let source = PathBuf::from("normal_file.1.txt");
            let expected = PathBuf::from("normal_file.1.txt");
            assert_eq!(expected, remove_dup_suffix(&source));
        }

        {
            let source = PathBuf::from("normal_folder");
            let expected = source.clone();
            assert_eq!(expected, remove_dup_suffix(&source));
        }

        {
            let source = PathBuf::from(String::from("normal_folder") + suf);
            let expected = PathBuf::from("normal_folder");
            assert_eq!(expected, remove_dup_suffix(&source));
        }
    }
}
