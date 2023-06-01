// Imports
use super::{visual_debug, EngineView};
use crate::utils::{GdkRGBAHelpers, GrapheneRectHelpers};
use crate::{Document, DrawOnDocBehaviour, RnoteEngine};
use gtk4::{gdk, graphene, gsk, prelude::*, Snapshot};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::helpers::{AabbHelpers, Affine2Helpers};

impl RnoteEngine {
    /// Update the background rendering for the current viewport.
    ///
    /// If the background pattern or zoom has changed, the background pattern needs to be regenerated first.
    pub fn update_background_rendering_current_viewport(&mut self) {
        let viewport = self.camera.viewport();
        let mut rendernodes: Vec<gsk::RenderNode> = vec![];

        if let Some(image) = &self.background_tile_image {
            // Only create the texture once, it is expensive
            let new_texture = match image.to_memtexture() {
                Ok(t) => t,
                Err(e) => {
                    log::error!(
                        "failed to generate memory-texture of background tile image, {e:?}"
                    );
                    return;
                }
            };

            for split_bounds in
                viewport.split_extended_origin_aligned(self.document.background.tile_size())
            {
                rendernodes.push(
                    gsk::TextureNode::new(
                        &new_texture,
                        &graphene::Rect::from_p2d_aabb(split_bounds),
                    )
                    .upcast(),
                );
            }
        }

        self.background_rendernodes = rendernodes;
    }

    /// Update the content rendering for the current viewport.
    pub fn update_content_rendering_current_viewport(&mut self) {
        let viewport = self.camera.viewport();
        let image_scale = self.camera.image_scale();

        self.store.regenerate_rendering_in_viewport_threaded(
            self.tasks_tx(),
            false,
            viewport,
            image_scale,
        );
    }

    /// Update the content and background rendering for the current viewport.
    ///
    /// If the background pattern or zoom has changed, the background pattern needs to be regenerated first.
    pub fn update_rendering_current_viewport(&mut self) {
        self.update_background_rendering_current_viewport();
        self.update_content_rendering_current_viewport();
    }

    /// Clear the rendering of the entire engine (e.g. when it becomes off-screen).
    pub fn clear_rendering(&mut self) {
        self.store.clear_rendering();
        self.background_tile_image.take();
        self.background_rendernodes.clear();
    }

    /// Regenerate the background tile image and updates the background rendering.
    pub fn background_regenerate_pattern(&mut self) {
        let image_scale = self.camera.image_scale();
        match self.document.background.gen_tile_image(image_scale) {
            Ok(image) => {
                self.background_tile_image = Some(image);
                self.update_background_rendering_current_viewport();
            }
            Err(e) => log::error!("regenerating background tile image failed, Err: {e:?}"),
        }
    }

    /// Draws the entire engine (doc, pens, strokes, selection, ..) to a GTK snapshot.
    pub fn draw_to_gtk_snapshot(
        &self,
        snapshot: &Snapshot,
        surface_bounds: Aabb,
    ) -> anyhow::Result<()> {
        let doc_bounds = self.document.bounds();
        let viewport = self.camera.viewport();
        let camera_transform = self.camera.transform_for_gtk_snapshot();

        snapshot.save();
        snapshot.transform(Some(&camera_transform));
        self.draw_document_shadow_to_gtk_snapshot(snapshot);
        self.draw_background_to_gtk_snapshot(snapshot)?;
        self.draw_format_borders_to_gtk_snapshot(snapshot)?;
        snapshot.restore();
        self.draw_origin_indicator_to_gtk_snapshot(snapshot)?;
        snapshot.save();
        snapshot.transform(Some(&camera_transform));
        self.store
            .draw_strokes_to_gtk_snapshot(snapshot, doc_bounds, viewport);
        /*
               let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(viewport));
               let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);
               self.store.draw_strokes_immediate(
                   &mut piet_cx,
                   doc_bounds,
                   viewport,
                   self.camera.image_scale(),
               );
        */
        snapshot.restore();
        self.penholder.draw_on_doc_to_gtk_snapshot(
            snapshot,
            &EngineView {
                tasks_tx: self.tasks_tx(),
                pens_config: &self.pens_config,
                doc: &self.document,
                store: &self.store,
                camera: &self.camera,
                audioplayer: &self.audioplayer,
            },
        )?;

        if self.visual_debug {
            snapshot.save();
            snapshot.transform(Some(&camera_transform));
            visual_debug::draw_stroke_debug_to_gtk_snapshot(snapshot, self, surface_bounds)?;
            snapshot.restore();

            visual_debug::draw_statistics_to_gtk_snapshot(snapshot, self, surface_bounds)?;
        }

        Ok(())
    }

