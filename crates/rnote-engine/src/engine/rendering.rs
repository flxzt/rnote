// Imports
use crate::render::Image;
use crate::{Engine, WidgetFlags};
use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use rnote_compose::color;

impl Engine {
    /// Update the background rendering for the current viewport.
    ///
    /// If the background pattern or zoom has changed, the background pattern needs to be regenerated first.
    pub fn update_background_rendering_current_viewport(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        #[cfg(feature = "ui")]
        {
            use crate::ext::GrapheneRectExt;
            use gtk4::{graphene, gsk, prelude::*};
            use rnote_compose::ext::AabbExt;
            use rnote_compose::SplitOrder;

            let viewport = self.camera.viewport();
            let mut rendernodes: Vec<gsk::RenderNode> = vec![];

            if let Some(image) = &self.background_tile_image {
                // Only create the texture once, it is expensive
                let new_texture = match image.to_memtexture() {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::error!(
                            "failed to generate memory-texture of background tile image, {e:?}"
                        );
                        return widget_flags;
                    }
                };

                for split_bounds in viewport.split_extended_origin_aligned(
                    self.document.background.tile_size(),
                    SplitOrder::default(),
                ) {
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

        #[cfg(feature = "ui")]
        {
            use crate::ext::GrapheneRectExt;
            use gtk4::{graphene, gsk, prelude::*};

            let total_zoom = self.camera.total_zoom();

            if let Some(image) = &self.origin_indicator_image {
                // Only create the texture once, it is expensive
                let new_texture = match image.to_memtexture() {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::error!(
                            "failed to generate memory-texture of origin indicator image, {e:?}"
                        );
                        return widget_flags;
                    }
                };

                self.origin_indicator_rendernode = Some(
                    gsk::TextureNode::new(
                        &new_texture,
                        &graphene::Rect::from_p2d_aabb(
                            origin_indicator_bounds()
                                .scaled(&na::Vector2::repeat(1.0 / total_zoom)),
                        ),
                    )
                    .upcast(),
                );
            }
        }

        widget_flags.redraw = true;
        widget_flags
    }

    /// Update the content rendering for the current viewport.
    pub fn update_content_rendering_current_viewport(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.store.regenerate_rendering_in_viewport_threaded(
            self.engine_tasks_tx(),
            false,
            self.camera.viewport(),
            self.camera.image_scale(),
        );
        widget_flags.redraw = true;
        widget_flags
    }

    /// Update the content and background rendering for the current viewport.
    ///
    /// If the background pattern or zoom has changed, the background pattern needs to be regenerated first.
    pub fn update_rendering_current_viewport(&mut self) -> WidgetFlags {
        self.update_background_rendering_current_viewport()
            | self.update_content_rendering_current_viewport()
    }

    /// Clear the rendering of the entire engine (e.g. when it becomes off-screen).
    pub fn clear_rendering(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.store.clear_rendering();
        self.background_tile_image.take();
        self.origin_indicator_image.take();
        #[cfg(feature = "ui")]
        {
            self.background_rendernodes.clear();
            self.origin_indicator_rendernode.take();
        }
        widget_flags.redraw = true;
        widget_flags
    }

    /// Regenerate the background tile image, origin indicator and updates the background rendering.
    pub fn background_rendering_regenerate(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        let image_scale = self.camera.image_scale();
        let scale_factor = self.camera.scale_factor();

        match self.document.background.gen_tile_image(image_scale) {
            Ok(image) => {
                self.background_tile_image = Some(image);
            }
            Err(e) => {
                tracing::error!("Regenerating background tile image failed, Err: {e:?}");
                return widget_flags;
            }
        }

        match gen_origin_indicator_image(scale_factor) {
            Ok(image) => {
                self.origin_indicator_image = Some(image);
            }
            Err(e) => {
                tracing::error!("Regenerating origin indicator image failed, Err: {e:?}");
                widget_flags.redraw = true;
                return widget_flags;
            }
        }

        widget_flags |= self.update_background_rendering_current_viewport();
        widget_flags.redraw = true;
        widget_flags
    }

    /// Draws the entire engine (doc, pens, strokes, selection, ..) to a GTK snapshot.
    #[cfg(feature = "ui")]
    pub fn draw_to_gtk_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
        surface_bounds: p2d::bounding_volume::Aabb,
    ) -> anyhow::Result<()> {
        use crate::drawable::DrawableOnDoc;
        use crate::engine::visual_debug;
        use crate::engine::EngineView;
        use gtk4::prelude::*;

        let doc_bounds = self.document.bounds();
        let viewport = self.camera.viewport();
        let camera_transform = self.camera.transform_for_gtk_snapshot();

        snapshot.save();
        snapshot.transform(Some(&camera_transform));
        self.draw_document_shadow_to_gtk_snapshot(snapshot);
        self.draw_background_to_gtk_snapshot(snapshot)?;
        self.draw_format_borders_to_gtk_snapshot(snapshot)?;
        self.draw_origin_indicator_to_gtk_snapshot(snapshot)?;
        self.store
            .draw_strokes_to_gtk_snapshot(snapshot, doc_bounds, viewport);
        snapshot.restore();
        /*
               let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(surface_bounds));
               let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);
               piet_cx.transform(self.camera.transform().to_kurbo());
               self.store.draw_strokes_immediate(
                   &mut piet_cx,
                   doc_bounds,
                   viewport,
                   self.camera.image_scale(),
               );
        */
        self.penholder.draw_on_doc_to_gtk_snapshot(
            snapshot,
            &EngineView {
                tasks_tx: self.engine_tasks_tx(),
                pens_config: &self.pens_config,
                document: &self.document,
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

    #[cfg(feature = "ui")]
    fn draw_document_shadow_to_gtk_snapshot(&self, snapshot: &gtk4::Snapshot) {
        use crate::ext::{GdkRGBAExt, GrapheneRectExt};
        use crate::Document;
        use gtk4::{gdk, graphene, gsk, prelude::*};

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

    #[cfg(feature = "ui")]
    fn draw_background_to_gtk_snapshot(&self, snapshot: &gtk4::Snapshot) -> anyhow::Result<()> {
        use crate::ext::{GdkRGBAExt, GrapheneRectExt};
        use gtk4::{gdk, graphene, gsk, prelude::*};

        let doc_bounds = self.document.bounds();

        snapshot.push_clip(&graphene::Rect::from_p2d_aabb(doc_bounds));

        // Fill with background color just in case there is any space left between the tiles
        snapshot.append_node(
            gsk::ColorNode::new(
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

    #[cfg(feature = "ui")]
    fn draw_format_borders_to_gtk_snapshot(&self, snapshot: &gtk4::Snapshot) -> anyhow::Result<()> {
        use crate::ext::{GdkRGBAExt, GrapheneRectExt};
        use gtk4::{gdk, graphene, gsk, prelude::*};
        use p2d::bounding_volume::BoundingVolume;
        use rnote_compose::ext::AabbExt;
        use rnote_compose::SplitOrder;

        if self.document.format.show_borders {
            let total_zoom = self.camera.total_zoom();
            let border_width = 1.0 / total_zoom;
            let viewport = self.camera.viewport();
            let doc_bounds = self.document.bounds();

            snapshot.push_clip(&graphene::Rect::from_p2d_aabb(doc_bounds.loosened(2.0)));

            for page_bounds in doc_bounds
                .split_extended_origin_aligned(self.document.format.size(), SplitOrder::default())
            {
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
    #[cfg(feature = "ui")]
    fn draw_origin_indicator_to_gtk_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
    ) -> anyhow::Result<()> {
        use gtk4::prelude::*;

        if self.document.format.show_origin_indicator {
            if let Some(r) = &self.origin_indicator_rendernode {
                snapshot.append_node(r);
            }
        }

        Ok(())
    }
}

/// Origin indicator bounds in document coordinate space.
fn origin_indicator_bounds() -> Aabb {
    const SIZE: na::Vector2<f64> = na::vector![17., 17.];
    Aabb::from_half_extents(na::Vector2::zeros().into(), SIZE * 0.5)
}

fn gen_origin_indicator_image(scale_factor: f64) -> anyhow::Result<Image> {
    const PATH_COLOR: piet::Color = color::GNOME_GREENS[4];
    const PATH_WIDTH: f64 = 1.5;
    let bounds = origin_indicator_bounds();

    Image::gen_with_piet(
        |piet_cx| {
            let path = kurbo::BezPath::from_iter([
                kurbo::PathEl::MoveTo(kurbo::Point::new(
                    bounds.mins[0] + PATH_WIDTH * 0.5,
                    bounds.mins[1] + PATH_WIDTH * 0.5,
                )),
                kurbo::PathEl::LineTo(kurbo::Point::new(
                    bounds.maxs[0] - PATH_WIDTH * 0.5,
                    bounds.maxs[1] - PATH_WIDTH * 0.5,
                )),
                kurbo::PathEl::MoveTo(kurbo::Point::new(
                    bounds.mins[0] + PATH_WIDTH * 0.5,
                    bounds.maxs[1] - PATH_WIDTH * 0.5,
                )),
                kurbo::PathEl::LineTo(kurbo::Point::new(
                    bounds.maxs[0] - PATH_WIDTH * 0.5,
                    bounds.mins[1] + PATH_WIDTH * 0.5,
                )),
            ]);
            piet_cx.stroke_styled(
                path,
                &PATH_COLOR,
                PATH_WIDTH,
                &piet::StrokeStyle::default().line_cap(piet::LineCap::Round),
            );
            Ok(())
        },
        bounds,
        scale_factor,
    )
}
