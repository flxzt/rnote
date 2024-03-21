// Modules
mod penevents;

// Imports
use super::pensconfig::selectorconfig::SelectorStyle;
use super::PenBehaviour;
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut, StrokeContent};
use crate::render::Svg;
use crate::snap::SnapCorner;
use crate::store::StrokeKey;
use crate::strokes::Content;
use crate::{Camera, DrawableOnDoc, Engine, WidgetFlags};
use futures::channel::oneshot;
use kurbo::Shape;
use p2d::bounding_volume::{Aabb, BoundingSphere, BoundingVolume};
use p2d::query::PointQuery;
use piet::RenderContext;
use rnote_compose::ext::{AabbExt, Vector2Ext};
use rnote_compose::penevent::{ModifierKey, PenEvent, PenProgress, PenState};
use rnote_compose::penpath::Element;
use rnote_compose::style::indicators;
use rnote_compose::EventResult;
use rnote_compose::{color, Color};
use std::time::Instant;

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
    Hover(na::Vector2<f64>),
    Translate {
        start_pos: na::Vector2<f64>,
        current_pos: na::Vector2<f64>,
        snap_corner: SnapCorner,
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
        last_rendered_bounds: Aabb,
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
    fn init(&mut self, _engine_view: &EngineView) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn deinit(&mut self) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn style(&self) -> PenStyle {
        PenStyle::Selector
    }

    fn update_state(&mut self, engine_view: &mut EngineViewMut) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        let selection = engine_view.store.selection_keys_as_rendered();

        self.state =
            if let Some(selection_bounds) = engine_view.store.bounds_for_strokes(&selection) {
                SelectorState::ModifySelection {
                    modify_state: ModifyState::default(),
                    selection,
                    selection_bounds,
                }
            } else {
                SelectorState::Idle
            };

        widget_flags.redraw = true;

        widget_flags
    }

    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        match event {
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
        }
    }

    fn fetch_clipboard_content(
        &self,
        engine_view: &EngineView,
    ) -> oneshot::Receiver<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>> {
        let widget_flags = WidgetFlags::default();
        let (sender, receiver) =
            oneshot::channel::<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>>();
        let mut clipboard_content = Vec::with_capacity(1);

        let stroke_content = if let SelectorState::ModifySelection { selection, .. } = &self.state {
            Some(engine_view.store.fetch_stroke_content(selection))
        } else {
            None
        };

        rayon::spawn(move || {
            let result = move || {
                if let Some(stroke_content) = stroke_content {
                    let stroke_content_svg = stroke_content.gen_svg(
                        false,
                        false,
                        false,
                        StrokeContent::CLIPBOARD_EXPORT_MARGIN,
                    )?;

                    // Add StrokeContent
                    clipboard_content.push((
                        serde_json::to_string(&stroke_content)?.into_bytes(),
                        StrokeContent::MIME_TYPE.to_string(),
                    ));
                    if let Some(stroke_content_svg) = stroke_content_svg {
                        // Add generated Svg
                        clipboard_content.push((
                            stroke_content_svg.svg_data.clone().into_bytes(),
                            Svg::MIME_TYPE.to_string(),
                        ));

                        // Add rendered Png
                        let image = stroke_content_svg
                            .gen_image(Engine::STROKE_EXPORT_IMAGE_SCALE)?
                            .into_encoded_bytes(image::ImageOutputFormat::Png)?;
                        clipboard_content.push((image, String::from("image/png")));
                    }
                }
                Ok((clipboard_content, widget_flags))
            };
            if sender.send(result()).is_err() {
                tracing::error!(
                    "Sending fetched selector clipboard content failed, receiver already dropped."
                );
            }
        });

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

        let stroke_content = if let SelectorState::ModifySelection { selection, .. } = &self.state {
            let c = Some(engine_view.store.cut_stroke_content(selection));
            self.state = SelectorState::Idle;
            widget_flags |= engine_view.store.record(Instant::now());
            widget_flags.store_modified = true;
            widget_flags.redraw = true;
            c
        } else {
            None
        };

        rayon::spawn(move || {
            let result = move || {
                if let Some(stroke_content) = stroke_content {
                    let stroke_content_svg = stroke_content.gen_svg(
                        false,
                        false,
                        false,
                        StrokeContent::CLIPBOARD_EXPORT_MARGIN,
                    )?;

                    // Add StrokeContent
                    clipboard_content.push((
                        serde_json::to_string(&stroke_content)?.into_bytes(),
                        StrokeContent::MIME_TYPE.to_string(),
                    ));
                    if let Some(stroke_content_svg) = stroke_content_svg {
                        // Add generated Svg
                        clipboard_content.push((
                            stroke_content_svg.svg_data.clone().into_bytes(),
                            Svg::MIME_TYPE.to_string(),
                        ));

                        // Add rendered Png
                        let image = stroke_content_svg
                            .gen_image(Engine::STROKE_EXPORT_IMAGE_SCALE)?
                            .into_encoded_bytes(image::ImageOutputFormat::Png)?;
                        clipboard_content.push((image, String::from("image/png")));
                    }
                }
                Ok((clipboard_content, widget_flags))
            };
            if sender.send(result()).is_err() {
                tracing::error!(
                    "Sending cut selector clipboard content failed, receiver already dropped."
                );
            }
        });

        receiver
    }
}

