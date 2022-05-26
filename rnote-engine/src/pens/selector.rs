use super::penbehaviour::{PenBehaviour, PenProgress};
use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::{Camera, DrawOnDocBehaviour, WidgetFlags};
use kurbo::Shape;
use p2d::query::PointQuery;
use piet::RenderContext;
use rnote_compose::helpers::{AABBHelpers, Vector2Helpers};
use rnote_compose::penhelpers::{KeyboardKey, PenState};
use rnote_compose::penhelpers::{PenEvent, ShortcutKey};
use rnote_compose::penpath::Element;
use rnote_compose::style::drawhelpers;
use rnote_compose::{color, Color};

use p2d::bounding_volume::{BoundingSphere, BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

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
        start_bounds: AABB,
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
        selection_bounds: AABB,
    },
}

impl Default for SelectorState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "selector_style")]
pub enum SelectorType {
    #[serde(rename = "polygon")]
    Polygon,
    #[serde(rename = "rectangle")]
    Rectangle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "selector")]
pub struct Selector {
    #[serde(rename = "style")]
    pub style: SelectorType,
    #[serde(rename = "resize_lock_aspectratio")]
    pub resize_lock_aspectratio: bool,
    #[serde(skip)]
    pub(super) state: SelectorState,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            style: SelectorType::Rectangle,
            resize_lock_aspectratio: false,
            state: SelectorState::default(),
        }
    }
}

