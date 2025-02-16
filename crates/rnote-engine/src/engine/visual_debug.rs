// Imports
use rnote_compose::Color;

pub const COLOR_POS: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const COLOR_STROKE_HITBOX: Color = Color {
    r: 0.0,
    g: 0.8,
    b: 0.2,
    a: 0.5,
};
pub const COLOR_STROKE_BOUNDS: Color = Color {
    r: 0.0,
    g: 0.8,
    b: 0.8,
    a: 1.0,
};
pub const COLOR_IMAGE_BOUNDS: Color = Color {
    r: 0.0,
    g: 0.5,
    b: 1.0,
    a: 1.0,
};
pub const COLOR_STROKE_RENDERING_DIRTY: Color = Color {
    r: 0.9,
    g: 0.0,
    b: 0.8,
    a: 0.10,
};
pub const COLOR_STROKE_RENDERING_BUSY: Color = Color {
    r: 0.0,
    g: 0.8,
    b: 1.0,
    a: 0.10,
};
pub const COLOR_SELECTOR_BOUNDS: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.8,
    a: 1.0,
};
pub const COLOR_DOC_BOUNDS: Color = Color {
    r: 0.8,
    g: 0.0,
    b: 0.8,
    a: 1.0,
};

#[cfg(feature = "ui")]
pub(crate) fn draw_bounds_to_gtk_snapshot(
    bounds: p2d::bounding_volume::Aabb,
    color: Color,
    snapshot: &gtk4::Snapshot,
    width: f64,
) {
    use crate::ext::GdkRGBAExt;
    use gtk4::{gdk, graphene, gsk, prelude::*};

    let bounds = graphene::Rect::new(
        bounds.mins[0] as f32,
        bounds.mins[1] as f32,
        (bounds.extents()[0]) as f32,
        (bounds.extents()[1]) as f32,
    );

    let rounded_rect = gsk::RoundedRect::new(
        bounds,
        graphene::Size::zero(),
        graphene::Size::zero(),
        graphene::Size::zero(),
        graphene::Size::zero(),
    );

    snapshot.append_border(
        &rounded_rect,
        &[width as f32, width as f32, width as f32, width as f32],
        &[
            gdk::RGBA::from_compose_color(color),
            gdk::RGBA::from_compose_color(color),
            gdk::RGBA::from_compose_color(color),
            gdk::RGBA::from_compose_color(color),
        ],
    )
}

#[cfg(feature = "ui")]
pub(crate) fn draw_pos_to_gtk_snapshot(
    snapshot: &gtk4::Snapshot,
    pos: na::Vector2<f64>,
    color: Color,
    width: f64,
) {
    use crate::ext::GdkRGBAExt;
    use gtk4::{gdk, graphene, prelude::*};

    snapshot.append_color(
        &gdk::RGBA::from_compose_color(color),
        &graphene::Rect::new(
            (pos[0] - 0.5 * width) as f32,
            (pos[1] - 0.5 * width) as f32,
            width as f32,
            width as f32,
        ),
    );
}

#[cfg(feature = "ui")]
pub(crate) fn draw_fill_to_gtk_snapshot(
    snapshot: &gtk4::Snapshot,
    rect: p2d::bounding_volume::Aabb,
    color: Color,
) {
    use crate::ext::{GdkRGBAExt, GrapheneRectExt};
    use gtk4::{gdk, graphene, prelude::*};

    snapshot.append_color(
        &gdk::RGBA::from_compose_color(color),
        &graphene::Rect::from_p2d_aabb(rect),
    );
}

