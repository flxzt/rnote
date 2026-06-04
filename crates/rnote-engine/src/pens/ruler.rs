// Imports
use crate::pens::pensconfig::rulerconfig::RulerConfig;
use p2d::bounding_volume::Aabb;
use p2d::math::Vector2;
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
use rnote_compose::color;
use rnote_compose::ext::Vector2Ext;

// Body, edge, tick and text colors are picked at runtime via `RulerConfig`
// helpers so they can adapt to dark / light backgrounds. The red rotation
// indicator stays the same in both modes — it's a high-contrast accent.
const INDICATOR_COLOR: piet::Color = color::GNOME_REDS[2];

/// Font size of the angle text, in surface pixels (constant on-screen).
const ANGLE_TEXT_SIZE_PX: f64 = 14.0;
/// Outer radius of the angle dial, in surface pixels (constant on-screen).
const DIAL_OUTER_RADIUS_PX: f64 = 32.0;
/// Length of dial minor ticks, in surface pixels.
const DIAL_MINOR_TICK_LEN_PX: f64 = 4.0;
/// Length of dial major ticks, in surface pixels.
const DIAL_MAJOR_TICK_LEN_PX: f64 = 7.0;
/// Number of degrees between dial minor ticks.
const DIAL_MINOR_TICK_STEP_DEG: f64 = 6.0;
/// Every Nth dial tick is a major tick.
const DIAL_MAJOR_TICK_EVERY: u32 = 5; // major every 30°
/// Size of each red direction-indicator triangle, in surface pixels.
const INDICATOR_SIZE_PX: f64 = 6.0;
/// Length of medium tick marks, in surface pixels. Drawn every 5th tick.
const EDGE_TICK_MEDIUM_LEN_PX: f64 = 10.0;

/// Compute the document-space `[min_t, max_t]` parameter range along the ruler
/// direction so that the segment `anchor_doc + t * direction` covers the full
/// viewport (with margin). Returns `None` for a degenerate viewport.
fn viewport_t_range(
    anchor_doc: Vector2,
    direction: Vector2,
    half_w_doc: f64,
    viewport: Aabb,
) -> Option<(f64, f64)> {
    let extent_along = viewport.extents().length() + half_w_doc * 2.0;
    if extent_along < 1e-6 {
        return None;
    }
    let corners = [
        Vector2::new(viewport.mins.x, viewport.mins.y),
        Vector2::new(viewport.maxs.x, viewport.mins.y),
        Vector2::new(viewport.maxs.x, viewport.maxs.y),
        Vector2::new(viewport.mins.x, viewport.maxs.y),
    ];
    let mut min_t = f64::INFINITY;
    let mut max_t = f64::NEG_INFINITY;
    for c in &corners {
        let t = (*c - anchor_doc).dot(direction);
        min_t = min_t.min(t);
        max_t = max_t.max(t);
    }
    let pad = half_w_doc * 4.0;
    Some((min_t - pad, max_t + pad))
}

/// Bounds of the ruler band on the document, when visible, including a small margin.
pub fn ruler_bounds_on_doc(
    ruler: &RulerConfig,
    viewport: Aabb,
    total_zoom: f64,
) -> Option<Aabb> {
    if !ruler.visible {
        return None;
    }
    let half_w = ruler.body_half_width_doc(total_zoom);
    let tick = RulerConfig::TICK_MAJOR_LEN_PX / total_zoom;
    let pad = half_w + tick + 4.0 / total_zoom;
    Some(Aabb::new(
        Vector2::new(viewport.mins.x - pad, viewport.mins.y - pad),
        Vector2::new(viewport.maxs.x + pad, viewport.maxs.y + pad),
    ))
}

