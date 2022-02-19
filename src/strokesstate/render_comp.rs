use std::sync::{Arc, RwLock};

use super::StateTask;
use super::{StrokeKey, StrokeStyle, StrokesState};
use crate::compose::color::Color;
use crate::drawbehaviour::DrawBehaviour;
use crate::render::{self, Renderer};
use crate::strokes::markerstroke::MarkerStroke;
use crate::ui::canvas;

use gtk4::{graphene, gsk, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
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
    pub rendernode: Option<gsk::RenderNode>,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            render: true,
            regenerate_flag: true,
            images: vec![],
            rendernode: None,
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

    pub fn regenerate_rendering_for_stroke(
        &mut self,
        key: StrokeKey,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        if let (Some(stroke), Some(render_comp)) =
            (self.strokes.get(key), self.render_components.get_mut(key))
        {
            match stroke.gen_images(zoom, renderer) {
                Ok(images) => {
                    match render::images_to_rendernode(&images, zoom) {
                        Ok(Some(rendernode)) => {
                            render_comp.rendernode = Some(rendernode);
                            render_comp.regenerate_flag = false;
                            render_comp.images = images;
                        }
                        Ok(None) => log::error!("image_to_rendernode() failed in regenerate_rendering_for_stroke(), returned None"),
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

    pub fn regenerate_rendering_for_strokes(
        &mut self,
        keys: &[StrokeKey],
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        keys.iter().for_each(|&key| {
            self.regenerate_rendering_for_stroke(key, Arc::clone(&renderer), zoom);
        })
    }

    pub fn regenerate_rendering_for_stroke_threaded(
        &mut self,
        key: StrokeKey,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        if let (Some(render_comp), Some(tasks_tx), Some(stroke)) = (
            self.render_components.get_mut(key),
            self.tasks_tx.clone(),
            self.strokes.get(key),
        ) {
            let stroke = stroke.clone();

            render_comp.regenerate_flag = false;

            // Spawn a new thread for image rendering
            self.threadpool.spawn(move || {
                match stroke.gen_images(zoom, renderer) {
                    Ok(images) => {
                        // Send each image individually, to not block too long
                        let mut images_iter = images.into_iter();
                        if let Some(first_image) = images_iter.next() {
                            // First image overrides any previous ( behaviour of UpdateStrokeWithImages )
                            tasks_tx.send(StateTask::UpdateStrokeWithImages {
                                key,
                                images: vec![first_image],
                            }).unwrap_or_else(|e| {
                                log::error!("tasks_tx.send() UpdateStrokeWithImages failed in regenerate_rendering_for_stroke_threaded() for stroke with key {:?}, with Err, {}",key, e);
                            });
                            while let Some(next_image) = images_iter.next() {
                                // Append the next images
                                tasks_tx.send(StateTask::AppendImagesToStroke {
                                    key,
                                    images: vec![next_image],
                                }).unwrap_or_else(|e| {
                                    log::error!("tasks_tx.send() AppendImagesToStroke failed in regenerate_rendering_for_stroke_threaded() for stroke with key {:?}, with Err, {}",key, e);
                                });
                            }
                        }
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
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        keys.iter().for_each(|&key| {
            self.regenerate_rendering_for_stroke_threaded(key, Arc::clone(&renderer), zoom);
        })
    }

    pub fn regenerate_rendering_current_view(
        &mut self,
        viewport: Option<AABB>,
        force_regenerate: bool,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        let keys = self.render_components.keys().collect::<Vec<StrokeKey>>();

        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);

            if let (Some(stroke), Some(render_comp)) =
                (self.strokes.get(key), self.render_components.get_mut(key))
            {
                // skip if stroke is not in viewport or does not need regeneration
                if let Some(viewport) = viewport {
                    if !viewport.intersects(&stroke.bounds()) {
                        return;
                    }
                }
                if !force_regenerate && !render_comp.regenerate_flag {
                    return;
                }

                match stroke.gen_images(zoom, Arc::clone(&renderer)) {
                    Ok(images) => {
                        match render::images_to_rendernode(&images, zoom) {
                            Ok(Some(rendernode)) => {
                                render_comp.rendernode = Some(rendernode);
                                render_comp.regenerate_flag = false;
                                render_comp.images = images;
                            }
                            Ok(None) => {}
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

    pub fn regenerate_rendering_current_view_threaded(
        &mut self,
        viewport: Option<AABB>,
        force_regenerate: bool,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        let keys = self.render_components.keys().collect::<Vec<StrokeKey>>();

        keys.iter().for_each(|&key| {
            if let (Some(stroke), Some(render_comp)) =
                (self.strokes.get(key), self.render_components.get_mut(key))
            {
                // skip if stroke is not in viewport or does not need regeneration
                if let Some(viewport) = viewport {
                    if !viewport.intersects(&stroke.bounds()) {
                        return;
                    }
                }
                if !force_regenerate && !render_comp.regenerate_flag {
                    return;
                }

                self.update_geometry_for_stroke(key);
                self.regenerate_rendering_for_stroke_threaded(key, Arc::clone(&renderer), zoom)
            } else {
                log::debug!(
                    "get stroke, render_comp returned None in regenerate_rendering_current_view_threaded() for stroke with key {:?}",
                    key
                );
            }
        })
    }

    /// Append the last elements to the render_comp of the stroke. The rendering for strokes that don't support generating rendering for only the last elements are regenerated completely
    pub fn append_rendering_new_elem(
        &mut self,
        key: StrokeKey,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        if let (Some(stroke), Some(render_comp)) =
            (self.strokes.get(key), self.render_components.get_mut(key))
        {
            match stroke {
                StrokeStyle::BrushStroke(brushstroke) => {
                    let elems_len = brushstroke.elements.len();

                    let elements = if elems_len >= 4 {
                        Some((
                            brushstroke.elements.get(elems_len - 4).unwrap(),
                            brushstroke.elements.get(elems_len - 3).unwrap(),
                            brushstroke.elements.get(elems_len - 2).unwrap(),
                            brushstroke.elements.get(elems_len - 1).unwrap(),
                        ))
                    } else {
                        None
                    };

                    if let Some(elements) = elements {
                        let offset = na::vector![0.0, 0.0];
                        if let Ok(Some(last_elems_svg)) =
                            brushstroke.gen_svg_for_elems(elements, offset, true)
                        {
                            let bounds = last_elems_svg.bounds;
                            match renderer.read().unwrap().gen_images(
                                zoom,
                                vec![last_elems_svg],
                                bounds,
                            ) {
                                Ok(mut images) => {
                                    match render::append_images_to_rendernode(
                                        render_comp.rendernode.as_ref(),
                                        &images,
                                        zoom,
                                    ) {
                                        Ok(rendernode) => {
                                            render_comp.rendernode = rendernode;
                                            render_comp.images.append(&mut images);
                                            render_comp.regenerate_flag = false;
                                        }
                                        Err(e) => log::error!("append_images_to_rendernode() failed in append_rendering_new_elem() with Err {}", e),
                                    }
                                }
                                Err(e) => {
                                    log::warn!("renderer.gen_image() failed in regenerate_image_new_elem() for stroke with key {:?}, with Err {}", key, e);
                                }
                            }
                        }
                    }
                }
                StrokeStyle::MarkerStroke(markerstroke) => {
                    let elems_len = markerstroke.elements.len();

                    let elements = if elems_len >= 4 {
                        Some((
                            markerstroke.elements.get(elems_len - 4).unwrap(),
                            markerstroke.elements.get(elems_len - 3).unwrap(),
                            markerstroke.elements.get(elems_len - 2).unwrap(),
                            markerstroke.elements.get(elems_len - 1).unwrap(),
                        ))
                    } else {
                        None
                    };

                    if let Some(elements) = elements {
                        let offset = na::vector![0.0, 0.0];
                        if let Some(last_elems_svg) = MarkerStroke::gen_svg_elem(
                            &markerstroke.options,
                            elements,
                            offset,
                            true,
                        ) {
                            let bounds = last_elems_svg.bounds;
                            match renderer.read().unwrap().gen_images(
                                zoom,
                                vec![last_elems_svg],
                                bounds,
                            ) {
                                Ok(mut images) => {
                                    match render::append_images_to_rendernode(
                                        render_comp.rendernode.as_ref(),
                                        &images,
                                        zoom,
                                    ) {
                                        Ok(rendernode) => {
                                            render_comp.rendernode = rendernode;
                                            render_comp.images.append(&mut images);
                                            render_comp.regenerate_flag = false;
                                        }
                                        Err(e) => log::error!("append_images_to_rendernode() failed in append_rendering_new_elem() with Err {}", e),
                                    }
                                }
                                Err(e) => {
                                    log::warn!("renderer.gen_image() failed in regenerate_image_new_elem() with Err {}", e);
                                }
                            }
                        }
                    }
                }
                // regenerate everything for strokes that don't support generating svgs for the last added elements
                StrokeStyle::ShapeStroke(_)
                | StrokeStyle::VectorImage(_)
                | StrokeStyle::BitmapImage(_) => {
                    match stroke.gen_images(zoom, renderer) {
                        Ok(images) => {
                            match render::images_to_rendernode(&images, zoom) {
                                Ok(Some(rendernode)) => {
                                    render_comp.rendernode = Some(rendernode);
                                    render_comp.regenerate_flag = false;
                                    render_comp.images = images;
                                }
                                Ok(None) => log::error!("image_to_rendernode() failed in regenerate_rendering_for_stroke(), returned None"),
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

    /// Append the last elements to the render_comp of the stroke threaded. The rendering for strokes that don't support generating rendering for only the last elements are regenerated completely
    pub fn append_rendering_new_elem_threaded(
        &mut self,
        key: StrokeKey,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
    ) {
        if let (Some(stroke), Some(render_comp), Some(tasks_tx)) = (
            self.strokes.get(key),
            self.render_components.get_mut(key),
            self.tasks_tx.clone(),
        ) {
            let stroke = stroke.clone();

            render_comp.regenerate_flag = true;

            self.threadpool.spawn(move || {
                match stroke {
                    StrokeStyle::MarkerStroke(markerstroke) => {
                        let elems_len = markerstroke.elements.len();

                        let elements = if elems_len >= 4 {
                            Some((
                                markerstroke.elements.get(elems_len - 4).unwrap(),
                                markerstroke.elements.get(elems_len - 3).unwrap(),
                                markerstroke.elements.get(elems_len - 2).unwrap(),
                                markerstroke.elements.get(elems_len - 1).unwrap(),
                            ))
                        } else {
                            None
                        };

                        if let Some(elements) = elements {
                            let offset = na::vector![0.0, 0.0];
                            if let Some(last_elems_svg) =
                                MarkerStroke::gen_svg_elem(&markerstroke.options, elements, offset, true)
                            {
                            let bounds = last_elems_svg.bounds;
                                match renderer.read().unwrap().gen_images(
                                    zoom,
                                    vec![last_elems_svg],
                                    bounds,
                                ) {
                                    Ok(images) => {
                                        tasks_tx.send(StateTask::AppendImagesToStroke {
                                            key,
                                            images,
                                        }).unwrap_or_else(|e| {
                                            log::error!("sending AppendImagesToStroke as task for markerstroke failed in regenerate_rendering_new_elem() for stroke with key {:?}, with Err {}", key, e);
                                        });
                                    }
                                    Err(e) => {
                                        log::warn!("renderer.gen_image() failed in regenerate_image_new_elem() for stroke with key {:?}, with Err {}",key, e);
                                    }
                                }
                            }
                        }
                    }
                    StrokeStyle::BrushStroke(brushstroke) => {
                        let elems_len = brushstroke.elements.len();

                        let elements = if elems_len >= 4 {
                            Some((
                                brushstroke.elements.get(elems_len - 4).unwrap(),
                                brushstroke.elements.get(elems_len - 3).unwrap(),
                                brushstroke.elements.get(elems_len - 2).unwrap(),
                                brushstroke.elements.get(elems_len - 1).unwrap(),
                            ))
                        } else {
                            None
                        };

                        if let Some(elements) = elements {
                            let offset = na::vector![0.0, 0.0];
                            if let Ok(Some(last_elems_svg)) =
                                brushstroke.gen_svg_for_elems(elements, offset, true)
                            {
                                let bounds = last_elems_svg.bounds;
                                match renderer.read().unwrap().gen_images(
                                    zoom,
                                    vec![last_elems_svg],
                                    bounds,
                                ) {
                                    Ok(images) => {
                                        tasks_tx.send(StateTask::AppendImagesToStroke {
                                            key,
                                            images,
                                        }).unwrap_or_else(|e| {
                                            log::error!("sending AppendImagesToStroke as task for markerstroke failed in regenerate_rendering_new_elem() for stroke with key {:?}, with Err, {}",key, e);
                                        });
                                    }
                                    Err(e) => {
                                        log::warn!("renderer.gen_image() failed in regenerate_image_new_elem() for stroke with key {:?} with Err {}", key, e);
                                    }
                                }
                            }
                        }
                    }
                    // regenerate everything for strokes that don't support generating svgs for the last added elements
                    StrokeStyle::ShapeStroke(_)
                    | StrokeStyle::VectorImage(_)
                    | StrokeStyle::BitmapImage(_) => {
                        match stroke.gen_images(zoom, renderer) {
                            Ok(images) => {
                                tasks_tx.send(StateTask::UpdateStrokeWithImages {
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

    pub fn regenerate_rendering_with_images(
        &mut self,
        key: StrokeKey,
        images: Vec<render::Image>,
        zoom: f64,
    ) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            match render::images_to_rendernode(&images, zoom) {
                Ok(Some(rendernode)) => {
                    render_comp.rendernode = Some(rendernode);
                    render_comp.regenerate_flag = false;
                    render_comp.images = images;
                }
                Ok(None) => {}
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

    pub fn append_images_to_rendering(
        &mut self,
        key: StrokeKey,
        mut images: Vec<render::Image>,
        zoom: f64,
    ) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            match render::append_images_to_rendernode(render_comp.rendernode.as_ref(), &images, zoom) {
                Ok(rendernode) => {
                    render_comp.rendernode = rendernode;
                    render_comp.regenerate_flag = false;
                    render_comp.images.append(&mut images);
                }
                Err(e) => log::error!(
                    "append_images_to_rendernode() failed in append_images_to_rendering() with Err {}",
                    e
                ),
            }
        }
    }

    /// Updates the cached rendernodes to the current zoom. Used to display the scaled (pixelated) images until new ones are generated with one of the regenerate_*_threaded funcs
    pub fn update_rendernodes_current_zoom(&mut self, zoom: f64) {
        self.render_components
            .iter_mut()
            .for_each(|(_key, render_comp)| {
                match render::images_to_rendernode(&render_comp.images, zoom) {
                    Ok(Some(rendernode)) => {
                        render_comp.rendernode = Some(rendernode);
                    }
                    Ok(None) => {}
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in update_rendernodes_current_zoom() with Err {}",
                        e
                    ),
                }
            });
    }

    /// Draws the strokes without the selection
    pub fn draw_strokes(&self, snapshot: &Snapshot, viewport: Option<AABB>) {
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

                if let Some(rendernode) = render_comp.rendernode.as_ref() {
                    snapshot.append_node(rendernode);
                }
            }
        });
    }

    /// Draws the selection
    pub fn draw_selection(&self, zoom: f64, snapshot: &Snapshot) {
        fn draw_selected_bounds(bounds: AABB, zoom: f64, snapshot: &Snapshot) {
            let bounds = graphene::Rect::new(
                bounds.mins[0] as f32,
                bounds.mins[1] as f32,
                (bounds.extents()[0]) as f32,
                (bounds.extents()[1]) as f32,
            )
            .scale(zoom as f32, zoom as f32);
            let border_color = Color {
                r: 0.0,
                g: 0.2,
                b: 0.8,
                a: 0.2,
            };
            let border_width = 2.0;

            snapshot.append_border(
                &gsk::RoundedRect::new(
                    graphene::Rect::new(bounds.x(), bounds.y(), bounds.width(), bounds.height()),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                ),
                &[border_width, border_width, border_width, border_width],
                &[
                    border_color.to_gdk(),
                    border_color.to_gdk(),
                    border_color.to_gdk(),
                    border_color.to_gdk(),
                ],
            );
        }

        self.selection_keys_as_rendered()
            .iter()
            .for_each(|&key| {
                let render_comp = self.render_components.get(key).unwrap();

                if let (Some(selection_comp), Some(stroke)) =
                    (self.selection_components.get(key), self.strokes.get(key))
                {
                    if selection_comp.selected {
                        if let Some(rendernode) = render_comp.rendernode.as_ref() {
                            snapshot.append_node(rendernode);
                        }

                        draw_selected_bounds(stroke.bounds(), zoom, snapshot);
                    }
                }
            });
    }

    pub fn draw_debug(&self, zoom: f64, snapshot: &Snapshot) {
        self.keys_sorted_chrono().iter().for_each(|&key| {
            let stroke = if let Some(stroke) = self.strokes.get(key) {
                stroke
            } else {
                return;
            };

            // Push blur and opacity for strokes which are normally hidden
            if let Some(render_comp) = self.render_components.get(key) {
                if let Some(trash_comp) = self.trash_components.get(key) {
                    if render_comp.render && trash_comp.trashed {
                        snapshot.push_blur(3.0);
                        snapshot.push_opacity(0.2);
                    }
                }
                if render_comp.regenerate_flag {
                    canvas::debug::draw_fill(
                        stroke.bounds(),
                        canvas::debug::COLOR_STROKE_REGENERATE_FLAG,
                        zoom,
                        snapshot,
                    );
                }
            }
            match stroke {
                StrokeStyle::MarkerStroke(markerstroke) => {
                    for element in markerstroke.elements.iter() {
                        canvas::debug::draw_pos(
                            element.inputdata.pos(),
                            canvas::debug::COLOR_POS,
                            zoom,
                            snapshot,
                        )
                    }
                    for &hitbox_elem in markerstroke.hitbox.iter() {
                        canvas::debug::draw_bounds(
                            hitbox_elem,
                            canvas::debug::COLOR_STROKE_HITBOX,
                            zoom,
                            snapshot,
                        );
                    }
                    canvas::debug::draw_bounds(
                        markerstroke.bounds,
                        canvas::debug::COLOR_STROKE_BOUNDS,
                        zoom,
                        snapshot,
                    );
                }
                StrokeStyle::BrushStroke(brushstroke) => {
                    for element in brushstroke.elements.iter() {
                        canvas::debug::draw_pos(
                            element.inputdata.pos(),
                            canvas::debug::COLOR_POS,
                            zoom,
                            snapshot,
                        )
                    }
                    for &hitbox_elem in brushstroke.hitboxes.iter() {
                        canvas::debug::draw_bounds(
                            hitbox_elem,
                            canvas::debug::COLOR_STROKE_HITBOX,
                            zoom,
                            snapshot,
                        );
                    }
                    canvas::debug::draw_bounds(
                        brushstroke.bounds,
                        canvas::debug::COLOR_STROKE_BOUNDS,
                        zoom,
                        snapshot,
                    );
                }
                StrokeStyle::ShapeStroke(shapestroke) => {
                    canvas::debug::draw_bounds(
                        shapestroke.bounds,
                        canvas::debug::COLOR_STROKE_BOUNDS,
                        zoom,
                        snapshot,
                    );
                }
                StrokeStyle::VectorImage(vectorimage) => {
                    canvas::debug::draw_bounds(
                        vectorimage.bounds,
                        canvas::debug::COLOR_STROKE_BOUNDS,
                        zoom,
                        snapshot,
                    );
                }
                StrokeStyle::BitmapImage(bitmapimage) => {
                    canvas::debug::draw_bounds(
                        bitmapimage.bounds,
                        canvas::debug::COLOR_STROKE_BOUNDS,
                        zoom,
                        snapshot,
                    );
                }
            }
            // Pop Blur and opacity for hidden strokes
            if let (Some(render_comp), Some(trash_comp)) = (
                self.render_components.get(key),
                self.trash_components.get(key),
            ) {
                if render_comp.render && trash_comp.trashed {
                    snapshot.pop();
                    snapshot.pop();
                }
            }
        });
    }
}
