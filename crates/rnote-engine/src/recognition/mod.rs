extern crate alloc;
use crate::store::text_comp::TextLine;

use crate::engine::{EngineTask, EngineTaskSender};
use crate::store::StrokeKey;
use crate::tasks::{OneOffTaskError, OneOffTaskHandle};
// 1. Bring the BoundingVolume trait into scope to allow calling `.merged()`
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::math::Vector2;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::error;

// Import Burn components
use burn::backend::NdArray;
use burn::tensor::{Tensor, TensorData};

// Use the pure-Rust NdArray backend (f32 element type)
pub type Backend = NdArray<f32>;

// Include the generated model directly in this module to fix the import error
pub mod my_model {
    // If your file was named student_model.onnx, the output will be student_model.rs
    include!(concat!(env!("OUT_DIR"), "/model/student_model.rs"));
}
use my_model::Model;

#[cfg(debug_assertions)]
pub mod debug_export;

// Expose the new segmentation module
pub mod segmentation;
use segmentation::segment_into_lines;

#[derive(Debug, Clone)]
pub struct RecognitionPoint {
    pub pos: Vector2,
}

#[derive(Debug, Clone)]
pub struct RecognitionStroke {
    pub id: StrokeKey,
    pub points: Vec<RecognitionPoint>,
}

impl RecognitionStroke {
    pub fn bounds(&self) -> Aabb {
        let mut aabb = Aabb::new_invalid();
        for pt in &self.points {
            aabb.take_point(pt.pos);
        }
        aabb
    }
}

pub fn get_strokes_bounds(strokes: &[RecognitionStroke]) -> Option<Aabb> {
    if strokes.is_empty() {
        return None;
    }
    Some(
        strokes
            .iter()
            .fold(Aabb::new_invalid(), |acc, s| acc.merged(&s.bounds())),
    )
}

fn get_global_center(strokes: &[RecognitionStroke]) -> Vector2 {
    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let mut count = 0.0_f64;

    for stroke in strokes {
        for pt in &stroke.points {
            sum_x += pt.pos.x as f64;
            sum_y += pt.pos.y as f64;
            count += 1.0;
        }
    }
    if count == 0.0 {
        return Vector2 { x: 0.0, y: 0.0 };
    }
    Vector2 {
        x: sum_x / count,
        y: sum_y / count,
    }
}

fn rotate_point(pt: &Vector2, origin: &Vector2, angle_rad: f64) -> Vector2 {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    let dx = pt.x as f64 - origin.x as f64;
    let dy = pt.y as f64 - origin.y as f64;

    Vector2 {
        x: origin.x as f64 + (dx * cos_a - dy * sin_a),
        y: origin.y as f64 + (dx * sin_a + dy * cos_a),
    }
}

fn get_stroke_centroid(stroke: &RecognitionStroke) -> Option<Vector2> {
    if stroke.points.is_empty() {
        return None;
    }
    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let count = stroke.points.len() as f64;

    for pt in &stroke.points {
        sum_x += pt.pos.x as f64;
        sum_y += pt.pos.y as f64;
    }
    Some(Vector2 {
        x: sum_x / count,
        y: sum_y / count,
    })
}

pub fn calculate_skew_angle(strokes: &[RecognitionStroke]) -> f64 {
    let mut centroids = Vec::with_capacity(strokes.len());
    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;

    for stroke in strokes {
        if let Some(centroid) = get_stroke_centroid(stroke) {
            sum_x += centroid.x as f64;
            sum_y += centroid.y as f64;
            centroids.push(centroid);
        }
    }

    let n = centroids.len() as f64;
    if n < 2.0 {
        return 0.0;
    }

    let mean_x = sum_x / n;
    let mean_y = sum_y / n;
    let mut numerator = 0.0_f64;
    let mut denominator = 0.0_f64;

    for centroid in &centroids {
        let dx = centroid.x as f64 - mean_x;
        let dy = centroid.y as f64 - mean_y;
        numerator += dx * dy;
        denominator += dx * dx;
    }

    if denominator.abs() < 1e-5 {
        return 0.0;
    }
    (numerator / denominator).atan()
}

pub fn deskew_strokes(strokes: &[RecognitionStroke]) -> Vec<RecognitionStroke> {
    if strokes.is_empty() {
        return vec![];
    }
    let angle_rad = calculate_skew_angle(strokes);
    if angle_rad.abs() < 0.05 {
        return strokes.to_vec();
    }

    let global_center = get_global_center(strokes);
    let rotation_angle = -angle_rad;

    strokes
        .iter()
        .map(|stroke| {
            let rotated_points = stroke
                .points
                .iter()
                .map(|pt| RecognitionPoint {
                    pos: rotate_point(&pt.pos, &global_center, rotation_angle),
                })
                .collect();
            RecognitionStroke {
                id: stroke.id,
                points: rotated_points,
            }
        })
        .collect()
}

