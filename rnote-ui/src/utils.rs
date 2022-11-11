use gtk4::{gio, glib, prelude::*, Widget};
use p2d::bounding_volume::AABB;

#[derive(Debug)]
pub enum FileType {
    Folder,
    RnoteFile,
    XoppFile,
    VectorImageFile,
    BitmapImageFile,
    PdfFile,
    Unsupported,
}

impl FileType {
    pub fn lookup_file_type(file: &gio::File) -> Self {
        if let Ok(info) = file.query_info(
            "standard::*",
            gio::FileQueryInfoFlags::NONE,
            None::<&gio::Cancellable>,
        ) {
            match info.file_type() {
                gio::FileType::Regular => {
                    if let Some(content_type) = info.content_type() {
                        match content_type.as_str() {
                            "application/rnote" => {
                                return Self::RnoteFile;
                            }
                            "application/x-xopp" => {
                                log::debug!(" is a xopp file ");
                                return Self::XoppFile;
                            }
                            "image/svg+xml" => {
                                return Self::VectorImageFile;
                            }
                            "image/png" | "image/jpeg" => {
                                return Self::BitmapImageFile;
                            }
                            "application/pdf" => {
                                return Self::PdfFile;
                            }
                            _ => {}
                        }
                    }
                }
                gio::FileType::Directory => {
                    return Self::Folder;
                }
                _ => {
                    log::warn!("unknown file type");
                    return Self::Unsupported;
                }
            }
        } else {
            log::warn!("failed to query FileInfo from file");
        }

        // match on file extensions as fallback
        if let Some(path) = file.path() {
            if let Some(extension_str) = path.extension() {
                match &*extension_str.to_string_lossy() {
                    "rnote" => {
                        return Self::RnoteFile;
                    }
                    "xopp" => {
                        return Self::XoppFile;
                    }
                    _ => {}
                }
            }
        } else {
            log::warn!("no path for file");
        };

        Self::Unsupported
    }
}

/// Translates a AABB to the coordinate space of the dest_widget. None if the widgets don't have a common ancestor
pub fn translate_aabb_to_widget(
    aabb: AABB,
    widget: &impl IsA<Widget>,
    dest_widget: &impl IsA<Widget>,
) -> Option<AABB> {
    let mins = {
        let coords = widget.translate_coordinates(dest_widget, aabb.mins[0], aabb.mins[1])?;
        na::point![coords.0, coords.1]
    };
    let maxs = {
        let coords = widget.translate_coordinates(dest_widget, aabb.maxs[0], aabb.maxs[1])?;
        na::point![coords.0, coords.1]
    };
    Some(AABB::new(mins, maxs))
}

/// Create a new file or replace if it already exists, asynchronously
pub async fn create_replace_file_future(bytes: Vec<u8>, file: &gio::File) -> anyhow::Result<()> {
    let output_stream = file
        .replace_future(
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
            glib::PRIORITY_HIGH_IDLE,
        )
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "file replace_future() failed in create_replace_file_future(), Err {}",
                e
            )
        })?;

    output_stream
        .write_all_future(bytes, glib::PRIORITY_HIGH_IDLE)
        .await
        .map_err(|(_, e)| {
            anyhow::anyhow!(
                "output_stream write_all_future() failed in create_replace_file_future(), Err {}",
                e
            )
        })?;
    output_stream
        .close_future(glib::PRIORITY_HIGH_IDLE)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "output_stream close_future() failed in create_replace_file_future(), Err {}",
                e
            )
        })?;

    Ok(())
}
