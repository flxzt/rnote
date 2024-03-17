// Imports
use super::{Stroke, StrokeKey, StrokeStore};
use crate::engine::{EngineTask, EngineTaskSender};
use crate::strokes::content::GeneratedContentImages;
use crate::strokes::Content;
use crate::{render, Drawable};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::ext::AabbExt;
use rnote_compose::shapes::Shapeable;

/// The tolerance where check between scale-factors are considered "equal".
pub(crate) const RENDER_IMAGE_SCALE_TOLERANCE: f64 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderCompState {
    Complete,
    ForViewport(Aabb),
    BusyRenderingInTask,
    Dirty,
}

impl Default for RenderCompState {
    fn default() -> Self {
        Self::Dirty
    }
}

#[derive(Debug, Clone)]
pub struct RenderComponent {
    pub(super) state: RenderCompState,
    pub(super) images: Vec<render::Image>,
    #[cfg(feature = "ui")]
    pub(super) rendernodes: Vec<gtk4::gsk::RenderNode>,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            state: RenderCompState::default(),
            images: vec![],
            #[cfg(feature = "ui")]
            rendernodes: vec![],
        }
    }
}

impl StrokeStore {
    /// Rebuild the slotmap with empty render components with the keys returned from the stroke components.
    pub(crate) fn rebuild_render_components_slotmap(&mut self) {
        self.render_components = slotmap::SecondaryMap::new();
        self.stroke_components.keys().for_each(|key| {
            self.render_components
                .insert(key, RenderComponent::default());
        });
    }

    /// Rebuild the render components slotmap while retaining the components for all currently stored strokes
    pub(crate) fn rebuild_retain_valid_keys_render_components(&mut self) {
        self.render_components
            .retain(|k, _| self.stroke_components.contains_key(k));
        self.stroke_components.keys().for_each(|k| {
            if !self.render_components.contains_key(k) {
                self.render_components.insert(k, RenderComponent::default());
            }
        });
    }

    /// Ability if rendering is supported.
    #[allow(unused)]
    pub(crate) fn can_render(&self, key: StrokeKey) -> bool {
        self.render_components.get(key).is_some()
    }

    pub(crate) fn render_comp_state(&self, key: StrokeKey) -> Option<RenderCompState> {
        self.render_components
            .get(key)
            .map(|render_comp| render_comp.state)
    }

