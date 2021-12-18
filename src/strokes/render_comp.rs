use super::StateTask;
use super::{StrokeKey, StrokeStyle, StrokesState};
use crate::compose;
use crate::render;
use crate::strokes::strokestyle::StrokeBehaviour;
use crate::ui::canvas;

use gtk4::gsk::IsRenderNode;
use gtk4::{gdk, graphene, gsk, Snapshot};
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderComponent {
    pub render: bool,
    pub regenerate_flag: bool,
    #[serde(skip)]
    pub image: render::Image,
    #[serde(skip, default = "render::default_rendernode")]
    pub rendernode: gsk::RenderNode,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            render: true,
            regenerate_flag: true,
            image: render::Image::default(),
            rendernode: render::default_rendernode(),
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
            log::warn!(
                "failed to get render_comp of stroke for key {:?}, invalid key used or stroke does not support rendering",
                key
            );
            None
        }
    }

    pub fn set_render(&mut self, key: StrokeKey, render: bool) {
        if let Some(render_component) = self.render_components.get_mut(key) {
            render_component.render = render;
        } else {
            log::warn!(
                "failed to get render_comp of stroke with key {:?}, invalid key used or stroke does not support rendering",
                key
            );
        }
    }

    pub fn regenerate_flag(&self, key: StrokeKey) -> Option<bool> {
        if let Some(render_comp) = self.render_components.get(key) {
            Some(render_comp.regenerate_flag)
        } else {
            None
        }
    }

    pub fn set_regenerate_flag(&mut self, key: StrokeKey, regenerate_flag: bool) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            render_comp.regenerate_flag = regenerate_flag;
        }
    }

    pub fn reset_regeneration_flag_all_strokes(&mut self) {
        self.render_components
            .iter_mut()
            .for_each(|(_key, render_comp)| {
                render_comp.regenerate_flag = true;
            });
    }

    pub fn regenerate_rendering_for_stroke(&mut self, key: StrokeKey) {
        if let (Some(stroke), Some(render_comp)) =
            (self.strokes.get(key), self.render_components.get_mut(key))
        {
            match stroke.gen_image(self.zoom, &self.renderer.read().unwrap()) {
                Ok(image) => {
                    render_comp.regenerate_flag = false;
                    render_comp.image = image;
                    render_comp.rendernode =
                        render::image_to_texturenode(&render_comp.image, self.zoom).upcast();
                }
                Err(e) => {
                    log::error!(
                        "Failed to generate rendernode for stroke with key: {:?}, {}",
                        key,
                        e
                    )
                }
            }
        } else {
            log::warn!(
                "failed to get stroke with key {:?}, invalid key used or stroke does not support rendering",
                key
            );
        }
    }

    pub fn regenerate_rendering_for_stroke_threaded(&mut self, key: StrokeKey) {
        let current_zoom = self.zoom;
        if let (Some(render_comp), Some(tasks_tx), Some(stroke)) = (
            self.render_components.get_mut(key),
            self.tasks_tx.clone(),
            self.strokes.get(key),
        ) {
            let stroke = stroke.clone();
            let offset = na::vector![0.0, 0.0];

            let renderer = self.renderer.clone();

            // Spawn a new thread for image rendering
            self.threadpool.spawn(move || {
                //std::thread::sleep(std::time::Duration::from_millis(500));

                match stroke.gen_svg_data(offset) {
                    Ok(svg_data) => {
                        let svg_data = compose::wrap_svg(svg_data.as_str(), Some(stroke.bounds()), Some(stroke.bounds()), true, false);

                        let svg = render::Svg {
                            bounds: stroke.bounds(),
                            svg_data,
                        };
                        match renderer.read().unwrap().gen_image(current_zoom, &svg) {
                            Ok(image) => {
                                tasks_tx.send(StateTask::UpdateStrokeWithImage {
                                    key,
                                    image,
                                    zoom: current_zoom,
                                }).unwrap_or_else(|e| {
                                    log::error!("render_tx.send() failed in update_rendering_for_stroke_threaded() with Err, {}", e);
                                });
                            }
                            Err(e) => {
                                log::error!("renderer.gen_image() failed in update_rendering_for_stroke_threaded() with Err {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("stroke.gen_svg_data() failed in update_rendering_for_stroke_threaded() with Err {}", e);
                    }
                }
            });

            render_comp.regenerate_flag = false;
        } else {
            log::error!("render_tx or stroke is None in update_rendering_for_stroke_threaded()");
        }
    }

    pub fn regenerate_rendering_for_selection(&mut self) {
        let selection_keys = self.keys_selection();

        selection_keys.iter().for_each(|&key| {
            self.regenerate_rendering_for_stroke(key);
        });
    }

    pub fn regenerate_rendering_for_selection_threaded(&mut self) {
        let selection_keys = self.keys_selection();

        selection_keys.iter().for_each(|&key| {
            self.regenerate_rendering_for_stroke_threaded(key);
        });
    }

    pub fn regenerate_rendering_newest_stroke(&mut self) {
        let last_stroke_key = self.last_stroke_key();
        if let Some(key) = last_stroke_key {
            self.regenerate_rendering_for_stroke(key);
        }
    }

    pub fn regenerate_rendering_newest_stroke_threaded(&mut self) {
        let last_stroke_key = self.last_stroke_key();
        if let Some(key) = last_stroke_key {
            self.regenerate_rendering_for_stroke_threaded(key);
        }
    }

    pub fn regenerate_rendering_newest_selected(&mut self) {
        let last_selection_key = self.last_selection_key();

        if let Some(last_selection_key) = last_selection_key {
            self.regenerate_rendering_for_stroke(last_selection_key);
        }
    }

    pub fn regenerate_rendering_newest_selected_threaded(&mut self) {
        let last_selection_key = self.last_selection_key();

        if let Some(last_selection_key) = last_selection_key {
            self.regenerate_rendering_for_stroke_threaded(last_selection_key);
        }
    }

    pub fn regenerate_rendering_current_view(
        &mut self,
        viewport: Option<p2d::bounding_volume::AABB>,
        force_regenerate: bool,
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

                match stroke.gen_image(self.zoom, &self.renderer.read().unwrap()) {
                    Ok(image) => {
                        render_comp.regenerate_flag = false;
                        render_comp.image = image;
                        render_comp.rendernode = render::image_to_texturenode(&render_comp.image, self.zoom).upcast();
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to generate rendernode for stroke with key: {:?}, {}",
                            key,
                            e
                        )
                    }
                }
            } else {
                log::warn!(
                    "failed to get stroke with key {:?}, invalid key used or stroke does not support rendering",
                    key
                );
            }
        })
    }

    pub fn regenerate_rendering_current_view_threaded(
        &mut self,
        viewport: Option<p2d::bounding_volume::AABB>,
        force_regenerate: bool,
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

                self.regenerate_rendering_for_stroke_threaded(key);
            } else {
                log::warn!(
                    "failed to get stroke with key {:?}, invalid key used or stroke does not support rendering",
                    key
                );
            }
        })
    }

    pub fn update_rendering_image(&mut self, key: StrokeKey, image: render::Image) {
        if let Some(render_comp) = self.render_components.get_mut(key) {
            render_comp.image = image;
            render_comp.regenerate_flag = false;
            render_comp.rendernode =
                render::image_to_texturenode(&render_comp.image, self.zoom).upcast();
        }
    }

    /// Updates the cached rendernodes to the current zoom. Used to display the scaled (pixelated) images until new ones are generated with one of the regenerate_*_threaded funcs
    pub fn update_rendernodes_current_zoom(&mut self, zoom: f64) {
        self.render_components
            .iter_mut()
            .for_each(|(_key, render_comp)| {
                render_comp.rendernode =
                    render::image_to_texturenode(&render_comp.image, zoom).upcast();
            });
    }

    pub fn draw_strokes(&self, snapshot: &Snapshot, viewport: Option<p2d::bounding_volume::AABB>) {
        let chrono_sorted = self.keys_sorted_chrono();

        chrono_sorted
            .iter()
            .filter(|&&key| {
                self.does_render(key).unwrap_or_else(|| false)
                    && !(self.trashed(key).unwrap_or_else(|| false))
                    && !(self.selected(key).unwrap_or_else(|| false))
            })
            .for_each(|&key| {
                if let (Some(stroke), Some(render_comp)) =
                    (self.strokes.get(key), self.render_components.get(key))
                {
                    // skip if stroke is not in viewport
                    if let Some(viewport) = viewport {
                        if !viewport.intersects(&stroke.bounds()) {
                            return;
                        }
                    }

                    snapshot.append_node(&render_comp.rendernode);
                }
            });
    }

    pub fn draw_selection(&self, zoom: f64, snapshot: &Snapshot) {
        fn draw_selected_bounds(
            bounds: p2d::bounding_volume::AABB,
            zoom: f64,
            snapshot: &Snapshot,
        ) {
            let bounds = graphene::Rect::new(
                bounds.mins[0] as f32,
                bounds.mins[1] as f32,
                (bounds.extents()[0]) as f32,
                (bounds.extents()[1]) as f32,
            )
            .scale(zoom as f32, zoom as f32);
            let border_color = gdk::RGBA {
                red: 0.0,
                green: 0.2,
                blue: 0.8,
                alpha: 0.4,
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
                &[border_color, border_color, border_color, border_color],
            );
        }

        let chrono_sorted = self.keys_sorted_chrono();

        chrono_sorted
            .iter()
            .filter(|&&key| {
                self.does_render(key).unwrap_or_else(|| false)
                    && !(self.trashed(key).unwrap_or_else(|| false))
                    && (self.selected(key).unwrap_or_else(|| false))
            })
            .for_each(|&key| {
                let render_comp = self.render_components.get(key).unwrap();

                if let (Some(selection_comp), Some(stroke)) =
                    (self.selection_components.get(key), self.strokes.get(key))
                {
                    if selection_comp.selected {
                        snapshot.append_node(&render_comp.rendernode);

                        draw_selected_bounds(stroke.bounds(), zoom, snapshot);
                    }
                }
            });
        self.draw_selection_bounds(zoom, snapshot);
    }

    pub fn draw_selection_bounds(&self, zoom: f64, snapshot: &Snapshot) {
        if let Some(selection_bounds) = self.selection_bounds {
            let selection_bounds = graphene::Rect::new(
                selection_bounds.mins[0] as f32,
                selection_bounds.mins[1] as f32,
                (selection_bounds.extents()[0]) as f32,
                (selection_bounds.extents()[1]) as f32,
            )
            .scale(zoom as f32, zoom as f32);

            let selection_border_color = gdk::RGBA {
                red: 0.49,
                green: 0.56,
                blue: 0.63,
                alpha: 0.3,
            };
            let selection_border_width = 4.0;

            snapshot.append_color(
                &gdk::RGBA {
                    red: 0.49,
                    green: 0.56,
                    blue: 0.63,
                    alpha: 0.1,
                },
                &selection_bounds,
            );
            snapshot.append_border(
                &gsk::RoundedRect::new(
                    graphene::Rect::new(
                        selection_bounds.x(),
                        selection_bounds.y(),
                        selection_bounds.width(),
                        selection_bounds.height(),
                    ),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                ),
                &[
                    selection_border_width,
                    selection_border_width,
                    selection_border_width,
                    selection_border_width,
                ],
                &[
                    selection_border_color,
                    selection_border_color,
                    selection_border_color,
                    selection_border_color,
                ],
            );
        }
    }

    pub fn draw_debug(&self, zoom: f64, snapshot: &Snapshot) {
        self.strokes.iter().for_each(|(key, stroke)| {
            // Blur debug rendering for strokes which are normally hidden
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
                    for &hitbox_elem in brushstroke.hitbox.iter() {
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
