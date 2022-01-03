use std::cell::RefCell;
use std::rc::Rc;
use std::time;

use crate::pens::PenStyle;
use crate::ui::appwindow::RnoteAppWindow;
use gst::prelude::*;
use gtk4::{glib, glib::clone};
use rand::Rng;

#[derive(Debug)]
pub struct RnoteAudioPlayer {
    pub enabled: bool,
    marker_file_srcs: Vec<gst::Element>,
    marker_pipeline: Option<gst::Pipeline>,
    brush_pipeline: Option<gst::Pipeline>,
    play_timeout_id: Rc<RefCell<Option<glib::SourceId>>>,
}

impl RnoteAudioPlayer {
    pub const PLAY_TIMEOUT_TIME: time::Duration = time::Duration::from_millis(500);
    /// Number of marker sound files installed in system-data-dir/sounds
    pub const MARKER_N_FILES: usize = 15;

    pub fn new() -> Self {
        Self {
            enabled: false,
            marker_file_srcs: vec![],
            brush_pipeline: None,
            marker_pipeline: None,
            play_timeout_id: Rc::new(RefCell::new(None)),
        }
    }

    pub fn init(&mut self, _appwindow: &RnoteAppWindow) -> Result<(), anyhow::Error> {
        let system_data_dirs = glib::system_data_dirs();

        // Init marker sounds
        {
            for mut path in system_data_dirs.clone() {
                path.push("rnote/sounds/");
                if path.exists() {
                    // File Uris
                    let mut marker_locations: Vec<(usize, String)> = vec![];

                    for i in 0..Self::MARKER_N_FILES {
                        let mut file_path = path.clone();
                        file_path.push(format!("marker_{:02}.wav", i));

                        if file_path.exists() {
                            if let Some(file_path_str) = file_path.to_str() {
                                marker_locations.push((i, String::from(file_path_str)));
                            }
                        } else {
                            return Err(anyhow::Error::msg(format!(
                                "failed to init audioplayer. File `{}` is missing.",
                                file_path.to_string_lossy()
                            )));
                        }
                    }

                    // Creating the pipeline
                    let pipeline = gst::Pipeline::new(Some("marker_pipeline"));
                    let selector =
                        gst::ElementFactory::make("input-selector", Some("marker_selector"))?;
                    let decodebin =
                        gst::ElementFactory::make("decodebin", Some("marker_decodebin"))?;
                    let audioconvert =
                        gst::ElementFactory::make("audioconvert", Some("marker_audioconvert"))?;
                    let sink = gst::ElementFactory::make("autoaudiosink", Some("marker_sink"))?;

                    pipeline.add_many(&[&selector, &decodebin, &audioconvert, &sink])?;

                    gst::Element::link_many(&[&selector, &decodebin])?;
                    gst::Element::link_many(&[&audioconvert, &sink])?;

                    let mut marker_file_srcs = vec![];

                    for marker_location in marker_locations {
                        let file_src = gst::ElementFactory::make(
                            "filesrc",
                            Some(format!("marker_file_{}", marker_location.0).as_str()),
                        )?;

                        file_src
                            .set_property("location", format!("{}", marker_location.1).as_str())?;

                        pipeline.add(&file_src)?;
                        file_src.link(&selector)?;

                        marker_file_srcs.push(file_src);
                    }

                    // the decodebin needs dynamic pad linking
                    decodebin.connect_pad_added(
                        clone!(@weak audioconvert => move |decodebin, src_pad| {
                        // Try to detect whether the raw stream decodebin provided us with audio capabilities
                        let (is_audio, _is_video) = {
                            let media_type = src_pad.current_caps().and_then(|caps| {
                                caps.structure(0).map(|s| {
                                    let name = s.name();
                                    (name.starts_with("audio/"), name.starts_with("video/"))
                                })
                            });

                            match media_type {
                                None => {
                                    gst::element_warning!(
                                        decodebin,
                                        gst::CoreError::Negotiation,
                                        ("Failed to get media type from pad {}", src_pad.name())
                                    );

                                    return;
                                }
                                Some(media_type) => media_type,
                            }
                        };


                        if is_audio {
                            match audioconvert.static_pad("sink") {
                                Some(sink_pad) => {
                                    if let Err(e) = src_pad.link(&sink_pad) {
                                        log::error!(
                                            "failed to link src_pad of decodebin to sink_pad of audioconvert inside pad_added callback of marker pipeline, Err {}",
                                            e
                                        );
                                    }
                                }
                                None => {
                                    log::error!("failed to get sink pad of marker_audioconvert in pad_added callback of marker pipeline. Is None");
                                }
                            }
                        }
                    }));

                    // Message handling
                    if let Some(bus) = pipeline.bus() {
                        if let Err(e) = bus.add_watch(clone!(@weak pipeline => @default-return glib::source::Continue(true), move |_bus, message| {
                            match message.view() {
                                gst::MessageView::Eos(_) => {
                                    if let Err(e) = pipeline.set_state(gst::State::Null) {
                                        log::error!("set_state(Null) failed in bus watch for marker_pipeline, Err {}", e);
                                    };
                                }
                                gst::MessageView::Error(err) => {
                                    log::error!("bus for marker_pipeline has message with Err \n{:?}", err);
                                }
                                _ => {
                                }
                            }
                            glib::source::Continue(true)
                        })) {
                            log::error!(
                                "adding bus watch for marker_pipeline failed with Err {}",
                                e
                            );
                        }
                    }

                    self.marker_file_srcs = marker_file_srcs;
                    self.marker_pipeline = Some(pipeline);
                    break;
                }
            }
        }

        // Init brush sounds
        {
            for mut path in system_data_dirs.clone() {
                path.push("rnote/sounds/brush.wav");
                if path.exists() {
                    let brush_file_location = path.to_string_lossy();

                    let brush_file_src = gst::ElementFactory::make(
                        "filesrc",
                        Some(format!("{}", brush_file_location).as_str()),
                    )?;
                    brush_file_src
                        .set_property("location", format!("{}", brush_file_location).as_str())?;

                    // Creating the pipeline
                    let pipeline = gst::Pipeline::new(Some("brush_pipeline"));
                    let decodebin =
                        gst::ElementFactory::make("decodebin", Some("brush_decodebin"))?;
                    let audioconvert =
                        gst::ElementFactory::make("audioconvert", Some("brush_audioconvert"))?;
                    let sink = gst::ElementFactory::make("autoaudiosink", Some("brush_sink"))?;

                    pipeline.add_many(&[&brush_file_src, &decodebin, &audioconvert, &sink])?;

                    gst::Element::link_many(&[&brush_file_src, &decodebin])?;
                    gst::Element::link_many(&[&audioconvert, &sink])?;

                    // the decodebin needs dynamic pad linking
                    decodebin.connect_pad_added(
                            clone!(@weak audioconvert => move |decodebin, src_pad| {
                            // Try to detect whether the raw stream decodebin provided us with audio capabilities
                            let (is_audio, _is_video) = {
                                let media_type = src_pad.current_caps().and_then(|caps| {
                                    caps.structure(0).map(|s| {
                                        let name = s.name();
                                        (name.starts_with("audio/"), name.starts_with("video/"))
                                    })
                                });

                                match media_type {
                                    None => {
                                        gst::element_warning!(
                                            decodebin,
                                            gst::CoreError::Negotiation,
                                            ("Failed to get media type from pad {}", src_pad.name())
                                        );

                                        return;
                                    }
                                    Some(media_type) => media_type,
                                }
                            };


                            if is_audio {
                                match audioconvert.static_pad("sink") {
                                    Some(sink_pad) => {
                                        if let Err(e) = src_pad.link(&sink_pad) {
                                            log::error!(
                                                "failed to link src_pad of decodebin to sink_pad of audioconvert inside pad_added callback of brush pipeline, Err {}",
                                                e
                                            );
                                        }
                                    }
                                    None => {
                                        log::error!("failed to get sink pad of marker_audioconvert in pad_added callback of brush pipeline. Is None");
                                    }
                                }
                            }
                        }));

                    // Message handling
                    if let Some(bus) = pipeline.bus() {
                        if let Err(e) = bus.add_watch(clone!(@weak pipeline => @default-return glib::source::Continue(true), move |_bus, message| {
                                match message.view() {
                                    gst::MessageView::Eos(_) => {
                                        if let Err(e) = pipeline.set_state(gst::State::Ready) {
                                            log::error!("set_state(Null) failed in bus watch for brush_pipeline, Err {}", e);
                                        };
                                        if let Err(e) = pipeline.set_state(gst::State::Playing) {
                                            log::error!("set_state(Null) failed in bus watch for brush_pipeline, Err {}", e);
                                        };
                                    }
                                    gst::MessageView::Error(err) => {
                                        log::error!("bus for marker_pipeline has message with Err \n{:?}", err);
                                    }
                                    _ => {
                                    }
                                }
                                glib::source::Continue(true)
                            })) {
                                log::error!(
                                    "adding bus watch for marker_pipeline failed with Err {}",
                                    e
                                );
                            }
                    }

                    self.brush_pipeline = Some(pipeline);
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn play_pen_sound_begin(&self, timeout_time: time::Duration, current_pen: PenStyle) {
        if self.enabled {
            match current_pen {
                PenStyle::Marker => {
                    self.play_marker_sound();
                }
                PenStyle::Brush => {
                    self.play_brush_sound_w_timeout(timeout_time);
                }
                _ => {}
            }
        }
    }

    pub fn play_pen_sound_motion(&self, timeout_time: time::Duration, current_pen: PenStyle) {
        if self.enabled {
            match current_pen {
                PenStyle::Brush => {
                    self.play_brush_sound_w_timeout(timeout_time);
                }
                _ => {}
            }
        }
    }

    pub fn stop(&self) {
        if self.enabled {
            if let Some(marker_pipeline) = &self.marker_pipeline {
                if let Err(e) = marker_pipeline.set_state(gst::State::Ready) {
                    log::error!(
                        "audioplayer pipeline set_state(Playing) failed in stop() with Err {}",
                        e
                    );
                };
            };
            if let Some(brush_pipeline) = &self.brush_pipeline {
                if let Err(e) = brush_pipeline.set_state(gst::State::Ready) {
                    log::error!(
                        "audioplayer pipeline set_state(Playing) failed in stop() with Err {}",
                        e
                    );
                };
            };
        }
    }

    fn play_marker_sound(&self) {
        if let Some(marker_pipeline) = self.marker_pipeline.as_ref() {
            let marker_selector = marker_pipeline.by_name("marker_selector").unwrap();

            // first set state to Ready to start playing from the beginning of the file
            if let Err(e) = marker_pipeline.set_state(gst::State::Ready) {
                log::error!(
                        "audioplayer marker_pipeline set_state(Playing) failed in play_marker_sound() with Err {}",
                        e
                    );
            };

            let mut rng = rand::thread_rng();

            // Play a random file out of the range
            let i = rng.gen_range(0..Self::MARKER_N_FILES);

            if self.marker_file_srcs.get(i).is_some() {
                match marker_selector.static_pad(format!("sink_{}", i).as_str()) {
                    Some(active_pad) => {
                        marker_selector
                            .set_property("active-pad", active_pad)
                            .unwrap();
                    }
                    None => {
                        log::error!(
                            "getting pad with index {} from marker_selector failed, pad is None",
                            i
                        );
                    }
                }
            } else {
                log::error!(
                    "selecting index {} out of marker_file_srcs failed, index is OOB",
                    i
                );
            }

            // Play the sound
            if let Err(e) = marker_pipeline.set_state(gst::State::Playing) {
                log::error!(
                        "audioplayer marker_pipeline set_state(Playing) failed in play_marker_sound() with Err {}",
                        e
                    );
            };
        };
    }

    fn play_brush_sound_w_timeout(&self, timeout_time: time::Duration) {
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
