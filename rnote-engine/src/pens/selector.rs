use std::time::Instant;

use super::penbehaviour::{PenBehaviour, PenProgress};
use super::pensconfig::selectorconfig::SelectorStyle;
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::{Camera, DrawOnDocBehaviour, WidgetFlags};
use kurbo::Shape;
use once_cell::sync::Lazy;
use p2d::query::PointQuery;
use piet::RenderContext;
use rnote_compose::helpers::{AabbHelpers, Vector2Helpers};
use rnote_compose::penevents::{KeyboardKey, PenState};
use rnote_compose::penevents::{PenEvent, ShortcutKey};
use rnote_compose::penpath::Element;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::drawhelpers;
use rnote_compose::{color, Color};

use p2d::bounding_volume::{Aabb, BoundingSphere, BoundingVolume};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ModifyState {
    Up,
    Translate {
        start_pos: na::Vector2<f64>,
        current_pos: na::Vector2<f64>,
    },
    Rotate {
        rotation_center: na::Point2<f64>,
        start_rotation_angle: f64,
        current_rotation_angle: f64,
    },
    Resize {
        from_corner: ResizeCorner,
        start_bounds: Aabb,
        start_pos: na::Vector2<f64>,
    },
}

impl Default for ModifyState {
    fn default() -> Self {
        Self::Up
    }
}

#[derive(Clone, Debug)]
pub(super) enum SelectorState {
    Idle,
    Selecting {
        path: Vec<Element>,
    },
    ModifySelection {
        modify_state: ModifyState,
        selection: Vec<StrokeKey>,
        selection_bounds: Aabb,
    },
}

impl Default for SelectorState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Debug)]
pub struct Selector {
    pub(super) state: SelectorState,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            state: SelectorState::default(),
        }
    }
}

impl PenBehaviour for Selector {
    fn style(&self) -> PenStyle {
        PenStyle::Selector
    }

    fn update_state(&mut self, engine_view: &mut EngineViewMut) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        let selection = engine_view.store.selection_keys_as_rendered();

        if let Some(selection_bounds) = engine_view.store.bounds_for_strokes(&selection) {
            self.state = SelectorState::ModifySelection {
                modify_state: ModifyState::default(),
                selection,
                selection_bounds,
            };

            widget_flags.redraw = true;
        } else {
            self.state = SelectorState::Idle;

            widget_flags.redraw = true;
        }

