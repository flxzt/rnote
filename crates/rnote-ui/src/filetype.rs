// Imports
use gtk4::{gio, prelude::*};
use tracing::warn;

/// File types supported by Rnote.
#[derive(Debug)]
pub(crate) enum FileType {
    Folder,
    RnoteFile,
    VectorImageFile,
    BitmapImageFile,
    XoppFile,
    PdfFile,
    PlaintextFile,
    Unsupported,
}

impl FileType {
    pub(crate) fn lookup_file_type(file: &gio::File) -> Self {
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
                            "image/svg+xml" => {
                                return Self::VectorImageFile;
                            }
                            "image/png" | "image/jpeg" => {
                                return Self::BitmapImageFile;
                            }
                            "application/x-xopp" => {
                                return Self::XoppFile;
                            }
                            "application/pdf" => {
                                return Self::PdfFile;
                            }
                            "text/plain" => {
                                return Self::PlaintextFile;
                            }
                            _ => {}
                        }
                    }
                }
                gio::FileType::Directory => {
                    return Self::Folder;
                }
                file_type => {
                    warn!("Looking up file type failed, unsupported file type `{file_type:?}`");
                    return Self::Unsupported;
                }
            }
        } else {
            warn!("Looking up file type failed, failed to query FileInfo from file `{file:?}`.");
        }

        // match on file extensions as fallback
        if let Some(path) = file.path() {
            if let Some(extension_str) = path.extension() {
                match &*extension_str.to_string_lossy() {
                    "rnote" => {
                        return Self::RnoteFile;
                    }
                    "svg" => {
                        return Self::VectorImageFile;
                    }
                    "jpg" | "jpeg" | "png" => {
                        return Self::BitmapImageFile;
                    }
                    "xopp" => {
                        return Self::XoppFile;
                    }
                    "pdf" => {
                        return Self::PdfFile;
                    }
                    "txt" => {
                        return Self::PlaintextFile;
                    }
                    _ => {}
                }
            }
        } else {
            warn!("Looking up file type failed, no path for file `{file:?}`.");
        };

        Self::Unsupported
    }
}
