use std::fs::File;
use std::path::PathBuf;
use std::time::{self, Duration};

use anyhow::Context;
use rand::Rng;
use rnote_compose::penhelpers::KeyboardKey;
use rodio::{Decoder, Source};

/// The audio player for pen sounds
#[allow(missing_debug_implementations, dead_code)]
pub struct AudioPlayer {
    /// enables / disables the player
    pub(super) enabled: bool,

    marker_sounds: Vec<rodio::source::Buffered<Decoder<File>>>,
    // we need to hold the outputstreams
    marker_outputstream: rodio::OutputStream,
    marker_outputstream_handle: rodio::OutputStreamHandle,

    brush_sound: rodio::source::Buffered<Decoder<File>>,
    // we need to hold the outputstreams
    brush_outputstream: rodio::OutputStream,
    brush_outputstream_handle: rodio::OutputStreamHandle,

    typewriter_key_sounds: Vec<rodio::source::Buffered<Decoder<File>>>,
    typewriter_insert_sound: rodio::source::Buffered<Decoder<File>>,
    typewriter_thump_sound: rodio::source::Buffered<Decoder<File>>,
    typewriter_bell_sound: rodio::source::Buffered<Decoder<File>>,
    typewriter_linefeed_sound: rodio::source::Buffered<Decoder<File>>,
    // we need to hold the outputstreams
    typewriter_outputstream: rodio::OutputStream,
    typewriter_outputstream_handle: rodio::OutputStreamHandle,

    brush_sink: Option<rodio::Sink>,
}

impl AudioPlayer {
    pub const PLAY_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(500);
    /// Number of marker sound files installed in system-data-dir/rnote/sounds
    pub const MARKER_N_FILES: usize = 15;

    pub const BRUSH_SEEK_TIMES_SEC: [f64; 5] = [0.0, 0.91, 4.129, 6.0, 8.56];

    pub const TYPEWRITER_N_FILES: usize = 30;

