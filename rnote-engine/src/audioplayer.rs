use std::fs::File;
use std::time::{self, Duration};

use anyhow::Context;
use gtk4::glib;
use rand::Rng;
use rodio::{Decoder, Source};

/// The audio player for pen sounds
#[allow(missing_debug_implementations, dead_code)]
pub struct AudioPlayer {
    /// enables / disables the player
    pub(super) enabled: bool,
    marker_sounds: Vec<rodio::source::Buffered<Decoder<File>>>,
    brush_sound: rodio::source::Buffered<Decoder<File>>,
    // we need to hold the outputstreams
    marker_outputstream: rodio::OutputStream,
    brush_outputstream: rodio::OutputStream,
    marker_outputstream_handle: rodio::OutputStreamHandle,
    brush_outputstream_handle: rodio::OutputStreamHandle,

    brush_sink: Option<rodio::Sink>,
}

impl AudioPlayer {
    pub const PLAY_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(500);
    /// Number of marker sound files installed in system-data-dir/rnote/sounds
    pub const MARKER_N_FILES: usize = 15;
    pub const BRUSH_SEEK_TIMES_SEC: [f64; 5] = [0.0, 0.91, 4.129, 6.0, 8.56];

    pub fn new() -> Result<Self, anyhow::Error> {
        let system_data_dirs = glib::system_data_dirs();
        let mut marker_sounds = vec![];
        let mut brush_sound = None;

        for mut path in system_data_dirs.clone() {
            path.push("rnote/sounds/");
            if path.exists() {
                // Init marker sounds
                for i in 0..Self::MARKER_N_FILES {
                    let mut marker_path = path.clone();
                    marker_path.push(format!("marker_{:02}.wav", i));

                    if marker_path.exists() {
                        let buffered = rodio::Decoder::new(
                            File::open(marker_path.clone()).with_context(|| {
                                anyhow::anyhow!("file open() for path {:?} failed", marker_path,)
                            })?,
                        )?
                        .buffered();
                        // Making sure buffer is initialized
                        buffered.clone().for_each(|_| {});

                        marker_sounds.push(buffered);
                    } else {
                        return Err(anyhow::Error::msg(format!(
                            "failed to init audioplayer. File `{:?}` is missing.",
                            marker_path
                        )));
                    }
                }

                // Init brush sounds
                let mut brush_path = path.clone();
                brush_path.push("brush.wav");
                if path.exists() {
                    let buffered =
                        rodio::Decoder::new(File::open(brush_path.clone()).with_context(
                            || anyhow::anyhow!("file open() for path {:?} failed", brush_path,),
                        )?)?
                        .buffered();
                    // Making sure buffer is initialized
                    buffered.clone().for_each(|_| {});

                    brush_sound = Some(buffered);
                }

                break;
            }
        }
        let brush_sound = match brush_sound {
            None => {
                return Err(anyhow::anyhow!("failed to initialize brush sound"));
            }
            Some(brush_sound) => brush_sound,
        };

        let (brush_outputstream, brush_outputstream_handle) = rodio::OutputStream::try_default()?;
        let (marker_outputstream, marker_outputstream_handle) = rodio::OutputStream::try_default()?;

        Ok(Self {
            enabled: true,
            marker_sounds,
            brush_sound,
            marker_outputstream,
            brush_outputstream,
            brush_outputstream_handle,
            marker_outputstream_handle,
            brush_sink: None,
        })
    }

    pub fn play_random_marker_sound(&self) {
        if !self.enabled {
            return;
        }

        let mut rng = rand::thread_rng();
        let marker_sound_index = rng.gen_range(0..Self::MARKER_N_FILES);

        match rodio::Sink::try_new(&self.marker_outputstream_handle) {
            Ok(sink) => {
                sink.append(self.marker_sounds[marker_sound_index].clone());
                sink.detach();
            }
            Err(e) => log::error!(
                "failed to create sink in play_random_marker_sound(), Err {}",
                e
            ),
        }
    }

    pub fn start_random_brush_sound(&mut self) {
        if !self.enabled {
            return;
        }

        let mut rng = rand::thread_rng();
        let brush_sound_seek_time_index = rng.gen_range(0..Self::BRUSH_SEEK_TIMES_SEC.len());

        match rodio::Sink::try_new(&self.brush_outputstream_handle) {
            Ok(sink) => {
                sink.append(self.brush_sound.clone().repeat_infinite().skip_duration(
                    Duration::from_millis(
                        (Self::BRUSH_SEEK_TIMES_SEC[brush_sound_seek_time_index] * 1000.0).round()
                            as u64,
                    ),
                ));

                self.brush_sink = Some(sink);
            }
            Err(e) => log::error!(
                "failed to create sink in start_play_random_brush_sound(), Err {}",
                e
            ),
        }
    }

    pub fn stop_random_brush_sond(&mut self) {
        if let Some(brush_sink) = self.brush_sink.take() {
            brush_sink.stop();
        }
    }
}
