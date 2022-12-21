use anyhow::Context;
use gtk4::{gdk, graphene, gsk, prelude::*, Snapshot};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::helpers::AabbHelpers;

use crate::utils::{GdkRGBAHelpers, GrapheneRectHelpers};
use crate::{Document, DrawOnDocBehaviour, RnoteEngine};

use super::{visual_debug, EngineView};

impl RnoteEngine {
    /// updates the background rendering for the current viewport.
    /// if the background pattern or zoom has changed, background_regenerate_pattern() needs to be called first.
    pub fn update_background_rendering_current_viewport(&mut self) -> anyhow::Result<()> {
        let viewport = self.camera.viewport();

        // Update background and strokes for the new viewport
        let mut rendernodes: Vec<gsk::RenderNode> = vec![];

        if let Some(image) = &self.background_tile_image {
            // Only create the texture once, it is expensive
            let new_texture = image
                .to_memtexture()
                .context("image to_memtexture() failed in gen_rendernode() of background.")?;

            for splitted_bounds in
                viewport.split_extended_origin_aligned(self.document.background.tile_size())
            {
                //log::debug!("splitted_bounds: {splitted_bounds:?}");

                rendernodes.push(
                    gsk::TextureNode::new(
                        &new_texture,
                        &graphene::Rect::from_p2d_aabb(splitted_bounds),
                    )
                    .upcast(),
                );
            }
        }

        self.background_rendernodes = rendernodes;

        Ok(())
    }

    /// updates the content rendering for the current viewport. including the background rendering.
    pub fn update_rendering_current_viewport(&mut self) -> anyhow::Result<()> {
        let viewport = self.camera.viewport();
        let image_scale = self.camera.image_scale();

        self.update_background_rendering_current_viewport()?;

        self.store.regenerate_rendering_in_viewport_threaded(
            self.tasks_tx(),
            false,
            viewport,
            image_scale,
        );

        Ok(())
    }

    /// regenerates the background tile image and updates the rendering.
    pub fn background_regenerate_pattern(&mut self) -> anyhow::Result<()> {
        let image_scale = self.camera.image_scale();
        self.background_tile_image = self.document.background.gen_tile_image(image_scale)?;

        self.update_background_rendering_current_viewport()?;
        Ok(())
    }
    /// Draws the entire engine (doc, pens, strokes, selection, ..) on a GTK snapshot.
    pub fn draw_on_gtk_snapshot(
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
        self.draw_format_borders_to_gtk4_snapshot(snapshot)?;
        self.draw_origin_indicator(snapshot)?;

        self.store
            .draw_strokes_to_gtk_snapshot(snapshot, doc_bounds, viewport);

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

        /*
               {
                   use crate::utils::GrapheneRectHelpers;
                   use gtk4::graphene;
                   use piet::RenderContext;
                   use rnote_compose::helpers::Affine2Helpers;

                   let zoom = self.camera.zoom();

                   let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(surface_bounds));
                   let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

                   // Transform to doc coordinate space
                   piet_cx.transform(self.camera.transform().to_kurbo());

                   piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
                   self.store
                       .draw_strokes_immediate_w_piet(&mut piet_cx, doc_bounds, viewport, zoom)?;
                   piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

                   piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

                   self.penholder
                       .draw_on_doc(&mut piet_cx, doc_bounds, &self.camera)?;
                   piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

                   piet_cx.finish().map_err(|e| anyhow::anyhow!("{e:?}"))?;
               }
        */

        // Overlay the visual debug on the canvas
        if self.visual_debug {
            snapshot.save();
            snapshot.transform(Some(&camera_transform));

            // visual debugging
            visual_debug::draw_debug(snapshot, self, surface_bounds)?;

            snapshot.restore();
        }

        // Show some statistics
        if self.visual_debug {
            visual_debug::draw_statistics_overlay(snapshot, self, surface_bounds)?;
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

    fn draw_format_borders_to_gtk4_snapshot(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
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

    fn draw_origin_indicator(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        const PATH_COLOR: piet::Color = color::GNOME_GREENS[4];
        let path_width: f64 = 1.0 / self.camera.total_zoom();

        let indicator_bounds = Aabb::from_half_extents(
            na::point![0.0, 0.0],
            na::Vector2::repeat(6.0 / self.camera.total_zoom()),
        );

        let cairo_node = gsk::CairoNode::new(&graphene::Rect::from_p2d_aabb(indicator_bounds));
        let cairo_cx = cairo_node.draw_context();
        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

        let mut indicator_path = kurbo::BezPath::new();
        indicator_path.move_to(kurbo::Point::new(
            indicator_bounds.mins[0],
            indicator_bounds.mins[1],
        ));
        indicator_path.line_to(kurbo::Point::new(
            indicator_bounds.maxs[0],
            indicator_bounds.maxs[1],
        ));
        indicator_path.move_to(kurbo::Point::new(
            indicator_bounds.mins[0],
            indicator_bounds.maxs[1],
        ));
        indicator_path.line_to(kurbo::Point::new(
            indicator_bounds.maxs[0],
            indicator_bounds.mins[1],
        ));

        piet_cx.stroke(indicator_path, &PATH_COLOR, path_width);

        piet_cx.finish().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        snapshot.append_node(cairo_node.upcast());
        Ok(())
    }
}
