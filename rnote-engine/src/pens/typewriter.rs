use std::ops::Range;

use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;
use rnote_compose::helpers::{AABBHelpers, Vector2Helpers};
use rnote_compose::penhelpers::{KeyboardKey, PenEvent, PenState, ShortcutKey};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::drawhelpers;
use rnote_compose::{color, Transform};
use serde::{Deserialize, Serialize};

use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::strokes::textstroke::{RangedTextAttribute, TextAlignment, TextAttribute, TextStyle};
use crate::strokes::{Stroke, TextStroke};
use crate::{Camera, Document, DrawOnDocBehaviour, StrokeStore, SurfaceFlags};

use super::penbehaviour::PenProgress;
use super::PenBehaviour;

#[derive(Debug, Clone)]
pub enum TypewriterState {
    Idle,
    Start(na::Vector2<f64>),
    Modifying {
        stroke_key: StrokeKey,
        cursor: unicode_segmentation::GraphemeCursor,
        pen_down: bool,
    },
    Selecting {
        stroke_key: StrokeKey,
        cursor: unicode_segmentation::GraphemeCursor,
        selection_cursor: unicode_segmentation::GraphemeCursor,
        /// If selecting is finished ( if true, will get resetted on the next click )
        finished: bool,
    },
    Translating {
        stroke_key: StrokeKey,
        cursor: unicode_segmentation::GraphemeCursor,
        start_pos: na::Vector2<f64>,
        current_pos: na::Vector2<f64>,
    },
    AdjustTextWidth {
        stroke_key: StrokeKey,
        cursor: unicode_segmentation::GraphemeCursor,
        start_text_width: f64,
        start_pos: na::Vector2<f64>,
        current_pos: na::Vector2<f64>,
    },
}

impl Default for TypewriterState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "typewriter")]
pub struct Typewriter {
    #[serde(rename = "text_style")]
    pub text_style: TextStyle,
    #[serde(rename = "max_width_enabled")]
    pub max_width_enabled: bool,
    #[serde(rename = "text_width")]
    pub text_width: f64,

    #[serde(skip)]
    state: TypewriterState,
}

impl Default for Typewriter {
    fn default() -> Self {
        Self {
            text_style: TextStyle::default(),
            max_width_enabled: true,
            text_width: 600.0,

            state: TypewriterState::default(),
        }
    }
}

impl DrawOnDocBehaviour for Typewriter {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<AABB> {
        let total_zoom = engine_view.camera.total_zoom();

        match &self.state {
            TypewriterState::Idle => None,
            TypewriterState::Start(pos) => Some(AABB::new(
                na::Point2::from(*pos),
                na::Point2::from(pos + na::vector![self.text_width, self.text_style.font_size]),
            )),
            TypewriterState::Modifying { stroke_key, .. }
            | TypewriterState::Selecting { stroke_key, .. }
            | TypewriterState::Translating { stroke_key, .. }
            | TypewriterState::AdjustTextWidth { stroke_key, .. } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    let text_rect = Self::text_rect_bounds(self.text_width, textstroke);

                    let typewriter_bounds = text_rect.extend_by(
                        Self::TRANSLATE_NODE_SIZE.maxs(&Self::ADJUST_TEXT_WIDTH_NODE_SIZE)
                            / total_zoom,
                    );

                    Some(typewriter_bounds)
                } else {
                    None
                }
            }
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        const OUTLINE_COLOR: piet::Color = color::GNOME_BRIGHTS[4].with_a8(0xf0);
        let total_zoom = engine_view.camera.total_zoom();

        let outline_width = 1.5 / total_zoom;
        let outline_corner_radius = 3.0 / total_zoom;