fn process_strokes_to_tensor(
    strokes: &[RecognitionStroke],
    max_w: usize,
    max_h: usize,
    padding: f32,
) -> Vec<f32> {
    let bounds = match get_strokes_bounds(strokes) {
        Some(b) => b,
        None => return vec![-1.0; 3 * max_w * max_h],
    };

    let min_x = bounds.mins.x as f32;
    let max_x = bounds.maxs.x as f32;
    let min_y = bounds.mins.y as f32;
    let max_y = bounds.maxs.y as f32;

    let total_points: usize = strokes.iter().map(|s| s.points.len()).sum();

    if total_points < 2 {
        return vec![-1.0; 3 * max_w * max_h];
    }

    let w = (max_x - min_x).max(1e-5);
    let h = (max_y - min_y).max(1e-5);

    let mut scale = (max_h as f32 - 2.0 * padding) / h;
    if w * scale > (max_w as f32 - 2.0 * padding) {
        scale = (max_w as f32 - 2.0 * padding) / w;
    }

    let y_offset = (max_h as f32 - (h * scale)) / 2.0;

    let mut buffer = vec![-1.0f32; 3 * max_w * max_h];

    let mut global_pt_idx = 0;
    let time_range = (total_points as f32 - 1.0).max(1e-5);

    for stroke in strokes {
        if stroke.points.len() < 2 {
            global_pt_idx += stroke.points.len();
            continue;
        }

        for i in 0..stroke.points.len() - 1 {
            let p1 = &stroke.points[i];
            let p2 = &stroke.points[i + 1];

            let x0 = (((p1.pos.x as f32) - min_x) * scale + padding).round() as i32;
            let y0 = (((p1.pos.y as f32) - min_y) * scale + y_offset).round() as i32;
            let x1 = (((p2.pos.x as f32) - min_x) * scale + padding).round() as i32;
            let y1 = (((p2.pos.y as f32) - min_y) * scale + y_offset).round() as i32;

            let dx = (p2.pos.x - p1.pos.x) as f32;
            let dy = (p2.pos.y - p1.pos.y) as f32;
            let angle = dy.atan2(dx);
            let norm_angle = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
            let norm_time = (global_pt_idx as f32) / time_range;

            let v_mask_scaled = 1.0;
            let v_time_scaled = (norm_time - 0.5) * 2.0;
            let v_angle_scaled = (norm_angle - 0.5) * 2.0;

            draw_line(
                &mut buffer,
                max_w,
                max_h,
                x0,
                y0,
                x1,
                y1,
                v_mask_scaled,
                v_time_scaled,
                v_angle_scaled,
            );
            global_pt_idx += 1;
        }
        global_pt_idx += 1;
    }

    buffer
}

fn draw_line(
    buf: &mut [f32],
    w: usize,
    h: usize,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    v_mask: f32,
    v_time: f32,
    v_angle: f32,
) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    let area = w * h;
    loop {
        if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
            let idx = (y as usize) * w + (x as usize);
            buf[idx] = v_mask;
            buf[area + idx] = v_time;
            buf[2 * area + idx] = v_angle;
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

#[derive(Clone)]
pub struct HandwritingRecognizer {
    task_handle: Arc<Mutex<Option<OneOffTaskHandle>>>,
    tasks_tx: EngineTaskSender,
    model: Arc<Model<Backend>>,
}

// Manually implement Debug to satisfy the Engine struct deriving Debug
// without depending on the inner traits.
impl std::fmt::Debug for HandwritingRecognizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandwritingRecognizer")
            .finish_non_exhaustive()
    }
}

impl HandwritingRecognizer {
    pub fn new(tasks_tx: EngineTaskSender) -> Self {
        // Initialize the built-in model
        let device = Default::default();
        let model: Model<Backend> = Model::new(&device);

        Self {
            task_handle: Arc::new(Mutex::new(None)),
            tasks_tx,
            model: Arc::new(model),
        }
    }

