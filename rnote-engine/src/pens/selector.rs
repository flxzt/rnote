use super::penbehaviour::{PenBehaviour, PenProgress};
use super::AudioPlayer;
use crate::sheet::Sheet;
use crate::store::StrokeKey;
use crate::{Camera, DrawOnSheetBehaviour, StrokeStore, SurfaceFlags};
use p2d::query::PointQuery;
use piet::RenderContext;
use rnote_compose::helpers::{AABBHelpers, Vector2Helpers};
use rnote_compose::penpath::Element;
use rnote_compose::style::drawhelpers::{self, NodeState};
use rnote_compose::{color, Color, PenEvent};

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
        pos: na::Vector2<f64>,
    },
    Rotate {
        rotation_center: na::Point2<f64>,
        start_rotation_angle: f64,
        current_rotation_angle: f64,
    },
    Resize {
        from_corner: ResizeCorner,
        start_bounds: AABB,
        resize_pos: na::Vector2<f64>,
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
        Self::Selecting { path: vec![] }
    }
}

impl SelectorState {
    fn reset() -> Self {
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
            style: SelectorType::Polygon,
            resize_lock_aspectratio: false,
            state: SelectorState::default(),
        }
    }
}

impl PenBehaviour for Selector {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _sheet: &mut Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) -> (PenProgress, SurfaceFlags) {
        let mut surface_flags = SurfaceFlags::default();
        let total_zoom = camera.total_zoom();

