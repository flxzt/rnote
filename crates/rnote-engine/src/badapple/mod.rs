use crate::engine::EngineTaskSender;
// Imports
use crate::store::StrokeKey;
use crate::strokes::{ShapeStroke, Stroke};
use crate::tasks::{PeriodicTaskHandle, PeriodicTaskResult};
use crate::StrokeStore;
use rnote_compose::shapes::Polyline;
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::{Shape, Style};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;

const FRAMES_N: usize = 5258;
/// 24 FPS
const FRAMETIME: Duration = Duration::from_micros(41_666);

pub(crate) struct BadApplePlayer {
    scale: f64,
    offset: na::Vector2<f64>,
    frame: usize,
    current_keys: Vec<StrokeKey>,
    task: Option<PeriodicTaskHandle>,
    pkg_data_dir: PathBuf,
    task_sender: EngineTaskSender,
    #[allow(unused)]
    audio_stream: rodio::OutputStream,
    audio_stream_handle: rodio::OutputStreamHandle,
    audio_sink: Option<rodio::Sink>,
}

impl std::fmt::Debug for BadApplePlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BadApplePlayer")
            .field("scale", &self.scale)
            .field("offset", &self.offset)
            .field("frame", &self.frame)
            .field("current_keys", &self.current_keys)
            .field("task", &self.task)
            .field("pkg_data_dir", &self.pkg_data_dir)
            .field("task_sender", &self.task_sender)
            .field("marker_outputstream", &"{.. no debug impl ..}")
            .field("marker_outputstream_handle", &"{.. no debug impl ..}")
            .finish()
    }
}

impl BadApplePlayer {
    pub(crate) fn new(
        scale: f64,
        offset: na::Vector2<f64>,
        pkg_data_dir: PathBuf,
        task_sender: EngineTaskSender,
    ) -> anyhow::Result<Self> {
        let (audio_stream, audio_stream_handle) = rodio::OutputStream::try_default()?;
        Ok(Self {
            scale,
            offset,
            frame: 1,
            pkg_data_dir,
            current_keys: Vec::new(),
            task: None,
            task_sender,
            audio_stream,
            audio_stream_handle,
            audio_sink: None,
        })
    }

    pub(crate) fn stroke_keys(&self) -> Vec<StrokeKey> {
        self.current_keys.clone()
    }

    pub(crate) fn play(&mut self) -> anyhow::Result<()> {
        let sender = self.task_sender.clone();
        let task = PeriodicTaskHandle::new(
            move || {
                sender.send(crate::engine::EngineTask::AdvanceBadAppleFrame);
                PeriodicTaskResult::Continue
            },
            FRAMETIME,
        );
        let audio_sink = self
            .audio_stream_handle
            .play_once(BufReader::new(File::open(
                self.pkg_data_dir.join("bad-apple/bad-apple-audio.wav"),
            )?))?;
        self.task = Some(task);
        self.audio_sink = Some(audio_sink);
        Ok(())
    }

    pub(crate) fn stop(&mut self, store: &mut StrokeStore) {
        for key in self.current_keys.drain(..) {
            store.remove_stroke(key);
        }
        if let Some(mut task) = self.task.take() {
            if let Err(e) = task.quit() {
                tracing::error!("Failed to quit bad apple task, Err: {e:?}");
            }
        }
        if let Some(audio_sink) = self.audio_sink.take() {
            audio_sink.stop();
        }
    }

    pub(crate) fn advance_frame(&mut self, store: &mut StrokeStore) -> anyhow::Result<bool> {
        if self.frame > FRAMES_N {
            return Ok(false);
        }
        for key in self.current_keys.drain(..) {
            store.remove_stroke(key);
        }
        for stroke in generate_frame_strokes(
            self.pkg_data_dir.join(&PathBuf::from("bad-apple/frames")),
            self.frame,
            self.scale,
            self.offset,
        )? {
            self.current_keys.push(store.insert_stroke(stroke, None));
        }
        self.frame += 1;
        Ok(true)
    }
}

fn generate_frame_strokes(
    frame_dir: PathBuf,
    mut frame_no: usize,
    scale: f64,
    offset: na::Vector2<f64>,
) -> anyhow::Result<Vec<Stroke>> {
    frame_no = frame_no.clamp(1, FRAMES_N);

    let image =
        image::io::Reader::open(&frame_dir.join(PathBuf::from(format!("frame{frame_no}.png"))))?
            .with_guessed_format()?
            .decode()?
            .into_rgba8();
    let image_width = image.width();
    let image_height = image.height();
    let svg = vtracer::convert(
        vtracer::ColorImage {
            pixels: image.into_vec(),
            width: image_width as usize,
            height: image_height as usize,
        },
        vtracer::Config {
            color_mode: vtracer::ColorMode::Binary,
            ..Default::default()
        },
    )
    .map_err(|s| anyhow::anyhow!(s))?;

    let strokes = svg
        .paths
        .into_iter()
        .flat_map(|p| vtracer_path_to_kurbo(p, scale, offset))
        .filter_map(|path| {
            let start = path.first()?;

            let mut polyline = Polyline::new(*start);
            polyline.extend(path);
            Some(Stroke::ShapeStroke(ShapeStroke::new(
                Shape::Polyline(polyline),
                Style::Smooth(SmoothOptions::default()),
            )))
        })
        .collect::<Vec<Stroke>>();

    Ok(strokes)
}

fn vtracer_path_to_kurbo(
    vtracer_path: vtracer::SvgPath,
    scale: f64,
    offset: na::Vector2<f64>,
) -> Vec<Vec<na::Vector2<f64>>> {
    vtracer_path
        .path
        .iter()
        .map(|e| match e {
            visioncortex::CompoundPathElement::PathI32(path) => path
                .to_path_f64()
                .path
                .into_iter()
                .map(|p| na::vector![p.x, p.y] * scale + offset)
                .collect::<Vec<na::Vector2<f64>>>(),
            visioncortex::CompoundPathElement::PathF64(path) => path
                .path
                .iter()
                .map(|p| na::vector![p.x, p.y] * scale + offset)
                .collect::<Vec<na::Vector2<f64>>>(),
            visioncortex::CompoundPathElement::Spline(spline) => spline
                .points
                .iter()
                .map(|p| na::vector![p.x, p.y] * scale + offset)
                .collect::<Vec<na::Vector2<f64>>>(),
        })
        .collect()
}
