use super::render_comp::RenderCompState;
use super::StrokeKey;
use crate::strokes::Stroke;
use crate::{render, StrokeStore};
use geo::intersects::Intersects;
use geo::prelude::Contains;
use rnote_compose::helpers;
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::transform::TransformBehaviour;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use std::sync::Arc;

/// Systems that are related to the stroke components.
impl StrokeStore {
    /// Gets a reference to a stroke
    pub fn get_stroke_ref(&self, key: StrokeKey) -> Option<&Stroke> {
        self.stroke_components.get(key).map(|stroke| &**stroke)
    }

    /// Gets a mutable reference to a stroke
    pub fn get_stroke_mut(&mut self, key: StrokeKey) -> Option<&mut Stroke> {
        Arc::make_mut(&mut self.stroke_components)
            .get_mut(key)
            .map(Arc::make_mut)
    }

    /// Gets a reference to the strokes
    pub fn get_strokes_ref(&self, keys: &[StrokeKey]) -> Vec<&Stroke> {
        keys.iter()
            .filter_map(|&key| self.stroke_components.get(key).map(|stroke| &**stroke))
            .collect::<Vec<&Stroke>>()
    }

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

    pub fn keys_unordered_intersecting_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        self.key_tree.keys_intersecting_bounds(bounds)
    }

    /// All stroke keys, unordered.
    pub fn stroke_keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components
            .keys()
            .filter(|&key| !(self.trashed(key).unwrap_or(false)))
            .collect()
    }

    /// Returns the stroke keys in the order that they should be rendered.
    pub fn stroke_keys_as_rendered(&self) -> Vec<StrokeKey> {
        self.keys_sorted_chrono()
            .into_iter()
            .filter(|&key| !(self.trashed(key).unwrap_or(false)))
            .collect::<Vec<StrokeKey>>()
    }

    /// Returns the stroke keys in the order that they should be rendered, intersecting the given bounds.
    pub fn stroke_keys_as_rendered_intersecting_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(bounds)
            .into_iter()
            .filter(|&key| !(self.trashed(key).unwrap_or(false)))
            .collect::<Vec<StrokeKey>>()
    }

    /// Clones the strokes for the given keys and returns them.
    pub fn clone_strokes(&self, keys: &[StrokeKey]) -> Vec<Stroke> {
        keys.iter()
            .filter_map(|&key| Some((**self.stroke_components.get(key)?).clone()))
            .collect::<Vec<Stroke>>()
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
                }
                Stroke::ShapeStroke(shapestroke) => {
                    shapestroke.update_geometry();
                }
                Stroke::TextStroke(_) | Stroke::VectorImage(_) | Stroke::BitmapImage(_) => {}
            }

            self.key_tree.update_with_key(key, stroke.bounds());
            self.set_rendering_dirty(key);
        }
    }

    /// Updates the strokes geometries.
    /// strokes then need to update their rendering
    pub fn update_geometry_for_strokes(&mut self, keys: &[StrokeKey]) {
        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);
        });
    }

    /// Calculates the height needed to fit all strokes
    pub fn calc_height(&self) -> f64 {
        let strokes_iter = self
            .stroke_keys_unordered()
            .into_iter()
            .filter_map(|key| self.stroke_components.get(key));

        let strokes_min_y = strokes_iter
            .clone()
            .fold(0.0, |acc, stroke| stroke.bounds().mins[1].min(acc));
        let strokes_max_y = strokes_iter.fold(0.0, |acc, stroke| stroke.bounds().maxs[1].max(acc));

        strokes_max_y - strokes_min_y
    }

    /// Generates the enclosing bounds for the given stroke keys
    pub fn bounds_for_strokes(&self, keys: &[StrokeKey]) -> Option<Aabb> {
        let mut keys_iter = keys.iter();
        let key = keys_iter.next()?;
        let first = self.stroke_components.get(*key)?;
        let mut bounds = first.bounds();

        keys_iter
            .filter_map(|&key| self.stroke_components.get(key))
            .for_each(|stroke| {
                bounds.merge(&stroke.bounds());
            });

        Some(bounds)
    }

    /// Collects all bounds for the given strokes
    pub fn strokes_bounds(&self, keys: &[StrokeKey]) -> Vec<Aabb> {
        keys.iter()
            .filter_map(|&key| Some(self.stroke_components.get(key)?.bounds()))
            .collect::<Vec<Aabb>>()
    }

    /// Translate the strokes with the offset.
    ///
    /// Strokes then need to update their geometry and rendering
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

    /// Translate the stroke renderin images
    ///
    /// Strokes then need to update their rendering
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
                        "images_to_rendernode() failed in translate_strokes_images() with Err: {e:?}"
                    ),
                }
            }
        });
    }

    /// Rotates the stroke with angle (rad) around the center.
    ///
    /// Strokes then need to update their rendering
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

    /// Rotates the stroke rendering images
    ///
    /// Strokes then need to update their rendering
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
                        "images_to_rendernode() failed in rotate_strokes() with Err: {e:?}"
                    ),
                }
            }
        });
    }

    /// Scales the strokes with the factor.
    ///
    /// Strokes then need to update their rendering
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

    /// Scales the stroke rendering images
    ///
    /// Strokes then need to update their rendering
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
                        "images_to_rendernode() failed in rotate_strokes() with Err: {e:?}"
                    ),
                }
            }
        });
    }

    /// Scales the strokes with a pivot as the scaling origin
    ///
    /// Strokes then need to update their rendering
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

    /// Scales the stroke rendering images with a pivot
    ///
    /// Strokes then need to update their rendering
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
    ///
    /// Strokes then need to update their rendering
    pub fn resize_strokes(&mut self, keys: &[StrokeKey], new_bounds: Aabb) {
        let old_bounds = match self.bounds_for_strokes(keys) {
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

    /// Resizes the strokes rendering images to new bounds.
    ///
    /// Strokes then need to update their rendering
    pub fn resize_strokes_images(&mut self, keys: &[StrokeKey], new_bounds: Aabb) {
        let old_bounds = match self.bounds_for_strokes(keys) {
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
                        "images_to_rendernode() failed in resize_strokes() with Err: {e:?}"
                    ),
                }
            }
        });
    }

    /// returns the strokes whose hitboxes are contained in the given polygon path.
    pub fn strokes_hitboxes_contained_in_path_polygon(
        &mut self,
        path: &[Element],
        viewport: Aabb,
    ) -> Vec<StrokeKey> {
        let selector_polygon = {
            let selector_path_points = path
                .iter()
                .map(|element| geo::Coord {
                    x: element.pos[0],
                    y: element.pos[1],
                })
                .collect::<Vec<geo::Coord<f64>>>();

            geo::Polygon::new(selector_path_points.into(), vec![])
        };

        self.keys_sorted_chrono_intersecting_bounds(viewport)
            .into_iter()
            .filter_map(|key| {
                // skip if stroke is trashed
                if self.trashed(key)? {
                    return None;
                }

                let stroke = self.stroke_components.get(key)?;
                let stroke_bounds = stroke.bounds();

                if selector_polygon.contains(&crate::utils::p2d_aabb_to_geo_polygon(stroke_bounds))
                {
                    return Some(key);
                } else if selector_polygon
                    .intersects(&crate::utils::p2d_aabb_to_geo_polygon(stroke_bounds))
                {
                    for &hitbox_elem in stroke.hitboxes().iter() {
                        if !selector_polygon
                            .contains(&crate::utils::p2d_aabb_to_geo_polygon(hitbox_elem))
                        {
                            return None;
                        }
                    }

                    return Some(key);
                }

                None
            })
            .collect()
    }

    /// returns the strokes whose hitboxes intersect in the given path.
    pub fn strokes_hitboxes_intersect_path(
        &mut self,
        path: &[Element],
        viewport: Aabb,
    ) -> Vec<StrokeKey> {
        let path_linestring = {
            let selector_path_points = path
                .iter()
                .map(|element| geo::Coord {
                    x: element.pos[0],
                    y: element.pos[1],
                })
                .collect::<Vec<geo::Coord<f64>>>();

            geo::LineString::new(selector_path_points)
        };

        self.keys_sorted_chrono_intersecting_bounds(viewport)
            .into_iter()
            .filter_map(|key| {
                // skip if stroke is trashed
                if self.trashed(key)? {
                    return None;
                }

                let stroke = self.stroke_components.get(key)?;
                let stroke_bounds = stroke.bounds();

                if path_linestring.intersects(&crate::utils::p2d_aabb_to_geo_polygon(stroke_bounds))
                {
                    for &hitbox_elem in stroke.hitboxes().iter() {
                        if path_linestring
                            .intersects(&crate::utils::p2d_aabb_to_geo_polygon(hitbox_elem))
                        {
                            return Some(key);
                        }
                    }
                }

                None
            })
            .collect()
    }

    /// returns the keys to the strokes whose hitboxes are contained in the given aabb
    pub fn strokes_hitboxes_contained_in_aabb(
        &mut self,
        aabb: Aabb,
        viewport: Aabb,
    ) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(viewport)
            .into_iter()
            .filter_map(|key| {
                // skip if stroke is trashed
                if self.trashed(key)? {
                    return None;
                }

                let stroke = self.stroke_components.get(key)?;
                let stroke_bounds = stroke.bounds();

                if aabb.contains(&stroke_bounds) {
                    return Some(key);
                } else if aabb.intersects(&stroke_bounds) {
                    for &hitbox_elem in stroke.hitboxes().iter() {
                        if !aabb.contains(&hitbox_elem) {
                            return None;
                        }
                    }

                    return Some(key);
                }

                None
            })
            .collect()
    }

    /// returns the strokes for the given coord is inside at least one of the stroke hitboxes
    pub fn stroke_hitboxes_contain_coord(
        &self,
        viewport: Aabb,
        coord: na::Vector2<f64>,
    ) -> Vec<StrokeKey> {
        self.stroke_keys_as_rendered_intersecting_bounds(viewport)
            .into_iter()
            .filter(|&key| {
                if let Some(stroke) = self.stroke_components.get(key) {
                    stroke
                        .hitboxes()
                        .into_iter()
                        .any(|hitbox| hitbox.contains_local_point(&na::Point2::from(coord)))
                } else {
                    false
                }
            })
            .collect()
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
}
