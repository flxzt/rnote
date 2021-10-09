use gtk4::{gdk, gio, glib, prelude::*};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32, // between 0.0 and 1.0
    pub g: f32, // between 0.0 and 1.0
    pub b: f32, // between 0.0 and 1.0
    pub a: f32, // between 0.0 and 1.0
}

impl Default for Color {
    fn default() -> Self {
        Self::black()
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn r(&self) -> f32 {
        self.r
    }

    pub fn g(&self) -> f32 {
        self.g
    }

    pub fn b(&self) -> f32 {
        self.b
    }

    pub fn a(&self) -> f32 {
        self.a
    }

    pub fn to_css_color(self) -> String {
        format!(
            "rgb({:03},{:03},{:03},{:.3})",
            (self.r * 255.0) as i32,
            (self.g * 255.0) as i32,
            (self.b * 255.0) as i32,
            ((1000.0 * self.a).round() / 1000.0),
        )
    }

    pub fn from_gdk(gdk_color: gdk::RGBA) -> Self {
        Self {
            r: gdk_color.red,
            g: gdk_color.green,
            b: gdk_color.blue,
            a: gdk_color.alpha,
        }
    }

    pub fn to_gdk(&self) -> gdk::RGBA {
        gdk::RGBA {
            red: self.r,
            green: self.g,
            blue: self.b,
            alpha: self.a,
        }
    }

    pub fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

pub fn now() -> String {
    match glib::DateTime::new_now_local() {
        Ok(datetime) => match datetime.format("%F_%T") {
            Ok(s) => s.to_string(),
            Err(_) => String::from("1970-01-01_12:00::00"),
        },
        Err(_) => String::from("1970-01-01_12:00:00"),
    }
}

// Return mins, maxs
pub fn vec2_mins_maxs(
    first: na::Vector2<f64>,
    second: na::Vector2<f64>,
) -> (na::Vector2<f64>, na::Vector2<f64>) {
    if first[0] < second[0] && first[1] < second[1] {
        (first, second)
    } else if first[0] > second[0] && first[1] < second[1] {
        (
            na::vector![second[0], first[1]],
            na::vector![first[0], second[1]],
        )
    } else if first[0] < second[0] && first[1] > second[1] {
        (
            na::vector![first[0], second[1]],
            na::vector![second[0], first[1]],
        )
    } else {
        (second, first)
    }
}

pub fn aabb_new_positive(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
) -> p2d::bounding_volume::AABB {
    if start[0] <= end[0] && start[1] <= end[1] {
        p2d::bounding_volume::AABB::new(na::point![start[0], start[1]], na::point![end[0], end[1]])
    } else if start[0] > end[0] && start[1] <= end[1] {
        p2d::bounding_volume::AABB::new(na::point![end[0], start[1]], na::point![start[0], end[1]])
    } else if start[0] <= end[0] && start[1] > end[1] {
        p2d::bounding_volume::AABB::new(na::point![start[0], end[1]], na::point![end[0], start[1]])
    } else {
        p2d::bounding_volume::AABB::new(na::point![end[0], end[1]], na::point![start[0], start[1]])
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

pub fn aabb_scale(
    aabb: p2d::bounding_volume::AABB,
    scalefactor: f64,
) -> p2d::bounding_volume::AABB {
    p2d::bounding_volume::AABB::new(
        na::Point2::<f64>::from(na::vector![0.0, 0.0].unscale(scalefactor)),
        na::Point2::<f64>::from(
            na::vector![aabb.maxs[0] - aabb.mins[0], aabb.maxs[1] - aabb.mins[0]]
                .unscale(scalefactor)
                + na::vector![aabb.mins[0], aabb.mins[1]],
        ),
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

#[derive(Debug)]
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
                            "image/png" | "image/jpeg" => {
                                return Self::BitmapImage;
                            }
                            _ => {}
                        }
                    }
                }
                gio::FileType::Directory => {
                    return Self::Folder;
                }
                _ => {
                    log::debug!("unkown file type");
                    return Self::Unknown;
                }
            }
        } else {
            log::debug!("failed to query FileInfo from file");
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
        } else {
            log::warn!("no path for file");
        };

        Self::Unknown
    }
}