    pub(crate) fn set_rendering_dirty(&mut self, key: StrokeKey) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            render_comp.state = RenderCompState::Dirty;
        }
    }

    pub(crate) fn set_rendering_dirty_for_strokes(&mut self, keys: &[StrokeKey]) {
        keys.iter().for_each(|&key| self.set_rendering_dirty(key));
    }

    #[allow(unused)]
    pub(crate) fn holds_images(&self, key: StrokeKey) -> bool {
        self.render_components
            .get(key)
            .map(|s| !s.images.is_empty())
            .unwrap_or(false)
    }

    pub(crate) fn regenerate_rendering_for_stroke(
        &mut self,
        key: StrokeKey,
        viewport: Aabb,
        image_scale: f64,
    ) {
        if let (Some(stroke), Some(render_comp)) = (
            self.stroke_components.get(key),
            self.render_components.get_mut(key),
        ) {
            if render_comp.state == RenderCompState::BusyRenderingInTask {
                return;
            }

            let viewport_extended =
                viewport.extend_by(viewport.extents() * render::VIEWPORT_EXTENTS_MARGIN_FACTOR);

            match stroke.gen_images(viewport_extended, image_scale) {
                Ok(GeneratedContentImages::Partial { images, viewport }) => {
                    #[cfg(feature = "ui")]
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.images = images;
                            render_comp.state = RenderCompState::ForViewport(viewport);
                        }
                        Err(e) => {
                            render_comp.state = RenderCompState::Dirty;
                            tracing::error!(
                                "Creating rendernodes from partial images failed while regenerating stroke rendering, Err: {e:?}"
                            );
                        }
                    }
                    #[cfg(not(feature = "ui"))]
                    {
                        render_comp.images = images;
                        render_comp.state = RenderCompState::ForViewport(viewport);
                    }
                }
                Ok(GeneratedContentImages::Full(images)) => {
                    #[cfg(feature = "ui")]
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.images = images;
                            render_comp.state = RenderCompState::Complete;
                        }
                        Err(e) => {
                            render_comp.state = RenderCompState::Dirty;
                            tracing::error!(
                                "Creating rendernodes from full images failed while regenerating stroke rendering, Err: {e:?}"
                            );
                        }
                    }
                    #[cfg(not(feature = "ui"))]
                    {
                        render_comp.images = images;
                        render_comp.state = RenderCompState::Complete;
                    }
                }
                Err(e) => {
                    render_comp.state = RenderCompState::Dirty;
                    tracing::error!(
                        "Generating images for stroke with key {key:?} failed, Err: {e:?}"
                    );
                }
            }
        }
    }

    pub(crate) fn regenerate_rendering_for_strokes(
        &mut self,
        keys: &[StrokeKey],
        viewport: Aabb,
        image_scale: f64,
    ) {
        for &key in keys {
            self.regenerate_rendering_for_stroke(key, viewport, image_scale);
        }
    }

    pub(crate) fn regenerate_rendering_for_stroke_threaded(
        &mut self,
        tasks_tx: EngineTaskSender,
        key: StrokeKey,
        viewport: Aabb,
        image_scale: f64,
    ) {
        if let (Some(render_comp), Some(stroke)) = (
            self.render_components.get_mut(key),
            self.stroke_components.get(key),
        ) {
            if render_comp.state == RenderCompState::BusyRenderingInTask {
                return;
            }

            let stroke = stroke.clone();
            let viewport_extended =
                viewport.extend_by(viewport.extents() * render::VIEWPORT_EXTENTS_MARGIN_FACTOR);

            // indicates that a task is now started rendering the stroke
            render_comp.state = RenderCompState::BusyRenderingInTask;

            // Spawn a new thread for image rendering
            rayon::spawn(
                move || match stroke.gen_images(viewport_extended, image_scale) {
                    Ok(images) => {
                        tasks_tx.send(EngineTask::UpdateStrokeWithImages {
                            key,
                            images,
                            image_scale,
                        });
                    }
                    Err(e) => {
                        tracing::error!(
                            "Generating images of stroke failed while regenerating stroke rendering, stroke key {key:?} , Err: {e:?}"
                        );
                    }
                },
            );
        }
    }

    pub(crate) fn regenerate_rendering_for_strokes_threaded(
        &mut self,
        tasks_tx: EngineTaskSender,
        keys: &[StrokeKey],
        viewport: Aabb,
        image_scale: f64,
    ) {
        for &key in keys {
            self.regenerate_rendering_for_stroke_threaded(
                tasks_tx.clone(),
                key,
                viewport,
                image_scale,
            );
        }
    }

    /// Regenerate the rendering of all keys for the given viewport that need to be rerendered.
    pub(crate) fn regenerate_rendering_in_viewport_threaded(
        &mut self,
        tasks_tx: EngineTaskSender,
        force_regenerate: bool,
        viewport: Aabb,
        image_scale: f64,
    ) {
        let keys = self.render_components.keys().collect::<Vec<StrokeKey>>();

        for key in keys {
            if let (Some(stroke), Some(render_comp)) = (
                self.stroke_components.get(key),
                self.render_components.get_mut(key),
            ) {
                let tasks_tx = tasks_tx.clone();
                let stroke_bounds = stroke.bounds();
                let viewport_extended =
                    viewport.extend_by(viewport.extents() * render::VIEWPORT_EXTENTS_MARGIN_FACTOR);

                // skip and clear image buffer if stroke is not in viewport
                if !viewport_extended.intersects(&stroke_bounds) {
                    #[cfg(feature = "ui")]
                    {
                        render_comp.rendernodes = vec![];
                    }
                    render_comp.images = vec![];
                    render_comp.state = RenderCompState::Dirty;
                    continue;
                }

                // only check if rerendering is not forced
                if !force_regenerate {
                    match render_comp.state {
                        RenderCompState::Complete | RenderCompState::BusyRenderingInTask => {
                            continue;
                        }
                        RenderCompState::ForViewport(old_viewport) => {
                            /// This factor is applied on top of the viewport extents margin factor,
                            /// so that rerendering is started a bit earlier to reaching
                            /// the edges of the viewport of the current rendered images.
                            const VIEWPORT_EXTENTS_MARGIN_RERENDER_THRESHOLD: f64 = 0.7;

                            if old_viewport.contains(
                                &(viewport.extend_by(
                                    viewport.extents()
                                        * render::VIEWPORT_EXTENTS_MARGIN_FACTOR
                                        * VIEWPORT_EXTENTS_MARGIN_RERENDER_THRESHOLD,
                                )),
                            ) {
                                continue;
                            }
                        }
                        RenderCompState::Dirty => {}
                    }
                }

                // indicates that a task has now started to render the stroke
                render_comp.state = RenderCompState::BusyRenderingInTask;
                let stroke = stroke.clone();

                // Spawn a new thread for image rendering
                rayon::spawn(
                    move || match stroke.gen_images(viewport_extended, image_scale) {
                        Ok(images) => {
                            tasks_tx.send(EngineTask::UpdateStrokeWithImages {
                                key,
                                images,
                                image_scale,
                            });
                        }
                        Err(e) => {
                            tracing::error!(
                                "Generating stroke images failed stroke while regenerating rendering in viewport `{viewport:?}`, stroke key: {key:?}, Err: {e:?}"
                            );
                        }
                    },
                );
            }
        }
    }

    /// Clear all rendering for all strokes.
    pub(crate) fn clear_rendering(&mut self) {
        for (_key, render_comp) in self.render_components.iter_mut() {
            #[cfg(feature = "ui")]
            {
                render_comp.rendernodes = vec![];
            }
            render_comp.images = vec![];
            render_comp.state = RenderCompState::Dirty;
        }
    }

    /// Generate images and appends them to the render component for the last segments of brushstrokes.
    ///
    /// For other strokes the rendering is regenerated completely.
    pub(crate) fn append_rendering_last_segments(
        &mut self,
        tasks_tx: EngineTaskSender,
        key: StrokeKey,
        n_last_segments: usize,
        viewport: Aabb,
        image_scale: f64,
    ) {
        if let (Some(stroke), Some(render_comp)) = (
            self.stroke_components.get(key),
            self.render_components.get_mut(key),
        ) {
            match stroke.as_ref() {
                Stroke::BrushStroke(brushstroke) => {
                    match brushstroke.gen_image_for_last_segments(n_last_segments, image_scale) {
                        Ok(Some(image)) => {
                            #[cfg(feature = "ui")]
                            match render::Image::images_to_rendernodes([&image]) {
                                Ok(mut rendernodes) => {
                                    render_comp.rendernodes.append(&mut rendernodes);
                                    render_comp.images.push(image);
                                }
                                Err(e) => {
                                    render_comp.state = RenderCompState::Dirty;
                                    tracing::error!("Failed to generated rendernodes while appending last segments rendering, Err: {e:?}");
                                }
                            }
                            #[cfg(not(feature = "ui"))]
                            {
                                render_comp.images.push(image);
                            }
                        }

                        Ok(None) => {}
                        Err(e) => {
                            render_comp.state = RenderCompState::Dirty;
                            tracing::error!(
                                "Failed to generate image while appending last segments rendering, Err: {e:?}"
                            );
                        }
                    }
                }
                // regenerate everything for strokes that don't support generating svgs for the last added elements
                Stroke::ShapeStroke(_)
                | Stroke::TextStroke(_)
                | Stroke::VectorImage(_)
                | Stroke::BitmapImage(_) => {
                    self.regenerate_rendering_for_stroke_threaded(
                        tasks_tx,
                        key,
                        viewport,
                        image_scale,
                    );
                }
            }
        }
    }

    /// Replace the entire current rendering with the given new images.
    ///
    /// Also updates the render component state.
    pub(crate) fn replace_rendering_with_images(
        &mut self,
        key: StrokeKey,
        images: GeneratedContentImages,
    ) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            match images {
                GeneratedContentImages::Partial { images, viewport } => {
                    #[cfg(feature = "ui")]
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.images = images;
                            render_comp.state = RenderCompState::ForViewport(viewport);
                        }
                        Err(e) => {
                            tracing::error!("Generating rendernodes failed while replacing rendering with partial images, Err {e:?}");
                            render_comp.state = RenderCompState::Dirty;
                        }
                    }
                    #[cfg(not(feature = "ui"))]
                    {
                        render_comp.images = images;
                        render_comp.state = RenderCompState::ForViewport(viewport);
                    }
                }
                GeneratedContentImages::Full(images) => {
                    #[cfg(feature = "ui")]
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.images = images;
                            render_comp.state = RenderCompState::Complete;
                        }
                        Err(e) => {
                            tracing::error!("Generating rendernodes failed while replacing rendering with full images, Err {e:?}");
                            render_comp.state = RenderCompState::Dirty;
                        }
                    }
                    #[cfg(not(feature = "ui"))]
                    {
                        render_comp.images = images;
                        render_comp.state = RenderCompState::Complete;
                    }
                }
            }
        }
    }

    /// Appends the images to the render component of the stroke.
    ///
    /// Not modifying the render component state, that is the responsibility of the caller.
    pub(crate) fn append_rendering_images(
        &mut self,
        key: StrokeKey,
        images: GeneratedContentImages,
    ) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            match images {
                GeneratedContentImages::Partial {
                    mut images,
                    viewport: _,
                }
                | GeneratedContentImages::Full(mut images) => {
                    #[cfg(feature = "ui")]
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(mut rendernodes) => {
                            render_comp.rendernodes.append(&mut rendernodes);
                            render_comp.images.append(&mut images);
                        }
                        Err(e) => {
                            tracing::error!("Generating rendernodes failed while appending rendering full images, Err {e:?}");
                            render_comp.state = RenderCompState::Dirty;
                        }
                    }
                    #[cfg(not(feature = "ui"))]
                    {
                        render_comp.images.append(&mut images);
                    }
                }
            }
        }
    }

    /// Draw all strokes on the gtk snapshot.
    #[cfg(feature = "ui")]
    pub(crate) fn draw_strokes_to_gtk_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
        doc_bounds: Aabb,
        viewport: Aabb,
    ) {
        use crate::ext::{GdkRGBAExt, GrapheneRectExt};
        use gtk4::{gdk, graphene, prelude::*};
        use rnote_compose::color;

        snapshot.push_clip(&graphene::Rect::from_p2d_aabb(doc_bounds));

        for key in self.stroke_keys_as_rendered_intersecting_bounds(viewport) {
            if let (Some(stroke), Some(render_comp)) = (
                self.stroke_components.get(key),
                self.render_components.get(key),
            ) {
                // if the stroke currently does not have a rendering and is will create one,
                // draw a placeholder filled rect
                if render_comp.rendernodes.is_empty()
                    && matches!(
                        render_comp.state,
                        RenderCompState::Dirty | RenderCompState::BusyRenderingInTask
                    )
                {
                    snapshot.append_color(
                        &gdk::RGBA::from_piet_color(color::GNOME_BRIGHTS[1].with_alpha(0.13)),
                        &graphene::Rect::from_p2d_aabb(stroke.bounds()),
                    );
                }

                for rendernode in render_comp.rendernodes.iter() {
                    snapshot.append_node(rendernode);
                }
            }
        }

        snapshot.pop();
    }

    /// Draw the strokes for the given keys on the [piet::RenderContext].
    ///
    /// This always draws all strokes for the given keys, even trashed ones.
    #[allow(unused)]
    pub(crate) fn draw_keys_immediate(
        &self,
        keys: &[StrokeKey],
        piet_cx: &mut impl piet::RenderContext,
        image_scale: f64,
    ) -> anyhow::Result<()> {
        for &key in keys {
            if let Some(stroke) = self.stroke_components.get(key) {
                stroke.draw(piet_cx, image_scale)?;
            }
        }

        Ok(())
    }

    /// Draw all strokes intersecting the viewport on the [piet::RenderContext].
    ///
    /// Immediate without any cached images.
    #[allow(unused)]
    pub(crate) fn draw_strokes_immediate(
        &self,
        piet_cx: &mut impl piet::RenderContext,
        _doc_bounds: Aabb,
        viewport: Aabb,
        image_scale: f64,
    ) {
        for key in self.stroke_keys_as_rendered_intersecting_bounds(viewport) {
            if let Some(stroke) = self.stroke_components.get(key) {
                if let Err(e) = stroke.draw(piet_cx, image_scale) {
                    tracing::error!(
                        "Drawing stroke immediate on piet RenderContext failed , Err: {e:?}"
                    );
                }
            }
        }
    }

    /// Draw bounds, positions, etc. for all strokes for visual debugging purposes.
    #[cfg(feature = "ui")]
    pub(crate) fn draw_debug_to_gtk_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
        engine: &crate::Engine,
        _surface_bounds: Aabb,
    ) -> anyhow::Result<()> {
        use crate::engine::visual_debug;
        use gtk4::prelude::*;

        let border_widths = 1.0 / engine.camera.total_zoom();

        for key in self.keys_sorted_chrono() {
            if let Some(stroke) = self.stroke_components.get(key) {
                // Push opacity for strokes which are normally hidden
                if let Some(trash_comp) = self.trash_components.get(key) {
                    if trash_comp.trashed {
                        snapshot.push_opacity(0.2);
                    }
                }

                if let Some(render_comp) = self.render_components.get(key) {
                    match render_comp.state {
                        RenderCompState::Dirty => {
                            visual_debug::draw_fill_to_gtk_snapshot(
                                snapshot,
                                stroke.bounds(),
                                visual_debug::COLOR_STROKE_RENDERING_DIRTY,
                            );
                        }
                        RenderCompState::BusyRenderingInTask => {
                            visual_debug::draw_fill_to_gtk_snapshot(
                                snapshot,
                                stroke.bounds(),
                                visual_debug::COLOR_STROKE_RENDERING_BUSY,
                            );
                        }
                        _ => {}
                    }
                    render_comp.images.iter().for_each(|image| {
                        visual_debug::draw_bounds_to_gtk_snapshot(
                            // a little tightened not to overlap with other bounds
                            image.rect.bounds().tightened(2.0 * border_widths),
                            visual_debug::COLOR_IMAGE_BOUNDS,
                            snapshot,
                            border_widths,
                        )
                    });
                }

                for &hitbox_elem in stroke.hitboxes().iter() {
                    visual_debug::draw_bounds_to_gtk_snapshot(
                        hitbox_elem,
                        visual_debug::COLOR_STROKE_HITBOX,
                        snapshot,
                        border_widths,
                    );
                }

                visual_debug::draw_bounds_to_gtk_snapshot(
                    stroke.bounds(),
                    visual_debug::COLOR_STROKE_BOUNDS,
                    snapshot,
                    border_widths,
                );

                match stroke.as_ref() {
                    // Draw positions for brushstrokes
                    Stroke::BrushStroke(brushstroke) => {
                        for element in brushstroke.path.clone().into_elements().iter() {
                            visual_debug::draw_pos_to_gtk_snapshot(
                                snapshot,
                                element.pos,
                                visual_debug::COLOR_POS,
                                border_widths * 4.0,
                            )
                        }
                    }
                    _ => {}
                }

                // Pop Blur and opacity for hidden strokes
                if let Some(trash_comp) = self.trash_components.get(key) {
                    if trash_comp.trashed {
                        snapshot.pop();
                    }
                }
            }
        }

        Ok(())
    }
}