        let pen_progress = match (&mut self.state, event) {
            (SelectorState::Idle, PenEvent::Down { element, .. }) => {
                // Deselect by default
                let keys = store.keys_sorted_chrono_intersecting_bounds(camera.viewport());
                store.set_selected_keys(&keys, false);

                self.state = SelectorState::Selecting {
                    path: vec![element],
                };

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(true);

                PenProgress::InProgress
            }
            (SelectorState::Idle, _) => {
                // already idle, so nothing to do
                PenProgress::Idle
            }
            (SelectorState::Selecting { path }, PenEvent::Down { element, .. }) => {
                Self::add_to_select_path(self.style, path, element);
                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (SelectorState::Selecting { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (SelectorState::Selecting { path }, PenEvent::Up { .. }) => {
                let mut state = SelectorState::reset();
                let mut pen_progress = PenProgress::Finished;

                if let Some(selection) = match self.style {
                    SelectorType::Polygon => {
                        if path.len() < 3 {
                            None
                        } else {
                            Some(store.update_selection_for_polygon_path(&path, camera.viewport()))
                        }
                    }
                    SelectorType::Rectangle => {
                        if let (Some(first), Some(last)) = (path.first(), path.last()) {
                            let aabb = AABB::new_positive(
                                na::Point2::from(first.pos),
                                na::Point2::from(last.pos),
                            );
                            Some(store.update_selection_for_aabb(aabb, camera.viewport()))
                        } else {
                            None
                        }
                    }
                } {
                    if let Some(selection_bounds) = store.gen_bounds(&selection) {
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

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(false);

                pen_progress
            }
            (SelectorState::Selecting { .. }, PenEvent::Cancel) => {
                self.state = SelectorState::reset();

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(false);
                PenProgress::Finished
            }
            (
                SelectorState::ModifySelection {
                    modify_state,
                    selection,
                    selection_bounds,
                },
                PenEvent::Down { element, .. },
            ) => {
                let mut pen_progress = PenProgress::InProgress;

                match modify_state {
                    ModifyState::Up => {
                        if Self::rotate_node_sphere(*selection_bounds, camera)
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
                            camera,
                        )
                        .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::TopLeft,
                                start_bounds: *selection_bounds,
                                resize_pos: element.pos,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::TopRight,
                            *selection_bounds,
                            camera,
                        )
                        .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::TopRight,
                                start_bounds: *selection_bounds,
                                resize_pos: element.pos,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::BottomLeft,
                            *selection_bounds,
                            camera,
                        )
                        .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::BottomLeft,
                                start_bounds: *selection_bounds,
                                resize_pos: element.pos,
                            }
                        } else if Self::resize_node_bounds(
                            ResizeCorner::BottomRight,
                            *selection_bounds,
                            camera,
                        )
                        .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Resize {
                                from_corner: ResizeCorner::BottomRight,
                                start_bounds: *selection_bounds,
                                resize_pos: element.pos,
                            }
                        } else if selection_bounds
                            .contains_local_point(&na::Point2::from(element.pos))
                        {
                            *modify_state = ModifyState::Translate { pos: element.pos };
                        } else {
                            // If clicking outside the selection, reset
                            store.set_selected_keys(selection, false);
                            self.state = SelectorState::reset();

                            pen_progress = PenProgress::Finished;
                        }
                    }
                    ModifyState::Translate { pos } => {
                        let offset = element.pos - *pos;

                        if offset.magnitude() > Self::TRANSLATE_MAGNITUDE_THRESHOLD / total_zoom {
                            store.translate_strokes(selection, offset);
                            *selection_bounds = selection_bounds.translate(offset);
                            // strokes that were far away previously might come into view
                            store.regenerate_rendering_in_viewport_threaded(
                                false,
                                camera.viewport_extended(),
                                camera.image_scale(),
                            );

                            *pos = element.pos;
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
                            store.rotate_strokes(selection, angle_delta, *rotation_center);
                            store.regenerate_rendering_in_viewport_threaded(
                                false,
                                camera.viewport_extended(),
                                camera.image_scale(),
                            );

                            if let Some(new_bounds) = store.gen_bounds(selection) {
                                *selection_bounds = new_bounds;
                            }
                            *current_rotation_angle = new_rotation_angle;
                        }
                    }
                    ModifyState::Resize {
                        from_corner,
                        start_bounds,
                        resize_pos,
                    } => {
                        let pos_offset = {
                            let pos_offset = element.pos - *resize_pos;

                            match from_corner {
                                ResizeCorner::TopLeft => -pos_offset,
                                ResizeCorner::TopRight => {
                                    na::vector![pos_offset[0], -pos_offset[1]]
                                }
                                ResizeCorner::BottomLeft => {
                                    na::vector![-pos_offset[0], pos_offset[1]]
                                }
                                ResizeCorner::BottomRight => pos_offset,
                            }
                        };

                        if pos_offset.magnitude() > Self::RESIZE_MAGNITUDE_THRESHOLD / total_zoom {
                            let new_extents = if self.resize_lock_aspectratio {
                                // Lock aspectratio
                                rnote_compose::helpers::scale_w_locked_aspectratio(
                                    start_bounds.extents(),
                                    selection_bounds.extents() + pos_offset,
                                )
                            } else {
                                selection_bounds.extents() + pos_offset
                            }
                            .maxs(&((Self::RESIZE_NODE_SIZE * 2.0) / camera.total_zoom()));

                            let new_bounds = match from_corner {
                                ResizeCorner::TopLeft => AABB::new(
                                    na::point![
                                        start_bounds.maxs[0] - new_extents[0],
                                        start_bounds.maxs[1] - new_extents[1]
                                    ],
                                    na::point![start_bounds.maxs[0], start_bounds.maxs[1]],
                                ),
                                ResizeCorner::TopRight => AABB::new(
                                    na::point![
                                        start_bounds.mins[0],
                                        start_bounds.maxs[1] - new_extents[1]
                                    ],
                                    na::point![
                                        start_bounds.mins[0] + new_extents[0],
                                        start_bounds.maxs[1]
                                    ],
                                ),
                                ResizeCorner::BottomLeft => AABB::new(
                                    na::point![
                                        start_bounds.maxs[0] - new_extents[0],
                                        start_bounds.mins[1]
                                    ],
                                    na::point![
                                        start_bounds.maxs[0],
                                        start_bounds.mins[1] + new_extents[1]
                                    ],
                                ),
                                ResizeCorner::BottomRight => AABB::new(
                                    na::point![start_bounds.mins[0], start_bounds.mins[1]],
                                    na::point![
                                        start_bounds.mins[0] + new_extents[0],
                                        start_bounds.mins[1] + new_extents[1]
                                    ],
                                ),
                            };

                            store.resize_strokes(selection, *selection_bounds, new_bounds);
                            store.regenerate_rendering_in_viewport_threaded(
                                false,
                                camera.viewport_extended(),
                                camera.image_scale(),
                            );

                            *resize_pos = element.pos;
                            *selection_bounds = new_bounds;
                        }
                    }
                }

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;

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
                if let Some(new_bounds) = store.gen_bounds(selection) {
                    *selection_bounds = new_bounds;
                }
                *modify_state = ModifyState::Up;

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;
                surface_flags.resize_to_fit_strokes = true;

                PenProgress::InProgress
            }
            (SelectorState::ModifySelection { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (SelectorState::ModifySelection { .. }, PenEvent::Cancel) => {
                self.state = SelectorState::reset();

                PenProgress::Finished
            }
        };

        (pen_progress, surface_flags)
    }
}

impl DrawOnSheetBehaviour for Selector {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, camera: &Camera) -> Option<AABB> {
        let total_zoom = camera.total_zoom();

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

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        let total_zoom = camera.total_zoom();

        match &self.state {
            SelectorState::Idle => Ok(()),
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

                Ok(())
            }
            SelectorState::ModifySelection {
                modify_state,
                selection_bounds,
                ..
            } => {
                Self::draw_selection_overlay(cx, *selection_bounds, modify_state, camera);

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
                            camera,
                        );
                    }
                    _ => {}
                }
                Ok(())
            }
        }
    }
}

impl Selector {
    /// The threshold where a translation is applied ( in offset magnitude, surface coords )
    const TRANSLATE_MAGNITUDE_THRESHOLD: f64 = 1.0;
    /// The threshold angle (rad) where a rotation is applied
    const ROTATE_ANGLE_THRESHOLD: f64 = (2.0 * std::f64::consts::PI) / 360.0;
    /// The threshold where a resize is applied ( in offset magnitude, surface coords )
    const RESIZE_MAGNITUDE_THRESHOLD: f64 = 1.0;

