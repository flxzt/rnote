// Imports
use super::{ModifyState, ResizeCorner, Selector, SelectorState};
use crate::engine::EngineViewMut;
use crate::pens::pensconfig::selectorconfig::SelectorStyle;
use crate::snap::SnapCorner;
use crate::store::StrokeKey;
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::Aabb;
use p2d::query::PointQuery;
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::ext::{AabbExt, Vector2Ext};
use rnote_compose::penevent::{KeyboardKey, ModifierKey, PenProgress};
use rnote_compose::penpath::Element;
use std::time::Instant;

impl Selector {
    pub(super) fn handle_pen_event_down(
        &mut self,
        element: Element,
        modifier_keys: Vec<ModifierKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let event_result = match &mut self.state {
            SelectorState::Idle => {
                // Deselect on start
                let selection_keys = engine_view.store.selection_keys_as_rendered();
                if !selection_keys.is_empty() {
                    engine_view.store.set_selected_keys(&selection_keys, false);
                    widget_flags.store_modified = true;
                }

                self.state = SelectorState::Selecting {
                    path: vec![element],
                };

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            SelectorState::Selecting { path } => {
                Self::add_to_select_path(
                    engine_view.pens_config.selector_config.style,
                    path,
                    element,
                );
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
            SelectorState::ModifySelection {
                modify_state,
                selection,
                selection_bounds,
            } => {
                let mut progress = PenProgress::InProgress;

                match modify_state {
                    ModifyState::Up | ModifyState::Hover(_) => {
                        // If we click on another, not-already selected stroke while in separate style or
                        // while pressing Shift, we add it to the selection
                        let key_to_add = engine_view
                            .store
                            .stroke_hitboxes_contain_coord(
                                engine_view.camera.viewport(),
                                element.pos,
                            )
                            .pop();

                        if (engine_view.pens_config.selector_config.style == SelectorStyle::Single
                            || modifier_keys.contains(&ModifierKey::KeyboardShift))
                            && key_to_add
                                .and_then(|key| engine_view.store.selected(key).map(|s| !s))
                                .unwrap_or(false)
                        {
                            let key_to_add = key_to_add.unwrap();
                            engine_view.store.set_selected(key_to_add, true);
                            selection.push(key_to_add);
                            if let Some(new_bounds) =
                                engine_view.store.bounds_for_strokes(selection)
                            {
                                *selection_bounds = new_bounds;
                            }
                        } else if Self::rotate_node_sphere(*selection_bounds, engine_view.camera)
                            .contains_local_point(&element.pos.into())
                        {
                            // clicking on the rotate node
                            let rotation_angle = {
                                let vec = element.pos - selection_bounds.center().coords;
                                na::Vector2::x().angle_ahead(&vec)
                            };

                            *modify_state = ModifyState::Rotate {
                                rotation_center: selection_bounds.center(),
                                start_rotation_angle: rotation_angle,
                                current_rotation_angle: rotation_angle,
                            };
                            // clicking on one of the resize nodes at the corners
                        } else if Self::resize_node_bounds(
                            ResizeCorner::TopLeft,
                            *selection_bounds,
                            engine_view.camera,
                        )
                        .contains_local_point(&element.pos.into())
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::TopLeft,
                                start_bounds: *selection_bounds,
                                start_pos: element.pos,
                                last_rendered_bounds: *selection_bounds,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::TopRight,
                            *selection_bounds,
                            engine_view.camera,
                        )
                        .contains_local_point(&element.pos.into())
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::TopRight,
                                start_bounds: *selection_bounds,
                                start_pos: element.pos,
                                last_rendered_bounds: *selection_bounds,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::BottomLeft,
                            *selection_bounds,
                            engine_view.camera,
                        )
                        .contains_local_point(&element.pos.into())
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::BottomLeft,
                                start_bounds: *selection_bounds,
                                start_pos: element.pos,
                                last_rendered_bounds: *selection_bounds,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::BottomRight,
                            *selection_bounds,
                            engine_view.camera,
                        )
                        .contains_local_point(&element.pos.into())
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::BottomRight,
                                start_bounds: *selection_bounds,
                                start_pos: element.pos,
                                last_rendered_bounds: *selection_bounds,
                            }
                        } else if selection_bounds.contains_local_point(&element.pos.into()) {
                            let snap_corner =
                                SnapCorner::determine_from_bounds(*selection_bounds, element.pos);

                            // clicking inside the selection bounds, triggering translation
                            *modify_state = ModifyState::Translate {
                                start_pos: element.pos,
                                current_pos: element.pos,
                                snap_corner,
                            };
                        } else {
                            // when clicking outside the selection bounds, reset
                            engine_view.store.set_selected_keys(selection, false);
                            self.state = SelectorState::Idle;

                            progress = PenProgress::Finished;
                        }
                    }
                    ModifyState::Translate {
                        start_pos: _,
                        current_pos,
                        snap_corner,
                    } => {
                        let snap_corner_pos = match snap_corner {
                            SnapCorner::TopLeft => selection_bounds.mins.coords,
                            SnapCorner::TopRight => {
                                na::vector![selection_bounds.maxs[0], selection_bounds.mins[1]]
                            }
                            SnapCorner::BottomLeft => {
                                na::vector![selection_bounds.mins[0], selection_bounds.maxs[1]]
                            }
                            SnapCorner::BottomRight => selection_bounds.maxs.coords,
                        };

                        let offset = engine_view
                            .document
                            .snap_position(snap_corner_pos + (element.pos - *current_pos))
                            - snap_corner_pos;

                        if offset.magnitude()
                            > Self::TRANSLATE_OFFSET_THRESHOLD / engine_view.camera.total_zoom()
                        {
                            // move selection
                            engine_view.store.translate_strokes(selection, offset);
                            engine_view
                                .store
                                .translate_strokes_images(selection, offset);
                            *selection_bounds = selection_bounds.translate(offset);
                            *current_pos += offset;
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
                    ModifyState::Rotate {
                        rotation_center,
                        start_rotation_angle: _,
                        current_rotation_angle,
                    } => {
                        let new_rotation_angle = {
                            let vec = element.pos - rotation_center.coords;
                            na::Vector2::x().angle_ahead(&vec)
                        };
                        let angle_delta = new_rotation_angle - *current_rotation_angle;

                        if angle_delta.abs() > Self::ROTATE_ANGLE_THRESHOLD {
                            engine_view.store.rotate_strokes(
                                selection,
                                angle_delta,
                                *rotation_center,
                            );
                            engine_view.store.rotate_strokes_images(
                                selection,
                                angle_delta,
                                *rotation_center,
                            );

                            if let Some(new_bounds) =
                                engine_view.store.bounds_for_strokes(selection)
                            {
                                *selection_bounds = new_bounds;
                            }
                            *current_rotation_angle = new_rotation_angle;
                        }
                    }
                    ModifyState::Resize {
                        from_corner,
                        start_bounds,
                        start_pos,
                        last_rendered_bounds,
                    } => {
                        let lock_aspectratio = engine_view
                            .pens_config
                            .selector_config
                            .resize_lock_aspectratio
                            || modifier_keys.contains(&ModifierKey::KeyboardCtrl);
                        let snap_corner_pos = match from_corner {
                            ResizeCorner::TopLeft => start_bounds.mins.coords,
                            ResizeCorner::TopRight => na::vector![
                                start_bounds.maxs.coords[0],
                                start_bounds.mins.coords[1]
                            ],
                            ResizeCorner::BottomLeft => na::vector![
                                start_bounds.mins.coords[0],
                                start_bounds.maxs.coords[1]
                            ],
                            ResizeCorner::BottomRight => start_bounds.maxs.coords,
                        };
                        let pivot = match from_corner {
                            ResizeCorner::TopLeft => start_bounds.maxs.coords,
                            ResizeCorner::TopRight => na::vector![
                                start_bounds.mins.coords[0],
                                start_bounds.maxs.coords[1]
                            ],
                            ResizeCorner::BottomLeft => na::vector![
                                start_bounds.maxs.coords[0],
                                start_bounds.mins.coords[1]
                            ],
                            ResizeCorner::BottomRight => start_bounds.mins.coords,
                        };
                        let mut offset_to_start = element.pos - *start_pos;
                        if !lock_aspectratio {
                            offset_to_start = engine_view
                                .document
                                .snap_position(snap_corner_pos + offset_to_start)
                                - snap_corner_pos;
                        }
                        offset_to_start = match from_corner {
                            ResizeCorner::TopLeft => -offset_to_start,
                            ResizeCorner::TopRight => {
                                na::vector![offset_to_start[0], -offset_to_start[1]]
                            }
                            ResizeCorner::BottomLeft => {
                                na::vector![-offset_to_start[0], offset_to_start[1]]
                            }
                            ResizeCorner::BottomRight => offset_to_start,
                        };
                        if lock_aspectratio {
                            let start_extents = start_bounds.extents();
                            let start_mean = start_extents.mean();
                            let offset_mean = offset_to_start.mean();
                            offset_to_start = start_extents * (offset_mean / start_mean);
                        }
                        let min_extents = na::Vector2::<f64>::from_element(2.0f64)
                            / engine_view.camera.total_zoom();
                        let scale = (start_bounds.extents() + offset_to_start)
                            .maxs(&min_extents)
                            .component_div(&selection_bounds.extents());

                        // resize strokes
                        engine_view
                            .store
                            .scale_strokes_with_pivot(selection, scale, pivot);
                        engine_view
                            .store
                            .scale_strokes_images_with_pivot(selection, scale, pivot);
                        *selection_bounds = selection_bounds
                            .translate(-pivot)
                            .scale_non_uniform(scale)
                            .translate(pivot);

                        // possibly nudge camera
                        widget_flags |= engine_view
                            .camera
                            .nudge_w_pos(element.pos, engine_view.document);
                        widget_flags |= engine_view
                            .document
                            .expand_autoexpand(engine_view.camera, engine_view.store);

                        // Rerender but based on some conditions
                        const RERENDER_BOUNDS_FACTOR: f64 = 1.5;
                        let last_rendered_bounds_scale = selection_bounds
                            .extents()
                            .component_div(&last_rendered_bounds.extents());

                        if last_rendered_bounds_scale[0] < 1. / RERENDER_BOUNDS_FACTOR
                            || last_rendered_bounds_scale[0] > RERENDER_BOUNDS_FACTOR
                            || last_rendered_bounds_scale[1] < 1. / RERENDER_BOUNDS_FACTOR
                            || last_rendered_bounds_scale[1] > RERENDER_BOUNDS_FACTOR
                        {
                            let selection_in_viewport: Vec<StrokeKey> = engine_view
                                .store
                                .filter_keys_intersecting_bounds::<&Vec<StrokeKey>>(
                                    selection,
                                    engine_view.camera.viewport(),
                                )
                                .copied()
                                .collect();
                            engine_view.store.regenerate_rendering_for_strokes(
                                &selection_in_viewport,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                            *last_rendered_bounds = *selection_bounds;
                        }
                    }
                }

                widget_flags.store_modified = true;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress,
                }
            }
        };