/// Draw the ruler band across the viewport with tick marks on its long edges
/// and an angle dial. The ruler's position is stored in scroller coordinates
/// and converted to document coordinates here using `camera_offset` and
/// `total_zoom`.
pub fn draw_ruler_on_doc(
    cx: &mut piet_cairo::CairoRenderContext,
    ruler: &RulerConfig,
    viewport: Aabb,
    camera_offset: Vector2,
    total_zoom: f64,
    background_color: &rnote_compose::Color,
) -> anyhow::Result<()> {
    if !ruler.visible {
        return Ok(());
    }
    let dark_mode = RulerConfig::dark_mode_for_background(background_color);
    let anchor_doc = ruler.anchor_doc(camera_offset, total_zoom);
    let dir = ruler.direction();
    let normal = ruler.normal();
    let half_w = ruler.body_half_width_doc(total_zoom);
    let Some((min_t, max_t)) = viewport_t_range(anchor_doc, dir, half_w, viewport) else {
        return Ok(());
    };

    let p_start = anchor_doc + min_t * dir;
    let p_end = anchor_doc + max_t * dir;
    let edge_a_s = p_start + half_w * normal;
    let edge_a_e = p_end + half_w * normal;
    let edge_b_s = p_start - half_w * normal;
    let edge_b_e = p_end - half_w * normal;

    cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // Body fill + outline. Fill opacity is user-configurable.
    let mut body = kurbo::BezPath::new();
    body.move_to(edge_a_s.to_kurbo_point());
    body.line_to(edge_a_e.to_kurbo_point());
    body.line_to(edge_b_e.to_kurbo_point());
    body.line_to(edge_b_s.to_kurbo_point());
    body.close_path();
    cx.fill(body.clone(), &ruler.body_fill_color(dark_mode));
    cx.stroke(body, &RulerConfig::body_stroke_color(dark_mode), 1.0 / total_zoom);

    // Tick marks on the long edges, three-tier (minor / medium / major).
    // Spacing is fixed in surface pixels — the ruler's scale is independent
    // of the document grid.
    let spacing = ruler.tick_spacing / total_zoom;
    let tick_major = RulerConfig::TICK_MAJOR_LEN_PX / total_zoom;
    let tick_medium = EDGE_TICK_MEDIUM_LEN_PX / total_zoom;
    let tick_minor = RulerConfig::TICK_MINOR_LEN_PX / total_zoom;
    let tick_w = 1.0 / total_zoom;
    let i_min = (min_t / spacing).ceil() as i64;
    let i_max = (max_t / spacing).floor() as i64;
    const MAX_TICKS: i64 = 4096;
    if i_max - i_min > MAX_TICKS {
        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        return Ok(());
    }
    for i in i_min..=i_max {
        let t = i as f64 * spacing;
        let p = anchor_doc + t * dir;
        let len = if i.rem_euclid(10) == 0 {
            tick_major
        } else if i.rem_euclid(5) == 0 {
            tick_medium
        } else {
            tick_minor
        };
        let a_outer = p + half_w * normal;
        let a_inner = p + (half_w - len) * normal;
        cx.stroke(
            kurbo::Line::new(a_outer.to_kurbo_point(), a_inner.to_kurbo_point()),
            &RulerConfig::tick_color(dark_mode),
            tick_w,
        );
        let b_outer = p - half_w * normal;
        let b_inner = p - (half_w - len) * normal;
        cx.stroke(
            kurbo::Line::new(b_outer.to_kurbo_point(), b_inner.to_kurbo_point()),
            &RulerConfig::tick_color(dark_mode),
            tick_w,
        );
    }

    if ruler.show_dial {
        let dial_pos_doc = ruler.dial_pos_doc(camera_offset, total_zoom);
        draw_angle_dial(cx, ruler, dial_pos_doc, total_zoom, dark_mode)?;
    }

    cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

fn draw_angle_dial(
    cx: &mut piet_cairo::CairoRenderContext,
    ruler: &RulerConfig,
    dial_pos_doc: Vector2,
    total_zoom: f64,
    dark_mode: bool,
) -> anyhow::Result<()> {
    let outer_r = DIAL_OUTER_RADIUS_PX / total_zoom;
    let minor_len = DIAL_MINOR_TICK_LEN_PX / total_zoom;
    let major_len = DIAL_MAJOR_TICK_LEN_PX / total_zoom;
    let tick_w = 1.0 / total_zoom;
    let indicator_size = INDICATOR_SIZE_PX / total_zoom;

    // Dial tick marks: drawn in the world frame (do NOT rotate with the
    // ruler), so they serve as a fixed reference for reading rotation.
    cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    cx.transform(kurbo::Affine::translate(dial_pos_doc.to_kurbo_vec()));

    let n_ticks = (360.0 / DIAL_MINOR_TICK_STEP_DEG).round() as i32;
    for i in 0..n_ticks {
        let theta = (i as f64) * DIAL_MINOR_TICK_STEP_DEG.to_radians();
        let (sin, cos) = (theta.sin(), theta.cos());
        let is_major = (i as u32).rem_euclid(DIAL_MAJOR_TICK_EVERY) == 0;
        let len = if is_major { major_len } else { minor_len };
        let p_out = kurbo::Point::new(cos * outer_r, sin * outer_r);
        let p_in = kurbo::Point::new(cos * (outer_r - len), sin * (outer_r - len));
        cx.stroke(kurbo::Line::new(p_out, p_in), &RulerConfig::tick_color(dark_mode), tick_w);
    }

    cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // Red direction-indicators: rotate WITH the ruler, so they point at the
    // current angle on the fixed dial.
    cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    cx.transform(kurbo::Affine::translate(dial_pos_doc.to_kurbo_vec()));
    cx.transform(kurbo::Affine::rotate(ruler.angle));
    for sign in [1.0_f64, -1.0_f64] {
        let tip = kurbo::Point::new(sign * (outer_r - major_len - indicator_size * 0.2), 0.0);
        let base_back = sign * (outer_r - major_len - indicator_size * 1.2);
        let p_left = kurbo::Point::new(base_back, -indicator_size * 0.55);
        let p_right = kurbo::Point::new(base_back, indicator_size * 0.55);
        let mut tri = kurbo::BezPath::new();
        tri.move_to(tip);
        tri.line_to(p_left);
        tri.line_to(p_right);
        tri.close_path();
        cx.fill(tri, &INDICATOR_COLOR);
    }
    cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // Angle text: world-horizontal, centered at the dial position. Ruler is
    // symmetric (θ and θ+π describe the same line), so display in [-90°, 90°).
    //
    // The layout is built once at a FIXED pixel font size so its measured
    // size is zoom-independent (otherwise sub-pixel hinting in the layout
    // wiggles the centered position as the zoom changes). We then undo the
    // camera's zoom inside this transform so the local frame is in surface
    // pixels — the text ends up at a stable, constant on-screen position.
    let normalized_deg = RulerConfig::normalize_angle(ruler.angle).to_degrees();
    // Round first so `-0.3°` doesn't surface as `-0°`; then canonicalize the
    // sign so `format!` doesn't print the negative zero.
    let rounded = normalized_deg.round();
    // Canonicalize: the displayed range is (-90°, 90°], so a rounded -90° is
    // the same orientation as +90° — show +90°. Also avoid printing "-0°".
    let display_value = if rounded == -90.0 {
        90.0
    } else if rounded == 0.0 {
        0.0
    } else {
        rounded
    };
    let text = format!("{display_value:.0}°");
    let layout = cx
        .text()
        .new_text_layout(text)
        .font(piet::FontFamily::SYSTEM_UI, ANGLE_TEXT_SIZE_PX)
        .text_color(RulerConfig::angle_text_color(dark_mode))
        .build()
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let text_size = layout.size();
    cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    cx.transform(kurbo::Affine::translate(dial_pos_doc.to_kurbo_vec()));
    cx.transform(kurbo::Affine::scale(1.0 / total_zoom));
    cx.draw_text(
        &layout,
        kurbo::Point::new(-text_size.width * 0.5, -text_size.height * 0.5),
    );
    cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