/// Draw some engine statistics for debugging purposes.
///
/// Expects that the snapshot is untransformed in surface coordinate space.
#[cfg(feature = "ui")]
pub(crate) fn draw_statistics_to_gtk_snapshot(
    snapshot: &gtk4::Snapshot,
    engine: &crate::Engine,
    surface_bounds: p2d::bounding_volume::Aabb,
) -> anyhow::Result<()> {
    use crate::ext::GrapheneRectExt;
    use gtk4::{graphene, prelude::*};
    use p2d::bounding_volume::Aabb;
    use piet::{RenderContext, Text, TextLayoutBuilder};
    use rnote_compose::ext::{AabbExt, Vector2Ext};

    // A statistics overlay
    {
        let text_bounds = Aabb::new(
            na::point![
                surface_bounds.maxs[0] - 320.0,
                surface_bounds.mins[1] + 20.0
            ],
            na::point![
                surface_bounds.maxs[0] - 20.0,
                surface_bounds.mins[1] + 120.0
            ],
        );
        let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(text_bounds));
        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

        // Gather statistics
        let strokes_total = engine.store.keys_unordered();
        let strokes_in_viewport = engine
            .store
            .keys_unordered_intersecting_bounds(engine.camera.viewport());
        let selected_strokes = engine.store.selection_keys_unordered();
        let trashed_strokes = engine.store.trashed_keys_unordered();
        let selected_stroke_layers = engine.store.debug_layers(&selected_strokes).join(" ");
        let strokes_hold_image = strokes_total
            .iter()
            .filter(|&&key| engine.store.holds_images(key))
            .count();

        let statistics_text_string = format!(
            "strokes in store:   {}\nstrokes in current viewport:   {}\nstrokes selected: {}\nstroke trashed: {}\nstrokes holding images: {}\nselection layers: {selected_stroke_layers}",
            strokes_total.len(),
            strokes_in_viewport.len(),
            selected_strokes.len(),
            trashed_strokes.len(),
            strokes_hold_image,
        );
        let text_layout = piet_cx
            .text()
            .new_text_layout(statistics_text_string)
            .text_color(piet::Color::rgba(0.8, 1.0, 1.0, 1.0))
            .max_width(text_bounds.extents()[0] - 20.0)
            .alignment(piet::TextAlignment::End)
            .font(piet::FontFamily::MONOSPACE, 10.0)
            .build()
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        piet_cx.fill(
            text_bounds.to_kurbo_rect(),
            &piet::Color::rgba(0.1, 0.1, 0.1, 0.8),
        );
        piet_cx.draw_text(
            &text_layout,
            (text_bounds.mins.coords + na::vector![10.0, 10.0]).to_kurbo_point(),
        );
        piet_cx.finish().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    }
    Ok(())
}

/// Draw stroke bounds, positions, etc. for visual debugging purposes.
#[cfg(feature = "ui")]
pub(crate) fn draw_stroke_debug_to_gtk_snapshot(
    snapshot: &gtk4::Snapshot,
    engine: &crate::Engine,
    surface_bounds: p2d::bounding_volume::Aabb,
) -> anyhow::Result<()> {
    use crate::drawable::DrawableOnDoc;
    use crate::engine_view;
    use p2d::bounding_volume::BoundingVolume;

    let viewport = engine.camera.viewport();
    let total_zoom = engine.camera.total_zoom();
    let doc_bounds = engine.document.bounds();
    let border_widths = 1.0 / total_zoom;

    draw_bounds_to_gtk_snapshot(doc_bounds, COLOR_DOC_BOUNDS, snapshot, border_widths);

    let tightened_viewport = viewport.tightened(2.0 / total_zoom);
    draw_bounds_to_gtk_snapshot(
        tightened_viewport,
        COLOR_STROKE_BOUNDS,
        snapshot,
        border_widths,
    );

    // Draw the strokes
    engine
        .store
        .draw_debug_to_gtk_snapshot(snapshot, engine, surface_bounds)?;

    // Draw the current pen bounds
    if let Some(bounds) = engine.penholder.bounds_on_doc(&engine_view!(engine)) {
        draw_bounds_to_gtk_snapshot(bounds, COLOR_SELECTOR_BOUNDS, snapshot, border_widths);
    }

    Ok(())
}
