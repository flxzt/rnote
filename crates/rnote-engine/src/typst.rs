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