        match &self.state {
            TypewriterState::Idle => {}
            TypewriterState::Start(pos) => {
                if let Some(bounds) = self.bounds_on_doc(engine_view) {
                    let rect = bounds
                        .tightened(outline_width * 0.5)
                        .to_kurbo_rect()
                        .to_rounded_rect(outline_corner_radius);

                    cx.stroke(rect, &OUTLINE_COLOR, outline_width);

                    let text = String::from("|");
                    let text_len = text.len();

                    // Draw the cursor
                    self.text_style.draw_cursor(
                        cx,
                        text,
                        &unicode_segmentation::GraphemeCursor::new(0, text_len, true),
                        &Transform::new_w_isometry(na::Isometry2::new(*pos, 0.0)),
                        engine_view.camera,
                    )?;
                }
            }
            TypewriterState::Modifying {
                stroke_key, cursor, ..
            } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    let text_rect = Self::text_rect_bounds(self.text_width, textstroke);

                    let text_drawrect = text_rect
                        .tightened(outline_width * 0.5)
                        .to_kurbo_rect()
                        .to_rounded_rect(outline_corner_radius);

                    cx.stroke(text_drawrect, &OUTLINE_COLOR, outline_width);

                    // Draw the cursor
                    textstroke.text_style.draw_cursor(
                        cx,
                        textstroke.text.clone(),
                        cursor,
                        &textstroke.transform,
                        engine_view.camera,
                    )?;

                    // Draw the text width adjust node
                    drawhelpers::draw_triangular_down_node(
                        cx,
                        PenState::Up,
                        Self::adjust_text_width_node_center(
                            text_rect.mins.coords,
                            self.text_width,
                            engine_view.camera,
                        ),
                        Self::ADJUST_TEXT_WIDTH_NODE_SIZE / total_zoom,
                        total_zoom,
                    );

                    if let Some(typewriter_bounds) = self.bounds_on_doc(engine_view) {
                        // draw translate Node
                        drawhelpers::draw_rectangular_node(
                            cx,
                            PenState::Up,
                            Self::translate_node_bounds(typewriter_bounds, engine_view.camera),
                            total_zoom,
                        );
                    }
                }
            }
            TypewriterState::Selecting {
                stroke_key,
                cursor,
                selection_cursor,
                ..
            } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    let text_rect = Self::text_rect_bounds(self.text_width, textstroke);

                    let text_drawrect = text_rect
                        .tightened(outline_width * 0.5)
                        .to_kurbo_rect()
                        .to_rounded_rect(outline_corner_radius);

                    cx.stroke(text_drawrect, &OUTLINE_COLOR, outline_width);

                    // Draw the text selection
                    textstroke.text_style.draw_text_selection(
                        cx,
                        textstroke.text.clone(),
                        cursor,
                        selection_cursor,
                        &textstroke.transform,
                        engine_view.camera,
                    );

                    // Draw the cursor
                    textstroke.text_style.draw_cursor(
                        cx,
                        textstroke.text.clone(),
                        cursor,
                        &textstroke.transform,
                        engine_view.camera,
                    )?;

                    // Draw the text width adjust node
                    drawhelpers::draw_triangular_down_node(
                        cx,
                        PenState::Up,
                        Self::adjust_text_width_node_center(
                            text_rect.mins.coords,
                            self.text_width,
                            engine_view.camera,
                        ),
                        Self::ADJUST_TEXT_WIDTH_NODE_SIZE / total_zoom,
                        total_zoom,
                    );

                    if let Some(typewriter_bounds) = self.bounds_on_doc(engine_view) {
                        // draw translate Node
                        drawhelpers::draw_rectangular_node(
                            cx,
                            PenState::Up,
                            Self::translate_node_bounds(typewriter_bounds, engine_view.camera),
                            total_zoom,
                        );
                    }
                }
            }
            TypewriterState::Translating { stroke_key, .. } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    let text_rect = Self::text_rect_bounds(self.text_width, textstroke);

                    let text_drawrect = text_rect
                        .tightened(outline_width * 0.5)
                        .to_kurbo_rect()
                        .to_rounded_rect(outline_corner_radius);

                    cx.stroke(text_drawrect, &OUTLINE_COLOR, outline_width);

                    // Draw the text width adjust node
                    drawhelpers::draw_triangular_down_node(
                        cx,
                        PenState::Up,
                        Self::adjust_text_width_node_center(
                            text_rect.mins.coords,
                            self.text_width,
                            engine_view.camera,
                        ),
                        Self::ADJUST_TEXT_WIDTH_NODE_SIZE / total_zoom,
                        total_zoom,
                    );

                    // Translate Node
                    if let Some(typewriter_bounds) = self.bounds_on_doc(engine_view) {
                        drawhelpers::draw_rectangular_node(
                            cx,
                            PenState::Down,
                            Self::translate_node_bounds(typewriter_bounds, engine_view.camera),
                            total_zoom,
                        );
                    }
                }
            }
            TypewriterState::AdjustTextWidth { stroke_key, .. } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    let text_rect = Self::text_rect_bounds(self.text_width, textstroke);

                    let text_drawrect = text_rect
                        .tightened(outline_width * 0.5)
                        .to_kurbo_rect()
                        .to_rounded_rect(outline_corner_radius);

                    cx.stroke(text_drawrect, &OUTLINE_COLOR, outline_width);

                    // Draw the text width adjust node
                    drawhelpers::draw_triangular_down_node(
                        cx,
                        PenState::Down,
                        Self::adjust_text_width_node_center(
                            text_rect.mins.coords,
                            self.text_width,
                            engine_view.camera,
                        ),
                        Self::ADJUST_TEXT_WIDTH_NODE_SIZE / total_zoom,
                        total_zoom,
                    );

                    // Translate Node
                    if let Some(typewriter_bounds) = self.bounds_on_doc(engine_view) {
                        drawhelpers::draw_rectangular_node(
                            cx,
                            PenState::Up,
                            Self::translate_node_bounds(typewriter_bounds, engine_view.camera),
                            total_zoom,
                        );
                    }
                }
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