impl PenBehaviour for Selector {
    fn handle_event(
        &mut self,
        event: PenEvent,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        //log::debug!("selector state: {:?}, event: {:?}", &self.state, &event);

        let pen_progress = match (&mut self.state, event) {
            (SelectorState::Idle, PenEvent::Down { element, .. }) => {
                widget_flags.merge_with_other(engine_view.store.record());

                // Deselect on start
                let selection_keys = engine_view
                    .store
                    .selection_keys_as_rendered_intersecting_bounds(engine_view.camera.viewport());
                engine_view.store.set_selected_keys(&selection_keys, false);

                self.state = SelectorState::Selecting {
                    path: vec![element],
                };

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;
                widget_flags.hide_scrollbars = Some(true);

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
                        let all_keys = engine_view.store.keys_sorted_chrono();

                        if let Some(new_selection_bounds) =
                            engine_view.store.bounds_for_strokes(&all_keys)
                        {
                            engine_view.store.set_selected_keys(&all_keys, true);

                            self.state = SelectorState::ModifySelection {
                                modify_state: ModifyState::default(),
                                selection: all_keys,
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
                Self::add_to_select_path(self.style, path, element);

                widget_flags.redraw = true;

                PenProgress::InProgress
            }
            (SelectorState::Selecting { path }, PenEvent::Up { .. }) => {
                let mut state = SelectorState::Idle;
                let mut pen_progress = PenProgress::Finished;

                if let Some(selection) =
                    match self.style {
                        SelectorType::Polygon => {
                            if path.len() < 3 {
                                None
                            } else {
                                Some(engine_view.store.select_keys_intersecting_polygon_path(
                                    path,
                                    engine_view.camera.viewport(),
                                ))
                            }
                        }
                        SelectorType::Rectangle => {
                            if let (Some(first), Some(last)) = (path.first(), path.last()) {
                                let aabb = AABB::new_positive(
                                    na::Point2::from(first.pos),
                                    na::Point2::from(last.pos),
                                );
                                Some(engine_view.store.select_keys_intersecting_aabb(
                                    aabb,
                                    engine_view.camera.viewport(),
                                ))
                            } else {
                                None
                            }
                        }
                    }
                {
                    if let Some(selection_bounds) = engine_view.store.bounds_for_strokes(&selection)
                    {
                        // Change to the modifiy state
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
                widget_flags.hide_scrollbars = Some(false);

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
                        let all_keys = engine_view.store.keys_sorted_chrono();

                        if let Some(new_selection_bounds) =
                            engine_view.store.bounds_for_strokes(&all_keys)
                        {
                            engine_view.store.set_selected_keys(&all_keys, true);

                            self.state = SelectorState::ModifySelection {
                                modify_state: ModifyState::default(),
                                selection: all_keys,
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
                let selection_keys = engine_view
                    .store
                    .selection_keys_as_rendered_intersecting_bounds(engine_view.camera.viewport());
                engine_view.store.set_selected_keys(&selection_keys, false);

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;
                widget_flags.hide_scrollbars = Some(false);

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
                        widget_flags.merge_with_other(engine_view.store.record());

                        if Self::rotate_node_sphere(*selection_bounds, engine_view.camera)
                            .contains_local_point(&na::Point2::from(element.pos))
                        {
                            let rotation_angle = {
                                let vec = element.pos - selection_bounds.center().coords;
                                na::Vector2::x().angle_ahead(&vec)
                            };

                            *modify_state = ModifyState::Rotate {
                                rotation_center: selection_bounds.center(),
                                start_rotation_angle: rotation_angle,
                                current_rotation_angle: rotation_angle,
                            };
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
                            *modify_state = ModifyState::Translate {
                                start_pos: element.pos,
                                current_pos: element.pos,
                            };
                        } else {
                            // If clicking outside the selection, reset
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
                                *selection_bounds = new_bounds;
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

                        let new_extents = if self.resize_lock_aspectratio
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
                    *selection_bounds = new_bounds;
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
                KeyboardKey::Unicode('a') => {
                    // Select all keys
                    if shortcut_keys.contains(&ShortcutKey::KeyboardCtrl) {
                        let all_keys = engine_view.store.keys_sorted_chrono();

                        if let Some(new_selection_bounds) =
                            engine_view.store.bounds_for_strokes(&all_keys)
                        {
                            engine_view.store.set_selected_keys(&all_keys, true);

                            self.state = SelectorState::ModifySelection {
                                modify_state: ModifyState::default(),
                                selection: all_keys,
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
        };

        (pen_progress, widget_flags)
    }

    fn update_internal_state(&mut self, engine_view: &EngineView) {
        let selection = engine_view.store.selection_keys_as_rendered();

        if let Some(selection_bounds) = engine_view.store.bounds_for_strokes(&selection) {
            self.state = SelectorState::ModifySelection {
                modify_state: ModifyState::default(),
                selection,
                selection_bounds,
            };
        } else {
            self.state = SelectorState::Idle;
        }
    }
}

impl DrawOnDocBehaviour for Selector {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<AABB> {
        let total_zoom = engine_view.camera.total_zoom();

        match &self.state {
            SelectorState::Idle => None,
            SelectorState::Selecting { path } => {
                // Making sure bounds are always outside of coord + width
                let mut path_iter = path.iter();
                if let Some(first) = path_iter.next() {
                    let mut new_bounds = AABB::from_half_extents(
                        na::Point2::from(first.pos),
                        na::Vector2::repeat(Self::SELECTION_OUTLINE_WIDTH / total_zoom),
                    );

                    path_iter.for_each(|element| {
                        let pos_bounds = AABB::from_half_extents(
                            na::Point2::from(element.pos),
                            na::Vector2::repeat(Self::SELECTION_OUTLINE_WIDTH / total_zoom),
                        );
                        new_bounds.merge(&pos_bounds);
                    });

                    Some(new_bounds)
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
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
        let total_zoom = engine_view.camera.total_zoom();

        match &self.state {
            SelectorState::Idle => {}
            SelectorState::Selecting { path } => {
                let mut bez_path = kurbo::BezPath::new();

                match self.style {
                    SelectorType::Polygon => {
                        for (i, element) in path.iter().enumerate() {
                            if i == 0 {
                                bez_path.move_to((element.pos).to_kurbo_point());
                            } else {
                                bez_path.line_to((element.pos).to_kurbo_point());
                            }
                        }
                    }
                    SelectorType::Rectangle => {
                        if let (Some(first), Some(last)) = (path.first(), path.last()) {
                            bez_path.move_to(first.pos.to_kurbo_point());
                            bez_path.line_to(kurbo::Point::new(last.pos[0], first.pos[1]));
                            bez_path.line_to(kurbo::Point::new(last.pos[0], last.pos[1]));
                            bez_path.line_to(kurbo::Point::new(first.pos[0], last.pos[1]));
                            bez_path.line_to(kurbo::Point::new(first.pos[0], first.pos[1]));
                        }
                    }
                }
                bez_path.close_path();

                cx.fill(bez_path.clone(), &Self::SELECTION_FILL_COLOR);
                cx.stroke_styled(
                    bez_path,
                    &Self::OUTLINE_COLOR,
                    Self::SELECTION_OUTLINE_WIDTH / total_zoom,
                    &piet::StrokeStyle::new().dash_pattern(&Self::SELECTING_DASH_PATTERN),
                );
            }
            SelectorState::ModifySelection {
                modify_state,
                selection_bounds,
                ..
            } => {
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

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

impl Selector {
    /// The threshold where a translation is applied ( in offset magnitude, surface coords )
    const TRANSLATE_MAGNITUDE_THRESHOLD: f64 = 1.0;
    /// The threshold angle (rad) where a rotation is applied
    const ROTATE_ANGLE_THRESHOLD: f64 = ((2.0 * std::f64::consts::PI) / 360.0) * 0.2;

    const SELECTION_OUTLINE_WIDTH: f64 = 1.5;
    const OUTLINE_COLOR: piet::Color = color::GNOME_BRIGHTS[4].with_a8(0xf0);
    const SELECTION_FILL_COLOR: piet::Color = color::GNOME_BRIGHTS[2].with_a8(0x17);
    const SELECTING_DASH_PATTERN: [f64; 2] = [12.0, 6.0];

    /// resize node size, in surface coords
    const RESIZE_NODE_SIZE: na::Vector2<f64> = na::vector![18.0, 18.0];
    /// rotate node size, in surface coords
    const ROTATE_NODE_SIZE: f64 = 18.0;

    fn add_to_select_path(style: SelectorType, path: &mut Vec<Element>, element: Element) {
        match style {
            SelectorType::Polygon => {
                path.push(element);
            }
            SelectorType::Rectangle => {
                path.push(element);

                if path.len() > 2 {
                    path.resize(2, Element::default());
                    path.insert(1, element);
                }
            }
        }
    }

    fn resize_node_bounds(position: ResizeCorner, selection_bounds: AABB, camera: &Camera) -> AABB {
        let total_zoom = camera.total_zoom();
        match position {
            ResizeCorner::TopLeft => AABB::from_half_extents(
                na::point![selection_bounds.mins[0], selection_bounds.mins[1]],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::TopRight => AABB::from_half_extents(
                na::point![selection_bounds.maxs[0], selection_bounds.mins[1]],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::BottomLeft => AABB::from_half_extents(
                na::point![selection_bounds.mins[0], selection_bounds.maxs[1]],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::BottomRight => AABB::from_half_extents(
                na::point![selection_bounds.maxs[0], selection_bounds.maxs[1]],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
        }
    }

    fn rotate_node_sphere(selection_bounds: AABB, camera: &Camera) -> BoundingSphere {
        let total_zoom = camera.total_zoom();
        let pos = na::point![
            selection_bounds.maxs[0],
            (selection_bounds.maxs[1] + selection_bounds.mins[1]) * 0.5
        ];
        BoundingSphere::new(pos, Self::ROTATE_NODE_SIZE * 0.5 / total_zoom)
    }

    fn draw_selection_overlay(
        piet_cx: &mut impl RenderContext,
        selection_bounds: AABB,
        modify_state: &ModifyState,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
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

        piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

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

        piet_cx.fill(selection_rect, &Selector::SELECTION_FILL_COLOR);
        piet_cx.stroke(
            selection_rect,
            &Selector::OUTLINE_COLOR,
            Selector::SELECTION_OUTLINE_WIDTH / total_zoom,
        );

        piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;

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

        piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }

    fn draw_rotation_indicator(
        piet_cx: &mut impl RenderContext,
        rotation_center: na::Point2<f64>,
        start_rotation_angle: f64,
        current_rotation_angle: f64,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
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
        piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(())
    }
}
