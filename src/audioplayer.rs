use std::cell::RefCell;
use std::rc::Rc;
use std::time;

use crate::pens::PenStyle;
use crate::ui::appwindow::RnoteAppWindow;
use gst::prelude::*;
use gtk4::{glib, glib::clone};

#[derive(Debug)]
pub struct RnoteAudioPlayer {
    pub enabled: bool,
    pub brush_pipeline: Option<gst::Element>,
    pub marker_pipeline: Option<gst::Element>,
    play_timeout_id: Rc<RefCell<Option<glib::SourceId>>>,
}

impl RnoteAudioPlayer {
    pub const PLAY_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(700);

    pub fn new() -> Self {
        Self {
            enabled: false,
            brush_pipeline: None,
            marker_pipeline: None,
            play_timeout_id: Rc::new(RefCell::new(None)),
        }
    }

    pub fn init(&mut self, _appwindow: &RnoteAppWindow) -> Result<(), anyhow::Error> {
        let system_data_dirs = glib::system_data_dirs();

        // Init marker sound
        for mut path in system_data_dirs.clone() {
            path.push("rnote/sounds/marker.wav");
            if path.exists() {
                if let Some(path_str) = path.to_str() {
                    let marker_audio_uri = format!("file://{}", path_str);
                    dbg!(&marker_audio_uri);

                    let marker_pipeline =
                        gst::parse_launch(&format!("playbin uri={}", marker_audio_uri))?;
                    self.marker_pipeline = Some(marker_pipeline);
                    break;
                }
            }
        }

        // Init brush sound
        for mut path in system_data_dirs.clone() {
            path.push("rnote/sounds/brush.wav");
            if path.exists() {
                if let Some(path_str) = path.to_str() {
                    let brush_audio_uri = format!("file://{}", path_str);
                    dbg!(&brush_audio_uri);

                    let brush_pipeline =
                        gst::parse_launch(&format!("playbin uri={}", brush_audio_uri))?;
                    self.brush_pipeline = Some(brush_pipeline);
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn play_w_timeout(&self, timeout_time: time::Duration, current_pen: PenStyle) {
        if self.enabled {
            match current_pen {
                PenStyle::Marker => {
                    self.play_w_timeout_marker(timeout_time);
                }
                PenStyle::Brush => {
                    self.play_w_timeout_brush(timeout_time);
                }
                _ => {}
            }
        }
    }

    pub fn stop(&self) {
        if self.enabled {
            if let Some(marker_pipeline) = &self.brush_pipeline {
                if let Err(e) = marker_pipeline.set_state(gst::State::Null) {
                    log::error!(
                        "audioplayer pipeline set_state(Playing) failed in stop() with Err {}",
                        e
                    );
                };
            };
        }
    }

    fn play_w_timeout_marker(&self, timeout_time: time::Duration) {
        if let Some(play_timeout_id) = self.play_timeout_id.borrow_mut().take() {
            glib::source::source_remove(play_timeout_id);
        } else {
            if let Some(pipeline) = &self.marker_pipeline {
                if let Err(e) = pipeline.set_state(gst::State::Playing) {
                    log::error!(
                        "audioplayer marker_pipeline set_state(Playing) failed in play_w_timeout() with Err {}",
                        e
                    );
                };
            };
        }

        self.play_timeout_id.borrow_mut()
            .replace(glib::source::timeout_add_local_once(
                timeout_time,
                clone!(@strong self.play_timeout_id as play_timeout_id, @strong self.marker_pipeline as marker_pipeline => move || {
                    if let Some(marker_pipeline) = &marker_pipeline {
                        if let Err(e) = marker_pipeline.set_state(gst::State::Null) {
                            log::error!(
                                "audioplayer marker_pipeline set_state(Null) failed in play_w_timeout() with Err {}",
                                e
                            );
                        };
                    };
                    play_timeout_id.borrow_mut().take();
                }),
            ));
    }

    fn play_w_timeout_brush(&self, timeout_time: time::Duration) {
        if let Some(play_timeout_id) = self.play_timeout_id.borrow_mut().take() {
            glib::source::source_remove(play_timeout_id);
        } else {
            if let Some(brush_pipeline) = &self.brush_pipeline {
                if let Err(e) = brush_pipeline.set_state(gst::State::Playing) {
                    log::error!(
                                    "audioplayer brush_pipeline set_state(Playing) failed in play_w_timeout() with Err {}",
                                    e
                                );
                };
            };
        }

        self.play_timeout_id.borrow_mut()
            .replace(glib::source::timeout_add_local_once(
                timeout_time,
                clone!(@strong self.play_timeout_id as play_timeout_id, @strong self.brush_pipeline as brush_pipeline => move || {
                    if let Some(brush_pipeline) = &brush_pipeline {
                        if let Err(e) = brush_pipeline.set_state(gst::State::Null) {
                            log::error!(
                                "audioplayer brush_pipeline set_state(Null) failed in play_w_timeout() with Err {}",
                                e
                            );
                        };
                    };
                    play_timeout_id.borrow_mut().take();
                }),
            ));
    }
}
