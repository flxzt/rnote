//! Helpers for drawing edit nodes, guides, etc.

use p2d::bounding_volume::{BoundingSphere, AABB};
use piet::RenderContext;

use crate::color;
use crate::helpers::{AABBHelpers, Vector2Helpers};

/// The node state
#[derive(Debug, Clone, Copy)]
pub enum NodeState {
    /// Up
    Up,
    /// Proximity
    Proximity,
    /// Down
    Down,
}

/// Draw a position indicator
pub fn draw_pos_indicator(
    cx: &mut impl RenderContext,
    node_state: NodeState,
    pos: na::Vector2<f64>,
    zoom: f64,
) {
    const RADIUS: f64 = 3.0;
    const FILL_COLOR: piet::Color = color::GNOME_BRIGHTS[2].with_a8(0x80);
    const OUTLINE_WIDTH: f64 = 1.8;
    const OUTLINE_COLOR: piet::Color = color::GNOME_REDS[4];

    let indicator_circle = kurbo::Circle::new(pos.to_kurbo_point(), RADIUS / zoom);
    match node_state {
        NodeState::Up => {}
        NodeState::Proximity => {}
        NodeState::Down => {
            cx.fill(indicator_circle, &FILL_COLOR);
        }
    }
    cx.stroke(indicator_circle, &OUTLINE_COLOR, OUTLINE_WIDTH / zoom);
}

/// Draw a vec indicator
pub fn draw_vec_indicator(
    cx: &mut impl RenderContext,
    node_state: NodeState,
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    zoom: f64,
) {
    const LINE_WIDTH: f64 = 1.8;

    let line = kurbo::Line::new(start.to_kurbo_point(), end.to_kurbo_point());
    let line_color = match node_state {
        NodeState::Up => color::GNOME_BRIGHTS[3].with_a8(0x60),
        NodeState::Proximity => color::GNOME_BRIGHTS[3].with_a8(0xa0),
        NodeState::Down => color::GNOME_DARKS[0].with_a8(0xa0),
    };

    cx.stroke(line, &line_color, LINE_WIDTH / zoom);
}

/// Draw a rectangular node
pub fn draw_rectangular_node(
    cx: &mut impl RenderContext,
    node_state: NodeState,
    bounds: AABB,
    zoom: f64,
) {
    const OUTLINE_WIDTH: f64 = 1.8;
    const RECT_RADIUS: f64 = 2.0;
    const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[4];
    const FILL_STATE_DOWN: piet::Color = color::GNOME_BLUES[0].with_a8(0x80);

    let node_rect = kurbo::RoundedRect::from_rect(bounds.to_kurbo_rect(), RECT_RADIUS / zoom);

    match node_state {
        NodeState::Up => {}
        NodeState::Proximity => {}
        NodeState::Down => {
            cx.fill(node_rect, &piet::PaintBrush::Color(FILL_STATE_DOWN));
        }
    }

    cx.stroke(
        node_rect,
        &piet::PaintBrush::Color(OUTLINE_COLOR),
        OUTLINE_WIDTH / zoom,
    );
}

/// Draw a circular node
pub fn draw_circular_node(
    cx: &mut impl RenderContext,
    node_state: NodeState,
    bounding_sphere: BoundingSphere,
    zoom: f64,
) {
    const OUTLINE_WIDTH: f64 = 1.8;
    const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[4];
    const FILL_STATE_DOWN: piet::Color = color::GNOME_BLUES[0].with_a8(0x80);

    let node_circle = kurbo::Circle::new(
        bounding_sphere.center.coords.to_kurbo_point(),
        bounding_sphere.radius,
    );
    cx.stroke(
        node_circle,
        &piet::PaintBrush::Color(OUTLINE_COLOR),
        OUTLINE_WIDTH / zoom,
    );

    match node_state {
        NodeState::Up => {}
        NodeState::Proximity => {}
        NodeState::Down => {
            cx.fill(node_circle, &piet::PaintBrush::Color(FILL_STATE_DOWN));
        }
    }
}
