use std::cell::RefCell;
use std::rc::Rc;
use std::time;

use crate::ui::appwindow::RnoteAppWindow;
use gst::prelude::*;
use gtk4::{glib, glib::clone};

#[derive(Debug)]
pub struct RnoteAudioPlayer {
    pub enabled: bool,
    pub pipeline: Option<gst::Element>,
    play_timeout_id: Rc<RefCell<Option<glib::SourceId>>>,
}

impl RnoteAudioPlayer {
    pub const PLAY_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(700);

    pub fn new() -> Self {
        Self {
            enabled: false,
            pipeline: None,
            play_timeout_id: Rc::new(RefCell::new(None)),
        }
    }

    pub fn init(&mut self, _appwindow: &RnoteAppWindow) -> Result<(), anyhow::Error> {
        let mut audio_file_uri: Option<String> = None;

        for mut path in glib::system_data_dirs() {
            path.push("rnote/sounds/writing_high_pitch.wav");
            if path.exists() {
                if let Some(path_str) = path.to_str() {
                    audio_file_uri = Some(format!("file://{}", path_str));
                    break;
                }
            }
        }

        if let Some(audio_file_uri) = audio_file_uri {
            let pipeline = gst::parse_launch(&format!("playbin uri={}", audio_file_uri))?;
            self.pipeline = Some(pipeline);
        } else {
            return Err(anyhow::Error::msg(
                "failed to initialize pipeline. Audio files not found.",
            ));
        }

        Ok(())
    }

    pub fn play_w_timeout(&self, timeout_time: time::Duration) {
        if self.enabled {
            if let Some(play_timeout_id) = self.play_timeout_id.borrow_mut().take() {
                glib::source::source_remove(play_timeout_id);
            } else {
                if let Some(pipeline) = &self.pipeline {
                    if let Err(e) = pipeline.set_state(gst::State::Playing) {
                        log::error!(
                    "audioplayer pipeline set_state(Playing) failed in play_w_timeout() with Err {}",
                    e
                );
                    };
                };
            }

            self.play_timeout_id.borrow_mut()
            .replace(glib::source::timeout_add_local_once(
                timeout_time,
                clone!(@strong self.play_timeout_id as play_timeout_id, @strong self.pipeline as pipeline => move || {
                    if let Some(pipeline) = &pipeline {
                        if let Err(e) = pipeline.set_state(gst::State::Null) {
                            log::error!(
                                "audioplayer pipeline set_state(Null) failed in play_w_timeout() with Err {}",
                                e
                            );
                        };
                    };
                    play_timeout_id.borrow_mut().take();
                }),
            ));
        }
    }

    pub fn stop(&self) {
        if self.enabled {
            if let Some(pipeline) = &self.pipeline {
                if let Err(e) = pipeline.set_state(gst::State::Null) {
                    log::error!(
                        "audioplayer pipeline set_state(Playing) failed in stop() with Err {}",
                        e
                    );
                };
            };
        }
    }
}