impl DrawableOnDoc for Selector {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        let total_zoom = engine_view.camera.total_zoom();

        match &self.state {
            SelectorState::Idle => None,
            SelectorState::Selecting { path } => {
                // Making sure bounds are always outside of coord + width
                let mut path_iter = path.iter();
                if let Some(first) = path_iter.next() {
                    let mut new_bounds = Aabb::from_half_extents(
                        first.pos.into(),
                        na::Vector2::repeat(Self::OUTLINE_STROKE_WIDTH / total_zoom),
                    );

                    path_iter.for_each(|element| {
                        let pos_bounds = Aabb::from_half_extents(
                            element.pos.into(),
                            na::Vector2::repeat(Self::OUTLINE_STROKE_WIDTH / total_zoom),
                        );
                        new_bounds.merge(&pos_bounds);
                    });

                    Some(new_bounds.loosened(Self::SELECTING_SINGLE_CIRCLE_RADIUS / total_zoom))
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

                            cx.fill(bez_path.clone(), &Self::SELECTION_FILL_COLOR);
                            cx.stroke_styled(
                                bez_path,
                                &Self::SELECTION_OUTLINE_COLOR,
                                Self::OUTLINE_STROKE_WIDTH / total_zoom,
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

                            cx.fill(select_rect, &Self::SELECTION_FILL_COLOR);
                            cx.stroke_styled(
                                select_rect,
                                &Self::SELECTION_OUTLINE_COLOR,
                                Self::OUTLINE_STROKE_WIDTH / total_zoom,
                                &stroke_style,
                            );
                        }
                    }
                    SelectorStyle::Single => {
                        if let Some(last) = path.last() {
                            cx.stroke(
                                kurbo::Circle::new(
                                    last.pos.to_kurbo_point(),
                                    Self::SELECTING_SINGLE_CIRCLE_RADIUS / total_zoom,
                                ),
                                &Self::SELECTION_OUTLINE_COLOR,
                                Self::OUTLINE_STROKE_WIDTH / total_zoom,
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
                                &Self::SELECTION_OUTLINE_COLOR,
                                Self::OUTLINE_STROKE_WIDTH / total_zoom,
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
                ..
            } => {
                // Draw the highlight for the selected strokes
                for stroke in engine_view.store.get_strokes_ref(selection) {
                    if let Err(e) = stroke.draw_highlight(cx, engine_view.camera.total_zoom()) {
                        tracing::error!("Failed to draw stroke highlight, Err: {e:?}");
                    }
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

impl Selector {
    /// The threshold where above it the translation is applied. In surface coordinates.
    const TRANSLATE_OFFSET_THRESHOLD: f64 = 1.414;
    /// The threshold angle (in radians) where above it the rotation is applied.
    const ROTATE_ANGLE_THRESHOLD: f64 = ((2.0 * std::f64::consts::PI) / 360.0) * 0.2;
    /// The outline stroke width when drawing a selection.
    const OUTLINE_STROKE_WIDTH: f64 = 2.0;
    /// The dash pattern while selecting.
    const SELECTING_DASH_PATTERN: [f64; 2] = [12.0, 6.0];
    /// The radius of the circle when selecting in single mode.
    const SELECTING_SINGLE_CIRCLE_RADIUS: f64 = 4.0;
    /// Resize node size, in surface coordinates.
    const RESIZE_NODE_SIZE: na::Vector2<f64> = na::vector![18.0, 18.0];
    /// Rotate node diameter, in surface coordinates.
    const ROTATE_NODE_DIAMETER: f64 = 18.0;
    /// The outline color when drawing a selection
    const SELECTION_OUTLINE_COLOR: piet::Color = color::GNOME_BRIGHTS[4].with_a8(240);
    /// The fill color when drawing a selection
    const SELECTION_FILL_COLOR: piet::Color = color::GNOME_BRIGHTS[2].with_a8(13);

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
                na::point![
                    selection_bounds.mins[0] - Self::RESIZE_NODE_SIZE[0] * 0.5 / total_zoom,
                    selection_bounds.mins[1] - Self::RESIZE_NODE_SIZE[0] * 0.5 / total_zoom
                ],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::TopRight => Aabb::from_half_extents(
                na::point![
                    selection_bounds.maxs[0] + Self::RESIZE_NODE_SIZE[0] * 0.5 / total_zoom,
                    selection_bounds.mins[1] - Self::RESIZE_NODE_SIZE[0] * 0.5 / total_zoom
                ],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::BottomLeft => Aabb::from_half_extents(
                na::point![
                    selection_bounds.mins[0] - Self::RESIZE_NODE_SIZE[0] * 0.5 / total_zoom,
                    selection_bounds.maxs[1] + Self::RESIZE_NODE_SIZE[0] * 0.5 / total_zoom
                ],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
            ResizeCorner::BottomRight => Aabb::from_half_extents(
                na::point![
                    selection_bounds.maxs[0] + Self::RESIZE_NODE_SIZE[0] * 0.5 / total_zoom,
                    selection_bounds.maxs[1] + Self::RESIZE_NODE_SIZE[0] * 0.5 / total_zoom
                ],
                Self::RESIZE_NODE_SIZE * 0.5 / total_zoom,
            ),
        }
    }

    fn rotate_node_sphere(selection_bounds: Aabb, camera: &Camera) -> BoundingSphere {
        let total_zoom = camera.total_zoom();
        let pos = na::point![
            selection_bounds.maxs[0] + Self::RESIZE_NODE_SIZE[0] / (2.0 * total_zoom),
            (selection_bounds.maxs[1] + selection_bounds.mins[1]) * 0.5
        ];
        BoundingSphere::new(pos, Self::ROTATE_NODE_DIAMETER * 0.5 / total_zoom)
    }

    fn draw_selection_overlay(
        piet_cx: &mut impl RenderContext,
        selection_bounds: Aabb,
        modify_state: &ModifyState,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let total_zoom = camera.total_zoom();

        let rotate_node_sphere = Self::rotate_node_sphere(selection_bounds, camera);
        let rotate_node_state = match modify_state {
            ModifyState::Rotate { .. } => PenState::Down,
            ModifyState::Hover(pos) => {
                if rotate_node_sphere.contains_local_point(&(*pos).into()) {
                    PenState::Proximity
                } else {
                    PenState::Up
                }
            }
            _ => PenState::Up,
        };

        let resize_tl_node_bounds =
            Self::resize_node_bounds(ResizeCorner::TopLeft, selection_bounds, camera);
        let resize_tl_node_state = match modify_state {
            ModifyState::Resize {
                from_corner: ResizeCorner::TopLeft,
                ..
            } => PenState::Down,
            ModifyState::Hover(pos) => {
                if resize_tl_node_bounds.contains_local_point(&(*pos).into()) {
                    PenState::Proximity
                } else {
                    PenState::Up
                }
            }
            _ => PenState::Up,
        };

        let resize_tr_node_bounds =
            Self::resize_node_bounds(ResizeCorner::TopRight, selection_bounds, camera);
        let resize_tr_node_state = match modify_state {
            ModifyState::Resize {
                from_corner: ResizeCorner::TopRight,
                ..
            } => PenState::Down,
            ModifyState::Hover(pos) => {
                if resize_tr_node_bounds.contains_local_point(&(*pos).into()) {
                    PenState::Proximity
                } else {
                    PenState::Up
                }
            }
            _ => PenState::Up,
        };

        let resize_bl_node_bounds =
            Self::resize_node_bounds(ResizeCorner::BottomLeft, selection_bounds, camera);
        let resize_bl_node_state = match modify_state {
            ModifyState::Resize {
                from_corner: ResizeCorner::BottomLeft,
                ..
            } => PenState::Down,
            ModifyState::Hover(pos) => {
                if resize_bl_node_bounds.contains_local_point(&(*pos).into()) {
                    PenState::Proximity
                } else {
                    PenState::Up
                }
            }
            _ => PenState::Up,
        };

        let resize_br_node_bounds =
            Self::resize_node_bounds(ResizeCorner::BottomRight, selection_bounds, camera);
        let resize_br_node_state = match modify_state {
            ModifyState::Resize {
                from_corner: ResizeCorner::BottomRight,
                ..
            } => PenState::Down,
            ModifyState::Hover(pos) => {
                if resize_br_node_bounds.contains_local_point(&(*pos).into()) {
                    PenState::Proximity
                } else {
                    PenState::Up
                }
            }
            _ => PenState::Up,
        };

        // Selection rect
        let selection_rect = selection_bounds.to_kurbo_rect();

        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let mut clip_path = kurbo::BezPath::new();
        clip_path.extend(
            indicators::rectangular_node_shape(
                resize_tl_node_state,
                resize_tl_node_bounds,
                total_zoom,
            )
            .path_elements(0.1),
        );
        clip_path.extend(
            indicators::rectangular_node_shape(
                resize_tr_node_state,
                resize_tr_node_bounds,
                total_zoom,
            )
            .path_elements(0.1),
        );
        clip_path.extend(
            indicators::rectangular_node_shape(
                resize_bl_node_state,
                resize_bl_node_bounds,
                total_zoom,
            )
            .path_elements(0.1),
        );
        clip_path.extend(
            indicators::rectangular_node_shape(
                resize_br_node_state,
                resize_br_node_bounds,
                total_zoom,
            )
            .path_elements(0.1),
        );

        clip_path.extend(
            indicators::circular_node_shape(rotate_node_state, rotate_node_sphere, total_zoom)
                .path_elements(0.1),
        );
        // enclosing the shapes with the selector (!) bounds ( in reversed winding ),
        // so that the inner shapes become the exterior for correct clipping
        clip_path.extend(
            kurbo::Rect::new(
                selection_bounds.maxs[0] + Self::OUTLINE_STROKE_WIDTH / total_zoom,
                selection_bounds.mins[1] - Self::OUTLINE_STROKE_WIDTH / total_zoom,
                selection_bounds.mins[0] - Self::OUTLINE_STROKE_WIDTH / total_zoom,
                selection_bounds.maxs[1] + Self::OUTLINE_STROKE_WIDTH / total_zoom,
            )
            .path_elements(0.1),
        );

        piet_cx.clip(clip_path);

        piet_cx.fill(selection_rect, &Self::SELECTION_FILL_COLOR);
        piet_cx.stroke(
            selection_rect,
            &Self::SELECTION_OUTLINE_COLOR,
            Self::OUTLINE_STROKE_WIDTH / total_zoom,
        );

        piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        // Rotate Node
        indicators::draw_circular_node(piet_cx, rotate_node_state, rotate_node_sphere, total_zoom);

        // Resize Nodes
        indicators::draw_rectangular_node(
            piet_cx,
            resize_tl_node_state,
            resize_tl_node_bounds,
            total_zoom,
        );
        indicators::draw_rectangular_node(
            piet_cx,
            resize_tr_node_state,
            resize_tr_node_bounds,
            total_zoom,
        );
        indicators::draw_rectangular_node(
            piet_cx,
            resize_bl_node_state,
            resize_bl_node_bounds,
            total_zoom,
        );
        indicators::draw_rectangular_node(
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

    fn select_all(
        &mut self,
        modifier_keys: Vec<ModifierKey>,
        engine_view: &mut EngineViewMut,
        widget_flags: &mut WidgetFlags,
    ) {
        if modifier_keys.contains(&ModifierKey::KeyboardCtrl) {
            // Select all keys
            let all_strokes = engine_view.store.stroke_keys_as_rendered();

            if let Some(new_bounds) = engine_view.store.bounds_for_strokes(&all_strokes) {
                engine_view.store.set_selected_keys(&all_strokes, true);
                *widget_flags |= engine_view
                    .document
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                self.state = SelectorState::ModifySelection {
                    modify_state: ModifyState::default(),
                    selection: all_strokes,
                    selection_bounds: new_bounds,
                };

                widget_flags.store_modified = true;
                widget_flags.deselect_color_setters = true;
            }
        }
    }
}

fn cancel_selection(selection: &[StrokeKey], engine_view: &mut EngineViewMut) -> WidgetFlags {
    let mut widget_flags = WidgetFlags::default();
    engine_view.store.set_selected_keys(selection, false);
    engine_view.store.update_geometry_for_strokes(selection);
    engine_view.store.regenerate_rendering_in_viewport_threaded(
        engine_view.tasks_tx.clone(),
        false,
        engine_view.camera.viewport(),
        engine_view.camera.image_scale(),
    );

    widget_flags |= engine_view.store.record(Instant::now());
    widget_flags.store_modified = true;
    widget_flags.resize = true;
    widget_flags
}
