// Imports
use crate::fileformats::xoppformat;
use geo::line_string;
use p2d::bounding_volume::Aabb;
use rnote_compose::Color;
use std::ops::Range;

#[cfg(feature = "ui")]
pub mod typst {
    use std::sync::OnceLock;
    use typst::Library;
    use typst::World;
    use typst::foundations::{Bytes, Datetime};
    use typst::syntax::{FileId, Source};
    use typst::text::{Font, FontBook};
    use typst::utils::LazyHash;

    /// Cached font data, initialized once on first use
    static FONT_DATA: OnceLock<(Vec<Font>, FontBook)> = OnceLock::new();

    /// Initialize and return cached font data
    fn get_font_data() -> &'static (Vec<Font>, FontBook) {
        FONT_DATA.get_or_init(|| {
            // Use embedded fonts with system fonts as fallback
            // Embedded fonts have higher priority by default
            let font_data = typst_kit::fonts::FontSearcher::new()
                .include_system_fonts(true)
                .search();

            let fonts: Vec<Font> = font_data
                .fonts
                .iter()
                .filter_map(|slot| slot.get())
                .collect();
            let book = font_data.book;

            (fonts, book)
        })
    }

    /// Compile Typst source code to SVG
    pub fn compile_to_svg(source: &str) -> anyhow::Result<String> {
        // Create a Typst world (compilation environment)
        let world = TypstWorld::new(source);

        // Compile the document
        let result = typst::compile(&world);
        let document = result.output.map_err(|errors| {
            let error_messages: Vec<String> =
                errors.iter().map(|e| format!("{}", e.message)).collect();
            anyhow::Error::msg(format!(
                "Typst compilation failed: {}",
                error_messages.join("; ")
            ))
        })?;

        // Render the first page to SVG
        if document.pages.is_empty() {
            anyhow::bail!("No pages in compiled document");
        }

        // TODO: allow for multiple pages
        let svg = typst_svg::svg(&document.pages[0]);
        Ok(svg)
    }

    /// A minimal Typst world implementation for compilation
    struct TypstWorld {
        source: Source,
        library: LazyHash<Library>,
        book: LazyHash<FontBook>,
        fonts: Vec<Font>,
    }

    impl TypstWorld {
        fn new(source: &str) -> Self {
            let (fonts, book) = get_font_data();

            Self {
                source: Source::detached(source),
                library: LazyHash::new(Library::default()),
                book: LazyHash::new(book.clone()),
                fonts: fonts.clone(),
            }
        }
    }

    impl World for TypstWorld {
        fn library(&self) -> &LazyHash<Library> {
            &self.library
        }

        fn book(&self) -> &LazyHash<FontBook> {
            &self.book
        }

        fn main(&self) -> FileId {
            self.source.id()
        }

        fn source(&self, id: FileId) -> typst::diag::FileResult<Source> {
            if id == self.main() {
                Ok(self.source.clone())
            } else {
                Err(typst::diag::FileError::NotFound(
                    id.vpath().as_rootless_path().to_path_buf(),
                ))
            }
        }

        fn file(&self, id: FileId) -> typst::diag::FileResult<Bytes> {
            Err(typst::diag::FileError::NotFound(
                id.vpath().as_rootless_path().to_path_buf(),
            ))
        }

        fn font(&self, index: usize) -> Option<Font> {
            self.fonts.get(index).cloned()
        }

        fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
            Datetime::from_ymd(2024, 1, 1)
        }
    }
}

pub const fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn color_from_xopp(xopp_color: xoppformat::XoppColor) -> Color {
    Color {
        r: f64::from(xopp_color.red) / 255.0,
        g: f64::from(xopp_color.green) / 255.0,
        b: f64::from(xopp_color.blue) / 255.0,
        a: f64::from(xopp_color.alpha) / 255.0,
    }
}

pub fn xoppcolor_from_color(color: Color) -> xoppformat::XoppColor {
    xoppformat::XoppColor {
        red: (color.r * 255.0).floor() as u8,
        green: (color.g * 255.0).floor() as u8,
        blue: (color.b * 255.0).floor() as u8,
        alpha: (color.a * 255.0).floor() as u8,
    }
}

pub fn now_formatted_string() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string()
}

pub fn doc_pages_files_names(file_stem_name: String, i: usize) -> String {
    file_stem_name + &format!(" - Page {i:02}")
}

pub fn convert_value_dpi(value: f64, current_dpi: f64, target_dpi: f64) -> f64 {
    (value / current_dpi) * target_dpi
}

pub fn convert_coord_dpi(
    coord: na::Vector2<f64>,
    current_dpi: f64,
    target_dpi: f64,
) -> na::Vector2<f64> {
    (coord / current_dpi) * target_dpi
}

#[cfg(feature = "ui")]
pub fn transform_to_gsk(transform: &rnote_compose::Transform) -> gtk4::gsk::Transform {
    gtk4::gsk::Transform::new().matrix(&gtk4::graphene::Matrix::from_2d(
        transform.affine[(0, 0)],
        transform.affine[(1, 0)],
        transform.affine[(0, 1)],
        transform.affine[(1, 1)],
        transform.affine[(0, 2)],
        transform.affine[(1, 2)],
    ))
}

/// Convert an [Aabb] to [`geo::Polygon<f64>`]
pub fn p2d_aabb_to_geo_polygon(aabb: Aabb) -> geo::Polygon<f64> {
    let line_string = line_string![
        (x: aabb.mins[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.mins[1]),
    ];
    geo::Polygon::new(line_string, vec![])
}

pub fn positive_range<I>(first: I, second: I) -> Range<I>
where
    I: PartialOrd,
{
    if first < second {
        first..second
    } else {
        second..first
    }
}

/// (De)Serialize a [glib::Bytes] with base64 encoding
pub mod glib_bytes_base64 {
    use serde::{Deserializer, Serializer};

    /// Serialize a [`Vec<u8>`] as base64 encoded
    pub fn serialize<S: Serializer>(v: &glib::Bytes, s: S) -> Result<S::Ok, S::Error> {
        rnote_compose::serialize::sliceu8_base64::serialize(v, s)
    }

    /// Deserialize base64 encoded [glib::Bytes]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<glib::Bytes, D::Error> {
        rnote_compose::serialize::sliceu8_base64::deserialize(d).map(glib::Bytes::from_owned)
    }
}
