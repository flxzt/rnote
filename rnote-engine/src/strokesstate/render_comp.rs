use super::StateTask;
use super::{Stroke, StrokeKey, StrokesState};
use crate::strokes::StrokeBehaviour;
use crate::utils::GrapheneRectHelpers;
use crate::{render, DrawBehaviour};
use rnote_compose::helpers::AABBHelpers;

use gtk4::{gsk, Snapshot, graphene};
use p2d::bounding_volume::{BoundingVolume, AABB};
use rnote_compose::shapes::ShapeBehaviour;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "render_component")]
pub struct RenderComponent {
    #[serde(rename = "render")]
    pub render: bool,
    #[serde(skip)]
    pub regenerate_flag: bool,
    #[serde(skip)]
    pub images: Vec<render::Image>,
    #[serde(skip)]
    pub rendernodes: Vec<gsk::RenderNode>,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            render: true,
            regenerate_flag: true,
            images: vec![],
            rendernodes: vec![],
        }
    }
}

impl StrokesState {
    /// Returns false if rendering is not supported
    pub fn can_render(&self, key: StrokeKey) -> bool {
        self.render_components.get(key).is_some()
    }

    /// Wether rendering is enabled. Returns None if rendering is not supported
    pub fn does_render(&self, key: StrokeKey) -> Option<bool> {
        if let Some(render_comp) = self.render_components.get(key) {
            Some(render_comp.render)
        } else {
            log::debug!(
                "get render_comp failed in does_render() of stroke for key {:?}, invalid key used or stroke does not support rendering",
                key
            );
            None
        }
    }

    pub fn set_render(&mut self, key: StrokeKey, render: bool) {
        if let Some(render_component) = self.render_components.get_mut(key) {
            render_component.render = render;
        } else {
            log::debug!(
                "get render_comp failed in set_render() of stroke for key {:?}, invalid key used or stroke does not support rendering",
                key
            );
        }
    }

    pub fn regenerate_flag(&self, key: StrokeKey) -> Option<bool> {
        if let Some(render_comp) = self.render_components.get(key) {
            Some(render_comp.regenerate_flag)
        } else {
            log::debug!(
                "get render_comp failed in regenerate_flag() of stroke for key {:?}, invalid key used or stroke does not support rendering",
                key
            );
            None
        }
    }

