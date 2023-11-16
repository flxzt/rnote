// Imports
use gtk4::{gio, prelude::*};

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
            log::warn!("no path for file");
        };

        Self::Unsupported
    }
}
