// Imports
use crate::color;
use crate::ext::{AabbExt, Vector2Ext};
use crate::penevent::PenState;
use p2d::bounding_volume::{Aabb, BoundingSphere, BoundingVolume};
use piet::RenderContext;

// Pos indicator

/// Position indicator radius.
pub const POS_INDICATOR_RADIUS: f64 = 3.0;
/// Position indicator outline width.
pub const POS_INDICATOR_OUTLINE_WIDTH: f64 = 1.5;

/// Position indicator shape.
pub fn pos_indicator_shape(
    _node_state: PenState,
    pos: na::Vector2<f64>,
    zoom: f64,
) -> kurbo::Circle {
    kurbo::Circle::new(
        pos.to_kurbo_point(),
        (POS_INDICATOR_RADIUS - POS_INDICATOR_OUTLINE_WIDTH * 0.5) / zoom,
    )
}

/// Draw a position indicator.
pub fn draw_pos_indicator(
    cx: &mut impl RenderContext,
    node_state: PenState,
    pos: na::Vector2<f64>,
    zoom: f64,
) {
    const FILL_COLOR: piet::Color = color::GNOME_REDS[3].with_a8(176);
    const OUTLINE_COLOR: piet::Color = color::GNOME_REDS[4];

    let pos_indicator = pos_indicator_shape(node_state, pos, zoom);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.fill(pos_indicator, &FILL_COLOR);
        }
    }
    cx.stroke(
        pos_indicator,
        &OUTLINE_COLOR,
        POS_INDICATOR_OUTLINE_WIDTH / zoom,
    );
}

// Vec indicator

/// Vector indicator line width.
pub const VEC_INDICATOR_LINE_WIDTH: f64 = 1.5;

/// Vector indicator shape.
pub fn vec_indicator_shape(
    _node_state: PenState,
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    _zoom: f64,
) -> kurbo::Line {
    kurbo::Line::new(start.to_kurbo_point(), end.to_kurbo_point())
}

/// Draw a vector indicator.
pub fn draw_vec_indicator(
    cx: &mut impl RenderContext,
    node_state: PenState,
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    zoom: f64,
) {
    let vec_indicator = vec_indicator_shape(node_state, start, end, zoom);

    let line_color = match node_state {
        PenState::Up => color::GNOME_DARKS[0].with_alpha(0.5),
        PenState::Proximity => color::GNOME_BRIGHTS[0].with_alpha(0.627),
        PenState::Down => color::GNOME_DARKS[1].with_alpha(0.627),
    };

    cx.stroke(vec_indicator, &line_color, VEC_INDICATOR_LINE_WIDTH / zoom);
}

// Finish indicator

/// Finish indicator radius.
pub const FINISH_INDICATOR_RADIUS: f64 = 5.0;
/// Finish indicator outline width.
pub const FINISH_INDICATOR_OUTLINE_WIDTH: f64 = 5.0;

/// A finish indicator shape.
pub fn finish_indicator_shape(
    _node_state: PenState,
    pos: na::Vector2<f64>,
    zoom: f64,
) -> kurbo::Circle {
    kurbo::Circle::new(
        pos.to_kurbo_point(),
        (FINISH_INDICATOR_RADIUS - FINISH_INDICATOR_OUTLINE_WIDTH * 0.5) / zoom,
    )
}

/// Draw a finish indicator.
pub fn draw_finish_indicator(
    cx: &mut impl RenderContext,
    node_state: PenState,
    pos: na::Vector2<f64>,
    zoom: f64,
) {
    const FILL_COLOR: piet::Color = color::GNOME_GREENS[3].with_a8(176);
    const OUTLINE_COLOR: piet::Color = color::GNOME_GREENS[4];

    let finish_indicator = finish_indicator_shape(node_state, pos, zoom);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.fill(finish_indicator, &FILL_COLOR);
        }
    }
    cx.stroke(
        finish_indicator,
        &OUTLINE_COLOR,
        POS_INDICATOR_OUTLINE_WIDTH / zoom,
    );
}

// Rectangular node

/// Rectangular node outline width.
pub const RECTANGULAR_NODE_OUTLINE_WIDTH: f64 = 1.5;

