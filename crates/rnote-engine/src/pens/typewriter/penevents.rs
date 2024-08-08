// Imports
use super::{ModifyState, Typewriter, TypewriterState};
use crate::engine::EngineViewMut;
use crate::pens::PenBehaviour;
use crate::strokes::{Stroke, TextStroke};
use crate::{DrawableOnDoc, StrokeStore, WidgetFlags};
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::penevent::{KeyboardKey, ModifierKey, PenProgress};
use rnote_compose::penpath::Element;
use rnote_compose::shapes::Shapeable;
use std::collections::HashSet;
use std::time::Instant;
use unicode_segmentation::GraphemeCursor;

impl Typewriter {
    pub(super) fn handle_pen_event_down(
        &mut self,
        element: Element,
        _modifier_keys: HashSet<ModifierKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();
        let typewriter_bounds = self.bounds_on_doc(&engine_view.as_im());
        let text_width = engine_view.pens_config.typewriter_config.text_width();

        let event_result = match &mut self.state {
            TypewriterState::Idle | TypewriterState::Start { .. } => {
                let mut refresh_state = false;
                let mut new_state =
                    TypewriterState::Start(engine_view.document.snap_position(element.pos));

                if let Some(&stroke_key) = engine_view
                    .store
                    .stroke_hitboxes_contain_coord(engine_view.camera.viewport(), element.pos)
                    .last()
                {
                    // When clicked on a textstroke, we start modifying it
                    if let Some(Stroke::TextStroke(textstroke)) =
                        engine_view.store.get_stroke_ref(stroke_key)
                    {
                        let cursor = if let Ok(new_cursor) =
                            // get the cursor for the current position
                            textstroke.get_cursor_for_global_coord(element.pos)
                        {
                            new_cursor
                        } else {
                            GraphemeCursor::new(0, textstroke.text.len(), true)
                        };

                        engine_view.store.update_chrono_to_last(stroke_key);

                        new_state = TypewriterState::Modifying {
                            modify_state: ModifyState::Up,
                            stroke_key,
                            cursor,
                            pen_down: true,
                        };
                        refresh_state = true;
                    }
                }

                self.state = new_state;
                self.reset_blink();

                // after setting new state
                if refresh_state {
                    // Update typewriter state for the current textstroke,
                    // and flag to update the UI
                    widget_flags |= self.update_state(engine_view);
                    widget_flags.refresh_ui = true;
                }

                // possibly nudge camera
                widget_flags |= engine_view
                    .camera
                    .nudge_w_pos(element.pos, engine_view.document);
                widget_flags |= engine_view
                    .document
                    .expand_autoexpand(engine_view.camera, engine_view.store);
                engine_view.store.regenerate_rendering_in_viewport_threaded(
                    engine_view.tasks_tx.clone(),
                    false,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                pen_down,
            } => {
                match modify_state {
                    ModifyState::Up | ModifyState::Hover(_) => {
                        let mut progress = PenProgress::InProgress;

                        if let (Some(typewriter_bounds), Some(Stroke::TextStroke(textstroke))) = (
                            typewriter_bounds,
                            engine_view.store.get_stroke_ref(*stroke_key),
                        ) {
                            if Self::translate_node_bounds(typewriter_bounds, engine_view.camera)
                                .contains_local_point(&element.pos.into())
                            {
                                // switch to translating state
                                self.state = TypewriterState::Modifying {
                                    modify_state: ModifyState::Translating {
                                        current_pos: element.pos,
                                    },
                                    stroke_key: *stroke_key,
                                    cursor: cursor.clone(),
                                    pen_down: true,
                                };
                            } else if Self::adjust_text_width_node_bounds(
                                Self::text_rect_bounds(text_width, textstroke).mins.coords,
                                text_width,
                                engine_view.camera,
                            )
                            .contains_local_point(&element.pos.into())
                            {
                                // switch to adjust text width
                                self.state = TypewriterState::Modifying {
                                    modify_state: ModifyState::AdjustTextWidth {
                                        start_text_width: text_width,
                                        start_pos: element.pos,
                                        current_pos: element.pos,
                                    },
                                    stroke_key: *stroke_key,
                                    cursor: cursor.clone(),
                                    pen_down: true,
                                };
                            // This is intentionally **not** the textstroke hitboxes
                            } else if typewriter_bounds.contains_local_point(&element.pos.into()) {
                                if let Some(Stroke::TextStroke(textstroke)) =
                                    engine_view.store.get_stroke_ref(*stroke_key)
                                {
                                    if let Ok(new_cursor) =
                                        textstroke.get_cursor_for_global_coord(element.pos)
                                    {
                                        if new_cursor.cur_cursor() != cursor.cur_cursor()
                                            && *pen_down
                                        {
                                            // switch to selecting state
                                            self.state = TypewriterState::Modifying {
                                                modify_state: ModifyState::Selecting {
                                                    selection_cursor: cursor.clone(),
                                                    finished: false,
                                                },
                                                stroke_key: *stroke_key,
                                                cursor: cursor.clone(),
                                                pen_down: true,
                                            };
                                        } else {
                                            *cursor = new_cursor;
                                            *pen_down = true;
                                            self.reset_blink();
                                        }
                                    }
                                }
                            } else {
                                // If we click outside, reset to idle
                                self.state = TypewriterState::Idle;
                                progress = PenProgress::Finished;
                            }
                        }

                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress,
                        }
                    }
                    ModifyState::Selecting { finished, .. } => {
                        let mut progress = PenProgress::InProgress;

                        if let Some(typewriter_bounds) = typewriter_bounds {
                            // Clicking on the translate node
                            if Self::translate_node_bounds(typewriter_bounds, engine_view.camera)
                                .contains_local_point(&element.pos.into())
                            {
                                self.state = TypewriterState::Modifying {
                                    modify_state: ModifyState::Translating {
                                        current_pos: element.pos,
                                    },
                                    stroke_key: *stroke_key,
                                    cursor: cursor.clone(),
                                    pen_down: true,
                                };
                            } else if typewriter_bounds.contains_local_point(&element.pos.into()) {
                                if let Some(Stroke::TextStroke(textstroke)) =
                                    engine_view.store.get_stroke_ref(*stroke_key)
                                {
                                    if *finished {
                                        if let Ok(new_cursor) =
                                            textstroke.get_cursor_for_global_coord(element.pos)
                                        {
                                            // If selecting is finished, return to modifying with the current pen position as cursor
                                            self.state = TypewriterState::Modifying {
                                                modify_state: ModifyState::Up,
                                                stroke_key: *stroke_key,
                                                cursor: new_cursor,
                                                pen_down: true,
                                            };
                                            self.reset_blink();
                                        }
                                    } else {
                                        // Updating the cursor for the clicked position
                                        if let Ok(new_cursor) =
                                            textstroke.get_cursor_for_global_coord(element.pos)
                                        {
                                            *cursor = new_cursor;
                                            self.reset_blink();
                                        }
                                    }
                                }
                            } else {
                                // If we click outside, reset to idle
                                self.state = TypewriterState::Idle;
                                progress = PenProgress::Finished;
                            }
                        }

                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress,
                        }
                    }
                    ModifyState::Translating { current_pos, .. } => {
                        if let Some(textstroke_bounds) = engine_view
                            .store
                            .get_stroke_ref(*stroke_key)
                            .map(|s| s.bounds())
                        {
                            let snap_corner_pos = textstroke_bounds.mins.coords;
                            let offset = engine_view
                                .document
                                .snap_position(snap_corner_pos + (element.pos - *current_pos))
                                - snap_corner_pos;

                            if offset.magnitude()
                                > Self::TRANSLATE_OFFSET_THRESHOLD / engine_view.camera.total_zoom()
                            {
                                // move text
                                engine_view.store.translate_strokes(&[*stroke_key], offset);
                                engine_view
                                    .store
                                    .translate_strokes_images(&[*stroke_key], offset);
                                *current_pos += offset;

                                widget_flags.store_modified = true;
                            }

                            // possibly nudge camera
                            widget_flags |= engine_view
                                .camera
                                .nudge_w_pos(element.pos, engine_view.document);
                            widget_flags |= engine_view
                                .document
                                .expand_autoexpand(engine_view.camera, engine_view.store);
                            engine_view.store.regenerate_rendering_in_viewport_threaded(
                                engine_view.tasks_tx.clone(),
                                false,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::InProgress,
                        }
                    }
                    ModifyState::AdjustTextWidth {
                        start_text_width,
                        start_pos,
                        current_pos,
                    } => {
                        let x_offset = element.pos[0] - current_pos[0];

                        if let Some(Stroke::TextStroke(textstroke)) =
                            engine_view.store.get_stroke_mut(*stroke_key)
                        {
                            if x_offset.abs()
                                > Self::ADJ_TEXT_WIDTH_THRESHOLD / engine_view.camera.total_zoom()
                            {
                                let new_text_width =
                                    *start_text_width + (element.pos[0] - start_pos[0]);
                                engine_view
                                    .pens_config
                                    .typewriter_config
                                    .set_text_width(new_text_width);
                                textstroke.text_style.set_max_width(Some(new_text_width));
                                engine_view.store.regenerate_rendering_for_stroke(
                                    *stroke_key,
                                    engine_view.camera.viewport(),
                                    engine_view.camera.image_scale(),
                                );

                                *current_pos = element.pos;

                                widget_flags.store_modified = true;
                            }
                        }

                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::InProgress,
                        }
                    }
                }
            }
        };

