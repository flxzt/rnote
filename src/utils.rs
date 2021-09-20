use gtk4::{gio, glib, prelude::*};
use std::error::Error;
use std::ops::Deref;
use std::path::PathBuf;
use tera::Tera;

use crate::config;

#[derive(Copy, Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "BoxedPos")]
pub struct BoxedPos {
    pub x: f64,
    pub y: f64,
}

pub fn now() -> String {
    let now = match glib::DateTime::new_now_local() {
        Ok(datetime) => match datetime.format("%F_%T") {
            Ok(s) => s.to_string(),
            Err(_) => String::from("1970-01-01_12:00::00"),
        },
        Err(_) => String::from("1970-01-01_12:00:00"),
    };

    now
}

pub fn aabb_new_positive(
    mins: na::Vector2<f64>,
    maxs: na::Vector2<f64>,
) -> p2d::bounding_volume::AABB {
    if (maxs - mins).norm() > 0.0 {
        p2d::bounding_volume::AABB::new(na::point![mins[0], mins[1]], na::point![maxs[0], maxs[1]])
    } else {
        p2d::bounding_volume::AABB::new(na::point![maxs[0], maxs[1]], na::point![mins[0], mins[1]])
    }
}

pub fn aabb_clamp(
    aabb: p2d::bounding_volume::AABB,
    min: Option<p2d::bounding_volume::AABB>,
    max: Option<p2d::bounding_volume::AABB>,
) -> p2d::bounding_volume::AABB {
    let mut aabb_mins_x = aabb.mins[0];
    let mut aabb_mins_y = aabb.mins[1];
    let mut aabb_maxs_x = aabb.maxs[0];
    let mut aabb_maxs_y = aabb.maxs[1];

    if let Some(min) = min {
        aabb_mins_x = aabb.mins[0].min(min.mins[0]);
        aabb_mins_y = aabb.mins[1].min(min.mins[1]);
        aabb_maxs_x = aabb.maxs[0].max(min.maxs[0]);
        aabb_maxs_y = aabb.maxs[1].max(min.maxs[1]);
    }
    if let Some(max) = max {
        aabb_mins_x = aabb.mins[0].max(max.mins[0]);
        aabb_mins_y = aabb.mins[1].max(max.mins[1]);
        aabb_maxs_x = aabb.maxs[0].min(max.maxs[0]);
        aabb_maxs_y = aabb.maxs[1].min(max.maxs[1]);
    }

    p2d::bounding_volume::AABB::new(
        na::point![aabb_mins_x, aabb_mins_y],
        na::point![aabb_maxs_x, aabb_maxs_y],
    )
}

pub fn load_string_from_resource(resource_path: &str) -> Result<String, Box<dyn Error>> {
    let imported_string = String::from_utf8(
        gio::resources_lookup_data(resource_path, gio::ResourceLookupFlags::NONE)?
            .deref()
            .to_vec(),
    )?;

    Ok(imported_string)
}

pub fn load_file_contents(file: &gio::File) -> Result<String, Box<dyn Error>> {
    let (result, _) = file.load_contents::<gio::Cancellable>(None)?;
    let contents = String::from_utf8(result)?;
    Ok(contents)
}

#[allow(dead_code)]
pub fn query_standard_file_info(file: &gio::File) -> Option<gio::FileInfo> {
    file.query_info::<gio::Cancellable>("standard::*", gio::FileQueryInfoFlags::NONE, None)
        .ok()
}

#[allow(dead_code)]
pub fn try_add_template(
    templates: &mut Tera,
    template_name: &str,
    template_str: &str,
) -> Result<(), Box<dyn Error>> {
    templates.add_raw_template(template_name, template_str)?;
    Ok(())
}

pub fn app_config_base_dirpath() -> Option<PathBuf> {
    let mut app_config_dirpath = glib::user_config_dir();
    app_config_dirpath.push(config::APP_NAME);
    let app_config_dir = gio::File::for_path(app_config_dirpath.clone());
    match app_config_dir.make_directory_with_parents::<gio::Cancellable>(None) {
        Ok(()) => Some(app_config_dirpath),
        Err(e) => match e.kind::<gio::IOErrorEnum>() {
            Some(gio::IOErrorEnum::Exists) => Some(app_config_dirpath),
            _ => {
                log::error!("failed to create app_config_dir, {}", e);
                None
            }
        },
    }
}

pub enum FileType {
    Folder,
    Rnote,
    Svg,
    BitmapImage,
    Unknown,
}

impl FileType {
    pub fn lookup_file_type(file: &gio::File) -> Self {
        if let Ok(info) =
            file.query_info::<gio::Cancellable>("standard::*", gio::FileQueryInfoFlags::NONE, None)
        {
            match info.file_type() {
                gio::FileType::Regular => {
                    if let Some(content_type) = info.content_type() {
                        match content_type.as_str() {
                            "image/svg+xml" => {
                                return Self::Svg;
                            }
                            "image/png" => {
                                return Self::BitmapImage;
                            }
                            _ => {}
                        }
                        if let Some(path) = file.path() {
                            if let Some(extension_str) = path.extension() {
                                match &*extension_str.to_string_lossy() {
                                    "rnote" => {
                                        return Self::Rnote;
                                    }
                                    _ => {}
                                }
                            }
                        };
                    }
                }
                gio::FileType::Directory => {
                    return Self::Folder;
                }
                _ => {
                    return Self::Unknown;
                }
            }
        }

        Self::Unknown
    }
}