    pub fn set_regenerate_flag(&mut self, key: StrokeKey, regenerate_flag: bool) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            render_comp.regenerate_flag = regenerate_flag;
        } else {
            log::debug!(
                "get render_comp failed in set_regenerate_flag() of stroke for key {:?}, invalid key used or stroke does not support rendering",
                key
            );
        }
    }

    pub fn reset_regenerate_flag_all_strokes(&mut self) {
        self.render_components
            .iter_mut()
            .for_each(|(_key, render_comp)| {
                render_comp.regenerate_flag = true;
            });
    }

    pub fn regenerate_rendering_for_stroke(&mut self, key: StrokeKey, image_scale: f64) {
        if let (Some(stroke), Some(render_comp)) =
            (self.strokes.get(key), self.render_components.get_mut(key))
        {
            match stroke.gen_images(image_scale) {
                Ok(images) => {
                    match render::Image::images_to_rendernodes(&images) {
                        Ok(rendernodes) => {
                            render_comp.rendernodes = rendernodes;
                            render_comp.regenerate_flag = false;
                            render_comp.images = images;
                        }
                        Err(e) => log::error!("image_to_rendernode() failed in regenerate_rendering_for_stroke() with Err {}", e),
                    }
                }
                Err(e) => {
                    log::debug!(
                        "gen_image() failed in regenerate_rendering_for_stroke() for stroke with key: {:?}, {}",
                        key,
                        e
                    )
                }
            }
        }
    }

    pub fn regenerate_rendering_for_strokes(&mut self, keys: &[StrokeKey], image_scale: f64) {
        keys.iter().for_each(|&key| {
            self.regenerate_rendering_for_stroke(key, image_scale);
        })
    }

    pub fn regenerate_rendering_for_stroke_threaded(&mut self, key: StrokeKey, image_scale: f64) {
        let tasks_tx = self.tasks_tx.clone();

        if let (Some(render_comp), Some(stroke)) =
            (self.render_components.get_mut(key), self.strokes.get(key))
        {
            let stroke = stroke.clone();

            render_comp.regenerate_flag = false;

            // Spawn a new thread for image rendering
            self.threadpool.spawn(move || {
                match stroke.gen_images(image_scale) {
                    Ok(images) => {
                        tasks_tx.unbounded_send(StateTask::UpdateStrokeWithImages {
                            key,
                            images,
                        }).unwrap_or_else(|e| {
                            log::error!("tasks_tx.send() UpdateStrokeWithImages failed in regenerate_rendering_for_stroke_threaded() for stroke with key {:?}, with Err, {}",key, e);
                        });
                    }
                    Err(e) => {
                        log::debug!("stroke.gen_image() failed in regenerate_rendering_for_stroke_threaded() for stroke with key {:?}, with Err {}", key, e);
                    }
                }
            });
        } else {
            log::debug!("getting stroke comp, tasks_tx or render_comp returned None in regenerate_rendering_for_stroke_threaded() for stroke {:?}", key);
        }
    }

    pub fn regenerate_rendering_for_strokes_threaded(
        &mut self,
        keys: &[StrokeKey],
        image_scale: f64,
    ) {
        keys.iter().for_each(|&key| {
            self.regenerate_rendering_for_stroke_threaded(key, image_scale);
        })
    }

    pub fn regenerate_rendering_in_viewport(
        &mut self,
        viewport: Option<AABB>,
        force_regenerate: bool,
        image_scale: f64,
    ) {
        let keys = self.render_components.keys().collect::<Vec<StrokeKey>>();

        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);

            if let (Some(stroke), Some(render_comp)) =
                (self.strokes.get(key), self.render_components.get_mut(key))
            {
                // skip if stroke is not in viewport or does not need regeneration
                if let Some(viewport) = viewport {
                    // Loosening the bounds to avoid strokes popping up
                    if !viewport.expand_by(viewport.extents()).intersects(&stroke.bounds()) {
                        render_comp.rendernodes = vec![];
                        render_comp.images = vec![];
                        render_comp.regenerate_flag = true;

                        return;
                    }
                }
                if !force_regenerate && !render_comp.regenerate_flag {
                    return;
                }

                match stroke.gen_images(image_scale) {
                    Ok(images) => {
                        match render::Image::images_to_rendernodes(&images) {
                            Ok(rendernodes) => {
                                render_comp.rendernodes = rendernodes;
                                render_comp.images = images;
                                render_comp.regenerate_flag = false;
                            }
                            Err(e) => log::error!("stroke.gen_images() failed in regenerate_stroke_current_view() with Err {}", e),
                        }
                    }
                    Err(e) => {
                        log::debug!(
                            "gen_image() failed in regenerate_rendering_current_view() for stroke with key: {:?}, with Err {}",
                            key,
                            e
                        )
                    }
                }
            } else {
                log::debug!(
                    "get stroke, render_comp returned None in regenerate_rendering_current_view() for stroke with key {:?}",
                    key
                );
            }
        })
    }

    pub fn regenerate_rendering_in_viewport_threaded(
        &mut self,
        force_regenerate: bool,
        viewport: Option<AABB>,
        image_scale: f64,
    ) {
        let keys = self.render_components.keys().collect::<Vec<StrokeKey>>();

        keys.iter().for_each(|&key| {
            if let (Some(stroke), Some(render_comp)) =
                (self.strokes.get(key), self.render_components.get_mut(key))
            {
                // skip if stroke is not in viewport or does not need regeneration
                if let Some(viewport) = viewport {
                    // Loosening the bounds to prerender around the viewport
                    if !viewport.expand_by(viewport.extents()).intersects(&stroke.bounds()) {
                        render_comp.rendernodes = vec![];
                        render_comp.images = vec![];
                        render_comp.regenerate_flag = true;

                        return;
                    }
                }
                if !force_regenerate && !render_comp.regenerate_flag {
                    return;
                }

                self.update_geometry_for_stroke(key);
                self.regenerate_rendering_for_stroke_threaded(key, image_scale)
            } else {
                log::debug!(
                    "get stroke, render_comp returned None in regenerate_rendering_current_view_threaded() for stroke with key {:?}",
                    key
                );
            }
        })
    }

    /// generates images and appends them to the render component for the last segments of brushstrokes. For other strokes all rendering is regenerated
    pub fn append_rendering_last_segments(
        &mut self,
        key: StrokeKey,
        no_last_segments: usize,
        image_scale: f64,
    ) {
        if let (Some(stroke), Some(render_comp)) =
            (self.strokes.get(key), self.render_components.get_mut(key))
        {
            match stroke {
                Stroke::BrushStroke(brushstroke) => {
                    match brushstroke.gen_images_for_last_segments(no_last_segments, image_scale) {
                        Ok(mut images) => {
                                match render::Image::images_to_rendernodes(
                                        &images,
                                    ) {
                                        Ok(mut rendernode) => {
                                            render_comp.rendernodes.append(&mut rendernode);
                                            render_comp.images.append(&mut images);
                                            render_comp.regenerate_flag = false;
                                        }
                                        Err(e) => log::error!("append_images_to_rendernode() failed in append_rendering_last_segments() with Err {}", e),
                                    }
                        }
                        Err(e) => {
                            log::error!("gen_images_for_last_segments() in append_rendering_last_segments() failed with Err {}", e);
                        }
                    }
                }
                // regenerate everything for strokes that don't support generating svgs for the last added elements
                Stroke::ShapeStroke(_) | Stroke::VectorImage(_) | Stroke::BitmapImage(_) => {
                    match stroke.gen_images(image_scale) {
                        Ok(images) => {
                            match render::Image::images_to_rendernodes(&images) {
                                Ok(rendernodes) => {
                                    render_comp.rendernodes = rendernodes;
                                    render_comp.regenerate_flag = false;
                                    render_comp.images = images;
                                }
                                Err(e) => log::error!("image_to_rendernode() failed in regenerate_rendering_for_stroke() with Err {}", e),
                            }
                        }
                        Err(e) => {
                            log::debug!(
                                "stroke.gen_image() failed in regenerate_rendering_newest_elem() for stroke with key: {:?}, with Err {}",
                                key,
                                e
                            )
                        }
                    }
                }
            }
        } else {
            log::debug!(
                "get stroke, render_comp returned None for stroke with key {:?}",
                key
            );
        }
    }

    /// generates images in another thread and appends them to the render component for the last segments of brushstrokes. For other strokes all rendering is regenerated
    pub fn append_rendering_last_segments_threaded(
        &mut self,
        key: StrokeKey,
        no_last_segments: usize,
        image_scale: f64,
    ) {
        let tasks_tx = self.tasks_tx.clone();

        if let (Some(stroke), Some(render_comp)) =
            (self.strokes.get(key), self.render_components.get_mut(key))
        {
            let stroke = stroke.clone();

            render_comp.regenerate_flag = false;

            self.threadpool.spawn(move || {
                match stroke {
                    Stroke::BrushStroke(brushstroke) => {
                        match brushstroke.gen_images_for_last_segments(no_last_segments, image_scale) {
                            Ok(images) => {
                                tasks_tx.unbounded_send(StateTask::AppendImagesToStroke {
                                    key,
                                    images,
                                }).unwrap_or_else(|e| {
                                    log::error!("sending AppendImagesToStroke as task for markerstroke failed in regenerate_rendering_new_elem() for stroke with key {:?}, with Err, {}",key, e);
                                });
                            }
                            Err(e) => {
                                log::error!("gen_images_for_last_segments() in append_rendering_last_segments() failed with Err {}", e);
                            }
                        }
                    }
                    // regenerate everything for strokes that don't support generating svgs for the last added elements
                    Stroke::ShapeStroke(_)
                    | Stroke::VectorImage(_)
                    | Stroke::BitmapImage(_) => {
                        match stroke.gen_images(image_scale) {
                            Ok(images) => {
                                tasks_tx.unbounded_send(StateTask::UpdateStrokeWithImages {
                                    key,
                                    images,
                                }).unwrap_or_else(|e| {
                                    log::error!("sending task UpdateStrokeWithImages failed in regenerate_rendering_newest_elem() for stroke with key {:?}, with Err {}", key, e);
                                });
                            }
                            Err(e) => {
                                log::debug!(
                                    "stroke.gen_image() failed in regenerate_rendering_newest_elem() for stroke with key: {:?}, with Err {}",
                                    key,
                                    e
                                )
                            }
                        }
                    }
                }
            });
        } else {
            log::debug!(
                "get stroke, render_comp, tasks_tx returned None for stroke with key {:?}",
                key
            );
        }
    }

    pub fn replace_rendering_with_images(&mut self, key: StrokeKey, images: Vec<render::Image>) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            match render::Image::images_to_rendernodes(&images) {
                Ok(rendernodes) => {
                    render_comp.rendernodes = rendernodes;
                    render_comp.images = images;
                    render_comp.regenerate_flag = false;
                }
                Err(e) => log::error!(
                    "image_to_rendernode() failed in regenerate_rendering_with_images() with Err {}",
                    e
                ),
            }
        } else {
            log::debug!(
                    "get render_comp returned None in regenerate_rendering_with_images() for stroke with key {:?}",
                    key
                );
        }
    }

    pub fn append_images_to_rendering(&mut self, key: StrokeKey, mut images: Vec<render::Image>) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            match render::Image::images_to_rendernodes(&images) {
                Ok(mut rendernodes) => {
                    render_comp.rendernodes.append(&mut rendernodes);
                    render_comp.images.append(&mut images);
                    render_comp.regenerate_flag = false;
                }
                Err(e) => log::error!(
                    "append_images_to_rendernode() failed in append_images_to_rendering() with Err {}",
                    e
                ),
            }
        }
    }

    /// Draws the strokes without the selection
    pub fn draw_strokes(&self, snapshot: &Snapshot, sheet_bounds: AABB, viewport: Option<AABB>) {
        snapshot.push_clip(&graphene::Rect::from_aabb(sheet_bounds));

        self.keys_as_rendered().iter().for_each(|&key| {
            if let (Some(stroke), Some(render_comp)) =
                (self.strokes.get(key), self.render_components.get(key))
            {
                // skip if stroke is not in viewport
                if let Some(viewport) = viewport {
                    if !viewport.intersects(&stroke.bounds()) {
                        return;
                    }
                }

                for rendernode in render_comp.rendernodes.iter() {
                    snapshot.append_node(rendernode);
                }
            }
        });

        snapshot.pop();
    }

    /// Draws the selection
    pub fn draw_selection(&self, snapshot: &Snapshot, _sheet_bounds: AABB, viewport: Option<AABB>) {
        self.selection_keys_as_rendered().iter().for_each(|&key| {
            let render_comp = self.render_components.get(key).unwrap();

            if let (Some(selection_comp), Some(stroke)) =
                (self.selection_components.get(key), self.strokes.get(key))
            {
                // skip if stroke is not in viewport
                if let Some(viewport) = viewport {
                    if !viewport.intersects(&stroke.bounds()) {
                        return;
                    }
                }

                if selection_comp.selected {
                    for rendernode in render_comp.rendernodes.iter() {
                        snapshot.append_node(rendernode);
                    }
                }
            }
        });
    }


    pub fn draw_strokes_immediate_w_piet(
        &self,
        piet_cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        viewport: Option<AABB>,
        image_scale: f64,
    ) -> Result<(), anyhow::Error> {
        self.keys_as_rendered()
            .into_iter()
            .for_each(|key| {
                if let Some(stroke) = self.strokes.get(key) {
                    if let Err(e) = || -> Result<(), anyhow::Error> {
                        // skip if stroke is not in viewport
                        if let Some(viewport) = viewport {
                            if !viewport.intersects(&stroke.bounds()) {
                                return Ok(());
                            }
                        }

                        piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
                        stroke
                            .draw(piet_cx, image_scale)
                            .map_err(|e| anyhow::anyhow!("{}", e))?;
                        piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
                        Ok(())
                    }() {
                        log::error!(
                            "drawing stroke in draw_strokes_immediate_w_piet() failed with Err {}",
                            e
                        );
                    }
                }
            });

        Ok(())
    }

    pub fn draw_selection_immediate_w_piet(
        &self,
        piet_cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        viewport: Option<AABB>,
        image_scale: f64,
    ) -> Result<(), anyhow::Error> {
        self.selection_keys_as_rendered()
            .into_iter()
            .for_each(|key| {
                if let Some(stroke) = self.strokes.get(key) {
                    if let Err(e) = || -> Result<(), anyhow::Error> {
                        // skip if stroke is not in viewport
                        if let Some(viewport) = viewport {
                            if !viewport.intersects(&stroke.bounds()) {
                                return Ok(());
                            }
                        }

                        piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
                        stroke
                            .draw(piet_cx, image_scale)
                            .map_err(|e| anyhow::anyhow!("{}", e))?;
                        piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
                        Ok(())
                    }() {
                        log::error!(
                            "drawing stroke in draw_strokes_immediate_w_piet() failed with Err {}",
                            e
                        );
                    }
                }
            });

        Ok(())
    }
}
