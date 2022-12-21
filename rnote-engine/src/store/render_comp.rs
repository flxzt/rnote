use super::{Stroke, StrokeKey, StrokeStore};
use crate::engine::visual_debug;
use crate::engine::{EngineTask, EngineTaskSender};
use crate::strokes::strokebehaviour::GeneratedStrokeImages;
use crate::strokes::StrokeBehaviour;
use crate::utils::{GdkRGBAHelpers, GrapheneRectHelpers};
use crate::{render, DrawBehaviour, RnoteEngine};

use gtk4::{gdk, graphene, gsk, prelude::*, Snapshot};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::color;
use rnote_compose::helpers::AabbHelpers;
use rnote_compose::shapes::ShapeBehaviour;
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
    pub images: Vec<render::Image>,
    pub rendernodes: Vec<gsk::RenderNode>,
    pub(super) state: RenderCompState,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            state: RenderCompState::default(),
            images: vec![],
            rendernodes: vec![],
        }
    }
}

impl StrokeStore {
    /// Reloads the slotmap with empty render components from the keys returned from the primary map, stroke_components.
    pub fn rebuild_render_components_slotmap(&mut self) {
        self.render_components = slotmap::SecondaryMap::new();
        self.stroke_components.keys().for_each(|key| {
            self.render_components
                .insert(key, RenderComponent::default());
        });
    }

    /// Returns false if rendering is not supported
    pub fn can_render(&self, key: StrokeKey) -> bool {
        self.render_components.get(key).is_some()
    }

    pub fn render_comp_state(&self, key: StrokeKey) -> Option<RenderCompState> {
        self.render_components
            .get(key)
            .map(|render_comp| render_comp.state)
    }

