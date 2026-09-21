//! Headless memory benchmark for the PDF import path.
//!
//! The GUI is the only place that normally drives PDF import, so this example calls the
//! same entry points directly and reports peak RSS, which is what we want to watch.
//!
//! Usage:
//!   cargo run -p rnote-engine --release --example pdf_import_bench -- <file.pdf> [vector|bitmap] [max_pages] [scalefactor]

use parry2d_f64::math::Vector2;
use rnote_engine::document::Format;
use rnote_engine::engine::import::{PdfImportPagesType, PdfImportPrefs};
use rnote_engine::image::ImageMemoryFormat;
use rnote_engine::strokes::{BitmapImage, VectorImage};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Peak *allocated* bytes. RSS (VmHWM) is not reliable here: the kernel swaps or compresses
// pages under memory pressure, which makes peak-resident undershoot the real peak.
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_BYTES: AtomicUsize = AtomicUsize::new(0);

struct TrackingAlloc;

unsafe impl GlobalAlloc for TrackingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let live = LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK_BYTES.fetch_max(live, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            if new_size >= layout.size() {
                let grown = new_size - layout.size();
                let live = LIVE_BYTES.fetch_add(grown, Ordering::Relaxed) + grown;
                PEAK_BYTES.fetch_max(live, Ordering::Relaxed);
            } else {
                LIVE_BYTES.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        new_ptr
    }
}

#[global_allocator]
static ALLOC: TrackingAlloc = TrackingAlloc;

fn peak_allocated_mb() -> f64 {
    PEAK_BYTES.load(Ordering::Relaxed) as f64 / 1e6
}

fn live_allocated_mb() -> f64 {
    LIVE_BYTES.load(Ordering::Relaxed) as f64 / 1e6
}
use std::time::Instant;

/// Peak resident set size in kB, from /proc/self/status (same figure `/usr/bin/time -v` reports).
fn peak_rss_kb() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            return rest
                .trim()
                .trim_end_matches("kB")
                .trim()
                .parse()
                .unwrap_or(0);
        }
    }
    0
}

#[allow(dead_code)]
fn current_rss_kb() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            return rest
                .trim()
                .trim_end_matches("kB")
                .trim()
                .parse()
                .unwrap_or(0);
        }
    }
    0
}

/// Total bytes the imported strokes will retain as raw RGBA in the document.
fn retained_bytes(images: &[(u32, u32)]) -> u64 {
    images
        .iter()
        .map(|(w, h)| 4 * (*w as u64) * (*h as u64))
        .sum()
}

/// FNV-1a over the exported pixel data, to prove the import output is unchanged.
fn fnv1a(bytes: &[u8], mut hash: u64) -> u64 {
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: <file.pdf> [vector|bitmap] [max_pages] [scalefactor]"))?;
    let mode = args.next().unwrap_or_else(|| "vector".to_string());
    let max_pages: Option<usize> = args.next().map(|s| s.parse::<usize>().unwrap());
    let scalefactor: Option<f64> = args.next().map(|s| s.parse::<f64>().unwrap());

    let bytes = std::fs::read(&path)?;
    let format = Format::default();

    let mut prefs = PdfImportPrefs::default();
    prefs.pages_type = match mode.as_str() {
        "bitmap" => PdfImportPagesType::Bitmap,
        "vector" => PdfImportPagesType::Vector,
        other => return Err(anyhow::anyhow!("unknown mode '{other}'")),
    };
    if let Some(sf) = scalefactor {
        prefs.bitmap_scalefactor = sf;
    }

    let started = Instant::now();

    // Import every page (page_range = None means "all pages", the default behaviour).
    let mut pixel_hash: u64 = 0xcbf2_9ce4_8422_2325;
    let (dimensions, retained): (Vec<(u32, u32)>, u64) = match prefs.pages_type {
        PdfImportPagesType::Bitmap => {
            let strokes = BitmapImage::from_pdf_bytes(
                &bytes,
                prefs,
                Vector2::ZERO,
                max_pages.map(|n| 0..n),
                &format,
                None,
            )?;
            let dims: Vec<(u32, u32)> = strokes
                .iter()
                .map(|s| (s.image.pixel_width, s.image.pixel_height))
                .collect();
            for s in strokes.iter() {
                pixel_hash = fnv1a(&s.image.data, pixel_hash);
            }
            let retained = retained_bytes(&dims);
            (dims, retained)
        }
        PdfImportPagesType::Vector => {
            let strokes = VectorImage::from_pdf_bytes(
                &bytes,
                prefs,
                Vector2::ZERO,
                max_pages.map(|n| 0..n),
                &format,
                None,
            )?;
            // What the document keeps resident is the SVG text itself.
            for s in strokes.iter() {
                pixel_hash = fnv1a(s.svg_data.as_bytes(), pixel_hash);
            }
            let svg_bytes: u64 = strokes.iter().map(|s| s.svg_data.len() as u64).sum();
            (Vec::new(), svg_bytes)
        }
    };

    let elapsed = started.elapsed();

    println!("file            {} ({:.1} MB)", path, bytes.len() as f64 / 1e6);
    println!(
        "mode            {} (scalefactor {})",
        mode,
        prefs.bitmap_scalefactor
    );
    if !dimensions.is_empty() {
        if let Some((w, h)) = dimensions.first() {
            println!("pages imported  {} ({w} x {h} px each)", dimensions.len());
        }
    } else {
        println!("pages imported  (vector mode)");
    }
    println!("retained        {:.1} MB", retained as f64 / 1e6);
    println!("\n-- peak memory --");
    println!("PEAK ALLOCATED  {:.1} MB", peak_allocated_mb());
    println!("live allocated  {:.1} MB", live_allocated_mb());
    println!(
        "peak / retained {:.2}x",
        peak_allocated_mb() / (retained as f64 / 1e6).max(1.0)
    );
    println!("peak RSS (HWM)  {:.1} MB", peak_rss_kb() as f64 / 1024.0);
    println!("elapsed         {:.2} s", elapsed.as_secs_f64());
    println!("pixel hash      {pixel_hash:016x}");
    println!("memory format   {:?}", ImageMemoryFormat::default());

    Ok(())
}