    fn draw_document_shadow_to_gtk_snapshot(&self, snapshot: &Snapshot) {
        let shadow_width = Document::SHADOW_WIDTH;
        let shadow_offset = Document::SHADOW_OFFSET;
        let doc_bounds = self.document.bounds();

        let corner_radius =
            graphene::Size::new(shadow_width as f32 * 0.25, shadow_width as f32 * 0.25);

        let rounded_rect = gsk::RoundedRect::new(
            graphene::Rect::from_p2d_aabb(doc_bounds),
            corner_radius,
            corner_radius,
            corner_radius,
            corner_radius,
        );

        snapshot.append_outset_shadow(
            &rounded_rect,
            &gdk::RGBA::from_compose_color(Document::SHADOW_COLOR),
            shadow_offset[0] as f32,
            shadow_offset[1] as f32,
            0.0,
            (shadow_width) as f32,
        );
    }

    fn draw_background_to_gtk_snapshot(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        let doc_bounds = self.document.bounds();

        snapshot.push_clip(&graphene::Rect::from_p2d_aabb(doc_bounds));

        // Fill with background color just in case there is any space left between the tiles
        snapshot.append_node(
            &gsk::ColorNode::new(
                &gdk::RGBA::from_compose_color(self.document.background.color),
                //&gdk::RGBA::RED,
                &graphene::Rect::from_p2d_aabb(doc_bounds),
            )
            .upcast(),
        );

        for r in self.background_rendernodes.iter() {
            snapshot.append_node(r);
        }

        snapshot.pop();
        Ok(())
    }

    fn draw_format_borders_to_gtk_snapshot(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        if self.document.format.show_borders {
            let total_zoom = self.camera.total_zoom();
            let border_width = 1.0 / total_zoom;
            let viewport = self.camera.viewport();
            let doc_bounds = self.document.bounds();

            snapshot.push_clip(&graphene::Rect::from_p2d_aabb(doc_bounds.loosened(2.0)));

            for page_bounds in doc_bounds.split_extended_origin_aligned(na::vector![
                self.document.format.width,
                self.document.format.height
            ]) {
                if !page_bounds.intersects(&viewport) {
                    continue;
                }

                let rounded_rect = gsk::RoundedRect::new(
                    graphene::Rect::from_p2d_aabb(page_bounds),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                );

                snapshot.append_border(
                    &rounded_rect,
                    &[
                        border_width as f32,
                        border_width as f32,
                        border_width as f32,
                        border_width as f32,
                    ],
                    &[
                        gdk::RGBA::from_compose_color(self.document.format.border_color),
                        gdk::RGBA::from_compose_color(self.document.format.border_color),
                        gdk::RGBA::from_compose_color(self.document.format.border_color),
                        gdk::RGBA::from_compose_color(self.document.format.border_color),
                    ],
                )
            }

            snapshot.pop();
        }

        Ok(())
    }

    /// Draw the document origin indicator cross.
    ///
    /// Expects that the snapshot is untransformed in surface coordinate space.
    fn draw_origin_indicator_to_gtk_snapshot(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        const PATH_COLOR: piet::Color = color::GNOME_GREENS[4];
        const PATH_WIDTH: f64 = 1.5;
        let total_zoom = self.camera.total_zoom();

        let bounds =
            Aabb::from_half_extents(na::point![0.0, 0.0], na::Vector2::repeat(5.0 / total_zoom));
        let bounds_on_surface = bounds
            .extend_by(na::Vector2::repeat(PATH_WIDTH / total_zoom))
            .scale(total_zoom)
            .translate(-self.camera.offset());

        let cairo_cx =
            snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(bounds_on_surface.ceil()));
        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);
        let path = kurbo::BezPath::from_iter([
            kurbo::PathEl::MoveTo(kurbo::Point::new(bounds.mins[0], bounds.mins[1])),
            kurbo::PathEl::LineTo(kurbo::Point::new(bounds.maxs[0], bounds.maxs[1])),
            kurbo::PathEl::MoveTo(kurbo::Point::new(bounds.mins[0], bounds.maxs[1])),
            kurbo::PathEl::LineTo(kurbo::Point::new(bounds.maxs[0], bounds.mins[1])),
        ]);
        piet_cx.transform(self.camera.transform().to_kurbo());
        piet_cx.stroke_styled(
            path,
            &PATH_COLOR,
            PATH_WIDTH / total_zoom,
            &piet::StrokeStyle::default().line_cap(piet::LineCap::Round),
        );

        Ok(())
    }
}
