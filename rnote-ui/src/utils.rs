// Imports
use gtk4::{gdk, gio, glib, prelude::*, Widget};
use p2d::bounding_volume::Aabb;
use std::cell::Ref;
use std::slice::Iter;

/// File types supported by Rnote.
#[derive(Debug)]
pub(crate) enum FileType {
    Folder,
    RnoteFile,
    VectorImageFile,
    BitmapImageFile,
    XoppFile,
    PdfFile,
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
                    _ => {}
                }
            }
        } else {
            log::warn!("no path for file");
        };

        Self::Unsupported
    }
}

/// Checks if the file is a temporary goutputstream file.
pub(crate) fn is_goutputstream_file(file: &gio::File) -> bool {
    if let Some(path) = file.path() {
        if let Some(file_name) = path.file_name() {
            if String::from(file_name.to_string_lossy()).starts_with(".goutputstream-") {
                return true;
            }
        }
    }

    false
}

/// Translates a Aabb from the the coordinate space of `widget` to `dest_widget`. None if the widgets don't have a common ancestor.
#[allow(unused)]
pub(crate) fn translate_aabb_to_widget(
    aabb: Aabb,
    widget: &impl IsA<Widget>,
    dest_widget: &impl IsA<Widget>,
) -> Option<Aabb> {
    let mins = {
        let coords = widget.translate_coordinates(dest_widget, aabb.mins[0], aabb.mins[1])?;
        na::point![coords.0, coords.1]
    };
    let maxs = {
        let coords = widget.translate_coordinates(dest_widget, aabb.maxs[0], aabb.maxs[1])?;
        na::point![coords.0, coords.1]
    };
    Some(Aabb::new(mins, maxs))
}

/// Create a new file or replace if it already exists, asynchronously.
pub(crate) async fn create_replace_file_future(
    bytes: Vec<u8>,
    file: &gio::File,
) -> anyhow::Result<()> {
    let output_stream = file
        .replace_future(
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
            glib::PRIORITY_HIGH,
        )
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "file replace_future() failed in create_replace_file_future(), Err: {e:?}"
            )
        })?;

    output_stream
        .write_all_future(bytes, glib::PRIORITY_HIGH)
        .await
        .map_err(|(_, e)| {
            anyhow::anyhow!(
                "output_stream write_all_future() failed in create_replace_file_future(), Err: {e:?}"
            )
        })?;
    output_stream
        .close_future(glib::PRIORITY_HIGH)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "output_stream close_future() failed in create_replace_file_future(), Err: {e:?}"
            )
        })?;

    Ok(())
}

pub(crate) fn str_from_u8_nul_utf8(utf8_src: &[u8]) -> Result<&str, std::str::Utf8Error> {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    std::str::from_utf8(&utf8_src[0..nul_range_end])
}

/// Gets the index of the AxisUse enum
///
/// TODO: Report to gtk-rs that [gdk::AxisUse] needs a [`Into<std::ops::Index>`] implementation for usage to retrieve pointer axes in [gdk::TimeCoord]
pub(crate) fn axis_use_idx(a: gdk::AxisUse) -> usize {
    match a {
        gdk::AxisUse::Ignore => 0,
        gdk::AxisUse::X => 1,
        gdk::AxisUse::Y => 2,
        gdk::AxisUse::DeltaX => 3,
        gdk::AxisUse::DeltaY => 4,
        gdk::AxisUse::Pressure => 5,
        gdk::AxisUse::Xtilt => 6,
        gdk::AxisUse::Ytilt => 7,
        gdk::AxisUse::Wheel => 8,
        gdk::AxisUse::Distance => 9,
        gdk::AxisUse::Rotation => 10,
        gdk::AxisUse::Slider => 11,
        _ => unreachable!(),
    }
}

pub fn default_file_title_for_export(
    output_file: Option<gio::File>,
    fallback: Option<&str>,
    suffix: Option<&str>,
) -> String {
    let mut title = output_file
        .and_then(|f| Some(f.basename()?.file_stem()?.to_string_lossy().to_string()))
        .unwrap_or_else(|| {
            fallback
                .map(|f| f.to_owned())
                .unwrap_or_else(rnote_engine::utils::now_formatted_string)
        });

    if let Some(suffix) = suffix {
        title += suffix;
    }

    title
}

/// Wrapper type that enables iterating over [`std::cell::RefCell<Vec<T>>`]
pub(crate) struct VecRefWrapper<'a, T: 'a> {
    r: Ref<'a, Vec<T>>,
}

impl<'a, 'b: 'a, T: 'a> IntoIterator for &'b VecRefWrapper<'a, T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Iter<'a, T> {
        self.r.iter()
    }
}

impl<'a, T> VecRefWrapper<'a, T>
where
    T: 'a,
{
    pub(crate) fn new(r: Ref<'a, Vec<T>>) -> Self {
        Self { r }
    }
}