        widget_flags
    }

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        //log::debug!("selector state: {:?}, event: {:?}", &self.state, &event);

        let pen_progress = match (&mut self.state, event) {
            (SelectorState::Idle, PenEvent::Down { element, .. }) => {
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
            (
                SelectorState::Idle,
                PenEvent::KeyPressed {
                    keyboard_key,
                    shortcut_keys,
                },
            ) => match keyboard_key {
                KeyboardKey::Unicode('a') => {
                    // Select all keys
                    if shortcut_keys.contains(&ShortcutKey::KeyboardCtrl) {
                        let all_strokes = engine_view.store.keys_sorted_chrono();

                        if let Some(new_selection_bounds) =
                            engine_view.store.bounds_for_strokes(&all_strokes)
                        {
                            engine_view.store.set_selected_keys(&all_strokes, true);

                            self.state = SelectorState::ModifySelection {
                                modify_state: ModifyState::default(),
                                selection: all_strokes,
                                selection_bounds: new_selection_bounds,
                            };

                            engine_view
                                .doc
                                .resize_autoexpand(engine_view.store, engine_view.camera);

                            widget_flags.redraw = true;
                            widget_flags.resize = true;
                            widget_flags.indicate_changed_store = true;
                        }

                        PenProgress::InProgress
                    } else {
                        PenProgress::InProgress
                    }
                }
                _ => PenProgress::InProgress,
            },
            (SelectorState::Idle, _) => PenProgress::Idle,
            (SelectorState::Selecting { path }, PenEvent::Down { element, .. }) => {
                Self::add_to_select_path(
                    engine_view.pens_config.selector_config.style,
                    path,
                    element,
                );

                widget_flags.redraw = true;

                PenProgress::InProgress
            }
            (SelectorState::Selecting { path }, PenEvent::Up { .. }) => {
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
                            selection_bounds,
                        };
                        pen_progress = PenProgress::InProgress;
                    }
                }

                self.state = state;

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                pen_progress
            }
            (SelectorState::Selecting { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (
                SelectorState::Selecting { .. },
                PenEvent::KeyPressed {
                    keyboard_key,
                    shortcut_keys,
                },
            ) => match keyboard_key {
                KeyboardKey::Unicode('a') => {
                    // Select all keys
                    if shortcut_keys.contains(&ShortcutKey::KeyboardCtrl) {
                        let all_strokes = engine_view.store.keys_sorted_chrono();

                        if let Some(new_selection_bounds) =
                            engine_view.store.bounds_for_strokes(&all_strokes)
                        {
                            engine_view.store.set_selected_keys(&all_strokes, true);

                            self.state = SelectorState::ModifySelection {
                                modify_state: ModifyState::default(),
                                selection: all_strokes,
                                selection_bounds: new_selection_bounds,
                            };

                            engine_view
                                .doc
                                .resize_autoexpand(engine_view.store, engine_view.camera);

                            widget_flags.redraw = true;
                            widget_flags.resize = true;
                            widget_flags.indicate_changed_store = true;
                        }

                        PenProgress::InProgress
                    } else {
                        PenProgress::InProgress
                    }
                }
                _ => PenProgress::InProgress,
            },
            (SelectorState::Selecting { .. }, PenEvent::Cancel) => {
                self.state = SelectorState::Idle;

                // Deselect on cancel
                let selection_keys = engine_view.store.selection_keys_as_rendered();
                engine_view.store.set_selected_keys(&selection_keys, false);

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::Finished
            }
            (
                SelectorState::ModifySelection {
                    modify_state,
                    selection,
                    selection_bounds,
                },
                PenEvent::Down {
                    element,
                    shortcut_keys,
                },
            ) => {
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
                            // clicking on on of the resize nodes at the corners
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
                                *selection_bounds = new_bounds
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
            (
                SelectorState::ModifySelection {
                    modify_state,
                    selection,
                    selection_bounds,
                    ..
                },
                PenEvent::Up { .. },
            ) => {
                engine_view.store.update_geometry_for_strokes(selection);
                engine_view.store.regenerate_rendering_in_viewport_threaded(
                    engine_view.tasks_tx.clone(),
                    false,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                if let Some(new_bounds) = engine_view.store.bounds_for_strokes(selection) {
                    *selection_bounds = new_bounds
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
            (SelectorState::ModifySelection { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (
                SelectorState::ModifySelection { selection, .. },
                PenEvent::KeyPressed {
                    keyboard_key,
                    shortcut_keys,
                },
            ) => match keyboard_key {
                KeyboardKey::Unicode('a') => {
                    // Select all keys
                    if shortcut_keys.contains(&ShortcutKey::KeyboardCtrl) {
                        let all_strokes = engine_view.store.keys_sorted_chrono();

                        if let Some(new_selection_bounds) =
                            engine_view.store.bounds_for_strokes(&all_strokes)
                        {
                            engine_view.store.set_selected_keys(&all_strokes, true);

                            self.state = SelectorState::ModifySelection {
                                modify_state: ModifyState::default(),
                                selection: all_strokes,
                                selection_bounds: new_selection_bounds,
                            };

                            engine_view
                                .doc
                                .resize_autoexpand(engine_view.store, engine_view.camera);

                            widget_flags.redraw = true;
                            widget_flags.resize = true;
                            widget_flags.indicate_changed_store = true;
                        }

                        PenProgress::InProgress
                    } else {
                        PenProgress::InProgress
                    }
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
            },
            (
                SelectorState::ModifySelection {
                    modify_state: _,
                    selection,
                    selection_bounds: _,
                },
                PenEvent::Cancel,
            ) => {
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
            (SelectorState::Selecting { .. }, PenEvent::Text { .. }) => PenProgress::InProgress,
            (SelectorState::ModifySelection { .. }, PenEvent::Text { .. }) => {
                PenProgress::InProgress
            }
        };

        (pen_progress, widget_flags)
    }
}

impl DrawOnDocBehaviour for Selector {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        let total_zoom = engine_view.camera.total_zoom();

        match &self.state {
            SelectorState::Idle => None,
            SelectorState::Selecting { path } => {
                // Making sure bounds are always outside of coord + width
                let mut path_iter = path.iter();
                if let Some(first) = path_iter.next() {
                    let mut new_bounds = Aabb::from_half_extents(
                        na::Point2::from(first.pos),
                        na::Vector2::repeat(Self::SELECTION_OUTLINE_WIDTH / total_zoom),
                    );

                    path_iter.for_each(|element| {
                        let pos_bounds = Aabb::from_half_extents(
                            na::Point2::from(element.pos),
                            na::Vector2::repeat(Self::SELECTION_OUTLINE_WIDTH / total_zoom),
                        );
                        new_bounds.merge(&pos_bounds);
                    });

                    Some(new_bounds.loosened(Self::SINGLE_SELECTING_CIRCLE_RADIUS / total_zoom))
                } else {
                    None
                }
            }
            SelectorState::ModifySelection {
                selection_bounds, ..
            } => Some(selection_bounds.extend_by(Self::RESIZE_NODE_SIZE / total_zoom)),
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let total_zoom = engine_view.camera.total_zoom();

        match &self.state {
            SelectorState::Idle => {}
            SelectorState::Selecting { path } => {
                match engine_view.pens_config.selector_config.style {
                    SelectorStyle::Polygon => {
                        let mut bez_path = kurbo::BezPath::new();
                        let mut path_iter = path.iter();

                        if let Some(first) = path_iter.next() {
                            bez_path.move_to(first.pos.to_kurbo_point());

                            for element in path_iter {
                                bez_path.line_to(element.pos.to_kurbo_point());
                            }

                            bez_path.close_path();

                            let mut stroke_style = piet::StrokeStyle::new();
                            stroke_style.set_dash_pattern(
                                Self::SELECTING_DASH_PATTERN
                                    .into_iter()
                                    .map(|x| x / total_zoom)
                                    .collect::<Vec<f64>>(),
                            );

                            cx.fill(bez_path.clone(), &*SELECTION_FILL_COLOR);
                            cx.stroke_styled(
                                bez_path,
                                &*OUTLINE_COLOR,
                                Self::SELECTION_OUTLINE_WIDTH / total_zoom,
                                &stroke_style,
                            );
                        }
                    }
                    SelectorStyle::Rectangle => {
                        if let (Some(first), Some(last)) = (path.first(), path.last()) {
                            let select_rect = kurbo::Rect::from_points(
                                first.pos.to_kurbo_point(),
                                last.pos.to_kurbo_point(),
                            );

                            let mut stroke_style = piet::StrokeStyle::new();
                            stroke_style.set_dash_pattern(
                                Self::SELECTING_DASH_PATTERN
                                    .into_iter()
                                    .map(|x| x / total_zoom)
                                    .collect::<Vec<f64>>(),
                            );

                            cx.fill(select_rect, &*SELECTION_FILL_COLOR);
                            cx.stroke_styled(
                                select_rect,
                                &*OUTLINE_COLOR,
                                Self::SELECTION_OUTLINE_WIDTH / total_zoom,
                                &stroke_style,
                            );
                        }
                    }
                    SelectorStyle::Single => {
                        if let Some(last) = path.last() {
                            cx.stroke(
                                kurbo::Circle::new(
                                    last.pos.to_kurbo_point(),
                                    Self::SINGLE_SELECTING_CIRCLE_RADIUS / total_zoom,
                                ),
                                &*OUTLINE_COLOR,
                                Self::SELECTION_OUTLINE_WIDTH / total_zoom,
                            );
                        }
                    }
                    SelectorStyle::IntersectingPath => {
                        let mut bez_path = kurbo::BezPath::new();
                        let mut path_iter = path.iter();

                        if let Some(first) = path_iter.next() {
                            bez_path.move_to(first.pos.to_kurbo_point());

                            for element in path_iter {
                                bez_path.line_to(element.pos.to_kurbo_point());
                            }

                            let mut stroke_style = piet::StrokeStyle::new();
                            stroke_style.set_dash_pattern(
                                Self::SELECTING_DASH_PATTERN
                                    .into_iter()
                                    .map(|x| x / total_zoom)
                                    .collect::<Vec<f64>>(),
                            );

                            cx.stroke_styled(
                                bez_path,
                                &*OUTLINE_COLOR,
                                Self::SELECTION_OUTLINE_WIDTH / total_zoom,
                                &stroke_style,
                            );
                        }
                    }
                }
            }
            SelectorState::ModifySelection {
                modify_state,
                selection,
                selection_bounds,
            } => {
                // Draw the bounds outlines for the selected strokes
                static SELECTED_BOUNDS_COLOR: Lazy<piet::Color> =
                    Lazy::new(|| color::GNOME_BLUES[1].with_alpha(0.376));

                let selected_bounds_width = 1.5 / total_zoom;
                for stroke in engine_view.store.get_strokes_ref(selection) {
                    cx.stroke(
                        stroke.bounds().to_kurbo_rect(),
                        &*SELECTED_BOUNDS_COLOR,
                        selected_bounds_width,
                    );
                }

                Self::draw_selection_overlay(
                    cx,
                    *selection_bounds,
                    modify_state,
                    engine_view.camera,
                )?;

                match modify_state {
                    ModifyState::Rotate {
                        rotation_center,
                        start_rotation_angle,
                        current_rotation_angle,
                    } => {
                        Self::draw_rotation_indicator(
                            cx,
                            *rotation_center,
                            *start_rotation_angle,
                            *current_rotation_angle,
                            engine_view.camera,
                        )?;
                    }
                    _ => {}
                }
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

static OUTLINE_COLOR: Lazy<piet::Color> = Lazy::new(|| color::GNOME_BRIGHTS[4].with_alpha(0.941));
static SELECTION_FILL_COLOR: Lazy<piet::Color> =
    Lazy::new(|| color::GNOME_BRIGHTS[2].with_alpha(0.090));

impl Selector {
    /// The threshold where a translation is applied ( in offset magnitude, surface coords )
    const TRANSLATE_MAGNITUDE_THRESHOLD: f64 = 1.0;
    /// The threshold angle (rad) where a rotation is applied
    const ROTATE_ANGLE_THRESHOLD: f64 = ((2.0 * std::f64::consts::PI) / 360.0) * 0.2;

    const SELECTION_OUTLINE_WIDTH: f64 = 1.5;
    const SELECTING_DASH_PATTERN: [f64; 2] = [12.0, 6.0];

    const SINGLE_SELECTING_CIRCLE_RADIUS: f64 = 4.0;

    /// resize node size, in surface coords
    const RESIZE_NODE_SIZE: na::Vector2<f64> = na::vector![18.0, 18.0];
    /// rotate node size, in surface coords
    const ROTATE_NODE_SIZE: f64 = 18.0;

    fn add_to_select_path(style: SelectorStyle, path: &mut Vec<Element>, element: Element) {
        match style {
            SelectorStyle::Polygon | SelectorStyle::Single | SelectorStyle::IntersectingPath => {
                path.push(element);
            }
            SelectorStyle::Rectangle => {
                path.push(element);

                if path.len() > 2 {
                    path.resize(2, Element::default());
                    path.insert(1, element);
                }
            }
        }
    }

    fn resize_node_bounds(position: ResizeCorner, selection_bounds: Aabb, camera: &Camera) -> Aabb {
        let total_zoom = camera.total_zoom();
        match position {
            ResizeCorner::TopLeft => Aabb::from_half_extents(
                na::point![selection_bounds.mins[0], selection_bounds.mins[1]],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::TopRight => Aabb::from_half_extents(
                na::point![selection_bounds.maxs[0], selection_bounds.mins[1]],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::BottomLeft => Aabb::from_half_extents(
                na::point![selection_bounds.mins[0], selection_bounds.maxs[1]],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::BottomRight => Aabb::from_half_extents(
                na::point![selection_bounds.maxs[0], selection_bounds.maxs[1]],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
        }
    }

    fn rotate_node_sphere(selection_bounds: Aabb, camera: &Camera) -> BoundingSphere {
        let total_zoom = camera.total_zoom();
        let pos = na::point![
            selection_bounds.maxs[0],
            (selection_bounds.maxs[1] + selection_bounds.mins[1]) * 0.5
        ];
        BoundingSphere::new(pos, Self::ROTATE_NODE_SIZE * 0.5 / total_zoom)
    }

    fn draw_selection_overlay(
        piet_cx: &mut impl RenderContext,
        selection_bounds: Aabb,
        modify_state: &ModifyState,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let total_zoom = camera.total_zoom();

        let rotate_node_state = match modify_state {
            ModifyState::Rotate { .. } => PenState::Down,
            _ => PenState::Up,
        };
        let rotate_node_sphere = Self::rotate_node_sphere(selection_bounds, camera);

        let resize_tl_node_state = match modify_state {
            ModifyState::Resize {
                from_corner: ResizeCorner::TopLeft,
                ..
            } => PenState::Down,
            _ => PenState::Up,
        };
        let resize_tl_node_bounds =
            Self::resize_node_bounds(ResizeCorner::TopLeft, selection_bounds, camera);

        let resize_tr_node_state = match modify_state {
            ModifyState::Resize {
                from_corner: ResizeCorner::TopRight,
                ..
            } => PenState::Down,
            _ => PenState::Up,
        };
        let resize_tr_node_bounds =
            Self::resize_node_bounds(ResizeCorner::TopRight, selection_bounds, camera);

        let resize_bl_node_state = match modify_state {
            ModifyState::Resize {
                from_corner: ResizeCorner::BottomLeft,
                ..
            } => PenState::Down,
            _ => PenState::Up,
        };
        let resize_bl_node_bounds =
            Self::resize_node_bounds(ResizeCorner::BottomLeft, selection_bounds, camera);

        let resize_br_node_state = match modify_state {
            ModifyState::Resize {
                from_corner: ResizeCorner::BottomRight,
                ..
            } => PenState::Down,
            _ => PenState::Up,
        };
        let resize_br_node_bounds =
            Self::resize_node_bounds(ResizeCorner::BottomRight, selection_bounds, camera);

        // Selection rect
        let selection_rect = selection_bounds.to_kurbo_rect();

        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let mut clip_path = kurbo::BezPath::new();
        clip_path.extend(
            drawhelpers::rectangular_node_shape(
                resize_tl_node_state,
                resize_tl_node_bounds,
                total_zoom,
            )
            .path_elements(0.1),
        );
        clip_path.extend(
            drawhelpers::rectangular_node_shape(
                resize_tr_node_state,
                resize_tr_node_bounds,
                total_zoom,
            )
            .path_elements(0.1),
        );
        clip_path.extend(
            drawhelpers::rectangular_node_shape(
                resize_bl_node_state,
                resize_bl_node_bounds,
                total_zoom,
            )
            .path_elements(0.1),
        );
        clip_path.extend(
            drawhelpers::rectangular_node_shape(
                resize_br_node_state,
                resize_br_node_bounds,
                total_zoom,
            )
            .path_elements(0.1),
        );

        clip_path.extend(
            drawhelpers::circular_node_shape(rotate_node_state, rotate_node_sphere, total_zoom)
                .path_elements(0.1),
        );
        // enclosing the shapes with the selector (!) bounds ( in reversed winding ),
        // so that the inner shapes become the exterior for correct clipping
        clip_path.extend(
            kurbo::Rect::new(
                selection_bounds.maxs[0] + Self::SELECTION_OUTLINE_WIDTH / total_zoom,
                selection_bounds.mins[1] - Self::SELECTION_OUTLINE_WIDTH / total_zoom,
                selection_bounds.mins[0] - Self::SELECTION_OUTLINE_WIDTH / total_zoom,
                selection_bounds.maxs[1] + Self::SELECTION_OUTLINE_WIDTH / total_zoom,
            )
            .path_elements(0.1),
        );

        piet_cx.clip(clip_path);

        piet_cx.fill(selection_rect, &*SELECTION_FILL_COLOR);
        piet_cx.stroke(
            selection_rect,
            &*OUTLINE_COLOR,
            Selector::SELECTION_OUTLINE_WIDTH / total_zoom,
        );

        piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        // Rotate Node
        drawhelpers::draw_circular_node(piet_cx, rotate_node_state, rotate_node_sphere, total_zoom);

        // Resize Nodes
        drawhelpers::draw_rectangular_node(
            piet_cx,
            resize_tl_node_state,
            resize_tl_node_bounds,
            total_zoom,
        );
        drawhelpers::draw_rectangular_node(
            piet_cx,
            resize_tr_node_state,
            resize_tr_node_bounds,
            total_zoom,
        );
        drawhelpers::draw_rectangular_node(
            piet_cx,
            resize_bl_node_state,
            resize_bl_node_bounds,
            total_zoom,
        );
        drawhelpers::draw_rectangular_node(
            piet_cx,
            resize_br_node_state,
            resize_br_node_bounds,
            total_zoom,
        );

        piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }

    fn draw_rotation_indicator(
        piet_cx: &mut impl RenderContext,
        rotation_center: na::Point2<f64>,
        start_rotation_angle: f64,
        current_rotation_angle: f64,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        const CENTER_CROSS_COLOR: Color = Color {
            r: 0.964,
            g: 0.380,
            b: 0.317,
            a: 1.0,
        };
        let total_zoom = camera.total_zoom();
        let center_cross_half_extents: f64 = 10.0 / total_zoom;
        let center_cross_path_width: f64 = 1.5 / total_zoom;

        let mut center_cross = kurbo::BezPath::new();
        center_cross.move_to(
            (rotation_center.coords + na::vector![-center_cross_half_extents, 0.0])
                .to_kurbo_point(),
        );
        center_cross.line_to(
            (rotation_center.coords + na::vector![center_cross_half_extents, 0.0]).to_kurbo_point(),
        );
        center_cross.move_to(
            (rotation_center.coords + na::vector![0.0, -center_cross_half_extents])
                .to_kurbo_point(),
        );
        center_cross.line_to(
            (rotation_center.coords + na::vector![0.0, center_cross_half_extents]).to_kurbo_point(),
        );

        piet_cx.transform(
            kurbo::Affine::translate(rotation_center.coords.to_kurbo_vec())
                * kurbo::Affine::rotate(current_rotation_angle - start_rotation_angle)
                * kurbo::Affine::translate(-rotation_center.coords.to_kurbo_vec()),
        );

        piet_cx.stroke(
            center_cross,
            &piet::Color::from(CENTER_CROSS_COLOR),
            center_cross_path_width,
        );
        piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        Ok(())
    }
}
