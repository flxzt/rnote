// Imports
use crate::ext::{AabbExt, Vector2Ext};
use crate::penevent::PenState;
use crate::{Color, color};
use kurbo::{BezPath, Shape, Stroke};
use p2d::bounding_volume::{Aabb, BoundingSphere, BoundingVolume};
use p2d::math::Vector2;
use vello_cpu::RenderContext;

// Pos indicator

/// Position indicator radius.
pub const POS_INDICATOR_RADIUS: f64 = 3.0;
/// Position indicator outline width.
pub const POS_INDICATOR_OUTLINE_WIDTH: f64 = 1.5;

/// Position indicator shape.
pub fn pos_indicator_shape(_node_state: PenState, pos: Vector2, zoom: f64) -> BezPath {
    kurbo::Circle::new(
        pos.to_kurbo_point(),
        (POS_INDICATOR_RADIUS - POS_INDICATOR_OUTLINE_WIDTH * 0.5) / zoom,
    )
    .to_path(0.1)
}

/// Draw a position indicator.
pub fn draw_pos_indicator(cx: &mut RenderContext, node_state: PenState, pos: Vector2, zoom: f64) {
    const FILL_COLOR: Color = color::GNOME_REDS[3].with_a8(176);
    const OUTLINE_COLOR: Color = color::GNOME_REDS[4];

    let pos_indicator = pos_indicator_shape(node_state, pos, zoom);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.set_paint(FILL_COLOR);
            cx.fill_path(&pos_indicator);
        }
    }
    cx.set_stroke(Stroke::new(POS_INDICATOR_OUTLINE_WIDTH / zoom));
    cx.set_paint(OUTLINE_COLOR);
    cx.stroke_path(&pos_indicator);
}

// Vec indicator

/// Vector indicator line width.
pub const VEC_INDICATOR_LINE_WIDTH: f64 = 1.5;

/// Vector indicator shape.
pub fn vec_indicator_shape(
    _node_state: PenState,
    start: Vector2,
    end: Vector2,
    _zoom: f64,
) -> BezPath {
    kurbo::Line::new(start.to_kurbo_point(), end.to_kurbo_point()).to_path(0.1)
}

/// Draw a vector indicator.
pub fn draw_vec_indicator(
    cx: &mut RenderContext,
    node_state: PenState,
    start: Vector2,
    end: Vector2,
    zoom: f64,
) {
    let vec_indicator = vec_indicator_shape(node_state, start, end, zoom);

    let line_color = match node_state {
        PenState::Up => color::GNOME_DARKS[0].with_a(0.5),
        PenState::Proximity => color::GNOME_BRIGHTS[0].with_a(0.627),
        PenState::Down => color::GNOME_DARKS[1].with_a(0.627),
    };

    cx.set_stroke(Stroke::new(VEC_INDICATOR_LINE_WIDTH / zoom));
    cx.set_paint(line_color);
    cx.stroke_path(&vec_indicator);
}

// Finish indicator

/// Finish indicator radius.
pub const FINISH_INDICATOR_RADIUS: f64 = 5.0;
/// Finish indicator outline width.
pub const FINISH_INDICATOR_OUTLINE_WIDTH: f64 = 5.0;

/// A finish indicator shape.
pub fn finish_indicator_shape(_node_state: PenState, pos: Vector2, zoom: f64) -> BezPath {
    kurbo::Circle::new(
        pos.to_kurbo_point(),
        (FINISH_INDICATOR_RADIUS - FINISH_INDICATOR_OUTLINE_WIDTH * 0.5) / zoom,
    )
    .to_path(0.1)
}

/// Draw a finish indicator.
pub fn draw_finish_indicator(
    cx: &mut RenderContext,
    node_state: PenState,
    pos: Vector2,
    zoom: f64,
) {
    const FILL_COLOR: Color = color::GNOME_GREENS[3].with_a8(176);
    const OUTLINE_COLOR: Color = color::GNOME_GREENS[4];

    let finish_indicator = finish_indicator_shape(node_state, pos, zoom);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.set_paint(FILL_COLOR);
            cx.fill_path(&finish_indicator);
        }
    }
    cx.set_stroke(Stroke::new(POS_INDICATOR_OUTLINE_WIDTH / zoom));
    cx.set_paint(OUTLINE_COLOR);
    cx.stroke_path(&finish_indicator);
}

// Rectangular node

/// Rectangular node outline width.
pub const RECTANGULAR_NODE_OUTLINE_WIDTH: f64 = 1.5;

/// Rectangular node shape.
pub fn rectangular_node_shape(_node_state: PenState, bounds: Aabb, zoom: f64) -> BezPath {
    const CORNER_RADIUS: f64 = 1.0;

    kurbo::RoundedRect::from_rect(
        bounds
            .tightened(RECTANGULAR_NODE_OUTLINE_WIDTH * 0.5 / zoom)
            .to_kurbo_rect(),
        CORNER_RADIUS / zoom,
    )
    .to_path(0.1)
}

