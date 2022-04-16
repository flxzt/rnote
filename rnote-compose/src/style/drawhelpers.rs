//! Helpers for drawing edit nodes, guides, etc.

use p2d::bounding_volume::{BoundingSphere, AABB};
use piet::RenderContext;

use crate::color;
use crate::helpers::{AABBHelpers, Vector2Helpers};
use crate::penhelpers::PenState;

/// the radius
pub const POS_INDICATOR_RADIUS: f64 = 3.0;
/// the outline width
pub const POS_INDICATOR_OUTLINE_WIDTH: f64 = 1.6;

/// Draw a position indicator
pub fn draw_pos_indicator(
    cx: &mut impl RenderContext,
    node_state: PenState,
    pos: na::Vector2<f64>,
    zoom: f64,
) {
    const FILL_COLOR: piet::Color = color::GNOME_REDS[3].with_a8(0xb0);
    const OUTLINE_COLOR: piet::Color = color::GNOME_REDS[4];

    let indicator_circle = kurbo::Circle::new(pos.to_kurbo_point(), POS_INDICATOR_RADIUS / zoom);
    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.fill(indicator_circle, &FILL_COLOR);
        }
    }
    cx.stroke(
        indicator_circle,
        &OUTLINE_COLOR,
        POS_INDICATOR_OUTLINE_WIDTH / zoom,
    );
}

/// the line width
pub const VEC_INDICATOR_LINE_WIDTH: f64 = 1.8;

/// Draw a vec indicator
pub fn draw_vec_indicator(
    cx: &mut impl RenderContext,
    node_state: PenState,
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    zoom: f64,
) {
    let line = kurbo::Line::new(start.to_kurbo_point(), end.to_kurbo_point());
    let line_color = match node_state {
        PenState::Up => color::GNOME_BRIGHTS[3].with_a8(0x60),
        PenState::Proximity => color::GNOME_BRIGHTS[3].with_a8(0xa0),
        PenState::Down => color::GNOME_DARKS[0].with_a8(0xa0),
    };

    cx.stroke(line, &line_color, VEC_INDICATOR_LINE_WIDTH / zoom);
}

/// the outline width
pub const RECTANGULAR_NODE_OUTLINE_WIDTH: f64 = 1.8;

/// Draw a rectangular node
pub fn draw_rectangular_node(
    cx: &mut impl RenderContext,
    node_state: PenState,
    bounds: AABB,
    zoom: f64,
) {
    const RECT_RADIUS: f64 = 2.0;
    const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[4];
    const FILL_COLOR_STATE_DOWN: piet::Color = color::GNOME_BLUES[0].with_a8(0x80);

    let node_rect = kurbo::RoundedRect::from_rect(bounds.to_kurbo_rect(), RECT_RADIUS / zoom);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.fill(node_rect, &FILL_COLOR_STATE_DOWN);
        }
    }

    cx.stroke(
        node_rect,
        &OUTLINE_COLOR,
        RECTANGULAR_NODE_OUTLINE_WIDTH / zoom,
    );
}

/// the outline width
pub const CIRCULAR_NODE_OUTLINE_WIDTH: f64 = 1.8;

/// Draw a circular node
pub fn draw_circular_node(
    cx: &mut impl RenderContext,
    node_state: PenState,
    bounding_sphere: BoundingSphere,
    zoom: f64,
) {
    const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[4];
    const FILL_STATE_DOWN: piet::Color = color::GNOME_BLUES[0].with_a8(0x80);

    let node_circle = kurbo::Circle::new(
        bounding_sphere.center.coords.to_kurbo_point(),
        bounding_sphere.radius,
    );
    cx.stroke(
        node_circle,
        &OUTLINE_COLOR,
        CIRCULAR_NODE_OUTLINE_WIDTH / zoom,
    );

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.fill(node_circle, &FILL_STATE_DOWN);
        }
    }
}