        (event_result, widget_flags)
    }

    pub(super) fn handle_pen_event_up(
        &mut self,
        element: Element,
        _modifier_keys: Vec<ModifierKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();
        let selector_bounds = self.bounds_on_doc(&engine_view.as_im());

        let event_result = match &mut self.state {
            SelectorState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            SelectorState::Selecting { path } => {
                let mut progress = PenProgress::Finished;

                let new_selection = match engine_view.pens_config.selector_config.style {
                    SelectorStyle::Polygon => {
                        if path.len() >= 3 {
                            engine_view
                                .store
                                .strokes_hitboxes_contained_in_path_polygon(
                                    path,
                                    engine_view.camera.viewport(),
                                )
                        } else {
                            vec![]
                        }
                    }
                    SelectorStyle::Rectangle => {
                        if let (Some(first), Some(last)) = (path.first(), path.last()) {
                            let aabb = Aabb::new_positive(first.pos.into(), last.pos.into());
                            engine_view.store.strokes_hitboxes_contained_in_aabb(
                                aabb,
                                engine_view.camera.viewport(),
                            )
                        } else {
                            vec![]
                        }
                    }
                    SelectorStyle::Single => {
                        if let Some(key) = path.last().and_then(|last| {
                            engine_view
                                .store
                                .stroke_hitboxes_contain_coord(
                                    engine_view.camera.viewport(),
                                    last.pos,
                                )
                                .pop()
                        }) {
                            vec![key]
                        } else {
                            vec![]
                        }
                    }
                    SelectorStyle::IntersectingPath => {
                        if path.len() >= 3 {
                            engine_view.store.strokes_hitboxes_intersect_path(
                                path,
                                engine_view.camera.viewport(),
                            )
                        } else {
                            vec![]
                        }
                    }
                };
                if !new_selection.is_empty() {
                    engine_view.store.set_selected_keys(&new_selection, true);
                    widget_flags.store_modified = true;
                    widget_flags.deselect_color_setters = true;

                    if let Some(new_bounds) = engine_view.store.bounds_for_strokes(&new_selection) {
                        // Change to the modify state
                        self.state = SelectorState::ModifySelection {
                            modify_state: ModifyState::default(),
                            selection: new_selection,
                            selection_bounds: new_bounds,
                        };
                        progress = PenProgress::InProgress;
                    }
                }

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress,
                }
            }
            SelectorState::ModifySelection {
                modify_state,
                selection,
                selection_bounds,
            } => {
                match modify_state {
                    ModifyState::Translate { .. }
                    | ModifyState::Rotate { .. }
                    | ModifyState::Resize { .. } => {
                        engine_view.store.update_geometry_for_strokes(selection);
                        widget_flags |= engine_view
                            .document
                            .resize_autoexpand(engine_view.store, engine_view.camera);
                        engine_view.store.regenerate_rendering_in_viewport_threaded(
                            engine_view.tasks_tx.clone(),
                            false,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        if let Some(new_bounds) = engine_view.store.bounds_for_strokes(selection) {
                            *selection_bounds = new_bounds;
                        }
                        // We would need to update bounds held in the modify state, but since we transition into either
                        // the up or hover state anyway that is not actually needed.

                        widget_flags |= engine_view.store.record(Instant::now());
                        widget_flags.store_modified = true;
                    }
                    _ => {}
                }

                *modify_state = if selector_bounds
                    .map(|b| b.contains_local_point(&element.pos.into()))
                    .unwrap_or(false)
                {
                    ModifyState::Hover(element.pos)
                } else {
                    ModifyState::Up
                };

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
        _modifier_keys: Vec<ModifierKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let widget_flags = WidgetFlags::default();
        let selector_bounds = self.bounds_on_doc(&engine_view.as_im());

        let event_result = match &mut self.state {
            SelectorState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            SelectorState::Selecting { .. } => EventResult {
                handled: true,
                propagate: EventPropagation::Stop,
                progress: PenProgress::InProgress,
            },
            SelectorState::ModifySelection { modify_state, .. } => {
                *modify_state = if selector_bounds
                    .map(|b| b.contains_local_point(&element.pos.into()))
                    .unwrap_or(false)
                {
                    ModifyState::Hover(element.pos)
                } else {
                    ModifyState::Up
                };
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
        modifier_keys: Vec<ModifierKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let event_result = match &mut self.state {
            SelectorState::Idle => match keyboard_key {
                KeyboardKey::Unicode('a') => {
                    self.select_all(modifier_keys, engine_view, &mut widget_flags);
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
            },
            SelectorState::Selecting { .. } => match keyboard_key {
                KeyboardKey::Unicode('a') => {
                    self.select_all(modifier_keys, engine_view, &mut widget_flags);
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
            },
            SelectorState::ModifySelection { selection, .. } => {
                match keyboard_key {
                    KeyboardKey::Unicode('a') => {
                        self.select_all(modifier_keys, engine_view, &mut widget_flags);
                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::InProgress,
                        }
                    }
                    KeyboardKey::Unicode('d') => {
                        //Duplicate selection
                        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
                            let duplicated = engine_view.store.duplicate_selection();
                            engine_view.store.update_geometry_for_strokes(&duplicated);
                            engine_view.store.regenerate_rendering_for_strokes_threaded(
                                engine_view.tasks_tx.clone(),
                                &duplicated,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );

                            widget_flags |= engine_view.store.record(Instant::now());
                            widget_flags.resize = true;
                            widget_flags.store_modified = true;
                        }
                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::Finished,
                        }
                    }
                    KeyboardKey::Delete | KeyboardKey::BackSpace => {
                        engine_view.store.set_trashed_keys(selection, true);
                        widget_flags |= super::cancel_selection(selection, engine_view);
                        self.state = SelectorState::Idle;
                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::Finished,
                        }
                    }
                    KeyboardKey::Escape => {
                        widget_flags |= super::cancel_selection(selection, engine_view);
                        self.state = SelectorState::Idle;
                        EventResult {
                            handled: true,
                            propagate: EventPropagation::Stop,
                            progress: PenProgress::Finished,
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

    pub(super) fn handle_pen_event_text(
        &mut self,
        _text: String,
        _now: Instant,
        _engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let widget_flags = WidgetFlags::default();

        let event_result = match &mut self.state {
            SelectorState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            SelectorState::Selecting { .. } => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            SelectorState::ModifySelection { .. } => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
        };

        (event_result, widget_flags)
    }

    pub(super) fn handle_pen_event_cancel(
        &mut self,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let event_result = match &mut self.state {
            SelectorState::Idle => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            SelectorState::Selecting { .. } => {
                self.state = SelectorState::Idle;
                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            SelectorState::ModifySelection { selection, .. } => {
                widget_flags |= super::cancel_selection(selection, engine_view);
                self.state = SelectorState::Idle;
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
