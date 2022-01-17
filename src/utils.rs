use crate::config;

use flate2::read::MultiGzDecoder;
use flate2::{Compression, GzBuilder};
use gtk4::{gdk, gio, glib, prelude::*};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Color {
    pub r: f32, // between 0.0 and 1.0
    pub g: f32, // between 0.0 and 1.0
    pub b: f32, // between 0.0 and 1.0
    pub a: f32, // between 0.0 and 1.0
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

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

    pub fn to_gdk(&self) -> gdk::RGBA {
        gdk::RGBA::new(self.r, self.g, self.b, self.a)
    }

    pub fn to_u32(&self) -> u32 {
        ((((self.r * 255.0).round() as u32) & 0xff) << 24)
            | ((((self.g * 255.0).round() as u32) & 0xff) << 16)
            | ((((self.b * 255.0).round() as u32) & 0xff) << 8)
            | (((self.a * 255.0).round() as u32) & 0xff)
    }
}

impl From<gdk::RGBA> for Color {
    fn from(gdk_color: gdk::RGBA) -> Self {
        Self {
            r: gdk_color.red(),
            g: gdk_color.green(),
            b: gdk_color.blue(),
            a: gdk_color.alpha(),
        }
    }
}

// u32 encoded as RGBA
impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self {
            r: ((value >> 24) & 0xff) as f32 / 255.0,
            g: ((value >> 16) & 0xff) as f32 / 255.0,
            b: ((value >> 8) & 0xff) as f32 / 255.0,
            a: ((value) & 0xff) as f32 / 255.0,
        }
    }
}

pub fn now() -> String {
    match glib::DateTime::now_local() {
        Ok(datetime) => match datetime.format("%F_%T") {
            Ok(s) => s.to_string(),
            Err(_) => String::from("1970-01-01_12:00::00"),
        },
        Err(_) => String::from("1970-01-01_12:00:00"),
    }
}

pub fn app_config_base_dirpath() -> Option<PathBuf> {
    let mut app_config_dirpath = glib::user_config_dir();
    app_config_dirpath.push(config::APP_NAME);
    let app_config_dir = gio::File::for_path(app_config_dirpath.clone());
    match app_config_dir.make_directory_with_parents(None::<&gio::Cancellable>) {
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
    RnoteFile,
    VectorImageFile,
    BitmapImageFile,
    Pdf,
    UnknownFile,
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
                            "image/svg+xml" => {
                                return Self::VectorImageFile;
                            }
                            "image/png" | "image/jpeg" => {
                                return Self::BitmapImageFile;
                            }
                            "application/pdf" => {
                                return Self::Pdf;
                            }
                            _ => {}
                        }
                    }
                }
                gio::FileType::Directory => {
                    return Self::Folder;
                }
                _ => {
                    log::warn!("unkown file type");
                    return Self::UnknownFile;
                }
            }
        } else {
            log::warn!("failed to query FileInfo from file");
        }

        if let Some(path) = file.path() {
            if let Some(extension_str) = path.extension() {
                match &*extension_str.to_string_lossy() {
                    "rnote" => {
                        return Self::RnoteFile;
                    }
                    _ => {}
                }
            }
        } else {
            log::warn!("no path for file");
        };

        Self::UnknownFile
    }
}

pub fn compress_to_gzip(to_compress: &[u8], file_name: &str) -> Result<Vec<u8>, anyhow::Error> {
    let compressed_bytes = Vec::<u8>::new();

    let mut encoder = GzBuilder::new()
        .filename(file_name)
        .comment("test")
        .write(compressed_bytes, Compression::default());

    encoder.write_all(to_compress)?;

    Ok(encoder.finish()?)
}

pub fn decompress_from_gzip(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut decoder = MultiGzDecoder::new(compressed);
    let mut bytes: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut bytes)?;

    Ok(bytes)
}

pub fn str_to_file(string: &str, file_path: &str) -> Result<(), anyhow::Error> {
    Ok(fs::write(PathBuf::from(file_path), string)?)
}

/// returns a new seed by generating a random value seeded from the old seed
pub fn seed_advance(seed: u64) -> u64 {
    let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
    rng.gen()
}
