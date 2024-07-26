// Modules
mod penevents;

// Imports
use super::pensconfig::TypewriterConfig;
use super::PenBehaviour;
use super::PenStyle;
use crate::engine::{EngineTask, EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::strokes::textstroke::{RangedTextAttribute, TextAttribute, TextStyle};
use crate::strokes::{Stroke, TextStroke};
use crate::{AudioPlayer, Camera, DrawableOnDoc, WidgetFlags};
use futures::channel::oneshot;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::ext::{AabbExt, Vector2Ext};
use rnote_compose::penevent::{KeyboardKey, PenEvent, PenProgress, PenState};
use rnote_compose::shapes::Shapeable;
use rnote_compose::style::indicators;
use rnote_compose::EventResult;
use rnote_compose::{color, Transform};
use std::ops::Range;
use std::time::{Duration, Instant};
use unicode_segmentation::GraphemeCursor;

#[derive(Debug, Clone)]
pub(super) enum ModifyState {
    Up,
    Hover(na::Vector2<f64>),
    Selecting {
        selection_cursor: GraphemeCursor,
        /// Whether selecting is finished.
        ///
        /// If true, the state will get reset on the next click.
        finished: bool,
    },
    Translating {
        current_pos: na::Vector2<f64>,
    },
    AdjustTextWidth {
        start_text_width: f64,
        start_pos: na::Vector2<f64>,
        current_pos: na::Vector2<f64>,
    },
}

#[derive(Debug, Clone)]
pub(super) enum TypewriterState {
    Idle,
    Start(na::Vector2<f64>),
    Modifying {
        modify_state: ModifyState,
        stroke_key: StrokeKey,
        cursor: GraphemeCursor,
        pen_down: bool,
    },
}

#[derive(Debug, Clone)]
pub struct Typewriter {
    state: TypewriterState,
    blink_task_handle: Option<crate::tasks::PeriodicTaskHandle>,
    cursor_visible: bool,
}

impl Default for Typewriter {
    fn default() -> Self {
        Self {
            state: TypewriterState::Idle,
            blink_task_handle: None,
            cursor_visible: true,
        }
    }
}

impl DrawableOnDoc for Typewriter {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        let total_zoom = engine_view.camera.total_zoom();

        match &self.state {
            TypewriterState::Idle => None,
            TypewriterState::Start(pos) => Some(Aabb::new(
                (*pos).into(),
                (pos + na::vector![
                    Self::STATE_START_TEXT_WIDTH,
                    engine_view
                        .pens_config
                        .typewriter_config
                        .text_style
                        .font_size
                ])
                .into(),
            )),
            TypewriterState::Modifying { stroke_key, .. } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    let text_rect = Self::text_rect_bounds(
                        engine_view.pens_config.typewriter_config.text_width(),
                        textstroke,
                    );
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
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let total_zoom = engine_view.camera.total_zoom();

        let draw_text_outline = |cx: &mut piet_cairo::CairoRenderContext, bounds: Aabb| {
            let stroke_width = Self::TEXT_OUTLINE_STROKE_WIDTH / total_zoom;

            cx.stroke(
                bounds.tightened(stroke_width * 0.5).to_kurbo_rect(),
                &Self::TEXT_OUTLINE_COLOR,
                stroke_width,
            );
        };

        match &self.state {
            TypewriterState::Idle => {}
            TypewriterState::Start(pos) => {
                if let Some(bounds) = self.bounds_on_doc(engine_view) {
                    // Draw the initial outline
                    draw_text_outline(cx, bounds);

                    // Draw the cursor
                    if self.cursor_visible {
                        let cursor_text = String::from('|');
                        let cursor_text_len = cursor_text.len();
                        engine_view
                            .pens_config
                            .typewriter_config
                            .text_style
                            .draw_cursor(
                                cx,
                                cursor_text,
                                &GraphemeCursor::new(0, cursor_text_len, true),
                                &Transform::new_w_isometry(na::Isometry2::new(*pos, 0.0)),
                                engine_view.camera,
                            )?;
                    }
                }
            }
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                ..
            } => {
                if let Some(Stroke::TextStroke(textstroke)) =
                    engine_view.store.get_stroke_ref(*stroke_key)
                {
                    let text_width = engine_view.pens_config.typewriter_config.text_width();
                    let text_bounds = Self::text_rect_bounds(text_width, textstroke);

                    // Draw text outline
                    draw_text_outline(cx, text_bounds);

                    // Draw the text selection
                    if let ModifyState::Selecting {
                        selection_cursor, ..
                    } = modify_state
                    {
                        textstroke.text_style.draw_text_selection(
                            cx,
                            textstroke.text.clone(),
                            cursor,
                            selection_cursor,
                            &textstroke.transform,
                            engine_view.camera,
                        );
                    }

                    // Draw the cursor
                    if self.cursor_visible {
                        textstroke.text_style.draw_cursor(
                            cx,
                            textstroke.text.clone(),
                            cursor,
                            &textstroke.transform,
                            engine_view.camera,
                        )?;
                    }

                    // Draw the text width adjust node
                    let adjust_text_width_node_bounds = Self::adjust_text_width_node_bounds(
                        text_bounds.mins.coords,
                        text_width,
                        engine_view.camera,
                    );
                    let adjust_text_width_node_state = match modify_state {
                        ModifyState::AdjustTextWidth { .. } => PenState::Down,
                        ModifyState::Hover(pos) => {
                            if adjust_text_width_node_bounds.contains_local_point(&(*pos).into()) {
                                PenState::Proximity
                            } else {
                                PenState::Up
                            }
                        }
                        _ => PenState::Up,
                    };
                    indicators::draw_triangular_node(
                        cx,
                        adjust_text_width_node_state,
                        Self::adjust_text_width_node_center(
                            text_bounds.mins.coords,
                            text_width,
                            engine_view.camera,
                        ),
                        Self::ADJUST_TEXT_WIDTH_NODE_SIZE / total_zoom,
                        total_zoom,
                    );

                    // Draw the translate Node
                    if let Some(typewriter_bounds) = self.bounds_on_doc(engine_view) {
                        let translate_node_bounds =
                            Self::translate_node_bounds(typewriter_bounds, engine_view.camera);
                        let translate_node_state = match modify_state {
                            ModifyState::Translating { .. } => PenState::Down,
                            ModifyState::Hover(pos) => {
                                if translate_node_bounds.contains_local_point(&(*pos).into()) {
                                    PenState::Proximity
                                } else {
                                    PenState::Up
                                }
                            }
                            _ => PenState::Up,
                        };
                        indicators::draw_rectangular_node(
                            cx,
                            translate_node_state,
                            translate_node_bounds,
                            total_zoom,
                        );
                    }
                }
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl PenBehaviour for Typewriter {
    fn init(&mut self, engine_view: &EngineView) -> WidgetFlags {
        let tasks_tx = engine_view.tasks_tx.clone();
        let blink_task = move || -> crate::tasks::PeriodicTaskResult {
            tasks_tx.send(EngineTask::BlinkTypewriterCursor);
            crate::tasks::PeriodicTaskResult::Continue
        };
        self.blink_task_handle = Some(crate::tasks::PeriodicTaskHandle::new(
            blink_task,
            Self::BLINK_TIME,
        ));
        WidgetFlags::default()
    }

    fn deinit(&mut self) -> WidgetFlags {
        self.blink_task_handle = None;
        WidgetFlags::default()
    }

    fn style(&self) -> PenStyle {
        PenStyle::Typewriter
    }

    fn update_state(&mut self, engine_view: &mut EngineViewMut) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        match &mut self.state {
            TypewriterState::Idle | TypewriterState::Start(_) => {}
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                ..
            } => match modify_state {
                ModifyState::Selecting {
                    selection_cursor, ..
                } => {
                    if let Some(Stroke::TextStroke(textstroke)) =
                        engine_view.store.get_stroke_ref(*stroke_key)
                    {
                        engine_view.pens_config.typewriter_config.text_style =
                            textstroke.text_style.clone();
                        engine_view
                            .pens_config
                            .typewriter_config
                            .text_style
                            .ranged_text_attributes
                            .clear();
                        engine_view.pens_config.typewriter_config.set_text_width(
                            textstroke
                                .text_style
                                .max_width()
                                .unwrap_or(TypewriterConfig::TEXT_WIDTH_DEFAULT),
                        );
                        update_cursors_for_textstroke(textstroke, cursor, Some(selection_cursor));

                        widget_flags.refresh_ui = true;
                    }
                }
                ModifyState::Up
                | ModifyState::Hover(_)
                | ModifyState::Translating { .. }
                | ModifyState::AdjustTextWidth { .. } => {
                    if let Some(Stroke::TextStroke(textstroke)) =
                        engine_view.store.get_stroke_ref(*stroke_key)
                    {
                        engine_view.pens_config.typewriter_config.text_style =
                            textstroke.text_style.clone();
                        engine_view
                            .pens_config
                            .typewriter_config
                            .text_style
                            .ranged_text_attributes
                            .clear();
                        engine_view.pens_config.typewriter_config.set_text_width(
                            textstroke
                                .text_style
                                .max_width()
                                .unwrap_or(TypewriterConfig::TEXT_WIDTH_DEFAULT),
                        );
                        update_cursors_for_textstroke(textstroke, cursor, None);

                        widget_flags.refresh_ui = true;
                    }
                }
            },
        }

        widget_flags.redraw = true;

        widget_flags
    }

    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let (event_result, widget_flags) = match event {
            PenEvent::Down {
                element,
                modifier_keys,
            } => self.handle_pen_event_down(element, modifier_keys, now, engine_view),
            PenEvent::Up {
                element,
                modifier_keys,
            } => self.handle_pen_event_up(element, modifier_keys, now, engine_view),
            PenEvent::Proximity {
                element,
                modifier_keys,
            } => self.handle_pen_event_proximity(element, modifier_keys, now, engine_view),
            PenEvent::KeyPressed {
                keyboard_key,
                modifier_keys,
            } => self.handle_pen_event_keypressed(keyboard_key, modifier_keys, now, engine_view),
            PenEvent::Text { text } => self.handle_pen_event_text(text, now, engine_view),
            PenEvent::Cancel => self.handle_pen_event_cancel(now, engine_view),
        };

        (event_result, widget_flags)
    }

    fn fetch_clipboard_content(
        &self,
        engine_view: &EngineView,
    ) -> oneshot::Receiver<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>> {
        let widget_flags = WidgetFlags::default();
        let (sender, receiver) =
            oneshot::channel::<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>>();
        let mut clipboard_content = Vec::with_capacity(1);

        match &self.state {
            TypewriterState::Idle | TypewriterState::Start(_) => {}
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                ..
            } => {
                match modify_state {
                    ModifyState::Selecting {
                        selection_cursor, ..
                    } => {
                        if let Some(Stroke::TextStroke(textstroke)) =
                            engine_view.store.get_stroke_ref(*stroke_key)
                        {
                            let selection_range = crate::utils::positive_range(
                                cursor.cur_cursor(),
                                selection_cursor.cur_cursor(),
                            );
                            // Current selection as clipboard text
                            let selection_text = textstroke
                                .get_text_slice_for_range(selection_range)
                                .to_string();
                            clipboard_content.push((
                                selection_text.into_bytes(),
                                String::from("text/plain;charset=utf-8"),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        if sender.send(Ok((clipboard_content, widget_flags))).is_err() {
            tracing::error!(
                "Sending fetched typewriter clipboard content failed, receiver already dropped."
            );
        }
        receiver
    }

    fn cut_clipboard_content(
        &mut self,
        engine_view: &mut EngineViewMut,
    ) -> oneshot::Receiver<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>> {
        let (sender, receiver) =
            oneshot::channel::<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>>();
        let mut widget_flags = WidgetFlags::default();
        let mut clipboard_content = Vec::with_capacity(1);

        match &mut self.state {
            TypewriterState::Idle | TypewriterState::Start(_) => {}
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                ..
            } => {
                match modify_state {
                    ModifyState::Selecting {
                        selection_cursor, ..
                    } => {
                        if let Some(Stroke::TextStroke(textstroke)) =
                            engine_view.store.get_stroke_mut(*stroke_key)
                        {
                            let selection_range = crate::utils::positive_range(
                                cursor.cur_cursor(),
                                selection_cursor.cur_cursor(),
                            );

                            // Current selection as clipboard text
                            let selection_text = textstroke
                                .get_text_slice_for_range(selection_range)
                                .to_string();

                            textstroke.replace_text_between_selection_cursors(
                                cursor,
                                selection_cursor,
                                String::from("").as_str(),
                            );

                            // Update stroke
                            engine_view.store.update_geometry_for_stroke(*stroke_key);
                            engine_view.store.regenerate_rendering_for_stroke(
                                *stroke_key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                            widget_flags |= engine_view
                                .document
                                .resize_autoexpand(engine_view.store, engine_view.camera);

                            // Back to modifying state
                            self.state = TypewriterState::Modifying {
                                modify_state: ModifyState::Up,
                                stroke_key: *stroke_key,
                                cursor: cursor.clone(),
                                pen_down: false,
                            };

                            widget_flags |= engine_view.store.record(Instant::now());
                            widget_flags.store_modified = true;
                            widget_flags.redraw = true;

                            clipboard_content.push((
                                selection_text.into_bytes(),
                                String::from("text/plain;charset=utf-8"),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        self.reset_blink();

        if sender.send(Ok((clipboard_content, widget_flags))).is_err() {
            tracing::error!(
                "Sending cut typewriter clipboard content failed, receiver already dropped."
            );
        }
        receiver
    }
}

// Update the cursors to valid positions and new text length.
fn update_cursors_for_textstroke(
    textstroke: &TextStroke,
    cursor: &mut GraphemeCursor,
    selection_cursor: Option<&mut GraphemeCursor>,
) {
    *cursor = GraphemeCursor::new(
        cursor.cur_cursor().min(textstroke.text.len()),
        textstroke.text.len(),
        true,
    );
    if let Some(selection_cursor) = selection_cursor {
        *selection_cursor = GraphemeCursor::new(
            selection_cursor.cur_cursor().min(textstroke.text.len()),
            textstroke.text.len(),
            true,
        );
    }
}

impl Typewriter {
    // The size of the translate node, located in the upper left corner.
    const TRANSLATE_NODE_SIZE: na::Vector2<f64> = na::vector![18.0, 18.0];
    /// The threshold where above it a transformation is applied. In surface coordinates.
    const TRANSLATE_OFFSET_THRESHOLD: f64 = 1.414;
    /// The threshold in x-axis direction where above it adjustments to the text width are applied. In surface coordinates.
    const ADJ_TEXT_WIDTH_THRESHOLD: f64 = 1.0;
    /// The size of the translate node, located in the upper right corner.
    const ADJUST_TEXT_WIDTH_NODE_SIZE: na::Vector2<f64> = na::vector![18.0, 18.0];
    /// The text width when the typewriter is in `Start` state.
    const STATE_START_TEXT_WIDTH: f64 = 10.0;
    /// The outline stroke width when drawing a text box outline
    const TEXT_OUTLINE_STROKE_WIDTH: f64 = 2.0;
    /// The time for the cursor blink.
    const BLINK_TIME: Duration = Duration::from_millis(800);
    /// The outline color when drawing a text box outline
    const TEXT_OUTLINE_COLOR: piet::Color = color::GNOME_BRIGHTS[4].with_a8(240);

    pub(crate) fn toggle_cursor_visibility(&mut self) {
        self.cursor_visible = !self.cursor_visible;
    }

    /// The range of the current selection, if available.
    pub(crate) fn selection_range(&self) -> Option<(Range<usize>, StrokeKey)> {
        if let TypewriterState::Modifying {
            modify_state:
                ModifyState::Selecting {
                    selection_cursor, ..
                },
            stroke_key,
            cursor,
            ..
        } = &self.state
        {
            let selection_range =
                crate::utils::positive_range(cursor.cur_cursor(), selection_cursor.cur_cursor());

            Some((selection_range, *stroke_key))
        } else {
            None
        }
    }

    /// The bounds of the text rect enclosing the textstroke.
    fn text_rect_bounds(text_width: f64, textstroke: &TextStroke) -> Aabb {
        let origin = textstroke.transform.translation_part();
        Aabb::new(origin.into(), na::point![origin[0] + text_width, origin[1]])
            .merged(&textstroke.bounds())
    }

    /// The bounds of the translate node.
    fn translate_node_bounds(typewriter_bounds: Aabb, camera: &Camera) -> Aabb {
        let total_zoom = camera.total_zoom();
        Aabb::from_half_extents(
            (typewriter_bounds.mins.coords + Self::TRANSLATE_NODE_SIZE * 0.5 / total_zoom).into(),
            Self::TRANSLATE_NODE_SIZE * 0.5 / total_zoom,
        )
    }

    /// The center of the adjust text width node.
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

    /// The bounds of the adjust text width node.
    fn adjust_text_width_node_bounds(
        text_rect_origin: na::Vector2<f64>,
        text_width: f64,
        camera: &Camera,
    ) -> Aabb {
        let total_zoom = camera.total_zoom();
        let center = Self::adjust_text_width_node_center(text_rect_origin, text_width, camera);
        Aabb::from_half_extents(
            center.into(),
            Self::ADJUST_TEXT_WIDTH_NODE_SIZE * 0.5 / total_zoom,
        )
    }

    /// Insert text either at the current cursor position or, if the state is idle, in a new textstroke.
    ///
    /// Inserts at the given position, if supplied. Else at a default offset.
    pub(crate) fn insert_text(
        &mut self,
        text: String,
        preferred_pos: Option<na::Vector2<f64>>,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let pos = preferred_pos.unwrap_or_else(|| {
            engine_view.camera.viewport().mins.coords + Stroke::IMPORT_OFFSET_DEFAULT
        });
        let mut widget_flags = WidgetFlags::default();
        let text_width = engine_view.pens_config.typewriter_config.text_width();
        let mut text_style = engine_view.pens_config.typewriter_config.text_style.clone();

        match &mut self.state {
            TypewriterState::Idle => {
                let text_len = text.len();
                text_style.ranged_text_attributes.clear();
                text_style.set_max_width(Some(text_width));
                let textstroke = TextStroke::new(text, pos, text_style);
                let cursor = GraphemeCursor::new(text_len, textstroke.text.len(), true);

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
                widget_flags.store_modified = true;
                widget_flags.resize = true;
            }
            TypewriterState::Start(pos) => {
                let text_len = text.len();
                text_style.ranged_text_attributes.clear();
                text_style.set_max_width(Some(text_width));
                let textstroke = TextStroke::new(text, *pos, text_style);
                let cursor = GraphemeCursor::new(text_len, textstroke.text.len(), true);

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
                widget_flags.store_modified = true;
                widget_flags.resize = true;
            }
            TypewriterState::Modifying {
                modify_state,
                stroke_key,
                cursor,
                ..
            } => match modify_state {
                ModifyState::Selecting {
                    selection_cursor, ..
                } => {
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
                _ => {
                    if let Some(Stroke::TextStroke(textstroke)) =
                        engine_view.store.get_stroke_mut(*stroke_key)
                    {
                        textstroke.insert_text_after_cursor(text.as_str(), cursor);
                        engine_view.store.update_geometry_for_stroke(*stroke_key);
                        engine_view.store.regenerate_rendering_for_stroke(
                            *stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );
                        widget_flags |= engine_view
                            .document
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        widget_flags |= engine_view.store.record(Instant::now());
                        widget_flags.store_modified = true;
                    }
                }
            },
        }

        self.reset_blink();
        widget_flags.redraw = true;

        widget_flags
    }

    // Change the text style of the text stroke that is currently being modified.
    pub(crate) fn change_text_style_in_modifying_stroke<F>(
        &mut self,
        modify_func: F,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags
    where
        F: FnOnce(&mut TextStyle),
    {
        let mut widget_flags = WidgetFlags::default();

        if let TypewriterState::Modifying { stroke_key, .. } = &mut self.state {
            if let Some(Stroke::TextStroke(textstroke)) =
                engine_view.store.get_stroke_mut(*stroke_key)
            {
                modify_func(&mut textstroke.text_style);
                engine_view.store.update_geometry_for_stroke(*stroke_key);
                engine_view.store.regenerate_rendering_for_stroke(
                    *stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.redraw = true;
                widget_flags.store_modified = true;
            }
        }

        widget_flags
    }

    pub(crate) fn toggle_text_attribute_current_selection(
        &mut self,
        text_attribute: TextAttribute,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if let Some((selection_range, stroke_key)) = self.selection_range() {
            if let Some(Stroke::TextStroke(textstroke)) =
                engine_view.store.get_stroke_mut(stroke_key)
            {
                textstroke.toggle_attrs_for_range(selection_range.clone(), text_attribute.clone());
                engine_view.store.update_geometry_for_stroke(stroke_key);
                engine_view.store.regenerate_rendering_for_stroke(
                    stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.redraw = true;
                widget_flags.store_modified = true;
            }
        }

        widget_flags
    }

    pub(crate) fn remove_text_attributes_current_selection(
        &mut self,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if let Some((selection_range, stroke_key)) = self.selection_range() {
            if let Some(Stroke::TextStroke(textstroke)) =
                engine_view.store.get_stroke_mut(stroke_key)
            {
                textstroke.remove_attrs_for_range(selection_range);
                engine_view.store.update_geometry_for_stroke(stroke_key);
                engine_view.store.regenerate_rendering_for_stroke(
                    stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.redraw = true;
                widget_flags.store_modified = true;
            }
        }

        widget_flags
    }

    pub(crate) fn add_text_attribute_current_selection(
        &mut self,
        text_attribute: TextAttribute,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if let Some((selection_range, stroke_key)) = self.selection_range() {
            if let Some(Stroke::TextStroke(textstroke)) =
                engine_view.store.get_stroke_mut(stroke_key)
            {
                textstroke
                    .text_style
                    .ranged_text_attributes
                    .push(RangedTextAttribute {
                        attribute: text_attribute,
                        range: selection_range,
                    });
                engine_view.store.update_geometry_for_stroke(stroke_key);
                engine_view.store.regenerate_rendering_for_stroke(
                    stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.redraw = true;
                widget_flags.store_modified = true;
            }
        }

        widget_flags
    }

    pub(crate) fn replace_text_attribute_current_selection(
        &mut self,
        text_attribute: TextAttribute,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if let Some((selection_range, stroke_key)) = self.selection_range() {
            if let Some(Stroke::TextStroke(textstroke)) =
                engine_view.store.get_stroke_mut(stroke_key)
            {
                textstroke.replace_attr_for_range(selection_range, text_attribute);
                engine_view.store.update_geometry_for_stroke(stroke_key);
                engine_view.store.regenerate_rendering_for_stroke(
                    stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.redraw = true;
                widget_flags.store_modified = true;
            }
        }

        widget_flags
    }

    /// Resets the blink
    fn reset_blink(&mut self) {
        if let Some(handle) = &mut self.blink_task_handle {
            if let Err(e) = handle.skip() {
                tracing::error!("Skipping blink task failed, Err: {e:?}");
            }
        }
        self.cursor_visible = true;
    }
}

fn play_sound(keyboard_key: Option<KeyboardKey>, audioplayer: &mut Option<AudioPlayer>) {
    if let Some(audioplayer) = audioplayer {
        audioplayer.play_typewriter_key_sound(keyboard_key);
    }
}
