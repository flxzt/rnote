// In src/debug_export.rs

// This ensures the entire file is only compiled in debug mode
#![cfg(debug_assertions)]

use serde::Serialize;
use std::fmt::Write;
use std::fs::File;
use std::io::BufWriter;

// You will need to import your crate's specific types here
use crate::recognition::RecognitionStroke;

#[derive(Serialize)]
pub struct AnnotationRecord {
    index: usize,
    text: String,
    strokes: Vec<Vec<(f64, f64, f64)>>,
}

pub fn export_for_annotation(
    segmented_lines: &[Vec<RecognitionStroke>],
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut records = Vec::with_capacity(segmented_lines.len());

    for (line_index, line_strokes) in segmented_lines.iter().enumerate() {
        let mut out_strokes = Vec::with_capacity(line_strokes.len());

        for stroke in line_strokes {
            let mut out_points = Vec::with_capacity(stroke.points.len());

            for pt in &stroke.points {
                let x = pt.pos.x as f64;
                let y = pt.pos.y as f64;
                let t = 0.0;
                out_points.push((x, y, t));
            }
            out_strokes.push(out_points);
        }

        records.push(AnnotationRecord {
            index: line_index,
            text: String::new(),
            strokes: out_strokes,
        });
    }

    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &records)?;

    Ok(())
}

pub fn export_debug_svg(lines: &[Vec<RecognitionStroke>], file_path: &str) {
    if lines.is_empty() {
        return;
    }

    // Find absolute boundaries for the canvas
    let mut global_min_x = f64::MAX;
    let mut global_min_y = f64::MAX;
    let mut global_max_x = f64::MIN;
    let mut global_max_y = f64::MIN;

    for line in lines {
        for stroke in line {
            let b = stroke.bounds();
            global_min_x = global_min_x.min(b.mins.x);
            global_min_y = global_min_y.min(b.mins.y);
            global_max_x = global_max_x.max(b.maxs.x);
            global_max_y = global_max_y.max(b.maxs.y);
        }
    }

    // Add 50px padding around the document
    let width = global_max_x - global_min_x + 100.0;
    let height = global_max_y - global_min_y + 100.0;

    let mut svg = String::new();
    let _ = writeln!(
        &mut svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" style="background-color: #1e1e1e;">"#,
        global_min_x - 50.0,
        global_min_y - 50.0,
        width,
        height
    );

    // Bounding box colors to easily see how the algorithm grouped the lines
    let box_colors = [
        "#ff5555", "#50fa7b", "#f1fa8c", "#bd93f9", "#ff79c6", "#8be9fd",
    ];

    for (i, line) in lines.iter().enumerate() {
        let box_color = box_colors[i % box_colors.len()];

        // Calculate Line Bounding Box
        let mut line_min_x = f64::MAX;
        let mut line_min_y = f64::MAX;
        let mut line_max_x = f64::MIN;
        let mut line_max_y = f64::MIN;

        for stroke in line {
            let b = stroke.bounds();
            line_min_x = line_min_x.min(b.mins.x);
            line_min_y = line_min_y.min(b.mins.y);
            line_max_x = line_max_x.max(b.maxs.x);
            line_max_y = line_max_y.max(b.maxs.y);
        }

        // Draw the background bounding box to verify segmentation
        let box_w = line_max_x - line_min_x;
        let box_h = line_max_y - line_min_y;
        let _ = writeln!(
            &mut svg,
            r#"  <rect x="{}" y="{}" width="{}" height="{}" fill="{}" fill-opacity="0.05" stroke="{}" stroke-width="2" stroke-dasharray="5,5"/>"#,
            line_min_x, line_min_y, box_w, box_h, box_color, box_color
        );

        let num_strokes = line.len();

        // Draw strokes with Time-Coloring (Blue -> Green -> Red)
        for (j, stroke) in line.iter().enumerate() {
            if stroke.points.is_empty() {
                continue;
            }

            // Calculate progress from 0.0 (First stroke) to 1.0 (Last stroke)
            let progress = if num_strokes > 1 {
                j as f64 / (num_strokes as f64 - 1.0)
            } else {
                0.5 // Default to green if there's only one stroke
            };

            // Hue shifts from 240 (Blue) down to 0 (Red)
            let hue = 240.0 * (1.0 - progress);
            let time_color = format!("hsl({}, 100%, 65%)", hue);

            let mut path_data = String::new();
            let _ = write!(
                &mut path_data,
                "M {} {} ",
                stroke.points[0].pos.x, stroke.points[0].pos.y
            );

            for pt in stroke.points.iter().skip(1) {
                let _ = write!(&mut path_data, "L {} {} ", pt.pos.x, pt.pos.y);
            }

            // Render the stroke path
            let _ = writeln!(
                &mut svg,
                r#"  <path d="{}" fill="none" stroke="{}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>"#,
                path_data, time_color
            );

            // Render a small white dot at the start of each stroke to visualize drawing direction
            let _ = writeln!(
                &mut svg,
                r#"  <circle cx="{}" cy="{}" r="2.5" fill='#ffffff' opacity="0.8"/>"#,
                stroke.points[0].pos.x, stroke.points[0].pos.y
            );
        }
    }

    let _ = writeln!(&mut svg, "</svg>");

    // Save to disk
    if let Err(e) = std::fs::write(file_path, svg) {
        tracing::error!("Failed to write debug SVG: {}", e);
    } else {
        tracing::info!("Wrote segmentation debug image to {}", file_path);
    }
}