/// Rectangular node shape.
pub fn rectangular_node_shape(
    _node_state: PenState,
    bounds: Aabb,
    zoom: f64,
) -> kurbo::RoundedRect {
    const CORNER_RADIUS: f64 = 1.0;

    kurbo::RoundedRect::from_rect(
        bounds
            .tightened(RECTANGULAR_NODE_OUTLINE_WIDTH * 0.5 / zoom)
            .to_kurbo_rect(),
        CORNER_RADIUS / zoom,
    )
}

/// Draw a rectangular node.
pub fn draw_rectangular_node(
    cx: &mut impl RenderContext,
    node_state: PenState,
    bounds: Aabb,
    zoom: f64,
) {
    const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[4];
    const FILL_STATE_PROXIMITY: piet::Color = color::GNOME_BLUES[0].with_a8(77);
    const FILL_STATE_DOWN: piet::Color = color::GNOME_BLUES[2].with_a8(128);

    let rectangular_node = rectangular_node_shape(node_state, bounds, zoom);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {
            cx.fill(rectangular_node, &FILL_STATE_PROXIMITY);
        }
        PenState::Down => {
            cx.fill(rectangular_node, &FILL_STATE_DOWN);
        }
    }

    cx.stroke(
        rectangular_node,
        &OUTLINE_COLOR,
        RECTANGULAR_NODE_OUTLINE_WIDTH / zoom,
    );
}

// Circular Node

/// Circular node outline width.
pub const CIRCULAR_NODE_OUTLINE_WIDTH: f64 = 1.5;

/// circular node shape.
pub fn circular_node_shape(
    _node_state: PenState,
    mut bounding_sphere: BoundingSphere,
    zoom: f64,
) -> kurbo::Circle {
    bounding_sphere.tighten(CIRCULAR_NODE_OUTLINE_WIDTH * 0.5 / zoom);

    kurbo::Circle::new(
        bounding_sphere.center.coords.to_kurbo_point(),
        bounding_sphere.radius,
    )
}

/// Draw a circular node.
pub fn draw_circular_node(
    cx: &mut impl RenderContext,
    node_state: PenState,
    bounding_sphere: BoundingSphere,
    zoom: f64,
) {
    const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[4];
    const FILL_STATE_PROXIMITY: piet::Color = color::GNOME_BLUES[0].with_a8(77);
    const FILL_STATE_DOWN: piet::Color = color::GNOME_BLUES[2].with_a8(128);

    let circular_node = circular_node_shape(node_state, bounding_sphere, zoom);

    cx.stroke(
        circular_node,
        &OUTLINE_COLOR,
        CIRCULAR_NODE_OUTLINE_WIDTH / zoom,
    );

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {
            cx.fill(circular_node, &FILL_STATE_PROXIMITY);
        }
        PenState::Down => {
            cx.fill(circular_node, &FILL_STATE_DOWN);
        }
    }
}

// Triangular down node

/// Triangular node outline width.
pub const TRIANGULAR_DOWN_NODE_OUTLINE_WIDTH: f64 = 1.5;

/// Triangular node shape.
pub fn triangular_down_node_shape(
    _node_state: PenState,
    center: na::Vector2<f64>,
    size: na::Vector2<f64>,
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
    cx: &mut impl RenderContext,
    node_state: PenState,
    center: na::Vector2<f64>,
    size: na::Vector2<f64>,
    zoom: f64,
) {
    const OUTLINE_COLOR: piet::Color = color::GNOME_ORANGES[4];
    const FILL_STATE_PROXIMITY: piet::Color = color::GNOME_ORANGES[0].with_a8(77);
    const FILL_STATE_DOWN: piet::Color = color::GNOME_ORANGES[3].with_a8(128);

    let triangular_down_node = triangular_down_node_shape(node_state, center, size, zoom);

    cx.stroke(
        triangular_down_node.clone(),
        &OUTLINE_COLOR,
        CIRCULAR_NODE_OUTLINE_WIDTH / zoom,
    );

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {
            cx.fill(triangular_down_node, &FILL_STATE_PROXIMITY);
        }
        PenState::Down => {
            cx.fill(triangular_down_node, &FILL_STATE_DOWN);
        }
    }
}
