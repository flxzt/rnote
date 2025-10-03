// Imports
use anyhow::Context;
use rand::Rng;
use rnote_compose::penevent::KeyboardKey;
use rodio::source::Buffered;
use rodio::{Decoder, Source};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use tracing::error;

/// The audio player for pen sounds.
pub struct AudioPlayer {
    marker_outputstream: rodio::OutputStream,
    brush_outputstream: rodio::OutputStream,
    typewriter_outputstream: rodio::OutputStream,

    sounds: HashMap<String, Buffered<Decoder<File>>>,
    brush_sound_task_handle: Option<crate::tasks::OneOffTaskHandle>,
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("marker_outputstream", &"{.. no debug impl ..}")
            .field("brush_outputstream", &"{.. no debug impl ..}")
            .field("typewriter_outputstream", &"{.. no debug impl ..}")
            .field("sounds", &"{.. no debug impl ..}")
            .field("brush_sound_task_handle", &self.brush_sound_task_handle)
            .finish()
    }
}

impl AudioPlayer {
    pub const BRUSH_SOUND_TIMEOUT: Duration = Duration::from_millis(600);
    pub const N_SOUND_FILES_MARKER: usize = 15;
    pub const N_SOUND_FILES_TYPEWRITER: usize = 30;
    pub const SOUND_FILE_BRUSH_SEEK_TIMES_MS: [f64; 5] = [0., 910., 4129., 6000., 8560.];

    /// Create and initialize new audioplayer.
    /// `pkg_data_dir` is the app data directory which has a "sounds" subfolder containing the sound files
    pub fn new_init(mut pkg_data_dir: PathBuf) -> Result<Self, anyhow::Error> {
        pkg_data_dir.push("sounds/");

        let mut sounds = HashMap::new();

        let brush_outputstream = rodio::OutputStreamBuilder::open_default_stream()?;
        let marker_outputstream = rodio::OutputStreamBuilder::open_default_stream()?;
        let typewriter_outputstream = rodio::OutputStreamBuilder::open_default_stream()?;

        // Init marker sounds
        for i in 0..Self::N_SOUND_FILES_MARKER {
            let name = format!("marker_{i:02}");
            let buffered = load_sound_from_path(pkg_data_dir.clone(), &name, "wav")?;

            sounds.insert(name, buffered);
        }

        // Init brush sounds
        {
            let name = String::from("brush");
            let buffered = load_sound_from_path(pkg_data_dir.clone(), &name, "wav")?;
            sounds.insert(name, buffered);
        }

        // Init typewriter sounds
        // the enumerated key sounds
        for i in 0..Self::N_SOUND_FILES_TYPEWRITER {
            let name = format!("typewriter_{i:02}");
            let buffered = load_sound_from_path(pkg_data_dir.clone(), &name, "wav")?;
            sounds.insert(name, buffered);
        }

        // the custom sounds
        {
            let name = String::from("typewriter_insert");
            let buffered = load_sound_from_path(pkg_data_dir.clone(), &name, "wav")?;
            sounds.insert(name, buffered);
        }

        {
            let name = String::from("typewriter_thump");
            let buffered = load_sound_from_path(pkg_data_dir.clone(), &name, "wav")?;
            sounds.insert(name, buffered);
        }

        {
            let name = String::from("typewriter_bell");
            let buffered = load_sound_from_path(pkg_data_dir.clone(), &name, "wav")?;
            sounds.insert(name, buffered);
        }

        {
            let name = String::from("typewriter_linefeed");
            let buffered = load_sound_from_path(pkg_data_dir.clone(), &name, "wav")?;
            sounds.insert(name, buffered);
        }

        Ok(Self {
            marker_outputstream,
            brush_outputstream,
            typewriter_outputstream,

            sounds,
            brush_sound_task_handle: None,
        })
    }

    pub fn play_random_marker_sound(&self) {
        let mut rng = rand::rng();
        let marker_sound_index = rng.random_range(0..Self::N_SOUND_FILES_MARKER);

        let sink = rodio::Sink::connect_new(self.marker_outputstream.mixer());
        sink.append(self.sounds[&format!("marker_{marker_sound_index:02}")].clone());
        sink.detach();
    }

    pub fn trigger_random_brush_sound(&mut self) {
        let mut rng = rand::rng();
        let brush_sound_seek_time_index =
            rng.random_range(0..Self::SOUND_FILE_BRUSH_SEEK_TIMES_MS.len());

        let mut reinstall_task = false;

        if let Some(handle) = self.brush_sound_task_handle.as_mut() {
            if !handle.timeout_reached() {
                if let Err(e) = handle.reset_timeout() {
                    error!("Resetting timeout on brush sound stop task failed, Err: {e:?}");
                    reinstall_task = true;
                }
            } else {
                reinstall_task = true;
            }
        } else {
            reinstall_task = true;
        }

        if reinstall_task {
            let sink = rodio::Sink::connect_new(self.brush_outputstream.mixer());

            sink.append(
                self.sounds["brush"]
                    .clone()
                    .repeat_infinite()
                    .skip_duration(Duration::from_millis(
                        (Self::SOUND_FILE_BRUSH_SEEK_TIMES_MS[brush_sound_seek_time_index]).round()
                            as u64,
                    )),
            );

            self.brush_sound_task_handle = Some(crate::tasks::OneOffTaskHandle::new(
                move || {
                    sink.stop();
                },
                Self::BRUSH_SOUND_TIMEOUT,
            ));
        }
    }

    /// Play a typewriter sound that fits the given key type, or a generic sound when None.
    pub fn play_typewriter_key_sound(&self, keyboard_key: Option<KeyboardKey>) {
        let sink = rodio::Sink::connect_new(self.typewriter_outputstream.mixer());

        match keyboard_key {
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
            // control characters are already filtered out of Unicode variant
            Some(KeyboardKey::Unicode(_))
            | Some(KeyboardKey::BackSpace)
            | Some(KeyboardKey::Delete)
            | Some(KeyboardKey::HorizontalTab)
            | None => {
                let mut rng = rand::rng();
                let typewriter_sound_index = rng.random_range(0..Self::N_SOUND_FILES_TYPEWRITER);

                sink.append(
                    self.sounds[&format!("typewriter_{typewriter_sound_index:02}")].clone(),
                );
                sink.detach();
            }
            _ => {
                sink.append(self.sounds["typewriter_thump"].clone());
                sink.detach();
            }
        }
    }
}

fn load_sound_from_path(
    mut resource_path: PathBuf,
    sound_name: &str,
    ending: &str,
) -> anyhow::Result<Buffered<Decoder<File>>> {
    resource_path.push(format!("{sound_name}.{ending}"));

    if resource_path.exists() {
        let buffered =
            rodio::Decoder::new(File::open(resource_path.clone()).with_context(|| {
                anyhow::anyhow!("Open file for path {:?} failed", resource_path,)
            })?)?
            .buffered();

        // initialize the buffer
        buffered.clone().for_each(|_| {});

        Ok(buffered)
    } else {
        Err(anyhow::anyhow!(
            "Failed to init audioplayer. file `{resource_path:?}` does not exist."
        ))
    }
}