impl PenBehaviour for Typewriter {
    fn handle_event(
        &mut self,
        event: PenEvent,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, SurfaceFlags) {
        //log::debug!("typewriter handle_event: state: {:#?}, event: {:#?}", self.state, event);

        let mut surface_flags = SurfaceFlags::default();

        let typewriter_bounds = self.bounds_on_doc(&engine_view.as_im());

        let pen_progress = match (&mut self.state, event) {
            (
                TypewriterState::Idle | TypewriterState::Start { .. },
                PenEvent::Down { element, .. },
            ) => {
                let mut new_state = TypewriterState::Start(element.pos);

                if let Some(stroke_key) = engine_view
                    .store
                    .query_stroke_hitboxes_contain_coord(engine_view.camera.viewport(), element.pos)
                {
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
                    }
                }

                self.state = new_state;

                PenProgress::InProgress
            }
            (TypewriterState::Idle, _) => PenProgress::Idle,
            (TypewriterState::Start(_), PenEvent::Proximity { .. } | PenEvent::Up { .. }) => {
                PenProgress::InProgress
            }
            (TypewriterState::Start(pos), PenEvent::KeyPressed { keyboard_key, .. }) => {
                match keyboard_key {
                    KeyboardKey::Unicode(keychar) => {
                        surface_flags.merge_with_other(engine_view.store.record());

                        let mut text_style = self.text_style.clone();
                        if self.max_width_enabled {
                            text_style.max_width = Some(self.text_width);
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
                            .insert_stroke(Stroke::TextStroke(textstroke));

                        if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                            stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_stroke() after inserting a new textstroke failed with Err {}", e);
                        }

                        self.state = TypewriterState::Modifying {
                            stroke_key,
                            cursor,
                            pen_down: false,
                        };
                    }
                    _ => {}
                }

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (TypewriterState::Start(_), PenEvent::Cancel) => {
                self.state = TypewriterState::Idle;

                surface_flags.redraw = true;

                PenProgress::Finished
            }
            (
                TypewriterState::Modifying {
                    stroke_key,
                    cursor,
                    pen_down,
                },
                PenEvent::Down { element, .. },
            ) => {
                let mut pen_progress = PenProgress::InProgress;

                if let (Some(typewriter_bounds), Some(Stroke::TextStroke(textstroke))) = (
                    typewriter_bounds,
                    engine_view.store.get_stroke_ref(*stroke_key),
                ) {
                    if Self::translate_node_bounds(typewriter_bounds, engine_view.camera)
                        .contains_local_point(&na::Point2::from(element.pos))
                    {
                        // switch to translating the text field
                        surface_flags.merge_with_other(engine_view.store.record());

                        self.state = TypewriterState::Translating {
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            start_pos: element.pos,
                            current_pos: element.pos,
                        };
                    } else if Self::adjust_text_width_node_bounds(
                        Self::text_rect_bounds(self.text_width, textstroke)
                            .mins
                            .coords,
                        self.text_width,
                        engine_view.camera,
                    )
                    .contains_local_point(&na::Point2::from(element.pos))
                    {
                        surface_flags.merge_with_other(engine_view.store.record());

                        // Clicking on the adjust text width node
                        self.state = TypewriterState::AdjustTextWidth {
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            start_text_width: self.text_width,
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

                surface_flags.redraw = true;

                pen_progress
            }
            (
                TypewriterState::Modifying { pen_down, .. },
                PenEvent::Proximity { .. } | PenEvent::Up { .. },
            ) => {
                *pen_down = false;
                PenProgress::InProgress
            }
            (
                TypewriterState::Modifying {
                    stroke_key,
                    cursor,
                    pen_down: down,
                },
                PenEvent::KeyPressed {
                    keyboard_key,
                    shortcut_keys,
                },
            ) => {
                //log::debug!("key: {:?}", keyboard_key);
                if let Some(Stroke::TextStroke(ref mut textstroke)) =
                    engine_view.store.get_stroke_mut(*stroke_key)
                {
                    let mut update_stroke = |store: &mut StrokeStore| {
                        surface_flags.merge_with_other(store.record());

                        store.update_geometry_for_stroke(*stroke_key);
                        store.regenerate_rendering_for_stroke_threaded(
                            engine_view.tasks_tx.clone(),
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        engine_view.doc.resize_autoexpand(store, engine_view.camera);

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.store_changed = true;
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
                        KeyboardKey::Linefeed => {
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

                    *down = false;

                    surface_flags.redraw = true;

                    if let Some(new_state) = new_state {
                        self.state = new_state;
                    }
                }

                PenProgress::InProgress
            }
            (TypewriterState::Modifying { .. }, PenEvent::Cancel) => {
                self.state = TypewriterState::Idle;

                surface_flags.redraw = true;

                PenProgress::Finished
            }
            (
                TypewriterState::Selecting {
                    stroke_key,
                    cursor,
                    finished,
                    ..
                },
                PenEvent::Down { element, .. },
            ) => {
                let mut pen_progress = PenProgress::InProgress;

                if let Some(typewriter_bounds) = typewriter_bounds {
                    // Clicking on the translate node
                    if Self::translate_node_bounds(typewriter_bounds, engine_view.camera)
                        .contains_local_point(&na::Point2::from(element.pos))
                    {
                        surface_flags.merge_with_other(engine_view.store.record());

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

                surface_flags.redraw = true;

                pen_progress
            }
            (TypewriterState::Selecting { finished, .. }, PenEvent::Up { .. }) => {
                // finished when drag ended
                *finished = true;

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (
                TypewriterState::Selecting {
                    stroke_key,
                    cursor,
                    selection_cursor,
                    finished,
                },
                PenEvent::KeyPressed {
                    keyboard_key,
                    shortcut_keys,
                },
            ) => {
                //log::debug!("key: {:?}", keyboard_key);

                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_mut(*stroke_key)
                {
                    let mut update_stroke = |store: &mut StrokeStore| {
                        surface_flags.merge_with_other(store.record());

                        store.update_geometry_for_stroke(*stroke_key);
                        store.regenerate_rendering_for_stroke_threaded(
                            engine_view.tasks_tx.clone(),
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        engine_view.doc.resize_autoexpand(store, engine_view.camera);

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.store_changed = true;
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
                        KeyboardKey::Linefeed => {
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

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (TypewriterState::Selecting { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }

            (TypewriterState::Selecting { .. }, PenEvent::Cancel) => {
                self.state = TypewriterState::Idle;

                surface_flags.redraw = true;

                PenProgress::Finished
            }
            (
                TypewriterState::Translating {
                    stroke_key,
                    current_pos,
                    ..
                },
                PenEvent::Down { element, .. },
            ) => {
                let offset = element.pos - *current_pos;

                if offset.magnitude()
                    > Self::TRANSLATE_MAGNITUDE_THRESHOLD / engine_view.camera.total_zoom()
                {
                    engine_view.store.translate_strokes(&[*stroke_key], offset);
                    engine_view
                        .store
                        .translate_strokes_images(&[*stroke_key], offset);

                    if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                        *stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    ) {
                        log::error!("regenerate_rendering_for_stroke() while translating textstroke failed with Err {}", e);
                    }

                    *current_pos = element.pos;

                    surface_flags.redraw = true;
                    surface_flags.store_changed = true;
                }

                PenProgress::InProgress
            }
            (
                TypewriterState::Translating {
                    stroke_key, cursor, ..
                },
                PenEvent::Up { .. },
            ) => {
                engine_view
                    .store
                    .update_geometry_for_strokes(&[*stroke_key]);
                if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                    *stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                ) {
                    log::error!("regenerate_rendering_for_stroke() while translating textstroke failed with Err {}", e);
                }

                self.state = TypewriterState::Modifying {
                    stroke_key: *stroke_key,
                    cursor: cursor.clone(),
                    pen_down: false,
                };

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.store_changed = true;

                PenProgress::InProgress
            }
            (
                TypewriterState::Translating { .. },
                PenEvent::Proximity { .. } | PenEvent::KeyPressed { .. },
            ) => PenProgress::InProgress,
            (TypewriterState::Translating { .. }, PenEvent::Cancel) => {
                self.state = TypewriterState::Idle;

                PenProgress::Finished
            }
            (
                TypewriterState::AdjustTextWidth {
                    stroke_key,
                    start_text_width,
                    start_pos,
                    current_pos,
                    ..
                },
                PenEvent::Down { element, .. },
            ) => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_mut(*stroke_key)
                {
                    let abs_x_offset = element.pos[0] - start_pos[0];

                    self.text_width = (*start_text_width + abs_x_offset).max(2.0);

                    if let Some(max_width) = &mut textstroke.text_style.max_width {
                        *max_width = *start_text_width + abs_x_offset;
                    }
                }

                if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                    *stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                ) {
                    log::error!("regenerate_rendering_for_stroke() while adjusting text width textstroke failed with Err {}", e);
                }

                *current_pos = element.pos;

                surface_flags.redraw = true;
                surface_flags.store_changed = true;

                PenProgress::InProgress
            }
            (
                TypewriterState::AdjustTextWidth {
                    stroke_key, cursor, ..
                },
                PenEvent::Up { .. },
            ) => {
                engine_view
                    .store
                    .update_geometry_for_strokes(&[*stroke_key]);
                if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                    *stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                ) {
                    log::error!("regenerate_rendering_for_stroke() while adjusting textstroke text width failed with Err {}", e);
                }

                self.state = TypewriterState::Modifying {
                    stroke_key: *stroke_key,
                    cursor: cursor.clone(),
                    pen_down: false,
                };

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.store_changed = true;

                PenProgress::InProgress
            }
            (
                TypewriterState::AdjustTextWidth { .. },
                PenEvent::Proximity { .. } | PenEvent::KeyPressed { .. },
            ) => PenProgress::InProgress,
            (TypewriterState::AdjustTextWidth { .. }, PenEvent::Cancel) => {
                self.state = TypewriterState::Idle;

                surface_flags.redraw = true;

                PenProgress::Finished
            }
        };

        (pen_progress, surface_flags)
    }

    fn paste_clipboard_content(
        &mut self,
        clipboard_content: &[u8],
        mime_types: Vec<String>,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, SurfaceFlags) {
        let mut surface_flags = SurfaceFlags::default();

        let pen_progress = match &mut self.state {
            TypewriterState::Start(pos) => {
                if mime_types
                    .iter()
                    .any(|mime_type| mime_type.contains("text/plain"))
                {
                    if let Ok(clipboard_text) = String::from_utf8(clipboard_content.to_vec()) {
                        let text_len = clipboard_text.len();

                        surface_flags.merge_with_other(engine_view.store.record());

                        let mut text_style = self.text_style.clone();
                        if self.max_width_enabled {
                            text_style.max_width = Some(self.text_width);
                        }

                        let textstroke = TextStroke::new(clipboard_text, *pos, text_style);

                        let cursor = unicode_segmentation::GraphemeCursor::new(
                            text_len,
                            textstroke.text.len(),
                            true,
                        );

                        let stroke_key = engine_view
                            .store
                            .insert_stroke(Stroke::TextStroke(textstroke));

                        if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                            stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_stroke() after inserting a new textstroke from clipboard contents in typewriter paste_clipboard_contents() failed with Err {}", e);
                        }

                        self.state = TypewriterState::Modifying {
                            stroke_key,
                            cursor,
                            pen_down: false,
                        };

                        surface_flags.redraw = true;
                    }
                }

                PenProgress::InProgress
            }
            TypewriterState::Modifying {
                stroke_key, cursor, ..
            } => {
                if mime_types
                    .iter()
                    .any(|mime_type| mime_type.contains("text/plain"))
                {
                    surface_flags.merge_with_other(engine_view.store.record());

                    if let (Some(Stroke::TextStroke(textstroke)), Ok(clipboard_text)) = (
                        engine_view.store.get_stroke_mut(*stroke_key),
                        String::from_utf8(clipboard_content.to_vec()),
                    ) {
                        textstroke.insert_text_after_cursor(clipboard_text.as_str(), cursor);

                        engine_view.store.update_geometry_for_stroke(*stroke_key);
                        engine_view.store.regenerate_rendering_for_stroke_threaded(
                            engine_view.tasks_tx.clone(),
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        engine_view
                            .doc
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.store_changed = true;
                    }
                }

                PenProgress::InProgress
            }
            TypewriterState::Selecting {
                stroke_key,
                cursor,
                selection_cursor,
                ..
            } => {
                if mime_types
                    .iter()
                    .any(|mime_type| mime_type.contains("text/plain"))
                {
                    surface_flags.merge_with_other(engine_view.store.record());

                    if let (Some(Stroke::TextStroke(textstroke)), Ok(clipboard_text)) = (
                        engine_view.store.get_stroke_mut(*stroke_key),
                        String::from_utf8(clipboard_content.to_vec()),
                    ) {
                        textstroke.replace_text_between_selection_cursors(
                            cursor,
                            selection_cursor,
                            clipboard_text.as_str(),
                        );

                        engine_view.store.update_geometry_for_stroke(*stroke_key);
                        engine_view.store.regenerate_rendering_for_stroke_threaded(
                            engine_view.tasks_tx.clone(),
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );
                        engine_view
                            .doc
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        self.state = TypewriterState::Modifying {
                            stroke_key: *stroke_key,
                            cursor: cursor.clone(),
                            pen_down: false,
                        };

                        surface_flags.resize = true;
                        surface_flags.redraw = true;
                        surface_flags.store_changed = true;
                    }
                }

                PenProgress::InProgress
            }
            TypewriterState::Idle
            | TypewriterState::Translating { .. }
            | TypewriterState::AdjustTextWidth { .. } => {
                // Do nothing when
                PenProgress::InProgress
            }
        };

        (pen_progress, surface_flags)
    }

    fn update_internal_state(&mut self, engine_view: &EngineView) {
        match &mut self.state {
            TypewriterState::Idle | TypewriterState::Start(_) => {}
            TypewriterState::Selecting {
                stroke_key,
                cursor,
                selection_cursor,
                ..
            } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    update_cursors_for_textstroke(textstroke, cursor, Some(selection_cursor));
                }
            }
            TypewriterState::Modifying {
                stroke_key, cursor, ..
            }
            | TypewriterState::Translating {
                stroke_key, cursor, ..
            }
            | TypewriterState::AdjustTextWidth {
                stroke_key, cursor, ..
            } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    if let Some(max_width) = textstroke.text_style.max_width {
                        self.text_width = max_width;
                    }

                    update_cursors_for_textstroke(textstroke, cursor, None);
                }
            }
        }
    }
}

// Updates the cursors to valid positions and new text length.
fn update_cursors_for_textstroke(
    textstroke: &TextStroke,
    cursor: &mut unicode_segmentation::GraphemeCursor,
    selection_cursor: Option<&mut unicode_segmentation::GraphemeCursor>,
) {
    *cursor = unicode_segmentation::GraphemeCursor::new(
        cursor.cur_cursor().min(textstroke.text.len()),
        textstroke.text.len(),
        true,
    );
    if let Some(selection_cursor) = selection_cursor {
        *selection_cursor = unicode_segmentation::GraphemeCursor::new(
            selection_cursor.cur_cursor().min(textstroke.text.len()),
            textstroke.text.len(),
            true,
        );
    }
}

impl Typewriter {
    // The size of the translate node, located in the upper left corner
    const TRANSLATE_NODE_SIZE: na::Vector2<f64> = na::vector![18.0, 18.0];
    /// The threshold where a translation is applied ( in offset magnitude, surface coords )
    const TRANSLATE_MAGNITUDE_THRESHOLD: f64 = 1.0;
    // The size of the translate node, located in the upper left corner
    const ADJUST_TEXT_WIDTH_NODE_SIZE: na::Vector2<f64> = na::vector![18.0, 18.0];

    fn text_rect_bounds(text_width: f64, textstroke: &TextStroke) -> AABB {
        let origin = textstroke.transform.translation_part();

        AABB::new(
            na::Point2::from(origin),
            na::point![origin[0] + text_width, origin[1]],
        )
        .merged(&textstroke.bounds())
    }

    fn translate_node_bounds(typewriter_bounds: AABB, camera: &Camera) -> AABB {
        let total_zoom = camera.total_zoom();

        AABB::from_half_extents(
            na::Point2::from(
                typewriter_bounds.mins.coords + Self::TRANSLATE_NODE_SIZE * 0.5 / total_zoom,
            ),
            Self::TRANSLATE_NODE_SIZE * 0.5 / total_zoom,
        )
    }

    fn adjust_text_width_node_center(
        text_rect_origin: na::Vector2<f64>,
        text_width: f64,
        camera: &Camera,
    ) -> na::Vector2<f64> {
        let total_zoom = camera.total_zoom();

        na::vector![
            text_rect_origin[0] + text_width,
            text_rect_origin[1] - Self::ADJUST_TEXT_WIDTH_NODE_SIZE[1] * 0.5 / total_zoom
        ]
    }

    fn adjust_text_width_node_bounds(
        text_rect_origin: na::Vector2<f64>,
        text_width: f64,
        camera: &Camera,
    ) -> AABB {
        let total_zoom = camera.total_zoom();
        let center = Self::adjust_text_width_node_center(text_rect_origin, text_width, camera);

        AABB::from_half_extents(
            na::Point2::from(center),
            Self::ADJUST_TEXT_WIDTH_NODE_SIZE * 0.5 / total_zoom,
        )
    }

    /// Returns the range of the current selection, if available
    pub fn selection_range(&self) -> Option<(Range<usize>, StrokeKey)> {
        if let TypewriterState::Selecting {
            stroke_key,
            cursor,
            selection_cursor,
            ..
        } = &self.state
        {
            let cursor_index = cursor.cur_cursor();
            let selection_cursor_index = selection_cursor.cur_cursor();

            if cursor_index < selection_cursor_index {
                Some((cursor_index..selection_cursor_index, *stroke_key))
            } else {
                Some((selection_cursor_index..cursor_index, *stroke_key))
            }
        } else {
            None
        }
    }

    // Sets the alignment for the text stroke that is currently being modified
    pub fn set_alignment_modifying_stroke(
        &mut self,
        alignment: TextAlignment,
        _doc: &mut Document,
        store: &mut StrokeStore,
        camera: &Camera,
    ) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();

        if let TypewriterState::Modifying { stroke_key, .. }
        | TypewriterState::Selecting { stroke_key, .. }
        | TypewriterState::Translating { stroke_key, .. }
        | TypewriterState::AdjustTextWidth { stroke_key, .. } = &mut self.state
        {
            surface_flags.merge_with_other(store.record());

            if let Some(Stroke::TextStroke(textstroke)) = store.get_stroke_mut(*stroke_key) {
                textstroke.text_style.alignment = alignment;

                store.update_geometry_for_stroke(*stroke_key);
                if let Err(e) = store.regenerate_rendering_for_stroke(
                    *stroke_key,
                    camera.viewport(),
                    camera.image_scale(),
                ) {
                    log::error!("regenerate_rendering_for_stroke() failed with Err {}", e);
                }

                surface_flags.redraw = true;
                surface_flags.store_changed = true;
            }
        }

        surface_flags
    }

    pub fn remove_text_attributes_current_selection(
        &mut self,
        _doc: &mut Document,
        store: &mut StrokeStore,
        camera: &Camera,
    ) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();

        if let Some((selection_range, stroke_key)) = self.selection_range() {
            surface_flags.merge_with_other(store.record());

            if let Some(Stroke::TextStroke(textstroke)) = store.get_stroke_mut(stroke_key) {
                textstroke.remove_attrs_for_range(selection_range);

                store.update_geometry_for_stroke(stroke_key);
                if let Err(e) = store.regenerate_rendering_for_stroke(
                    stroke_key,
                    camera.viewport(),
                    camera.image_scale(),
                ) {
                    log::error!("regenerate_rendering_for_stroke() failed with Err {}", e);
                }

                surface_flags.redraw = true;
                surface_flags.store_changed = true;
            }
        }

        surface_flags
    }

    pub fn add_text_attribute_current_selection(
        &mut self,
        text_attribute: TextAttribute,
        _doc: &mut Document,
        store: &mut StrokeStore,
        camera: &Camera,
    ) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();

        if let Some((selection_range, stroke_key)) = self.selection_range() {
            surface_flags.merge_with_other(store.record());

            if let Some(Stroke::TextStroke(textstroke)) = store.get_stroke_mut(stroke_key) {
                textstroke
                    .text_style
                    .ranged_text_attributes
                    .push(RangedTextAttribute {
                        attribute: text_attribute,
                        range: selection_range,
                    });

                store.update_geometry_for_stroke(stroke_key);
                if let Err(e) = store.regenerate_rendering_for_stroke(
                    stroke_key,
                    camera.viewport(),
                    camera.image_scale(),
                ) {
                    log::error!("regenerate_rendering_for_stroke() failed with Err {}", e);
                }

                surface_flags.redraw = true;
                surface_flags.store_changed = true;
            }
        }

        surface_flags
    }
}
