// Imports
use crate::workspacebrowser::RnFileRow;
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::prelude::FileExt;
use gtk4::{gio, glib, glib::clone};
use once_cell::sync::Lazy;
use regex::Regex;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use tracing::debug;

/// The regex used to search for duplicated files
/// ```text
/// - Look for original filename lazily
/// |         - Look for `<delim>1`/`<delim>2`/`<delim>123`/...
/// |         |        - Look for the file extension after the `<delim><num>` part
/// |         |       |       - At the end of the file name
/// |         |       |       |
/// |         (?P<delim{}\d+)$
/// |                 |
/// ^(.*?)            (\.\w+)
/// ```
static DUP_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Incorporate FILE_DUP_SUFFIX_DELIM_REGEX dynamically into the regex pattern
    let regex_pattern: String = format!(
        r"^(.*?)(?P<delim>{}\d*)?(\.\w+)?$",
        crate::utils::FILE_DUP_SUFFIX_DELIM_REGEX
    );

    Regex::new(&regex_pattern).unwrap()
});

/// Create a new `duplicate` action.
pub(crate) fn duplicate(filerow: &RnFileRow, appwindow: &RnAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("duplicate", None);

    action.connect_activate(clone!(@weak filerow, @weak appwindow => move |_, _| {
        glib::spawn_future_local(clone!(@weak filerow, @weak appwindow => async move {
            let Some(current_path) = filerow.current_file().and_then(|f| f.path()) else {
                appwindow.overlays().dispatch_toast_error(&gettext("Can't duplicate an unsaved document"));
                debug!("Could not duplicate file, current file is None.");
                return;
            };

            let mut success = true;
            appwindow.overlays().progressbar_start_pulsing();

            if current_path.is_file() {
                if let Err(e) = duplicate_file(&current_path).await {
                    appwindow.overlays().dispatch_toast_error(&gettext("Duplicating the file failed"));
                    debug!("Duplicating file for path `{current_path:?}` failed, Err: {e:?}");
                    success = false;
                }
            } else if current_path.is_dir() {
                if let Err(e) = duplicate_dir(&current_path).await {
                    appwindow.overlays().dispatch_toast_error(&gettext("Duplicating the directory failed"));
                    debug!("Duplicating directory for path `{current_path:?}` failed, Err: {e:?}");
                    success = false;
                }
            } else {
                success = false;
            }

            if success {
                appwindow.overlays().progressbar_finish();
            } else {
                appwindow.overlays().progressbar_abort();
            }
        }));
    }));

    action
}

async fn duplicate_file(source: impl AsRef<Path>) -> anyhow::Result<()> {
    let destination = generate_destination_path(&source)?;
    async_fs::copy(source, destination).await?;
    Ok(())
}

async fn duplicate_dir(source: impl AsRef<Path>) -> anyhow::Result<()> {
    let destination = generate_destination_path(&source)?;
    fs_extra::copy_items(
        &[source.as_ref()],
        destination,
        &fs_extra::dir::CopyOptions {
            copy_inside: true,
            ..Default::default()
        },
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

    let source_extension = adjusted_source_path.extension();

    // Loop to find the next available duplicate filename
    loop {
        destination_path.set_file_name(generate_duplicate_filename(
            source_stem,
            source_extension,
            duplicate_index,
        ));

        if !destination_path.exists() {
            return Ok(destination_path);
        }

        debug!("File '{destination_path:?}' already exists. Incrementing duplication index.",);
        duplicate_index += 1;
    }
}

/// Creates the duplicate-filename by the given information about the source.
///
/// For example:
/// "test.txt" => "test - 1.txt" => "test - 2.txt"
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
    let original = source.as_ref().to_string_lossy().to_string();
    // Preserve the base and extension, remove duplicate suffix
    let new = DUP_REGEX.replace(&original, "$1$3").to_string();

    PathBuf::from(new)
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

        {
            let source = PathBuf::from("normal - file.rnote");
            let expected = source.clone();
            assert_eq!(expected, remove_dup_suffix(&source));
        }

        {
            let source = PathBuf::from("normal - file - 1.rnote");
            let expected = PathBuf::from("normal - file.rnote");
            assert_eq!(expected, remove_dup_suffix(&source));
        }
    }
}
