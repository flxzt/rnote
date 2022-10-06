use std::collections::HashMap;
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

    // we need to hold the output streams too
    marker_outputstream: rodio::OutputStream,
    marker_outputstream_handle: rodio::OutputStreamHandle,
    brush_outputstream: rodio::OutputStream,
    brush_outputstream_handle: rodio::OutputStreamHandle,
    typewriter_outputstream: rodio::OutputStream,
    typewriter_outputstream_handle: rodio::OutputStreamHandle,

    sounds: HashMap<String, rodio::source::Buffered<Decoder<File>>>,

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

        let mut sounds = HashMap::new();

        let load_sound_from_path =
            |sounds: &mut HashMap<String, rodio::source::Buffered<Decoder<File>>>,
             mut resource_path: PathBuf,
             sound_name: String,
             ending: &str|
             -> anyhow::Result<()> {
                resource_path.push(format!("{sound_name}.{ending}"));

                if resource_path.exists() {
                    let buffered =
                        rodio::Decoder::new(File::open(resource_path.clone()).with_context(
                            || anyhow::anyhow!("file open() for path {:?} failed", resource_path,),
                        )?)?
                        .buffered();

                    // Making sure buffer is initialized
                    buffered.clone().for_each(|_| {});

                    sounds.insert(sound_name, buffered);
                } else {
                    return Err(anyhow::Error::msg(format!(
                        "failed to init audioplayer. File `{:?}` is missing.",
                        resource_path
                    )));
                }
                Ok(())
            };

        // Init marker sounds
        for i in 0..Self::MARKER_N_FILES {
            load_sound_from_path(
                &mut sounds,
                data_dir.clone(),
                format!("marker_{:02}", i),
                "wav",
            )?;
        }

        // Init brush sounds
        load_sound_from_path(&mut sounds, data_dir.clone(), format!("brush"), "wav")?;

        // Init typewriter sounds
        // the enumerated key sounds
        for i in 0..Self::TYPEWRITER_N_FILES {
            load_sound_from_path(
                &mut sounds,
                data_dir.clone(),
                format!("typewriter_{:02}", i),
                "wav",
            )?;
        }

        // the custom sounds
        load_sound_from_path(
            &mut sounds,
            data_dir.clone(),
            format!("typewriter_insert"),
            "wav",
        )?;

        load_sound_from_path(
            &mut sounds,
            data_dir.clone(),
            format!("typewriter_thump"),
            "wav",
        )?;

        load_sound_from_path(
            &mut sounds,
            data_dir.clone(),
            format!("typewriter_bell"),
            "wav",
        )?;

        load_sound_from_path(
            &mut sounds,
            data_dir.clone(),
            format!("typewriter_linefeed"),
            "wav",
        )?;

        let (brush_outputstream, brush_outputstream_handle) = rodio::OutputStream::try_default()?;
        let (marker_outputstream, marker_outputstream_handle) = rodio::OutputStream::try_default()?;
        let (typewriter_outputstream, typewriter_outputstream_handle) =
            rodio::OutputStream::try_default()?;

        Ok(Self {
            enabled: true,

            marker_outputstream,
            marker_outputstream_handle,
            brush_outputstream,
            brush_outputstream_handle,
            typewriter_outputstream,
            typewriter_outputstream_handle,

            sounds,

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
                sink.append(self.sounds[&format!("marker_{:02}", marker_sound_index)].clone());
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
                sink.append(
                    self.sounds["brush"]
                        .clone()
                        .repeat_infinite()
                        .skip_duration(Duration::from_millis(
                            (Self::BRUSH_SEEK_TIMES_SEC[brush_sound_seek_time_index] * 1000.0)
                                .round() as u64,
                        )),
                );

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

    /// Play a typewriter sound that fits the given key type, or a generic sound when None
    pub fn play_typewriter_key_sound(&self, keyboard_key: Option<KeyboardKey>) {
        if !self.enabled {
            return;
        }

        match rodio::Sink::try_new(&self.typewriter_outputstream_handle) {
            Ok(sink) => match keyboard_key {
                Some(KeyboardKey::CarriageReturn) | Some(KeyboardKey::Linefeed) => {
                    sink.append(
                        self.sounds["typewriter_bell"].clone().mix(
                            self.sounds["typewriter_linefeed"]
                                .clone()
                                .delay(Duration::from_millis(200)),
                        ),
                    );
                    sink.detach();
                }
                // control characters are already filtered out of unicode
                Some(KeyboardKey::Unicode(_))
                | Some(KeyboardKey::BackSpace)
                | Some(KeyboardKey::Delete)
                | Some(KeyboardKey::HorizontalTab)
                | None => {
                    let mut rng = rand::thread_rng();
                    let typewriter_sound_index = rng.gen_range(0..Self::TYPEWRITER_N_FILES);

                    sink.append(
                        self.sounds[&format!("typewriter_{:02}", typewriter_sound_index)].clone(),
                    );
                    sink.detach();
                }
                _ => {
                    sink.append(self.sounds["typewriter_thump"].clone());
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