    pub fn set_rendering_dirty(&mut self, key: StrokeKey) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            render_comp.state = RenderCompState::Dirty;
        }
    }

    pub fn set_rendering_dirty_for_strokes(&mut self, keys: &[StrokeKey]) {
        keys.iter().for_each(|&key| self.set_rendering_dirty(key));
    }

    pub fn gen_bounds_for_stroke_images(&self, key: StrokeKey) -> Option<Aabb> {
        if let Some(render_comp) = self.render_components.get(key) {
            if render_comp.images.is_empty() {
                return None;
            }
            return Some(
                render_comp
                    .images
                    .iter()
                    .map(|image| image.rect.bounds())
                    .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x)),
            );
        }
        None
    }

    pub fn gen_bounds_for_strokes_images(&self, keys: &[StrokeKey]) -> Option<Aabb> {
        let images_bounds = keys
            .iter()
            .filter_map(|&key| self.gen_bounds_for_stroke_images(key))
            .collect::<Vec<Aabb>>();
        if images_bounds.is_empty() {
            return None;
        }

        Some(
            images_bounds
                .into_iter()
                .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x)),
        )
    }

    pub fn regenerate_rendering_for_stroke(
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

            // extending the viewport by the factor
            let viewport_render_margins =
                viewport.extents() * render::VIEWPORT_EXTENTS_MARGIN_FACTOR;
            let viewport = viewport.extend_by(viewport_render_margins);

            match stroke.gen_images(viewport, image_scale) {
                Ok(GeneratedStrokeImages::Partial { images, viewport }) => {
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.images = images;
                            render_comp.state = RenderCompState::ForViewport(viewport);
                        }
                        Err(e) => {
                            render_comp.state = RenderCompState::Dirty;
                            log::error!(" image_to_rendernode() failed in regenerate_rendering_for_stroke(), Err: {e:?}");
                        }
                    }
                }
                Ok(GeneratedStrokeImages::Full(images)) => {
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.images = images;
                            render_comp.state = RenderCompState::Complete;
                        }
                        Err(e) => {
                            render_comp.state = RenderCompState::Dirty;
                            log::error!(" image_to_rendernode() failed in regenerate_rendering_for_stroke(), Err: {e:?}");
                        }
                    }
                }
                Err(e) => {
                    log::error!("generating images for stroke with key {key:?} failed, Err: {e:?}");
                }
            }
        }
    }

    pub fn regenerate_rendering_for_strokes(
        &mut self,
        keys: &[StrokeKey],
        viewport: Aabb,
        image_scale: f64,
    ) {
        for &key in keys {
            self.regenerate_rendering_for_stroke(key, viewport, image_scale);
        }
    }

    pub fn regenerate_rendering_for_stroke_threaded(
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

            // extending the viewport by the factor
            let viewport_render_margins =
                viewport.extents() * render::VIEWPORT_EXTENTS_MARGIN_FACTOR;
            let viewport = viewport.extend_by(viewport_render_margins);

            // indicates that a task is now started rendering the stroke
            render_comp.state = RenderCompState::BusyRenderingInTask;

            // Spawn a new thread for image rendering
            rayon::spawn(move || match stroke.gen_images(viewport, image_scale) {
                Ok(images) => {
                    tasks_tx.unbounded_send(EngineTask::UpdateStrokeWithImages {
                            key,
                            images,
                        }).unwrap_or_else(|e| {
                            log::error!("tasks_tx.send() UpdateStrokeWithImages failed in regenerate_rendering_for_stroke_threaded() for stroke with key {key:?}, with Err, {e:?}");
                        });
                }
                Err(e) => {
                    log::debug!("stroke.gen_image() failed in regenerate_rendering_for_stroke_threaded() for stroke with key {key:?}, with Err: {e:?}");
                }
            });
        }
    }

    /// Regenerates the rendering of all keys for the given viewport that need rerendering
    pub fn regenerate_rendering_in_viewport_threaded(
        &mut self,
        tasks_tx: EngineTaskSender,
        force_regenerate: bool,
        viewport: Aabb,
        image_scale: f64,
    ) {
        let keys = self.render_components.keys().collect::<Vec<StrokeKey>>();

        keys.iter().for_each(|&key| {
            if let (Some(stroke), Some(render_comp)) =
                (self.stroke_components.get(key), self.render_components.get_mut(key))
            {
                let tasks_tx = tasks_tx.clone();
                let stroke_bounds = stroke.bounds();

                // extending the viewport by the factor
                let viewport_render_margins = viewport.extents() * render::VIEWPORT_EXTENTS_MARGIN_FACTOR;
                let viewport = viewport.extend_by(viewport_render_margins);

                // skip and empty image buffer if stroke is not in viewport
                if !viewport.intersects(&stroke_bounds) {
                    render_comp.rendernodes = vec![];
                    render_comp.images = vec![];
                    render_comp.state = RenderCompState::Dirty;

                    return;
                }

                // only check if rerendering is not forced
                if !force_regenerate {
                    match render_comp.state {
                        RenderCompState::Complete | RenderCompState::BusyRenderingInTask => {
                            return;
                        }
                        RenderCompState::ForViewport(old_viewport) => {
                            // We don't skip if we pass the threshold in context to the margin, so the stroke gets rerendered in time. between 0.0 and 1.0
                            const SKIP_RERENDER_MARGIN_THRESHOLD: f64 = 0.7;

                            let diff  = (old_viewport.center().coords - viewport.center().coords).abs();
                            if diff[0] < viewport_render_margins[0] * SKIP_RERENDER_MARGIN_THRESHOLD && diff[1] < viewport_render_margins[1] * SKIP_RERENDER_MARGIN_THRESHOLD {
                                // We don't update the state, to have the old bounds on the next call
                                // so we only update the rendering after it crossed the margin threshold
                                return;
                            }
                        }
                        RenderCompState::Dirty => {}
                    }
                }

                // indicates that a task is now started rendering the stroke
                render_comp.state = RenderCompState::BusyRenderingInTask;

                let stroke = stroke.clone();

                //log::debug!("updating stroke with viewport: {:#?}", viewport);

                // Spawn a new thread for image rendering
                rayon::spawn(move || {
                    match stroke.gen_images(viewport, image_scale) {
                        Ok(images) => {
                            tasks_tx.unbounded_send(EngineTask::UpdateStrokeWithImages {
                                key,
                                images,
                            }).unwrap_or_else(|e| {
                                log::error!("tasks_tx.send() UpdateStrokeWithImages failed in regenerate_rendering_in_viewport_threaded(), with Err, {e}");
                            });
                        }
                        Err(e) => {
                            log::debug!("stroke.gen_image() failed in regenerate_rendering_in_viewport_threaded(), with Err: {e:?}");
                        }
                    }
                });
            }
        })
    }

    /// generates images and appends them to the render component for the last segments of brushstrokes. For other strokes the rendering is regenerated completely
    pub fn append_rendering_last_segments(
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
                        Ok(Some(image)) => match render::Image::images_to_rendernodes([&image]) {
                            Ok(mut rendernodes) => {
                                render_comp.rendernodes.append(&mut rendernodes);
                                render_comp.images.push(image);
                            }
                            Err(e) => {
                                render_comp.state = RenderCompState::Dirty;
                                log::error!("failed to generated rendernodes in append_rendering_last_segments(), Err: {e:?}");
                            }
                        },
                        Ok(None) => {}
                        Err(e) => {
                            render_comp.state = RenderCompState::Dirty;
                            log::error!(
                                "failed to generated image in append_rendering_last_segments(), Err: {e:?}"
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

    /// Replaces the entire current rendering with the given new images. Also updates the renderstate
    pub fn replace_rendering_with_images(&mut self, key: StrokeKey, images: GeneratedStrokeImages) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            match images {
                GeneratedStrokeImages::Partial { images, viewport } => {
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.images = images;
                            render_comp.state = RenderCompState::ForViewport(viewport);
                        }
                        Err(e) => {
                            log::error!("failed to generate rendernodes in replace_rendering_with_images(), Err {e:?}");
                            render_comp.state = RenderCompState::Dirty;
                        }
                    }
                }
                GeneratedStrokeImages::Full(images) => {
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.images = images;
                            render_comp.state = RenderCompState::Complete;
                        }
                        Err(e) => {
                            log::error!("failed to generate rendernodes in replace_rendering_with_images(), Err {e:?}");
                            render_comp.state = RenderCompState::Dirty;
                        }
                    }
                }
            }
        }
    }

    /// Not changing the render component state, that is the responsibility of the caller
    pub fn append_rendering_images(&mut self, key: StrokeKey, images: GeneratedStrokeImages) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            match images {
                GeneratedStrokeImages::Partial {
                    mut images,
                    viewport: _,
                }
                | GeneratedStrokeImages::Full(mut images) => {
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(mut rendernodes) => {
                            render_comp.rendernodes.append(&mut rendernodes);
                            render_comp.images.append(&mut images);
                        }
                        Err(e) => {
                            log::error!("failed to generate rendernodes in append_rendering_images(), Err {e:?}");
                            render_comp.state = RenderCompState::Dirty;
                        }
                    }
                }
            }
        }
    }

    /// Draws all strokes on the snapshot
    pub fn draw_strokes_to_gtk_snapshot(
        &self,
        snapshot: &Snapshot,
        doc_bounds: Aabb,
        viewport: Aabb,
    ) {
        snapshot.push_clip(&graphene::Rect::from_p2d_aabb(doc_bounds));

        self.stroke_keys_as_rendered_intersecting_bounds(viewport)
            .iter()
            .for_each(|&key| {
                if let (Some(stroke), Some(render_comp)) = (
                    self.stroke_components.get(key),
                    self.render_components.get(key),
                ) {
                    // Draws a placeholder for the given stroke bounds
                    if render_comp.rendernodes.is_empty() {
                        snapshot.append_color(
                            &gdk::RGBA::from_piet_color(color::GNOME_BRIGHTS[1].with_alpha(0.564)),
                            &graphene::Rect::from_p2d_aabb(stroke.bounds()),
                        );
                    }

                    for rendernode in render_comp.rendernodes.iter() {
                        snapshot.append_node(rendernode);
                    }
                }
            });

        snapshot.pop();
    }

    // Draws the given strokes on a piet render context. Note that every given stroke get drawn, even the trashed ones.
    pub fn draw_stroke_keys_to_piet(
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

    /// Draws all strokes on the piet context. In immediate mode, without the image cache.
    pub fn draw_strokes_immediate_w_piet(
        &self,
        piet_cx: &mut impl piet::RenderContext,
        _doc_bounds: Aabb,
        viewport: Aabb,
        image_scale: f64,
    ) -> anyhow::Result<()> {
        self.stroke_keys_as_rendered_intersecting_bounds(viewport)
            .into_iter()
            .for_each(|key| {
                if let Some(stroke) = self.stroke_components.get(key) {
                    if let Err(e) = || -> anyhow::Result<()> {
                        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
                        stroke
                            .draw(piet_cx, image_scale)
                            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
                        piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
                        Ok(())
                    }() {
                        log::error!(
                            "drawing stroke in draw_strokes_immediate_w_piet() failed with Err: {e:?}"
                        );
                    }
                }
            });

        Ok(())
    }

    /// Draws the selection on the piet context. In immediate mode, without the image cache.
    pub fn draw_selection_immediate_w_piet(
        &self,
        piet_cx: &mut impl piet::RenderContext,
        _doc_bounds: Aabb,
        viewport: Aabb,
        image_scale: f64,
    ) -> anyhow::Result<()> {
        self.selection_keys_as_rendered_intersecting_bounds(viewport)
            .into_iter()
            .for_each(|key| {
                if let Some(stroke) = self.stroke_components.get(key) {
                    if let Err(e) = || -> anyhow::Result<()> {
                        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
                        stroke
                            .draw(piet_cx, image_scale)
                            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
                        piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
                        Ok(())
                    }() {
                        log::error!(
                            "drawing stroke in draw_selection_immediate_w_piet() failed with Err: {e:?}"
                        );
                    }
                }
            });

        Ok(())
    }

    /// Draws bounds, positions, etc. for all strokes for visual debugging
    pub fn draw_debug(
        &self,
        snapshot: &Snapshot,
        engine: &RnoteEngine,
        _surface_bounds: Aabb,
    ) -> anyhow::Result<()> {
        let border_widths = 1.0 / engine.camera.total_zoom();

        self.keys_sorted_chrono().into_iter().for_each(|key| {
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
                            visual_debug::draw_fill(
                                stroke.bounds(),
                                visual_debug::COLOR_STROKE_RENDERING_DIRTY,
                                snapshot,
                            );
                        }
                        RenderCompState::BusyRenderingInTask => {
                            visual_debug::draw_fill(
                                stroke.bounds(),
                                visual_debug::COLOR_STROKE_RENDERING_BUSY,
                                snapshot,
                            );
                        }
                        _ => {}
                    }
                    render_comp.images.iter().for_each(|image| {
                        visual_debug::draw_bounds(
                            // a little tightened not to overlap with other bounds
                            image.rect.bounds().tightened(2.0 * border_widths),
                            visual_debug::COLOR_IMAGE_BOUNDS,
                            snapshot,
                            border_widths,
                        )
                    });
                }

                for &hitbox_elem in stroke.hitboxes().iter() {
                    visual_debug::draw_bounds(
                        hitbox_elem,
                        visual_debug::COLOR_STROKE_HITBOX,
                        snapshot,
                        border_widths,
                    );
                }

                visual_debug::draw_bounds(
                    stroke.bounds(),
                    visual_debug::COLOR_STROKE_BOUNDS,
                    snapshot,
                    border_widths,
                );

                match stroke.as_ref() {
                    // Draw positions for brushstrokes
                    Stroke::BrushStroke(brushstroke) => {
                        for element in brushstroke.path.clone().into_elements().iter() {
                            visual_debug::draw_pos(
                                element.pos,
                                visual_debug::COLOR_POS,
                                snapshot,
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
        });

        Ok(())
    }
}