    /// A new audioplayer for the given data dir.
    pub fn new(mut data_dir: PathBuf) -> Result<Self, anyhow::Error> {
        data_dir.push("sounds/");

        // Init marker sounds
        let marker_sounds = {
            let mut marker_sounds = Vec::with_capacity(Self::MARKER_N_FILES);

            for i in 0..Self::MARKER_N_FILES {
                let mut file_path = data_dir.clone();
                file_path.push(format!("marker_{:02}.wav", i));

                if file_path.exists() {
                    let buffered =
                        rodio::Decoder::new(File::open(file_path.clone()).with_context(|| {
                            anyhow::anyhow!("file open() for path {:?} failed", file_path,)
                        })?)?
                        .buffered();
                    // Making sure buffer is initialized
                    buffered.clone().for_each(|_| {});

                    marker_sounds.push(buffered);
                } else {
                    return Err(anyhow::Error::msg(format!(
                        "failed to init audioplayer. File `{:?}` is missing.",
                        file_path
                    )));
                }
            }

            marker_sounds
        };

        // Init brush sounds
        let brush_sound = {
            let mut file_path = data_dir.clone();
            file_path.push("brush.wav");
            let buffered =
                rodio::Decoder::new(File::open(file_path.clone()).with_context(|| {
                    anyhow::anyhow!("file open() for path {:?} failed", file_path,)
                })?)?
                .buffered();
            buffered.clone().for_each(|_| {});

            buffered
        };

        // Init typewriter sounds
        // First the enumerated key sounds
        let typewriter_key_sounds = {
            let mut typewriter_key_sounds = Vec::with_capacity(Self::TYPEWRITER_N_FILES);

            for i in 0..Self::TYPEWRITER_N_FILES {
                let mut file_path = data_dir.clone();
                file_path.push(format!("typewriter_{:02}.wav", i));

                if file_path.exists() {
                    let buffered =
                        rodio::Decoder::new(File::open(file_path.clone()).with_context(|| {
                            anyhow::anyhow!("file open() for path {:?} failed", file_path,)
                        })?)?
                        .buffered();
                    buffered.clone().for_each(|_| {});

                    typewriter_key_sounds.push(buffered);
                } else {
                    return Err(anyhow::Error::msg(format!(
                        "failed to init audioplayer. File `{:?}` is missing.",
                        file_path
                    )));
                }
            }

            typewriter_key_sounds
        };

        // then the custom sounds
        let typewriter_insert_sound = {
            let mut file_path = data_dir.clone();
            file_path.push("typewriter_insert.wav");
            let buffered =
                rodio::Decoder::new(File::open(file_path.clone()).with_context(|| {
                    anyhow::anyhow!("file open() for path {:?} failed", file_path,)
                })?)?
                .buffered();
            buffered.clone().for_each(|_| {});

            buffered
        };

        let typewriter_thump_sound = {
            let mut file_path = data_dir.clone();
            file_path.push("typewriter_thump.wav");
            let buffered =
                rodio::Decoder::new(File::open(file_path.clone()).with_context(|| {
                    anyhow::anyhow!("file open() for path {:?} failed", file_path,)
                })?)?
                .buffered();
            buffered.clone().for_each(|_| {});

            buffered
        };

        let typewriter_bell_sound = {
            let mut file_path = data_dir.clone();
            file_path.push("typewriter_bell.wav");
            let buffered =
                rodio::Decoder::new(File::open(file_path.clone()).with_context(|| {
                    anyhow::anyhow!("file open() for path {:?} failed", file_path,)
                })?)?
                .buffered();
            buffered.clone().for_each(|_| {});

            buffered
        };

        let typewriter_linefeed_sound = {
            let mut file_path = data_dir.clone();
            file_path.push("typewriter_linefeed.wav");
            let buffered =
                rodio::Decoder::new(File::open(file_path.clone()).with_context(|| {
                    anyhow::anyhow!("file open() for path {:?} failed", file_path,)
                })?)?
                .buffered();
            buffered.clone().for_each(|_| {});

            buffered
        };

        let (brush_outputstream, brush_outputstream_handle) = rodio::OutputStream::try_default()?;
        let (marker_outputstream, marker_outputstream_handle) = rodio::OutputStream::try_default()?;
        let (typewriter_outputstream, typewriter_outputstream_handle) =
            rodio::OutputStream::try_default()?;

        Ok(Self {
            enabled: true,

            marker_sounds,
            marker_outputstream,
            marker_outputstream_handle,

            brush_sound,
            brush_outputstream,
            brush_outputstream_handle,

            typewriter_key_sounds,
            typewriter_insert_sound,
            typewriter_thump_sound,
            typewriter_bell_sound,
            typewriter_linefeed_sound,
            typewriter_outputstream,
            typewriter_outputstream_handle,

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

    pub fn play_typewriter_key_sound(&self, keyboard_key: KeyboardKey) {
        if !self.enabled {
            return;
        }

        match rodio::Sink::try_new(&self.typewriter_outputstream_handle) {
            Ok(sink) => match keyboard_key {
                KeyboardKey::Linefeed => {
                    sink.append(
                        self.typewriter_bell_sound.clone().mix(
                            self.typewriter_linefeed_sound
                                .clone()
                                .delay(Duration::from_millis(200)),
                        ),
                    );
                    sink.detach();
                }
                // control characters are already filtered out of unicode
                KeyboardKey::Unicode(_)
                | KeyboardKey::BackSpace
                | KeyboardKey::Delete
                | KeyboardKey::HorizontalTab => {
                    let mut rng = rand::thread_rng();
                    let typewriter_sound_index = rng.gen_range(0..Self::TYPEWRITER_N_FILES);

                    sink.append(self.typewriter_key_sounds[typewriter_sound_index].clone());
                    sink.detach();
                }
                _ => {
                    sink.append(self.typewriter_thump_sound.clone());
                    sink.detach();
                }
            },
            Err(e) => log::error!(
                "failed to create sink in play_typewriter_sound(), Err {}",
                e
            ),
        }
    }
}
