use std::time::Instant;

use rnote_compose::penevents::{KeyboardKey, ShortcutKey};
use rnote_compose::penpath::Element;

use crate::engine::EngineViewMut;
use crate::pens::penbehaviour::PenProgress;
use crate::pens::PenBehaviour;
use crate::strokes::{Stroke, TextStroke};
use crate::{DrawOnDocBehaviour, StrokeStore, WidgetFlags};

use super::{Typewriter, TypewriterState};

impl Typewriter {
    pub(super) fn handle_pen_event_down(
        &mut self,
        element: Element,
        _shortcut_keys: Vec<ShortcutKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let typewriter_bounds = self.bounds_on_doc(&engine_view.as_im());

        let text_width = engine_view.pens_config.typewriter_config.text_width;

        let pen_progress = match &mut self.state {
            TypewriterState::Idle | TypewriterState::Start { .. } => {
                let mut refresh_state = false;
                let mut new_state = TypewriterState::Start(element.pos);

                if let Some(&stroke_key) = engine_view
                    .store
                    .stroke_hitboxes_contain_coord(engine_view.camera.viewport(), element.pos)
                    .last()
                {
                    // When clicked on a textstroke, we start modifying it
                    if let Some(Stroke::TextStroke(textstroke)) =
                        engine_view.store.get_stroke_ref(stroke_key)
                    {
                        let mut cursor = unicode_segmentation::GraphemeCursor::new(
                            0,
                            textstroke.text.len(),
                            true,
                        );

                        // Updating the cursor for the clicked position
                        if let Ok(new_cursor) = textstroke.get_cursor_for_global_coord(element.pos)
                        {
                            cursor = new_cursor;
                        }

                        engine_view.store.update_chrono_to_last(stroke_key);

                        new_state = TypewriterState::Modifying {
                            stroke_key,
                            cursor,
                            pen_down: true,
                        };
                        refresh_state = true;
                    }
                }

                self.state = new_state;

                // after setting new state
                if refresh_state {
                    // Update typewriter state for the current textstroke, and indicate that the penholder has changed, to update the UI
                    widget_flags.merge(self.update_state(engine_view));

                    widget_flags.redraw = true;
                    widget_flags.refresh_ui = true;
                }

                PenProgress::InProgress
            }
            TypewriterState::Modifying {
                stroke_key,
                cursor,
                pen_down,
            } => {
                let mut pen_progress = PenProgress::InProgress;

                if let (Some(typewriter_bounds), Some(Stroke::TextStroke(textstroke))) = (
                    typewriter_bounds,
                    engine_view.store.get_stroke_ref(*stroke_key),
                ) {
                    if Self::translate_node_bounds(typewriter_bounds, engine_view.camera)
                        .contains_local_point(&na::Point2::from(element.pos))
                    {
                        // switch to translating the text field
                        widget_flags.merge(engine_view.store.record(Instant::now()));

                        self.state = TypewriterState::Translating {
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            start_pos: element.pos,
                            current_pos: element.pos,
                        };
                    } else if Self::adjust_text_width_node_bounds(
                        Self::text_rect_bounds(text_width, textstroke).mins.coords,
                        text_width,
                        engine_view.camera,
                    )
                    .contains_local_point(&na::Point2::from(element.pos))
                    {
                        widget_flags.merge(engine_view.store.record(Instant::now()));

                        // Clicking on the adjust text width node
                        self.state = TypewriterState::AdjustTextWidth {
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            start_text_width: text_width,
                            start_pos: element.pos,
                            current_pos: element.pos,
                        };
                    // This is intentionally **not** the textstroke hitboxes
                    } else if typewriter_bounds.contains_local_point(&na::Point2::from(element.pos))
                    {
                        if let Some(Stroke::TextStroke(textstroke)) =
                            engine_view.store.get_stroke_ref(*stroke_key)
                        {
                            if let Ok(new_cursor) =
                                textstroke.get_cursor_for_global_coord(element.pos)
                            {
                                if new_cursor.cur_cursor() != cursor.cur_cursor() && *pen_down {
                                    // switch to selecting
                                    self.state = TypewriterState::Selecting {
                                        stroke_key: *stroke_key,
                                        cursor: cursor.clone(),
                                        selection_cursor: cursor.clone(),
                                        finished: false,
                                    };
                                } else {
                                    *cursor = new_cursor;
                                    *pen_down = true;
                                }
                            }
                        }
                    } else {
                        // If we click outside, reset to idle
                        self.state = TypewriterState::Idle;

                        pen_progress = PenProgress::Finished;
                    }
                }

                widget_flags.redraw = true;

                pen_progress
            }
            TypewriterState::Selecting {
                stroke_key,
                cursor,
                finished,
                ..
            } => {
                let mut pen_progress = PenProgress::InProgress;

                if let Some(typewriter_bounds) = typewriter_bounds {
                    // Clicking on the translate node
                    if Self::translate_node_bounds(typewriter_bounds, engine_view.camera)
                        .contains_local_point(&na::Point2::from(element.pos))
                    {
                        widget_flags.merge(engine_view.store.record(Instant::now()));

                        self.state = TypewriterState::Translating {
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            start_pos: element.pos,
                            current_pos: element.pos,
                        };
                    } else if typewriter_bounds.contains_local_point(&na::Point2::from(element.pos))
                    {
                        if let Some(Stroke::TextStroke(textstroke)) =
                            engine_view.store.get_stroke_ref(*stroke_key)
                        {
                            // If selecting is finished, return to modifying with the current pen position as cursor
                            if *finished {
                                if let Ok(new_cursor) =
                                    textstroke.get_cursor_for_global_coord(element.pos)
                                {
                                    self.state = TypewriterState::Modifying {
                                        stroke_key: *stroke_key,
                                        cursor: new_cursor,
                                        pen_down: false,
                                    };
                                }
                            } else {
                                // Updating the cursor for the clicked position
                                if let Ok(new_cursor) =
                                    textstroke.get_cursor_for_global_coord(element.pos)
                                {
                                    *cursor = new_cursor
                                }
                            }
                        }
                    } else {
                        // If we click outside, reset to idle
                        self.state = TypewriterState::Idle;

                        pen_progress = PenProgress::Finished;
                    }
                }

                widget_flags.redraw = true;

                pen_progress
            }
            TypewriterState::Translating {
                stroke_key,
                current_pos,
                ..
            } => {
                let offset = element.pos - *current_pos;

                if offset.magnitude()
                    > Self::TRANSLATE_MAGNITUDE_THRESHOLD / engine_view.camera.total_zoom()
                {
                    engine_view.store.translate_strokes(&[*stroke_key], offset);
                    engine_view
                        .store
                        .translate_strokes_images(&[*stroke_key], offset);

                    engine_view.store.regenerate_rendering_for_stroke(
                        *stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    );

                    *current_pos = element.pos;

                    widget_flags.redraw = true;
                    widget_flags.indicate_changed_store = true;
                }

                PenProgress::InProgress
            }
            TypewriterState::AdjustTextWidth {
                stroke_key,
                start_text_width,
                start_pos,
                current_pos,
                ..
            } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_mut(*stroke_key)
                {
                    let abs_x_offset = element.pos[0] - start_pos[0];

                    engine_view.pens_config.typewriter_config.text_width =
                        (*start_text_width + abs_x_offset).max(2.0);

                    if let Some(max_width) = &mut textstroke.text_style.max_width {
                        *max_width = *start_text_width + abs_x_offset;
                    }
                }

                engine_view.store.regenerate_rendering_for_stroke(
                    *stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                *current_pos = element.pos;

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
        };

        (pen_progress, widget_flags)
    }

    pub(super) fn handle_pen_event_up(
        &mut self,
        _element: Element,
        _shortcut_keys: Vec<ShortcutKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let pen_progress = match &mut self.state {
            TypewriterState::Idle => PenProgress::Idle,
            TypewriterState::Start(_) => PenProgress::InProgress,
            TypewriterState::Modifying { pen_down, .. } => {
                *pen_down = false;
                PenProgress::InProgress
            }
            TypewriterState::Selecting { finished, .. } => {
                // finished when drag ended
                *finished = true;

                widget_flags.redraw = true;

                PenProgress::InProgress
            }
            TypewriterState::Translating {
                stroke_key, cursor, ..
            } => {
                engine_view
                    .store
                    .update_geometry_for_strokes(&[*stroke_key]);
                engine_view.store.regenerate_rendering_for_stroke(
                    *stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.state = TypewriterState::Modifying {
                    stroke_key: *stroke_key,
                    cursor: cursor.clone(),
                    pen_down: false,
                };

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
            TypewriterState::AdjustTextWidth {
                stroke_key, cursor, ..
            } => {
                engine_view
                    .store
                    .update_geometry_for_strokes(&[*stroke_key]);
                engine_view.store.regenerate_rendering_for_stroke(
                    *stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.state = TypewriterState::Modifying {
                    stroke_key: *stroke_key,
                    cursor: cursor.clone(),
                    pen_down: false,
                };

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
        };

        (pen_progress, widget_flags)
    }

    pub(super) fn handle_pen_event_proximity(
        &mut self,
        _element: Element,
        _shortcut_keys: Vec<ShortcutKey>,
        _now: Instant,
        _engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let widget_flags = WidgetFlags::default();

        let pen_progress = match &mut self.state {
            TypewriterState::Idle => PenProgress::Idle,
            TypewriterState::Start(_) => PenProgress::InProgress,
            TypewriterState::Modifying { pen_down, .. } => {
                *pen_down = false;
                PenProgress::InProgress
            }
            TypewriterState::Selecting { .. } => PenProgress::InProgress,
            TypewriterState::Translating { .. } => PenProgress::InProgress,
            TypewriterState::AdjustTextWidth { .. } => PenProgress::InProgress,
        };

        (pen_progress, widget_flags)
    }

    pub(super) fn handle_pen_event_keypressed(
        &mut self,
        keyboard_key: KeyboardKey,
        shortcut_keys: Vec<ShortcutKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let text_width = engine_view.pens_config.typewriter_config.text_width;
        let mut text_style = engine_view.pens_config.typewriter_config.text_style.clone();
        let max_width_enabled = engine_view.pens_config.typewriter_config.max_width_enabled;

        let pen_progress = match &mut self.state {
            TypewriterState::Idle => PenProgress::Idle,
            TypewriterState::Start(pos) => {
                widget_flags.merge(engine_view.store.record(Instant::now()));
                Self::start_audio(Some(keyboard_key), engine_view.audioplayer);

                match keyboard_key {
                    KeyboardKey::Unicode(keychar) => {
                        text_style.ranged_text_attributes.clear();

                        if max_width_enabled {
                            text_style.max_width = Some(text_width);
                        }

                        let textstroke = TextStroke::new(String::from(keychar), *pos, text_style);

                        let mut cursor = unicode_segmentation::GraphemeCursor::new(
                            0,
                            textstroke.text.len(),
                            true,
                        );
                        textstroke.move_cursor_forward(&mut cursor);

                        let stroke_key = engine_view
                            .store
                            .insert_stroke(Stroke::TextStroke(textstroke), None);

                        engine_view.store.regenerate_rendering_for_stroke(
                            stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        self.state = TypewriterState::Modifying {
                            stroke_key,
                            cursor,
                            pen_down: false,
                        };
                    }
                    _ => {}
                }

                widget_flags.redraw = true;

                PenProgress::InProgress
            }
            TypewriterState::Modifying {
                stroke_key,
                cursor,
                pen_down,
            } => {
                //log::debug!("key: {:?}", keyboard_key);
                widget_flags.merge(engine_view.store.record(Instant::now()));
                Self::start_audio(Some(keyboard_key), engine_view.audioplayer);

                if let Some(Stroke::TextStroke(ref mut textstroke)) =
                    engine_view.store.get_stroke_mut(*stroke_key)
                {
                    let mut update_stroke = |store: &mut StrokeStore| {
                        store.update_geometry_for_stroke(*stroke_key);
                        store.regenerate_rendering_for_stroke(
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        engine_view.doc.resize_autoexpand(store, engine_view.camera);

                        widget_flags.redraw = true;
                        widget_flags.resize = true;
                        widget_flags.indicate_changed_store = true;
                    };

                    // Handling keyboard input
                    let new_state = match keyboard_key {
                        KeyboardKey::Unicode(keychar) => {
                            if keychar == 'a' && shortcut_keys.contains(&ShortcutKey::KeyboardCtrl)
                            {
                                // Select entire text

                                Some(TypewriterState::Selecting {
                                    stroke_key: *stroke_key,
                                    cursor: unicode_segmentation::GraphemeCursor::new(
                                        textstroke.text.len(),
                                        textstroke.text.len(),
                                        true,
                                    ),
                                    selection_cursor: unicode_segmentation::GraphemeCursor::new(
                                        0,
                                        textstroke.text.len(),
                                        true,
                                    ),
                                    finished: true,
                                })
                            } else {
                                textstroke
                                    .insert_text_after_cursor(keychar.to_string().as_str(), cursor);
                                update_stroke(engine_view.store);
                                None
                            }
                        }
                        KeyboardKey::BackSpace => {
                            textstroke.remove_grapheme_before_cursor(cursor);
                            update_stroke(engine_view.store);
                            None
                        }
                        KeyboardKey::HorizontalTab => {
                            textstroke.insert_text_after_cursor("\t", cursor);
                            update_stroke(engine_view.store);
                            None
                        }
                        KeyboardKey::CarriageReturn | KeyboardKey::Linefeed => {
                            textstroke.insert_text_after_cursor("\n", cursor);
                            update_stroke(engine_view.store);

                            None
                        }
                        KeyboardKey::Delete => {
                            textstroke.remove_grapheme_after_cursor(cursor);
                            update_stroke(engine_view.store);

                            None
                        }
                        KeyboardKey::NavLeft => {
                            if shortcut_keys.contains(&ShortcutKey::KeyboardShift) {
                                let mut new_cursor = cursor.clone();
                                textstroke.move_cursor_back(&mut new_cursor);

                                Some(TypewriterState::Selecting {
                                    stroke_key: *stroke_key,
                                    cursor: new_cursor,
                                    selection_cursor: cursor.clone(),
                                    finished: false,
                                })
                            } else {
                                textstroke.move_cursor_back(cursor);

                                None
                            }
                        }
                        KeyboardKey::NavRight => {
                            if shortcut_keys.contains(&ShortcutKey::KeyboardShift) {
                                let mut new_cursor = cursor.clone();
                                textstroke.move_cursor_forward(&mut new_cursor);

                                Some(TypewriterState::Selecting {
                                    stroke_key: *stroke_key,
                                    cursor: new_cursor,
                                    selection_cursor: cursor.clone(),
                                    finished: false,
                                })
                            } else {
                                textstroke.move_cursor_forward(cursor);

                                None
                            }
                        }
                        KeyboardKey::NavUp => {
                            if shortcut_keys.contains(&ShortcutKey::KeyboardShift) {
                                let mut new_cursor = cursor.clone();
                                textstroke.move_cursor_line_up(&mut new_cursor);

                                Some(TypewriterState::Selecting {
                                    stroke_key: *stroke_key,
                                    cursor: new_cursor,
                                    selection_cursor: cursor.clone(),
                                    finished: false,
                                })
                            } else {
                                textstroke.move_cursor_line_up(cursor);

                                None
                            }
                        }
                        KeyboardKey::NavDown => {
                            if shortcut_keys.contains(&ShortcutKey::KeyboardShift) {
                                let mut new_cursor = cursor.clone();
                                textstroke.move_cursor_line_down(&mut new_cursor);

                                Some(TypewriterState::Selecting {
                                    stroke_key: *stroke_key,
                                    cursor: new_cursor,
                                    selection_cursor: cursor.clone(),
                                    finished: false,
                                })
                            } else {
                                textstroke.move_cursor_line_down(cursor);

                                None
                            }
                        }
                        _ => None,
                    };

                    *pen_down = false;

                    widget_flags.redraw = true;

                    if let Some(new_state) = new_state {
                        self.state = new_state;
                    }
                }

                PenProgress::InProgress
            }
            TypewriterState::Selecting {
                stroke_key,
                cursor,
                selection_cursor,
                finished,
            } => {
                //log::debug!("key: {:?}", keyboard_key);
                widget_flags.merge(engine_view.store.record(Instant::now()));
                Self::start_audio(Some(keyboard_key), engine_view.audioplayer);

                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_mut(*stroke_key)
                {
                    let mut update_stroke = |store: &mut StrokeStore| {
                        store.update_geometry_for_stroke(*stroke_key);
                        store.regenerate_rendering_for_stroke(
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        engine_view.doc.resize_autoexpand(store, engine_view.camera);

                        widget_flags.redraw = true;
                        widget_flags.resize = true;
                        widget_flags.indicate_changed_store = true;
                    };

                    // Handle keyboard keys
                    let quit_selecting = match keyboard_key {
                        KeyboardKey::Unicode(keychar) => {
                            if keychar == 'a' && shortcut_keys.contains(&ShortcutKey::KeyboardCtrl)
                            {
                                textstroke.update_selection_entire_text(cursor, selection_cursor);
                                *finished = true;

                                false
                            } else {
                                textstroke.replace_text_between_selection_cursors(
                                    cursor,
                                    selection_cursor,
                                    String::from(keychar).as_str(),
                                );

                                update_stroke(engine_view.store);
                                true
                            }
                        }
                        KeyboardKey::NavLeft => {
                            if shortcut_keys.contains(&ShortcutKey::KeyboardShift) {
                                textstroke.move_cursor_back(cursor);
                                false
                            } else {
                                true
                            }
                        }
                        KeyboardKey::NavRight => {
                            if shortcut_keys.contains(&ShortcutKey::KeyboardShift) {
                                textstroke.move_cursor_forward(cursor);
                                false
                            } else {
                                true
                            }
                        }
                        KeyboardKey::NavUp => {
                            if shortcut_keys.contains(&ShortcutKey::KeyboardShift) {
                                textstroke.move_cursor_line_up(cursor);
                                false
                            } else {
                                true
                            }
                        }
                        KeyboardKey::NavDown => {
                            if shortcut_keys.contains(&ShortcutKey::KeyboardShift) {
                                textstroke.move_cursor_line_down(cursor);
                                false
                            } else {
                                true
                            }
                        }
                        KeyboardKey::CarriageReturn | KeyboardKey::Linefeed => {
                            textstroke.replace_text_between_selection_cursors(
                                cursor,
                                selection_cursor,
                                "\n",
                            );

                            update_stroke(engine_view.store);
                            true
                        }
                        KeyboardKey::BackSpace | KeyboardKey::Delete => {
                            textstroke.replace_text_between_selection_cursors(
                                cursor,
                                selection_cursor,
                                "",
                            );

                            update_stroke(engine_view.store);
                            true
                        }
                        KeyboardKey::HorizontalTab => {
                            textstroke.replace_text_between_selection_cursors(
                                cursor,
                                selection_cursor,
                                "\t",
                            );

                            update_stroke(engine_view.store);
                            true
                        }
                        KeyboardKey::CtrlLeft
                        | KeyboardKey::CtrlRight
                        | KeyboardKey::ShiftLeft
                        | KeyboardKey::ShiftRight => false,
                        _ => true,
                    };

                    if quit_selecting {
                        // Back to modifying
                        self.state = TypewriterState::Modifying {
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            pen_down: false,
                        };
                    }
                }

                widget_flags.redraw = true;

                PenProgress::InProgress
            }
            TypewriterState::Translating { .. } => PenProgress::InProgress,
            TypewriterState::AdjustTextWidth { .. } => PenProgress::InProgress,
        };

        (pen_progress, widget_flags)
    }

    pub(super) fn handle_pen_event_text(
        &mut self,
        text: String,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let text_width = engine_view.pens_config.typewriter_config.text_width;
        let mut text_style = engine_view.pens_config.typewriter_config.text_style.clone();
        let max_width_enabled = engine_view.pens_config.typewriter_config.max_width_enabled;

        let pen_progress = match &mut self.state {
            TypewriterState::Idle => PenProgress::Idle,
            TypewriterState::Start(pos) => {
                widget_flags.merge(engine_view.store.record(Instant::now()));
                Self::start_audio(None, engine_view.audioplayer);

                text_style.ranged_text_attributes.clear();

                if max_width_enabled {
                    text_style.max_width = Some(text_width);
                }
                let text_len = text.len();

                let textstroke = TextStroke::new(text, *pos, text_style);

                let cursor = unicode_segmentation::GraphemeCursor::new(text_len, text_len, true);

                let stroke_key = engine_view
                    .store
                    .insert_stroke(Stroke::TextStroke(textstroke), None);

                engine_view.store.regenerate_rendering_for_stroke(
                    stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.state = TypewriterState::Modifying {
                    stroke_key,
                    cursor,
                    pen_down: false,
                };

                PenProgress::InProgress
            }
            TypewriterState::Modifying {
                stroke_key,
                cursor,
                pen_down,
            } => {
                // Only record between words
                if text.contains(' ') {
                    widget_flags.merge(engine_view.store.record(Instant::now()));
                }
                Self::start_audio(None, engine_view.audioplayer);

                if let Some(Stroke::TextStroke(ref mut textstroke)) =
                    engine_view.store.get_stroke_mut(*stroke_key)
                {
                    textstroke.insert_text_after_cursor(&text, cursor);

                    engine_view.store.update_geometry_for_stroke(*stroke_key);
                    engine_view.store.regenerate_rendering_for_stroke(
                        *stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    );

                    engine_view
                        .doc
                        .resize_autoexpand(engine_view.store, engine_view.camera);

                    widget_flags.redraw = true;
                    widget_flags.resize = true;
                    widget_flags.indicate_changed_store = true;

                    *pen_down = false;
                }

                PenProgress::InProgress
            }
            TypewriterState::Selecting {
                stroke_key,
                cursor,
                selection_cursor,
                finished,
            } => {
                if text.contains(' ') {
                    widget_flags.merge(engine_view.store.record(Instant::now()));
                }
                Self::start_audio(None, engine_view.audioplayer);

                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_mut(*stroke_key)
                {
                    textstroke.replace_text_between_selection_cursors(
                        cursor,
                        selection_cursor,
                        text.as_str(),
                    );

                    engine_view.store.update_geometry_for_stroke(*stroke_key);
                    engine_view.store.regenerate_rendering_for_stroke(
                        *stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    );

                    engine_view
                        .doc
                        .resize_autoexpand(engine_view.store, engine_view.camera);

                    widget_flags.redraw = true;
                    widget_flags.resize = true;
                    widget_flags.indicate_changed_store = true;

                    *finished = true
                }

                PenProgress::InProgress
            }
            TypewriterState::Translating { .. } => PenProgress::InProgress,
            TypewriterState::AdjustTextWidth { .. } => PenProgress::InProgress,
        };

        (pen_progress, widget_flags)
    }

    pub(super) fn handle_pen_event_cancel(
        &mut self,
        _now: Instant,
        _engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let pen_progress = match &mut self.state {
            TypewriterState::Idle => PenProgress::Idle,
            TypewriterState::Start(_) => {
                self.state = TypewriterState::Idle;

                widget_flags.redraw = true;

                PenProgress::Finished
            }
            TypewriterState::Modifying { .. } => {
                self.state = TypewriterState::Idle;

                widget_flags.redraw = true;

                PenProgress::Finished
            }
            TypewriterState::Selecting { .. } => {
                self.state = TypewriterState::Idle;

                widget_flags.redraw = true;

                PenProgress::Finished
            }
            TypewriterState::Translating { .. } => {
                self.state = TypewriterState::Idle;

                widget_flags.redraw = true;

                PenProgress::Finished
            }
            TypewriterState::AdjustTextWidth { .. } => {
                self.state = TypewriterState::Idle;

                widget_flags.redraw = true;

                PenProgress::Finished
            }
        };

        (pen_progress, widget_flags)
    }
}
