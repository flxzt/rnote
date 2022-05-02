use super::render_comp::RenderCompState;
use super::StrokeKey;
use crate::engine::{EngineTask, EngineTaskSender};
use crate::pens::tools::DragProximityTool;
use crate::strokes::Stroke;
use crate::strokes::VectorImage;
use crate::strokes::{BitmapImage, StrokeBehaviour};
use crate::{render, StrokeStore};
use rnote_compose::helpers;
use rnote_compose::penpath::Segment;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::transform::TransformBehaviour;

use p2d::bounding_volume::{BoundingSphere, BoundingVolume, AABB};
use std::sync::Arc;

/// Systems that are related to the stroke components.
impl StrokeStore {
    /// Adds a segment to the brush stroke. If the stroke is not a brushstroke this does nothing.
    /// stroke then needs to update its geometry and its rendering
    pub fn add_segment_to_brushstroke(&mut self, key: StrokeKey, segment: Segment) {
        if let Some(Stroke::BrushStroke(brushstroke)) = Arc::make_mut(&mut self.stroke_components)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            brushstroke.push_segment(segment);

            self.set_rendering_dirty(key);
        }
    }

    /// All stroke keys unordered
    pub fn keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components.keys().collect()
    }

    /// All stroke keys unordered, excluding selected or trashed keys
    pub fn stroke_keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components
            .keys()
            .filter_map(|key| {
                if !(self.trashed(key).unwrap_or(false)) && !(self.selected(key).unwrap_or(false)) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the stroke keys in the order that they should be rendered. Exluding selected or trashed keys.
    pub fn stroke_keys_as_rendered(&self) -> Vec<StrokeKey> {
        self.keys_sorted_chrono()
            .into_iter()
            .filter_map(|key| {
                if !(self.trashed(key).unwrap_or(false)) && !(self.selected(key).unwrap_or(false)) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    /// Returns the stroke keys in the order that they should be rendered, intersecting the given bounds.
    /// Exluding selected or trashed keys.
    pub fn stroke_keys_as_rendered_intersecting_bounds(&self, bounds: AABB) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(bounds)
            .into_iter()
            .filter(|&key| {
                !(self.trashed(key).unwrap_or(false)) && !(self.selected(key).unwrap_or(false))
            })
            .collect::<Vec<StrokeKey>>()
    }

    /// Clones the strokes for the given keys and returns them.
    pub fn clone_strokes(&self, keys: &[StrokeKey]) -> Vec<Stroke> {
        keys.iter()
            .filter_map(|&key| Some((**self.stroke_components.get(key)?).clone()))
            .collect::<Vec<Stroke>>()
    }

    pub fn insert_vectorimage_bytes_threaded(
        &mut self,
        tasks_tx: EngineTaskSender,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
    ) {
        let all_strokes = self.keys_unordered();
        self.set_selected_keys(&all_strokes, false);

        rayon::spawn(move || match String::from_utf8(bytes) {
            Ok(svg) => match VectorImage::import_from_svg_data(svg.as_str(), pos, None) {
                Ok(vectorimage) => {
                    let vectorimage = Stroke::VectorImage(vectorimage);

                    tasks_tx.unbounded_send(EngineTask::InsertStroke {
                                    stroke: vectorimage
                                }).unwrap_or_else(|e| {
                                    log::error!("tasks_tx.send() failed in insert_vectorimage_bytes_threaded() with Err, {}", e);
                                });
                }
                Err(e) => {
                    log::error!("VectorImage::import_from_svg_data() failed in insert_vectorimage_bytes_threaded() with Err, {}", e);
                }
            },
            Err(e) => {
                log::error!("from_utf8() failed in thread from insert_vectorimages_bytes_threaded() with Err {}", e);
            }
        });
    }

    pub fn insert_bitmapimage_bytes_threaded(
        &mut self,
        tasks_tx: EngineTaskSender,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
    ) {
        let all_strokes = self.keys_unordered();
        self.set_selected_keys(&all_strokes, false);

        rayon::spawn(
            move || match BitmapImage::import_from_image_bytes(&bytes, pos) {
                Ok(bitmapimage) => {
                    let bitmapimage = Stroke::BitmapImage(bitmapimage);

                    tasks_tx.unbounded_send(EngineTask::InsertStroke {
                            stroke: bitmapimage
                        }).unwrap_or_else(|e| {
                            log::error!("tasks_tx.send() failed in insert_bitmapimage_bytes_threaded() with Err, {}", e);
                        });
                }
                Err(e) => {
                    log::error!("BitmapImage::import_from_svg_data() failed in insert_bitmapimage_bytes_threaded() with Err, {}", e);
                }
            },
        );
    }

    pub fn insert_pdf_bytes_as_vector_threaded(
        &mut self,
        tasks_tx: EngineTaskSender,
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
        bytes: Vec<u8>,
    ) {
        let all_strokes = self.keys_unordered();
        self.set_selected_keys(&all_strokes, false);

        rayon::spawn(
            move || match VectorImage::import_from_pdf_bytes(&bytes, pos, page_width) {
                Ok(images) => {
                    for image in images {
                        tasks_tx.unbounded_send(EngineTask::InsertStroke {
                                stroke: Stroke::VectorImage(image)
                            }).unwrap_or_else(|e| {
                                log::error!("tasks_tx.send() failed in insert_pdf_bytes_as_vector_threaded() with Err, {}", e);
                            });
                    }
                }
                Err(e) => {
                    log::error!("VectorImage::import_from_pdf_bytes() failed in insert_pdf_bytes_as_vector_threaded() with Err, {}", e);
                }
            },
        );
    }

    pub fn insert_pdf_bytes_as_bitmap_threaded(
        &mut self,
        tasks_tx: EngineTaskSender,
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
        bytes: Vec<u8>,
    ) {
        let all_strokes = self.keys_unordered();
        self.set_selected_keys(&all_strokes, false);

        rayon::spawn(
            move || match BitmapImage::import_from_pdf_bytes(&bytes, pos, page_width) {
                Ok(images) => {
                    for image in images {
                        let image = Stroke::BitmapImage(image);

                        tasks_tx.unbounded_send(EngineTask::InsertStroke {
                                stroke: image
                            }).unwrap_or_else(|e| {
                                log::error!("tasks_tx.send() failed in insert_pdf_bytes_as_bitmap_threaded() with Err, {}", e);
                            });
                    }
                }
                Err(e) => {
                    log::error!("BitmapImage::import_from_pdf_bytes() failed in insert_pdf_bytes_as_bitmap_threaded() with Err, {}", e);
                }
            },
        );
    }

    /// Updates the stroke geometry.
    /// stroke then needs to update its rendering
    pub fn update_geometry_for_stroke(&mut self, key: StrokeKey) {
        if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            match stroke {
                Stroke::BrushStroke(ref mut brushstroke) => {
                    brushstroke.update_geometry();
                    self.key_tree.update_with_key(key, stroke.bounds());

                    self.set_rendering_dirty(key);
                }
                Stroke::ShapeStroke(shapestroke) => {
                    shapestroke.update_geometry();
                    self.key_tree.update_with_key(key, stroke.bounds());

                    self.set_rendering_dirty(key);
                }
                Stroke::VectorImage(_) => {}
                Stroke::BitmapImage(_) => {}
            }
        }
    }

    /// Updates the strokes geometries.
    /// strokes then need to update their rendering
    pub fn update_geometry_for_strokes(&mut self, keys: &[StrokeKey]) {
        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);
        });
    }

    /// Calculates the width needed to fit all strokes
    pub fn calc_width(&self) -> f64 {
        let new_width = if let Some(stroke) = self
            .stroke_components
            .iter()
            .filter_map(|(key, stroke)| {
                if let Some(trash_comp) = self.trash_components.get(key) {
                    if !trash_comp.trashed {
                        return Some(stroke);
                    }
                }
                None
            })
            .max_by_key(|&stroke| stroke.bounds().maxs[0].round() as i32)
        {
            // max_by_key() returns the element, so we need to extract the width again
            stroke.bounds().maxs[0]
        } else {
            0.0
        };

        new_width
    }

    /// Calculates the height needed to fit all strokes
    pub fn calc_height(&self) -> f64 {
        let new_height = if let Some(stroke) = self
            .stroke_keys_unordered()
            .into_iter()
            .filter_map(|key| self.stroke_components.get(key))
            .max_by_key(|&stroke| stroke.bounds().maxs[1].round() as i32)
        {
            // max_by_key() returns the element, so we need to extract the height again
            stroke.bounds().maxs[1]
        } else {
            0.0
        };

        new_height
    }

    /// Generates the enclosing bounds for the given stroke keys
    pub fn gen_bounds_for_strokes(&self, keys: &[StrokeKey]) -> Option<AABB> {
        let mut keys_iter = keys.iter();
        if let Some(&key) = keys_iter.next() {
            if let Some(first) = self.stroke_components.get(key) {
                let mut bounds = first.bounds();

                keys_iter
                    .filter_map(|&key| self.stroke_components.get(key))
                    .for_each(|stroke| {
                        bounds.merge(&stroke.bounds());
                    });

                return Some(bounds);
            }
        }

        None
    }

    /// Collects all bounds for the given strokes
    pub fn bounds_for_strokes(&self, keys: &[StrokeKey]) -> Vec<AABB> {
        keys.iter()
            .filter_map(|&key| Some(self.stroke_components.get(key)?.bounds()))
            .collect::<Vec<AABB>>()
    }

    /// Generates a Svg for all strokes as drawn onto the canvas without xml headers or svg roots. Does not include the selection.
    pub fn gen_svgs_for_strokes(&self, keys: &[StrokeKey]) -> Vec<render::Svg> {
        keys.iter()
            .filter_map(|&key| {
                let stroke = self.stroke_components.get(key)?;

                match stroke.gen_svg() {
                    Ok(svgs) => Some(svgs),
                    Err(e) => {
                        log::error!(
                            "stroke.gen_svg() failed in gen_svg_for_strokes() with Err {}",
                            e
                        );
                        None
                    }
                }
            })
            .collect::<Vec<render::Svg>>()
    }

    /// Translate the strokes with the offset.
    /// strokes then need to update their rendering
    pub fn translate_strokes(&mut self, keys: &[StrokeKey], offset: na::Vector2<f64>) {
        keys.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    // translate the stroke geometry
                    stroke.translate(offset);
                    self.key_tree.update_with_key(key, stroke.bounds());
                }
            }
        });
    }

    pub fn translate_strokes_images(&mut self, keys: &[StrokeKey], offset: na::Vector2<f64>) {
        keys.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                for image in render_comp.images.iter_mut() {
                    image.translate(offset);
                }

                match render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in translate_strokes_images() with Err {}",
                        e
                    ),
                }
            }
        });
    }

    /// Rotates the stroke with angle (rad) around the center.
    /// strokes then need to update their rendering
    pub fn rotate_strokes(&mut self, keys: &[StrokeKey], angle: f64, center: na::Point2<f64>) {
        keys.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    // rotate the stroke geometry
                    stroke.rotate(angle, center);
                    self.key_tree.update_with_key(key, stroke.bounds());
                }
            }
        });
    }

    pub fn rotate_strokes_images(
        &mut self,
        keys: &[StrokeKey],
        angle: f64,
        center: na::Point2<f64>,
    ) {
        keys.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.state = RenderCompState::Dirty;

                for image in render_comp.images.iter_mut() {
                    image.rotate(angle, center);
                }

                match render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in rotate_strokes() with Err {}",
                        e
                    ),
                }
            }
        });
    }

    /// Scales the strokes with the factor.
    /// strokes then need to update their rendering
    pub fn scale_strokes(&mut self, keys: &[StrokeKey], scale: na::Vector2<f64>) {
        keys.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    // rotate the stroke geometry
                    stroke.scale(scale);
                    self.key_tree.update_with_key(key, stroke.bounds());
                }
            }
        });
    }

    pub fn scale_strokes_images(&mut self, keys: &[StrokeKey], scale: na::Vector2<f64>) {
        keys.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.state = RenderCompState::Dirty;

                for image in render_comp.images.iter_mut() {
                    image.scale(scale);
                }

                match render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in rotate_strokes() with Err {}",
                        e
                    ),
                }
            }
        });
    }

    /// Scales the strokes with a pivot as the scaling origin
    /// strokes then need to update their rendering
    pub fn scale_strokes_with_pivot(
        &mut self,
        keys: &[StrokeKey],
        scale: na::Vector2<f64>,
        pivot: na::Vector2<f64>,
    ) {
        self.translate_strokes(keys, -pivot);
        self.scale_strokes(keys, scale);
        self.translate_strokes(keys, pivot);
    }

    pub fn scale_strokes_images_with_pivot(
        &mut self,
        strokes: &[StrokeKey],
        scale: na::Vector2<f64>,
        pivot: na::Vector2<f64>,
    ) {
        self.translate_strokes_images(strokes, -pivot);
        self.scale_strokes_images(strokes, scale);
        self.translate_strokes_images(strokes, pivot);
    }

    /// Resizes the strokes to new bounds.
    /// strokes then need to update their rendering
    pub fn resize_strokes(&mut self, keys: &[StrokeKey], new_bounds: AABB) {
        let old_bounds = match self.gen_bounds_for_strokes(keys) {
            Some(old_bounds) => old_bounds,
            None => return,
        };

        keys.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    // resize the stroke geometry
                    let old_stroke_bounds = stroke.bounds();
                    let new_stroke_bounds = helpers::scale_inner_bounds_in_context_new_outer_bounds(
                        old_stroke_bounds,
                        old_bounds,
                        new_bounds,
                    );
                    let scale = new_stroke_bounds
                        .extents()
                        .component_div(&old_stroke_bounds.extents());
                    let rel_offset = new_stroke_bounds.center() - old_stroke_bounds.center();

                    // Translate in relation to the outer bounds
                    stroke.translate(rel_offset - old_stroke_bounds.center().coords);
                    stroke.scale(scale);
                    stroke.translate(old_stroke_bounds.center().coords);

                    self.key_tree.update_with_key(key, stroke.bounds());
                }
            }
        });
    }

    pub fn resize_strokes_images(&mut self, keys: &[StrokeKey], new_bounds: AABB) {
        let old_bounds = match self.gen_bounds_for_strokes(keys) {
            Some(old_bounds) => old_bounds,
            None => return,
        };

        keys.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.state = RenderCompState::Dirty;

                for image in render_comp.images.iter_mut() {
                    // resize the stroke geometry
                    let old_image_bounds = image.rect.bounds();
                    let new_image_bounds = helpers::scale_inner_bounds_in_context_new_outer_bounds(
                        old_image_bounds,
                        old_bounds,
                        new_bounds,
                    );
                    let scale = new_image_bounds
                        .extents()
                        .component_div(&old_image_bounds.extents());
                    let rel_offset = new_image_bounds.center() - old_image_bounds.center();

                    // Translate in relation to the outer bounds
                    image.translate(rel_offset - old_image_bounds.center().coords);
                    image.scale(scale);
                    image.translate(old_image_bounds.center().coords);
                }

                match render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => log::error!(
                        "images_to_rendernode() failed in resize_strokes() with Err {}",
                        e
                    ),
                }
            }
        });
    }

    /// Returns all keys below the y_pos
    pub fn keys_below_y_pos(&self, y_pos: f64) -> Vec<StrokeKey> {
        self.stroke_components
            .iter()
            .filter_map(|(key, stroke)| {
                if stroke.bounds().mins[1] > y_pos {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    /// Unimplemented!
    /// strokes then need to update their rendering
    pub fn drag_strokes_proximity(&mut self, drag_proximity_tool: &DragProximityTool) {
        let _sphere = BoundingSphere {
            center: na::Point2::from(drag_proximity_tool.pos),
            radius: drag_proximity_tool.radius,
        };

        #[allow(dead_code)]
        fn calc_distance_ratio(
            pos: na::Vector2<f64>,
            tool_pos: na::Vector2<f64>,
            radius: f64,
        ) -> f64 {
            // Zero when right at drag_proximity_tool position, One when right at the radius
            (1.0 - (pos - tool_pos).magnitude() / radius).clamp(0.0, 1.0)
        }

        unimplemented!()
    }
}
