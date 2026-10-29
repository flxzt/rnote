// Imports
#[cfg(feature = "ui")]
use gtk4::pango;
#[cfg(feature = "ui")]
use p2d::math::Vector2;
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
    pos: Vector2,
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

    // A statistics overlay
    {
        let text_bounds = Aabb::new(
            Vector2::new(
                surface_bounds.maxs[0] - 260.0,
                surface_bounds.mins[1] + 10.0,
            ),
            Vector2::new(
                surface_bounds.maxs[0] - 10.0,
                surface_bounds.mins[1] + 110.0,
            ),
        );
        let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(text_bounds));

        // Gather statistics
        let strokes_total = engine.store.keys_unordered();
        let strokes_in_viewport = engine
            .store
            .keys_unordered_intersecting_bounds(engine.camera.viewport());
        let selected_strokes = engine.store.selection_keys_unordered();
        let trashed_strokes = engine.store.trashed_keys_unordered();
        let strokes_hold_image = strokes_total
            .iter()
            .filter(|&&key| engine.store.holds_images(key))
            .count();

        let statistics = format!(
            "strokes in store:\t\t{}
strokes in viewport:\t\t{}
strokes selected:\t\t{}
stroke trashed:\t\t\t{}
strokes holding images:\t\t{}",
            strokes_total.len(),
            strokes_in_viewport.len(),
            selected_strokes.len(),
            trashed_strokes.len(),
            strokes_hold_image,
        );
        let pango_cx = pangocairo::functions::create_context(&cairo_cx);
        let text_layout = pango::Layout::new(&pango_cx);
        text_layout.set_text(&statistics);
        text_layout.set_font_description(Some(&pango::FontDescription::from_string(
            "Monospace Regular 8",
        )));
        cairo_cx.move_to(text_bounds.mins.x, text_bounds.mins.y);
        cairo_cx.line_to(text_bounds.maxs.x, text_bounds.mins.y);
        cairo_cx.line_to(text_bounds.maxs.x, text_bounds.maxs.y);
        cairo_cx.line_to(text_bounds.mins.x, text_bounds.maxs.y);
        cairo_cx.close_path();
        cairo_cx.set_source_rgba(0.1, 0.1, 0.1, 0.9);
        cairo_cx.fill()?;
        cairo_cx.move_to(text_bounds.mins.x + 10., text_bounds.mins.y + 10.);
        cairo_cx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        pangocairo::functions::update_layout(&cairo_cx, &text_layout);
        pangocairo::functions::show_layout(&cairo_cx, &text_layout);
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
    use crate::Previewable;
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
    if let Some(bounds) = engine.penholder.preview_bounds(&engine_view!(engine)) {
        draw_bounds_to_gtk_snapshot(bounds, COLOR_SELECTOR_BOUNDS, snapshot, border_widths);
    }

    Ok(())
}
