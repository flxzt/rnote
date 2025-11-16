// Imports
use std::sync::OnceLock;
use typst::Library;
use typst::LibraryExt;
use typst::World;
use typst::foundations::{Bytes, Datetime};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst_kit::download::{Downloader, ProgressSink};
use typst_kit::package::PackageStorage;

/// Cached font data, initialized once on first use
static FONT_DATA: OnceLock<(Vec<Font>, FontBook)> = OnceLock::new();

/// Cached package storage, initialized once on first use
static PACKAGE_STORAGE: OnceLock<PackageStorage> = OnceLock::new();

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

/// Initialize and return cached package storage
fn get_package_storage() -> &'static PackageStorage {
    PACKAGE_STORAGE.get_or_init(|| {
        let downloader = Downloader::new(concat!("rnote/", env!("CARGO_PKG_VERSION")));
        PackageStorage::new(None, None, downloader)
    })
}

/// Compile Typst source code to SVG
pub fn compile_to_svg(source: &str) -> anyhow::Result<String> {
    // Create a Typst world (compilation environment)
    let world = TypstWorld::new(source);

    // Compile the document
    let document: PagedDocument = typst::compile(&world)
        .output
        .map_err(|errors| anyhow::Error::msg(format!("Typst compilation failed: {:?}", errors)))?;

    // Render the first page to SVG
    if document.pages.is_empty() {
        anyhow::bail!("No pages in compiled document");
    }

    // TODO: allow for multiple pages
    let svg = typst_svg::svg(&document.pages[0]);
    Ok(svg)
}

/// A Typst world implementation for compilation with package support
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
            // Try to load from package
            let data = self.file(id)?;
            let text = String::from_utf8(data.to_vec())
                .map_err(|_| typst::diag::FileError::InvalidUtf8)?;
            Ok(Source::new(id, text))
        }
    }

    fn file(&self, id: FileId) -> typst::diag::FileResult<Bytes> {
        // Check if this is a package file
        if let Some(spec) = id.package() {
            let storage = get_package_storage();
            let package_dir = storage.prepare_package(spec, &mut ProgressSink)?;
            let file_path = package_dir.join(id.vpath().as_rootless_path());

            std::fs::read(&file_path)
                .map(|data| Bytes::new(data))
                .map_err(|err| typst::diag::FileError::from_io(err, file_path.as_path()))
        } else {
            Err(typst::diag::FileError::NotFound(
                id.vpath().as_rootless_path().to_path_buf(),
            ))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd(2024, 1, 1)
    }
}