    pub fn trigger_recognition_debounced(
        &self,
        raw_stroke_data: Vec<RecognitionStroke>,
        existing_text: Vec<TextLine>,
        handwriting_debounce_ms: u32,
    ) {
        let timeout = Duration::from_millis(u64::from(handwriting_debounce_ms));
        let mut reinstall_task = false;

        let tasks_tx = self.tasks_tx.clone();
        let model = self.model.clone();

        let recognition_task = move || {
            if raw_stroke_data.is_empty() {
                return;
            }

            let lines = segment_into_lines(&raw_stroke_data, 0.0);
            let mut recognized_lines: Vec<TextLine> = Vec::new();

            let max_w = 512;
            let max_h = 32;

            // Reconstruct NdArray backend device directly here in the worker thread
            let device = Default::default();

            for (line_index, line_strokes) in lines.into_iter().enumerate() {
                if line_strokes.is_empty() {
                    continue;
                }

                let mut current_stroke_ids: Vec<StrokeKey> =
                    line_strokes.iter().map(|s| s.id).collect();
                current_stroke_ids.sort_unstable();

                let already_recognized = existing_text
                    .iter()
                    .find(|old_line| old_line.stroke_keys == current_stroke_ids);

                if let Some(reusable_line) = already_recognized {
                    recognized_lines.push(reusable_line.clone());
                    continue;
                }

                let original_bounds =
                    get_strokes_bounds(&line_strokes).unwrap_or_else(|| Aabb::new_invalid());

                let buffer = process_strokes_to_tensor(&line_strokes, max_w, max_h, 2.0);

                // Build pure-Rust Burn tensor
                let tensor_data = TensorData::new(buffer, [1, 3, max_h, max_w]);
                let input_tensor = Tensor::<Backend, 4>::from_data(tensor_data, &device);

                // Run forward pass
                let output_tensor = model.forward(input_tensor);

                // Extract Burn tensor data
                let output_data = output_tensor.into_data();

                // 2. Add an explicit type mapping matching the 3-dimensional tensor extraction
                let dims: [usize; 3] = output_data.shape.dims();
                let data = match output_data.to_vec::<f32>() {
                    Ok(d) => d,
                    Err(e) => {
                        error!("Failed to extract tensor for line {}: {}", line_index, e);
                        continue;
                    }
                };

                let recognized_text: Option<String> = {
                    if dims.len() >= 3 {
                        let time_steps = dims[1];
                        let vocab_size = dims[2];

                        let vocab: &[char] = &[
                            '\0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
                            'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'ä',
                            'ö', 'ü', 'ß', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
                            'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y',
                            'Z', 'Ä', 'Ö', 'Ü', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
                            '.', ',', ':', ';', '!', '?', '-', '"', '(', ')', '[', ']', '+', '*',
                            '/', '\\', '=', '<', '>', '^', '_', '%', '°', '$', '€', '@', '#', '&',
                            '§', ' ',
                        ];

                        let mut decoded_chars = Vec::new();
                        let mut last_token = None;

                        for t in 0..time_steps {
                            let start_idx = t * vocab_size;
                            let logits_slice = &data[start_idx..start_idx + vocab_size];

                            let mut max_idx = 0;
                            let mut max_val = logits_slice[0];
                            for (vocab_idx, &val) in logits_slice.iter().enumerate().skip(1) {
                                if val > max_val {
                                    max_val = val;
                                    max_idx = vocab_idx;
                                }
                            }

                            if max_idx != 0 && Some(max_idx) != last_token {
                                if let Some(&ch) = vocab.get(max_idx) {
                                    decoded_chars.push(ch);
                                }
                            }
                            last_token = Some(max_idx);
                        }

                        Some(decoded_chars.into_iter().collect())
                    } else {
                        None
                    }
                };

                if let Some(recognized_text) = recognized_text {
                    recognized_lines.push(TextLine {
                        text: recognized_text,
                        bounds: original_bounds,
                        stroke_keys: current_stroke_ids,
                    });
                }
            }

            if !recognized_lines.is_empty() {
                tasks_tx.send(EngineTask::HandwritingRecognitionResult {
                    lines: recognized_lines,
                });
            };
        };

        let mut handle_lock = self.task_handle.lock().unwrap();
        if let Some(handle) = handle_lock.as_mut() {
            match handle.replace_task(recognition_task.clone()) {
                Ok(()) => {}
                Err(OneOffTaskError::TimeoutReached) => reinstall_task = true,
                Err(e) => {
                    error!("Could not replace task for handwriting recognition, Err: {e:?}");
                    reinstall_task = true;
                }
            }
        } else {
            reinstall_task = true;
        }

        if reinstall_task {
            *handle_lock = Some(OneOffTaskHandle::new(recognition_task, timeout));
        }
    }
}
