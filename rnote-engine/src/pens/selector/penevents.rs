use std::time::Instant;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::query::PointQuery;
use rnote_compose::helpers::{AabbHelpers, Vector2Helpers};
use rnote_compose::penevents::{KeyboardKey, ShortcutKey};
use rnote_compose::penpath::Element;

use crate::engine::EngineViewMut;
use crate::pens::penbehaviour::PenProgress;
use crate::pens::pensconfig::selectorconfig::SelectorStyle;
use crate::WidgetFlags;

use super::{ModifyState, ResizeCorner, Selector, SelectorState};

impl Selector {
    pub(super) fn handle_pen_event_down(
        &mut self,
        element: Element,
        shortcut_keys: Vec<ShortcutKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let progress = match &mut self.state {
            SelectorState::Idle => {
                widget_flags.merge(engine_view.store.record(Instant::now()));

                // Deselect on start
                let selection_keys = engine_view.store.selection_keys_as_rendered();
                engine_view.store.set_selected_keys(&selection_keys, false);

                self.state = SelectorState::Selecting {
                    path: vec![element],
                };

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
            SelectorState::Selecting { path } => {
                Self::add_to_select_path(
                    engine_view.pens_config.selector_config.style,
                    path,
                    element,
                );

                widget_flags.redraw = true;

                PenProgress::InProgress
            }
            SelectorState::ModifySelection {
                modify_state,
                selection,
                selection_bounds,
            } => {
                let mut pen_progress = PenProgress::InProgress;

                match modify_state {
                    ModifyState::Up => {
                        widget_flags.merge(engine_view.store.record(Instant::now()));

                        // If we click on another, not-already selected stroke while in separate style or while pressing Shift, we add it to the selection
                        let keys = engine_view.store.stroke_hitboxes_contain_coord(
                            engine_view.camera.viewport(),
                            element.pos,
                        );
                        let key_to_add = keys.last();

                        if (engine_view.pens_config.selector_config.style == SelectorStyle::Single
                            || shortcut_keys.contains(&ShortcutKey::KeyboardShift))
                            && key_to_add
                                .and_then(|&key| engine_view.store.selected(key).map(|s| !s))
                                .unwrap_or(false)
                        {
                            let key_to_add = *key_to_add.unwrap();
                            engine_view.store.set_selected(key_to_add, true);

                            selection.push(key_to_add);

                            if let Some(new_bounds) =
                                engine_view.store.bounds_for_strokes(selection)
                            {
                                *selection_bounds = new_bounds;
                            }
                        } else if Self::rotate_node_sphere(*selection_bounds, engine_view.camera)
                            .contains_local_point(&na::Point2::from(element.pos))
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
                        .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::TopLeft,
                                start_bounds: *selection_bounds,
                                start_pos: element.pos,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::TopRight,
                            *selection_bounds,
                            engine_view.camera,
                        )
                        .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::TopRight,
                                start_bounds: *selection_bounds,
                                start_pos: element.pos,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::BottomLeft,
                            *selection_bounds,
                            engine_view.camera,
                        )
                        .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::BottomLeft,
                                start_bounds: *selection_bounds,
                                start_pos: element.pos,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::BottomRight,
                            *selection_bounds,
                            engine_view.camera,
                        )
                        .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::BottomRight,
                                start_bounds: *selection_bounds,
                                start_pos: element.pos,
                            }
                        } else if selection_bounds
                            .contains_local_point(&na::Point2::from(element.pos))
                        {
                            // clicking inside the selection bounds, triggering translation
                            *modify_state = ModifyState::Translate {
                                start_pos: element.pos,
                                current_pos: element.pos,
                            };
                        } else {
                            // If clicking outside the selection bounds, reset
                            engine_view.store.set_selected_keys(selection, false);
                            self.state = SelectorState::Idle;

                            pen_progress = PenProgress::Finished;
                        }
                    }
                    ModifyState::Translate {
                        start_pos: _,
                        current_pos,
                    } => {
                        let offset = element.pos - *current_pos;

                        if offset.magnitude()
                            > Self::TRANSLATE_MAGNITUDE_THRESHOLD / engine_view.camera.total_zoom()
                        {
                            engine_view.store.translate_strokes(selection, offset);
                            engine_view
                                .store
                                .translate_strokes_images(selection, offset);
                            *selection_bounds = selection_bounds.translate(offset);

                            // strokes that were far away previously might come into view
                            engine_view.store.regenerate_rendering_in_viewport_threaded(
                                engine_view.tasks_tx.clone(),
                                false,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );

                            *current_pos = element.pos;
                        }
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
                                *selection_bounds =
                                    new_bounds.loosened(Self::SELECTION_BOUNDS_MARGIN);
                            }
                            *current_rotation_angle = new_rotation_angle;
                        }
                    }
                    ModifyState::Resize {
                        from_corner,
                        start_bounds,
                        start_pos,
                    } => {
                        let (pos_offset, pivot) = {
                            let pos_offset = element.pos - *start_pos;

                            match from_corner {
                                ResizeCorner::TopLeft => (-pos_offset, start_bounds.maxs.coords),
                                ResizeCorner::TopRight => (
                                    na::vector![pos_offset[0], -pos_offset[1]],
                                    na::vector![
                                        start_bounds.mins.coords[0],
                                        start_bounds.maxs.coords[1]
                                    ],
                                ),
                                ResizeCorner::BottomLeft => (
                                    na::vector![-pos_offset[0], pos_offset[1]],
                                    na::vector![
                                        start_bounds.maxs.coords[0],
                                        start_bounds.mins.coords[1]
                                    ],
                                ),
                                ResizeCorner::BottomRight => (pos_offset, start_bounds.mins.coords),
                            }
                        };

                        let new_extents = if engine_view
                            .pens_config
                            .selector_config
                            .resize_lock_aspectratio
                            || shortcut_keys.contains(&ShortcutKey::KeyboardCtrl)
                        {
                            // Lock aspectratio
                            rnote_compose::helpers::scale_w_locked_aspectratio(
                                start_bounds.extents(),
                                start_bounds.extents() + pos_offset,
                            )
                        } else {
                            start_bounds.extents() + pos_offset
                        }
                        .maxs(&((Self::RESIZE_NODE_SIZE * 2.0) / engine_view.camera.total_zoom()));

                        let scale = new_extents.component_div(&selection_bounds.extents());

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
                    }
                }

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                pen_progress
            }
        };

        (progress, widget_flags)
    }

    pub(super) fn handle_pen_event_up(
        &mut self,
        _element: Element,
        _shortcut_keys: Vec<ShortcutKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let progress = match &mut self.state {
            SelectorState::Idle => PenProgress::Idle,
            SelectorState::Selecting { path } => {
                let mut state = SelectorState::Idle;
                let mut pen_progress = PenProgress::Finished;

                if let Some(selection) = match engine_view.pens_config.selector_config.style {
                    SelectorStyle::Polygon => {
                        if path.len() < 3 {
                            None
                        } else {
                            let new_keys = engine_view
                                .store
                                .strokes_hitboxes_contained_in_path_polygon(
                                    path,
                                    engine_view.camera.viewport(),
                                );
                            if !new_keys.is_empty() {
                                engine_view.store.set_selected_keys(&new_keys, true);
                                Some(new_keys)
                            } else {
                                None
                            }
                        }
                    }
                    SelectorStyle::Rectangle => {
                        if let (Some(first), Some(last)) = (path.first(), path.last()) {
                            let aabb = Aabb::new_positive(
                                na::Point2::from(first.pos),
                                na::Point2::from(last.pos),
                            );
                            let new_keys = engine_view.store.strokes_hitboxes_contained_in_aabb(
                                aabb,
                                engine_view.camera.viewport(),
                            );
                            if !new_keys.is_empty() {
                                engine_view.store.set_selected_keys(&new_keys, true);
                                Some(new_keys)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    SelectorStyle::Single => {
                        if let Some(last) = path.last() {
                            if let Some(&new_key) = engine_view
                                .store
                                .stroke_hitboxes_contain_coord(
                                    engine_view.camera.viewport(),
                                    last.pos,
                                )
                                .last()
                            {
                                engine_view.store.set_selected(new_key, true);

                                Some(vec![new_key])
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    SelectorStyle::IntersectingPath => {
                        if path.len() < 3 {
                            None
                        } else {
                            let intersecting_keys =
                                engine_view.store.strokes_hitboxes_intersect_path(
                                    path,
                                    engine_view.camera.viewport(),
                                );
                            if !intersecting_keys.is_empty() {
                                engine_view
                                    .store
                                    .set_selected_keys(&intersecting_keys, true);
                                Some(intersecting_keys)
                            } else {
                                None
                            }
                        }
                    }
                } {
                    if let Some(selection_bounds) = engine_view.store.bounds_for_strokes(&selection)
                    {
                        // Change to the modify state
                        state = SelectorState::ModifySelection {
                            modify_state: ModifyState::default(),
                            selection,
                            selection_bounds: selection_bounds
                                .loosened(Self::SELECTION_BOUNDS_MARGIN),
                        };
                        pen_progress = PenProgress::InProgress;
                    }
                }

                self.state = state;

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                pen_progress
            }
            SelectorState::ModifySelection {
                modify_state,
                selection,
                selection_bounds,
            } => {
                engine_view.store.update_geometry_for_strokes(selection);
                engine_view.store.regenerate_rendering_in_viewport_threaded(
                    engine_view.tasks_tx.clone(),
                    false,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                if let Some(new_selection_bounds) = engine_view.store.bounds_for_strokes(selection)
                {
                    *selection_bounds =
                        new_selection_bounds.loosened(Self::SELECTION_BOUNDS_MARGIN);
                }
                *modify_state = ModifyState::Up;

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
        };

        (progress, widget_flags)
    }

    pub(super) fn handle_pen_event_proximity(
        &mut self,
        _element: Element,
        _shortcut_keys: Vec<ShortcutKey>,
        _now: Instant,
        _engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let widget_flags = WidgetFlags::default();

        let progress = match &mut self.state {
            SelectorState::Idle => PenProgress::Idle,
            SelectorState::Selecting { .. } => PenProgress::InProgress,
            SelectorState::ModifySelection { .. } => PenProgress::InProgress,
        };

        (progress, widget_flags)
    }

    pub(super) fn handle_pen_event_keypressed(
        &mut self,
        keyboard_key: KeyboardKey,
        shortcut_keys: Vec<ShortcutKey>,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let progress = match &mut self.state {
            SelectorState::Idle => {
                match keyboard_key {
                    KeyboardKey::Unicode('a') => {
                        if shortcut_keys.contains(&ShortcutKey::KeyboardCtrl) {
                            // Select all keys
                            let all_strokes = engine_view.store.keys_sorted_chrono();

                            if let Some(selection_bounds) =
                                engine_view.store.bounds_for_strokes(&all_strokes)
                            {
                                engine_view.store.set_selected_keys(&all_strokes, true);

                                self.state = SelectorState::ModifySelection {
                                    modify_state: ModifyState::default(),
                                    selection: all_strokes,
                                    selection_bounds: selection_bounds
                                        .loosened(Self::SELECTION_BOUNDS_MARGIN),
                                };

                                engine_view
                                    .doc
                                    .resize_autoexpand(engine_view.store, engine_view.camera);

                                widget_flags.redraw = true;
                                widget_flags.resize = true;
                                widget_flags.indicate_changed_store = true;
                            }
                        }

                        PenProgress::InProgress
                    }
                    _ => PenProgress::InProgress,
                }
            }
            SelectorState::Selecting { .. } => {
                match keyboard_key {
                    KeyboardKey::Unicode('a') => {
                        if shortcut_keys.contains(&ShortcutKey::KeyboardCtrl) {
                            // Select all keys
                            let all_strokes = engine_view.store.keys_sorted_chrono();

                            if let Some(selection_bounds) =
                                engine_view.store.bounds_for_strokes(&all_strokes)
                            {
                                engine_view.store.set_selected_keys(&all_strokes, true);

                                self.state = SelectorState::ModifySelection {
                                    modify_state: ModifyState::default(),
                                    selection: all_strokes,
                                    selection_bounds: selection_bounds
                                        .loosened(Self::SELECTION_BOUNDS_MARGIN),
                                };

                                engine_view
                                    .doc
                                    .resize_autoexpand(engine_view.store, engine_view.camera);

                                widget_flags.redraw = true;
                                widget_flags.resize = true;
                                widget_flags.indicate_changed_store = true;
                            }
                        }

                        PenProgress::InProgress
                    }
                    _ => PenProgress::InProgress,
                }
            }

            SelectorState::ModifySelection { selection, .. } => {
                match keyboard_key {
                    KeyboardKey::Unicode('a') => {
                        // Select all keys
                        if shortcut_keys.contains(&ShortcutKey::KeyboardCtrl) {
                            let all_strokes = engine_view.store.keys_sorted_chrono();

                            if let Some(selection_bounds) =
                                engine_view.store.bounds_for_strokes(&all_strokes)
                            {
                                engine_view.store.set_selected_keys(&all_strokes, true);

                                self.state = SelectorState::ModifySelection {
                                    modify_state: ModifyState::default(),
                                    selection: all_strokes,
                                    selection_bounds: selection_bounds
                                        .loosened(Self::SELECTION_BOUNDS_MARGIN),
                                };

                                engine_view
                                    .doc
                                    .resize_autoexpand(engine_view.store, engine_view.camera);

                                widget_flags.redraw = true;
                                widget_flags.resize = true;
                                widget_flags.indicate_changed_store = true;
                            }
                        }

                        PenProgress::InProgress
                    }
                    KeyboardKey::Delete | KeyboardKey::BackSpace => {
                        engine_view.store.set_trashed_keys(selection, true);
                        self.state = SelectorState::Idle;

                        engine_view
                            .doc
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        widget_flags.redraw = true;
                        widget_flags.resize = true;
                        widget_flags.indicate_changed_store = true;

                        PenProgress::Finished
                    }
                    KeyboardKey::Escape => {
                        engine_view.store.set_selected_keys(selection, false);
                        self.state = SelectorState::Idle;

                        engine_view
                            .doc
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        widget_flags.redraw = true;
                        widget_flags.resize = true;
                        widget_flags.indicate_changed_store = true;

                        PenProgress::Finished
                    }
                    _ => PenProgress::InProgress,
                }
            }
        };

        (progress, widget_flags)
    }

    pub(super) fn handle_pen_event_text(
        &mut self,
        _text: String,
        _now: Instant,
        _engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let widget_flags = WidgetFlags::default();

        let progress = match &mut self.state {
            SelectorState::Idle => PenProgress::Idle,
            SelectorState::Selecting { .. } => PenProgress::InProgress,
            SelectorState::ModifySelection { .. } => PenProgress::InProgress,
        };

        (progress, widget_flags)
    }

    pub(super) fn handle_pen_event_cancel(
        &mut self,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let progress = match &mut self.state {
            SelectorState::Idle => PenProgress::Idle,
            SelectorState::Selecting { .. } => {
                self.state = SelectorState::Idle;

                // Deselect on cancel
                let selection_keys = engine_view.store.selection_keys_as_rendered();
                engine_view.store.set_selected_keys(&selection_keys, false);

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::Finished
            }
            SelectorState::ModifySelection { selection, .. } => {
                engine_view.store.set_selected_keys(selection, false);
                self.state = SelectorState::Idle;

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::Finished
            }
        };

        (progress, widget_flags)
    }
}