    const SELECTION_OUTLINE_WIDTH: f64 = 1.8;
    const OUTLINE_COLOR: piet::Color = color::GNOME_BRIGHTS[4].with_a8(0xf0);
    const SELECTION_FILL_COLOR: piet::Color = color::GNOME_BRIGHTS[2].with_a8(0x17);
    const SELECTING_DASH_PATTERN: [f64; 2] = [12.0, 6.0];

    const RESIZE_NODE_SIZE: na::Vector2<f64> = na::vector![16.0, 16.0];
    const ROTATE_NODE_SIZE: f64 = 16.0;

    /// Sets the state to a selection
    pub fn set_selection(&mut self, selection: Vec<StrokeKey>, selection_bounds: AABB) {
        self.state = SelectorState::ModifySelection {
            modify_state: ModifyState::default(),
            selection,
            selection_bounds,
        };
    }

    pub fn reset(&mut self) {
        self.state = SelectorState::reset();
    }

    pub fn update_selection_from_state(&mut self, store: &StrokeStore) {
        let selection = store.selection_keys_unordered();
        let selection_bounds = store.gen_bounds(&selection);
        if let Some(selection_bounds) = selection_bounds {
            self.set_selection(selection, selection_bounds);
        } else {
            self.reset();
        }
    }

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
    ) {
        let total_zoom = camera.total_zoom();

        // Selection rect
        {
            let rect = selection_bounds
                .tightened(Selector::SELECTION_OUTLINE_WIDTH / total_zoom)
                .to_kurbo_rect();

            piet_cx.fill(rect.clone(), &Selector::SELECTION_FILL_COLOR);
            piet_cx.stroke(
                rect,
                &Selector::OUTLINE_COLOR,
                Selector::SELECTION_OUTLINE_WIDTH / total_zoom,
            );
        }

        // Rotate Node
        {
            let rotate_node_state = match modify_state {
                ModifyState::Rotate { .. } => NodeState::Down,
                _ => NodeState::Up,
            };

            drawhelpers::draw_circular_node(
                piet_cx,
                rotate_node_state,
                Self::rotate_node_sphere(selection_bounds, camera),
                total_zoom,
            );
        }

        // Resize Nodes
        {
            let tl_node_state = match modify_state {
                ModifyState::Resize {
                    from_corner: ResizeCorner::TopLeft,
                    ..
                } => NodeState::Down,
                _ => NodeState::Up,
            };

            drawhelpers::draw_rectangular_node(
                piet_cx,
                tl_node_state,
                Self::resize_node_bounds(ResizeCorner::TopLeft, selection_bounds, camera),
                total_zoom,
            );
        }

        {
            let tr_node_state = match modify_state {
                ModifyState::Resize {
                    from_corner: ResizeCorner::TopRight,
                    ..
                } => NodeState::Down,
                _ => NodeState::Up,
            };

            drawhelpers::draw_rectangular_node(
                piet_cx,
                tr_node_state,
                Self::resize_node_bounds(ResizeCorner::TopRight, selection_bounds, camera),
                total_zoom,
            );
        }

        {
            let bl_node_state = match modify_state {
                ModifyState::Resize {
                    from_corner: ResizeCorner::BottomLeft,
                    ..
                } => NodeState::Down,
                _ => NodeState::Up,
            };

            drawhelpers::draw_rectangular_node(
                piet_cx,
                bl_node_state,
                Self::resize_node_bounds(ResizeCorner::BottomLeft, selection_bounds, camera),
                total_zoom,
            );
        }

        {
            let br_node_state = match modify_state {
                ModifyState::Resize {
                    from_corner: ResizeCorner::BottomRight,
                    ..
                } => NodeState::Down,
                _ => NodeState::Up,
            };

            drawhelpers::draw_rectangular_node(
                piet_cx,
                br_node_state,
                Self::resize_node_bounds(ResizeCorner::BottomRight, selection_bounds, camera),
                total_zoom,
            );
        }
    }

    fn draw_rotation_indicator(
        piet_cx: &mut impl RenderContext,
        rotation_center: na::Point2<f64>,
        start_rotation_angle: f64,
        current_rotation_angle: f64,
        camera: &Camera,
    ) {
        const CENTER_CROSS_COLOR: Color = Color {
            r: 0.964,
            g: 0.380,
            b: 0.317,
            a: 1.0,
        };
        let total_zoom = camera.total_zoom();
        let center_cross_radius: f64 = 10.0 / total_zoom;
        let center_cross_path_width: f64 = 1.0 / total_zoom;

        let mut center_cross = kurbo::BezPath::new();
        center_cross.move_to(
            (rotation_center.coords + na::vector![-center_cross_radius, 0.0]).to_kurbo_point(),
        );
        center_cross.line_to(
            (rotation_center.coords + na::vector![center_cross_radius, 0.0]).to_kurbo_point(),
        );
        center_cross.move_to(
            (rotation_center.coords + na::vector![0.0, -center_cross_radius]).to_kurbo_point(),
        );
        center_cross.line_to(
            (rotation_center.coords + na::vector![0.0, center_cross_radius]).to_kurbo_point(),
        );

        piet_cx.save().unwrap();
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
        piet_cx.restore().unwrap();
    }
}