/// Draw a rectangular node.
pub fn draw_rectangular_node(
    cx: &mut RenderContext,
    node_state: PenState,
    bounds: Aabb,
    zoom: f64,
    background_color: Color,
) {
    const OUTLINE_COLOR: Color = color::GNOME_BLUES[4];
    const FILL_STATE_PROXIMITY: Color = color::GNOME_BLUES[0].with_a8(77);
    const FILL_STATE_DOWN: Color = color::GNOME_BLUES[2].with_a8(128);

    let rectangular_node = rectangular_node_shape(node_state, bounds, zoom);
    cx.set_paint(background_color);
    cx.fill_path(&rectangular_node);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {
            cx.set_paint(FILL_STATE_PROXIMITY);
            cx.fill_path(&rectangular_node);
        }
        PenState::Down => {
            cx.set_paint(FILL_STATE_DOWN);
            cx.fill_path(&rectangular_node);
        }
    }

    cx.set_stroke(Stroke::new(RECTANGULAR_NODE_OUTLINE_WIDTH / zoom));
    cx.set_paint(OUTLINE_COLOR);
    cx.stroke_path(&rectangular_node);
}

// Circular Node

/// Circular node outline width.
pub const CIRCULAR_NODE_OUTLINE_WIDTH: f64 = 1.5;

/// circular node shape.
pub fn circular_node_shape(
    _node_state: PenState,
    mut bounding_sphere: BoundingSphere,
    zoom: f64,
) -> BezPath {
    bounding_sphere.tighten(CIRCULAR_NODE_OUTLINE_WIDTH * 0.5 / zoom);

    kurbo::Circle::new(
        bounding_sphere.center.to_kurbo_point(),
        bounding_sphere.radius,
    )
    .to_path(0.1)
}

/// Draw a circular node.
pub fn draw_circular_node(
    cx: &mut RenderContext,
    node_state: PenState,
    bounding_sphere: BoundingSphere,
    zoom: f64,
    background_color: Color,
) {
    const OUTLINE_COLOR: Color = color::GNOME_BLUES[4];
    const FILL_STATE_PROXIMITY: Color = color::GNOME_BLUES[0].with_a8(77);
    const FILL_STATE_DOWN: Color = color::GNOME_BLUES[2].with_a8(128);

    let circular_node = circular_node_shape(node_state, bounding_sphere, zoom);

    cx.set_paint(background_color);
    cx.fill_path(&circular_node);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {
            cx.set_paint(FILL_STATE_PROXIMITY);
            cx.fill_path(&circular_node);
        }
        PenState::Down => {
            cx.set_paint(FILL_STATE_DOWN);
            cx.fill_path(&circular_node);
        }
    }

    cx.set_stroke(Stroke::new(CIRCULAR_NODE_OUTLINE_WIDTH / zoom));
    cx.set_paint(OUTLINE_COLOR);
    cx.stroke_path(&circular_node);
}

// Triangular down node

/// Triangular node outline width.
pub const TRIANGULAR_DOWN_NODE_OUTLINE_WIDTH: f64 = 1.5;

/// Triangular node shape.
pub fn triangular_down_node_shape(
    _node_state: PenState,
    center: Vector2,
    size: Vector2,
    zoom: f64,
) -> kurbo::BezPath {
    let outline_half_width = TRIANGULAR_DOWN_NODE_OUTLINE_WIDTH * 0.5 / zoom;
    kurbo::BezPath::from_iter([
        kurbo::PathEl::MoveTo(kurbo::Point::new(
            center[0] - size[0] * 0.5 + outline_half_width,
            center[1] - size[1] * 0.5 + outline_half_width,
        )),
        kurbo::PathEl::LineTo(kurbo::Point::new(
            center[0] + size[0] * 0.5 - outline_half_width,
            center[1] - size[1] * 0.5 + outline_half_width,
        )),
        kurbo::PathEl::LineTo(kurbo::Point::new(
            center[0],
            center[1] + size[1] * 0.5 - outline_half_width,
        )),
        kurbo::PathEl::ClosePath,
    ])
}

/// Draw a triangular node.
pub fn draw_triangular_node(
    cx: &mut RenderContext,
    node_state: PenState,
    center: Vector2,
    size: Vector2,
    zoom: f64,
    background_color: Color,
) {
    const OUTLINE_COLOR: Color = color::GNOME_ORANGES[4];
    const FILL_STATE_PROXIMITY: Color = color::GNOME_ORANGES[0].with_a8(77);
    const FILL_STATE_DOWN: Color = color::GNOME_ORANGES[3].with_a8(128);

    let triangular_down_node = triangular_down_node_shape(node_state, center, size, zoom);

    cx.set_paint(background_color);
    cx.fill_path(&triangular_down_node);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {
            cx.set_paint(FILL_STATE_PROXIMITY);
            cx.fill_path(&triangular_down_node);
        }
        PenState::Down => {
            cx.set_paint(FILL_STATE_DOWN);
            cx.fill_path(&triangular_down_node);
        }
    }

    cx.set_stroke(Stroke::new(CIRCULAR_NODE_OUTLINE_WIDTH / zoom));
    cx.set_paint(OUTLINE_COLOR);
    cx.stroke_path(&triangular_down_node);
}