        (event_result, widget_flags)
    }

    pub(super) fn handle_pen_event_up(
        &mut self,
        element: Element,
        _modifier_keys: HashSet<ModifierKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();
        let typewriter_bounds = self.bounds_on_doc(&engine_view.as_im());

        let event_result = match &mut self.state {
            TypewriterState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            TypewriterState::Start(_) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                pen_down,
                ..
            } => {
                *pen_down = false;

                match modify_state {
                    ModifyState::Up | ModifyState::Hover(_) => {
                        // detect hover state
                        *modify_state = if typewriter_bounds
                            .map(|b| b.contains_local_point(&element.pos.into()))
                            .unwrap_or(false)
                        {
                            ModifyState::Hover(element.pos)
                        } else {
                            ModifyState::Up
                        }
                    }
                    ModifyState::Selecting { finished, .. } => {
                        // finished when drag ended
                        *finished = true;
                    }
                    ModifyState::Translating { .. } => {
                        engine_view
                            .store
                            .update_geometry_for_strokes(&[*stroke_key]);
                        engine_view.store.regenerate_rendering_for_stroke(
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );
                        widget_flags |= engine_view
                            .document
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        self.state = TypewriterState::Modifying {
                            modify_state: ModifyState::Up,
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            pen_down: false,
                        };

                        widget_flags |= engine_view.store.record(Instant::now());
                        widget_flags.store_modified = true;
                    }
                    ModifyState::AdjustTextWidth { .. } => {
                        engine_view
                            .store
                            .update_geometry_for_strokes(&[*stroke_key]);
                        engine_view.store.regenerate_rendering_for_stroke(
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );
                        widget_flags |= engine_view
                            .document
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        self.state = TypewriterState::Modifying {
                            modify_state: ModifyState::Up,
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            pen_down: false,
                        };

                        widget_flags |= engine_view.store.record(Instant::now());
                        widget_flags.store_modified = true;
                    }
                }

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
        };

        (event_result, widget_flags)
    }

    pub(super) fn handle_pen_event_proximity(
        &mut self,
        element: Element,
        _modifier_keys: HashSet<ModifierKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let widget_flags = WidgetFlags::default();
        let typewriter_bounds = self.bounds_on_doc(&engine_view.as_im());

        let event_result = match &mut self.state {
            TypewriterState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            TypewriterState::Start(_) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            TypewriterState::Modifying {
                modify_state,
                pen_down,
                ..
            } => {
                // detect hover state
                *modify_state = if typewriter_bounds
                    .map(|b| b.contains_local_point(&element.pos.into()))
                    .unwrap_or(false)
                {
                    ModifyState::Hover(element.pos)
                } else {
                    ModifyState::Up
                };
                *pen_down = false;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
        };

        (event_result, widget_flags)
    }

    pub(super) fn handle_pen_event_keypressed(
        &mut self,
        keyboard_key: KeyboardKey,
        modifier_keys: HashSet<ModifierKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let text_width = engine_view.pens_config.typewriter_config.text_width();
        let mut text_style = engine_view.pens_config.typewriter_config.text_style.clone();

        let event_result = match &mut self.state {
            TypewriterState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            TypewriterState::Start(pos) => {
                super::play_sound(Some(keyboard_key), engine_view.audioplayer);

                match keyboard_key {
                    KeyboardKey::Unicode(keychar) => {
                        text_style.ranged_text_attributes.clear();
                        text_style.set_max_width(Some(text_width));
                        let textstroke = TextStroke::new(String::from(keychar), *pos, text_style);
                        let mut cursor = GraphemeCursor::new(0, textstroke.text.len(), true);

                        textstroke.move_cursor_forward(&mut cursor);
                        let stroke_key = engine_view
                            .store
                            .insert_stroke(Stroke::TextStroke(textstroke), None);
                        widget_flags |= engine_view
                            .document
                            .resize_autoexpand(engine_view.store, engine_view.camera);
                        engine_view.store.regenerate_rendering_for_stroke(
                            stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        self.state = TypewriterState::Modifying {
                            modify_state: ModifyState::Up,
                            stroke_key,
                            cursor,
                            pen_down: false,
                        };

                        widget_flags |= engine_view.store.record(Instant::now());
                        widget_flags.store_modified = true;

                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::InProgress,
                        }
                    }
                    _ => EventResult {
                        handled: false,
                        propagate: EventPropagation::Proceed,
                        progress: PenProgress::InProgress,
                    },
                }
            }
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                pen_down,
            } => {
                match modify_state {
                    ModifyState::Up | ModifyState::Hover(_) => {
                        super::play_sound(Some(keyboard_key), engine_view.audioplayer);

                        if let Some(Stroke::TextStroke(ref mut textstroke)) =
                            engine_view.store.get_stroke_mut(*stroke_key)
                        {
                            let mut update_stroke =
                                |store: &mut StrokeStore, keychar_is_whitespace: bool| {
                                    store.update_geometry_for_stroke(*stroke_key);
                                    store.regenerate_rendering_for_stroke(
                                        *stroke_key,
                                        engine_view.camera.viewport(),
                                        engine_view.camera.image_scale(),
                                    );
                                    widget_flags |= engine_view
                                        .document
                                        .resize_autoexpand(store, engine_view.camera);
                                    if keychar_is_whitespace {
                                        widget_flags |= store.record(Instant::now());
                                    } else {
                                        widget_flags |=
                                            store.update_latest_history_entry(Instant::now());
                                    }

                                    widget_flags.store_modified = true;
                                };

                            *pen_down = false;

                            // Handling keyboard input
                            match keyboard_key {
                                KeyboardKey::Unicode(keychar) => {
                                    if keychar == 'a'
                                        && modifier_keys.contains(&ModifierKey::KeyboardCtrl)
                                    {
                                        cursor.set_cursor(textstroke.text.len());
                                        // Select entire text
                                        *modify_state = ModifyState::Selecting {
                                            selection_cursor: GraphemeCursor::new(
                                                0,
                                                textstroke.text.len(),
                                                true,
                                            ),
                                            finished: true,
                                        };
                                    } else {
                                        textstroke.insert_text_after_cursor(
                                            keychar.to_string().as_str(),
                                            cursor,
                                        );
                                        update_stroke(engine_view.store, keychar.is_whitespace());
                                    }

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::BackSpace => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                        textstroke.remove_word_before_cursor(cursor);
                                    } else {
                                        textstroke.remove_grapheme_before_cursor(cursor);
                                    }
                                    update_stroke(engine_view.store, false);

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::HorizontalTab => {
                                    textstroke.insert_text_after_cursor("\t", cursor);
                                    update_stroke(engine_view.store, false);

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::CarriageReturn | KeyboardKey::Linefeed => {
                                    textstroke.insert_text_after_cursor("\n", cursor);
                                    update_stroke(engine_view.store, true);

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::Delete => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                        textstroke.remove_word_after_cursor(cursor);
                                    } else {
                                        textstroke.remove_grapheme_after_cursor(cursor);
                                    }
                                    update_stroke(engine_view.store, false);

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::NavLeft => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        let old_cursor = cursor.clone();
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_word_back(cursor);
                                        } else {
                                            textstroke.move_cursor_back(cursor);
                                        }

                                        *modify_state = ModifyState::Selecting {
                                            selection_cursor: old_cursor,
                                            finished: false,
                                        }
                                    } else {
                                        #[allow(clippy::collapsible_else_if)]
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_word_back(cursor);
                                        } else {
                                            textstroke.move_cursor_back(cursor);
                                        }
                                    }

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::NavRight => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        let old_cursor = cursor.clone();
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_word_forward(cursor);
                                        } else {
                                            textstroke.move_cursor_forward(cursor);
                                        }

                                        *modify_state = ModifyState::Selecting {
                                            selection_cursor: old_cursor,
                                            finished: false,
                                        };
                                    } else {
                                        #[allow(clippy::collapsible_else_if)]
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_word_forward(cursor);
                                        } else {
                                            textstroke.move_cursor_forward(cursor);
                                        }
                                    }

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::NavUp => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        let old_cursor = cursor.clone();
                                        textstroke.move_cursor_line_up(cursor);

                                        *modify_state = ModifyState::Selecting {
                                            selection_cursor: old_cursor,
                                            finished: false,
                                        };
                                    } else {
                                        textstroke.move_cursor_line_up(cursor);
                                    }

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::NavDown => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        let old_cursor = cursor.clone();
                                        textstroke.move_cursor_line_down(cursor);

                                        *modify_state = ModifyState::Selecting {
                                            selection_cursor: old_cursor,
                                            finished: false,
                                        };
                                    } else {
                                        textstroke.move_cursor_line_down(cursor);
                                    }

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::Home => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        let old_cursor = cursor.clone();
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_text_start(cursor);
                                        } else {
                                            textstroke.move_cursor_line_start(cursor);
                                        }

                                        *modify_state = ModifyState::Selecting {
                                            selection_cursor: old_cursor,
                                            finished: false,
                                        };
                                    } else {
                                        #[allow(clippy::collapsible_else_if)]
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_text_start(cursor);
                                        } else {
                                            textstroke.move_cursor_line_start(cursor);
                                        }
                                    }

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::End => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        let old_cursor = cursor.clone();
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_text_end(cursor);
                                        } else {
                                            textstroke.move_cursor_line_end(cursor);
                                        }

                                        *modify_state = ModifyState::Selecting {
                                            selection_cursor: old_cursor,
                                            finished: false,
                                        };
                                    } else {
                                        #[allow(clippy::collapsible_else_if)]
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_text_end(cursor);
                                        } else {
                                            textstroke.move_cursor_line_end(cursor);
                                        }
                                    }

                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                _ => EventResult {
                                    handled: false,
                                    propagate: EventPropagation::Proceed,
                                    progress: PenProgress::InProgress,
                                },
                            }
                        } else {
                            EventResult {
                                handled: false,
                                propagate: EventPropagation::Proceed,
                                progress: PenProgress::InProgress,
                            }
                        }
                    }
                    ModifyState::Selecting {
                        selection_cursor,
                        finished,
                    } => {
                        super::play_sound(Some(keyboard_key), engine_view.audioplayer);

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
                                widget_flags |= engine_view
                                    .document
                                    .resize_autoexpand(store, engine_view.camera)
                                    | store.record(Instant::now());
                                widget_flags.store_modified = true;
                            };
                            let mut quit_selecting = false;

                            // Handle keyboard keys
                            let event_result = match keyboard_key {
                                KeyboardKey::Unicode(keychar) => {
                                    if keychar == 'a'
                                        && modifier_keys.contains(&ModifierKey::KeyboardCtrl)
                                    {
                                        textstroke
                                            .update_selection_entire_text(cursor, selection_cursor);
                                        *finished = true;
                                    } else {
                                        textstroke.replace_text_between_selection_cursors(
                                            cursor,
                                            selection_cursor,
                                            String::from(keychar).as_str(),
                                        );
                                        update_stroke(engine_view.store);
                                        quit_selecting = true;
                                    }
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::NavLeft => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_word_back(cursor);
                                        } else {
                                            textstroke.move_cursor_back(cursor);
                                        }
                                    } else {
                                        cursor.set_cursor(
                                            cursor.cur_cursor().min(selection_cursor.cur_cursor()),
                                        );
                                        quit_selecting = true;
                                    }
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::NavRight => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                            textstroke.move_cursor_word_forward(cursor);
                                        } else {
                                            textstroke.move_cursor_forward(cursor);
                                        }
                                    } else {
                                        cursor.set_cursor(
                                            cursor.cur_cursor().max(selection_cursor.cur_cursor()),
                                        );
                                        quit_selecting = true;
                                    }
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::NavUp => {
                                    textstroke.move_cursor_line_up(cursor);
                                    if !modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        quit_selecting = true;
                                    }
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::NavDown => {
                                    textstroke.move_cursor_line_down(cursor);
                                    if !modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        quit_selecting = true;
                                    }
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::Home => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                        textstroke.move_cursor_text_start(cursor);
                                    } else {
                                        textstroke.move_cursor_line_start(cursor);
                                    }
                                    if !modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        quit_selecting = true;
                                    }
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::End => {
                                    if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                                        textstroke.move_cursor_text_end(cursor);
                                    } else {
                                        textstroke.move_cursor_line_end(cursor);
                                    }
                                    if !modifier_keys.contains(&ModifierKey::KeyboardShift) {
                                        quit_selecting = true;
                                    }
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::CarriageReturn | KeyboardKey::Linefeed => {
                                    textstroke.replace_text_between_selection_cursors(
                                        cursor,
                                        selection_cursor,
                                        "\n",
                                    );
                                    update_stroke(engine_view.store);
                                    quit_selecting = true;
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::BackSpace | KeyboardKey::Delete => {
                                    textstroke.replace_text_between_selection_cursors(
                                        cursor,
                                        selection_cursor,
                                        "",
                                    );
                                    update_stroke(engine_view.store);
                                    quit_selecting = true;
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::HorizontalTab => {
                                    textstroke.replace_text_between_selection_cursors(
                                        cursor,
                                        selection_cursor,
                                        "\t",
                                    );
                                    update_stroke(engine_view.store);
                                    quit_selecting = true;
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                                KeyboardKey::CtrlLeft
                                | KeyboardKey::CtrlRight
                                | KeyboardKey::ShiftLeft
                                | KeyboardKey::ShiftRight => EventResult {
                                    handled: false,
                                    propagate: EventPropagation::Proceed,
                                    progress: PenProgress::InProgress,
                                },
                                _ => {
                                    quit_selecting = true;
                                    EventResult {
                                        handled: true,
                                        propagate: EventPropagation::Stop,
                                        progress: PenProgress::InProgress,
                                    }
                                }
                            };

                            if quit_selecting {
                                self.state = TypewriterState::Modifying {
                                    modify_state: ModifyState::Up,
                                    stroke_key: *stroke_key,
                                    cursor: cursor.clone(),
                                    pen_down: false,
                                };
                            }

                            event_result
                        } else {
                            EventResult {
                                handled: false,
                                propagate: EventPropagation::Proceed,
                                progress: PenProgress::InProgress,
                            }
                        }
                    }
                    _ => EventResult {
                        handled: false,
                        propagate: EventPropagation::Proceed,
                        progress: PenProgress::InProgress,
                    },
                }
            }
        };

        self.reset_blink();

        (event_result, widget_flags)
    }

    pub(super) fn handle_pen_event_text(
        &mut self,
        text: String,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let text_width = engine_view.pens_config.typewriter_config.text_width();
        let mut text_style = engine_view.pens_config.typewriter_config.text_style.clone();

        self.reset_blink();

        let event_result = match &mut self.state {
            TypewriterState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            TypewriterState::Start(pos) => {
                super::play_sound(None, engine_view.audioplayer);

                text_style.ranged_text_attributes.clear();
                text_style.set_max_width(Some(text_width));
                let text_len = text.len();
                let textstroke = TextStroke::new(text, *pos, text_style);
                let cursor = GraphemeCursor::new(text_len, text_len, true);

                let stroke_key = engine_view
                    .store
                    .insert_stroke(Stroke::TextStroke(textstroke), None);
                engine_view.store.regenerate_rendering_for_stroke(
                    stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.state = TypewriterState::Modifying {
                    modify_state: ModifyState::Up,
                    stroke_key,
                    cursor,
                    pen_down: false,
                };

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.resize = true;
                widget_flags.store_modified = true;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                pen_down,
            } => {
                match modify_state {
                    ModifyState::Up | ModifyState::Hover(_) => {
                        super::play_sound(None, engine_view.audioplayer);

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
                            widget_flags |= engine_view
                                .document
                                .resize_autoexpand(engine_view.store, engine_view.camera);

                            *pen_down = false;

                            // only record new history entry if the text contains ascii-whitespace,
                            // else only update history
                            if text.contains(char::is_whitespace) {
                                widget_flags |= engine_view.store.record(Instant::now());
                            } else {
                                widget_flags |= engine_view
                                    .store
                                    .update_latest_history_entry(Instant::now());
                            }

                            widget_flags.store_modified = true;
                        }

                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::InProgress,
                        }
                    }
                    ModifyState::Selecting {
                        selection_cursor,
                        finished,
                    } => {
                        super::play_sound(None, engine_view.audioplayer);

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
                            widget_flags |= engine_view
                                .document
                                .resize_autoexpand(engine_view.store, engine_view.camera);

                            *finished = true;

                            // only record new history entry if the text contains ascii-whitespace,
                            // else only update history
                            if text.contains(char::is_whitespace) {
                                widget_flags |= engine_view.store.record(Instant::now());
                            } else {
                                widget_flags |= engine_view
                                    .store
                                    .update_latest_history_entry(Instant::now());
                            }
                            widget_flags.store_modified = true;
                        }

                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::InProgress,
                        }
                    }
                    _ => EventResult {
                        handled: false,
                        propagate: EventPropagation::Proceed,
                        progress: PenProgress::InProgress,
                    },
                }
            }
        };

        (event_result, widget_flags)
    }

    pub(super) fn handle_pen_event_cancel(
        &mut self,
        _now: Instant,
        _engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let widget_flags = WidgetFlags::default();

        let event_result = match &mut self.state {
            TypewriterState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            _ => {
                self.state = TypewriterState::Idle;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
        };

        (event_result, widget_flags)
    }
}
