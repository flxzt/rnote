// Imports
use super::render_comp::RenderCompState;
use super::StrokeKey;
use crate::engine::StrokeContent;
use crate::strokes::{Content, Stroke};
use crate::{StrokeStore, WidgetFlags};
use geo::intersects::Intersects;
use geo::prelude::Contains;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::penpath::Element;
use rnote_compose::shapes::Shapeable;
use rnote_compose::transform::Transformable;
use rnote_compose::Color;
use std::sync::Arc;

/// Systems that are related to the stroke components.
impl StrokeStore {
    /// Gets a immutable reference to a stroke.
    pub(crate) fn get_stroke_ref(&self, key: StrokeKey) -> Option<&Stroke> {
        self.stroke_components.get(key).map(|stroke| &**stroke)
    }

    /// Gets a mutable reference to a stroke.
    pub(crate) fn get_stroke_mut(&mut self, key: StrokeKey) -> Option<&mut Stroke> {
        Arc::make_mut(&mut self.stroke_components)
            .get_mut(key)
            .map(Arc::make_mut)
    }

    /// Gets the stroke by cloning the Arc that is wrapped around it.
    #[allow(unused)]
    pub(crate) fn get_stroke_arc(&self, key: StrokeKey) -> Option<Arc<Stroke>> {
        self.stroke_components.get(key).cloned()
    }

    /// Gets immutable references to the strokes.
    pub(crate) fn get_strokes_ref(&self, keys: &[StrokeKey]) -> Vec<&Stroke> {
        keys.iter()
            .filter_map(|&key| self.stroke_components.get(key).map(|stroke| &**stroke))
            .collect()
    }

    /// Gets the strokes by cloning the Arc's that are wrapped around them.
    pub(crate) fn get_strokes_arc(&self, keys: &[StrokeKey]) -> Vec<Arc<Stroke>> {
        keys.iter()
            .filter_map(|&key| self.stroke_components.get(key).cloned())
            .collect()
    }

    /// All keys from the stroke components slotmap, unordered.
    pub(crate) fn keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components.keys().collect()
    }

    #[allow(unused)]
    pub(crate) fn keys_unordered_intersecting_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        self.key_tree.keys_intersecting_bounds(bounds)
    }

    /// All stroke keys that are not trashed, unordered.
    pub(crate) fn stroke_keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components
            .keys()
            .filter(|&key| !(self.trashed(key).unwrap_or(false)))
            .collect()
    }

    /// Storke keys in the order that they should be rendered.
    pub(crate) fn stroke_keys_as_rendered(&self) -> Vec<StrokeKey> {
        self.keys_sorted_chrono()
            .into_iter()
            .filter(|&key| !(self.trashed(key).unwrap_or(false)))
            .collect::<Vec<StrokeKey>>()
    }

    /// Stroke keys intersecting the given bounds, in the order that they should be rendered.
    pub(crate) fn stroke_keys_as_rendered_intersecting_bounds(
        &self,
        bounds: Aabb,
    ) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(bounds)
            .into_iter()
            .filter(|&key| !(self.trashed(key).unwrap_or(false)))
            .collect::<Vec<StrokeKey>>()
    }

    /// Stroke keys contained in the given bounds, in the order that they should be rendered.
    pub(crate) fn stroke_keys_as_rendered_in_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_in_bounds(bounds)
            .into_iter()
            .filter(|&key| !(self.trashed(key).unwrap_or(false)))
            .collect::<Vec<StrokeKey>>()
    }

    /// Clone the strokes for the given keys.
    #[allow(unused)]
    pub(crate) fn clone_strokes(&self, keys: &[StrokeKey]) -> Vec<Stroke> {
        keys.iter()
            .filter_map(|&key| Some((**self.stroke_components.get(key)?).clone()))
            .collect::<Vec<Stroke>>()
    }

    /// Updates the stroke geometry.
    ///
    /// The stroke then needs to update its rendering.
    pub(crate) fn update_geometry_for_stroke(&mut self, key: StrokeKey) {
        if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            stroke.update_geometry();
            self.key_tree.update_with_key(key, stroke.bounds());
            self.set_rendering_dirty(key);
        }
    }

    /// Updates the strokes geometries.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn update_geometry_for_strokes(&mut self, keys: &[StrokeKey]) {
        keys.iter().for_each(|&key| {
            self.update_geometry_for_stroke(key);
        });
    }

    /// Calculate the height needed to fit all strokes.
    pub(crate) fn calc_height(&self) -> f64 {
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

    /// Calculate the width needed to fit all strokes.
    #[allow(unused)]
    pub(crate) fn calc_width(&self) -> f64 {
        let strokes_iter = self
            .stroke_keys_unordered()
            .into_iter()
            .filter_map(|key| self.stroke_components.get(key));

        let strokes_min_x = strokes_iter
            .clone()
            .fold(0.0, |acc, stroke| stroke.bounds().mins[0].min(acc));
        let strokes_max_x = strokes_iter.fold(0.0, |acc, stroke| stroke.bounds().maxs[0].max(acc));

        strokes_max_x - strokes_min_x
    }

    /// Generate the enclosing bounds for the given keys.
    pub(crate) fn bounds_for_strokes(&self, keys: &[StrokeKey]) -> Option<Aabb> {
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

    /// Collect all stroke bounds for the given keys.
    pub(crate) fn strokes_bounds(&self, keys: &[StrokeKey]) -> Vec<Aabb> {
        keys.iter()
            .filter_map(|&key| Some(self.stroke_components.get(key)?.bounds()))
            .collect::<Vec<Aabb>>()
    }

    pub(crate) fn set_stroke_pos(&mut self, key: StrokeKey, pos: na::Vector2<f64>) {
        let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
            .get_mut(key)
            .map(Arc::make_mut)
        else {
            return;
        };
        stroke.set_pos(pos);
    }

    /// Translate the strokes by the offset.
    ///
    /// The strokes then need to update their geometry and rendering.
    pub(crate) fn translate_strokes(&mut self, keys: &[StrokeKey], offset: na::Vector2<f64>) {
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

    /// Translate the stroke rendering images.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn translate_strokes_images(
        &mut self,
        keys: &[StrokeKey],
        offset: na::Vector2<f64>,
    ) {
        keys.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                for image in render_comp.images.iter_mut() {
                    image.translate(offset);
                }

                #[cfg(feature = "ui")]
                match crate::render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => tracing::error!(
                        "Generating rendernodes from images failed while translating stroke images , Err: {e:?}"
                    ),
                }
            }
        });
    }

    /// Rotate the stroke by the given angle (in radians) around the center.
    ///
    /// Strokes then need to update their rendering.
    pub(crate) fn rotate_strokes(
        &mut self,
        keys: &[StrokeKey],
        angle: f64,
        center: na::Point2<f64>,
    ) {
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

    /// Change the stroke and text color for the given keys.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn change_stroke_colors(&mut self, keys: &[StrokeKey], color: Color) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if keys.is_empty() {
            return widget_flags;
        }

        keys.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    match stroke {
                        Stroke::BrushStroke(brush_stroke) => {
                            brush_stroke.style.set_stroke_color(color);
                            self.set_rendering_dirty(key);
                        }
                        Stroke::ShapeStroke(shape_stroke) => {
                            shape_stroke.style.set_stroke_color(color);
                            self.set_rendering_dirty(key);
                        }
                        Stroke::TextStroke(text_stroke) => {
                            text_stroke.text_style.color = color;
                            self.set_rendering_dirty(key);
                        }
                        _ => {}
                    }
                }
            }
        });

        widget_flags.redraw = true;
        widget_flags.store_modified = true;

        widget_flags
    }

    /// Invert the stroke, text and fill color of the given keys.
    ///
    /// Strokes then need to update their rendering.
    pub fn invert_color_brightness(&mut self, keys: &[StrokeKey]) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if keys.is_empty() {
            return widget_flags;
        }

        keys.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                let stroke_modified = stroke.set_to_inverted_brightness_color();

                if stroke_modified {
                    self.set_rendering_dirty(key);
                }
            }
        });

        widget_flags.redraw = true;
        widget_flags.store_modified = true;

        widget_flags
    }

    /// Change the fill color of the given keys.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn change_fill_colors(&mut self, keys: &[StrokeKey], color: Color) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if keys.is_empty() {
            return widget_flags;
        }

        keys.iter().for_each(|&key| {
            if let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
                .get_mut(key)
                .map(Arc::make_mut)
            {
                {
                    match stroke {
                        Stroke::BrushStroke(brush_stroke) => {
                            brush_stroke.style.set_fill_color(color);
                            self.set_rendering_dirty(key);
                        }
                        Stroke::ShapeStroke(shape_stroke) => {
                            shape_stroke.style.set_fill_color(color);
                            self.set_rendering_dirty(key);
                        }
                        _ => {}
                    }
                }
            }
        });

        widget_flags.redraw = true;
        widget_flags.store_modified = true;

        widget_flags
    }

    /// Rotate the stroke rendering images.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn rotate_strokes_images(
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

                #[cfg(feature = "ui")]
                match crate::render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => tracing::error!(
                        "Generating rendernodes from images failed while rotating stroke images , Err: {e:?}"
                    ),
                }
            }
        });
    }

    /// Scale the strokes with the factor.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn scale_strokes(&mut self, keys: &[StrokeKey], scale: na::Vector2<f64>) {
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

    /// Scale the stroke rendering images.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn scale_strokes_images(&mut self, keys: &[StrokeKey], scale: na::Vector2<f64>) {
        keys.iter().for_each(|&key| {
            if let Some(render_comp) = self.render_components.get_mut(key) {
                render_comp.state = RenderCompState::Dirty;

                for image in render_comp.images.iter_mut() {
                    image.scale(scale);
                }

                #[cfg(feature = "ui")]
                match crate::render::Image::images_to_rendernodes(&render_comp.images) {
                    Ok(rendernodes) => {
                        render_comp.rendernodes = rendernodes;
                    }
                    Err(e) => tracing::error!(
                        "Generating rendernodes from images failed while scaling stroke images, Err: {e:?}"
                    ),
                }
            }
        });
    }

    /// Scale the strokes with a pivot as the scaling origin.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn scale_strokes_with_pivot(
        &mut self,
        keys: &[StrokeKey],
        scale: na::Vector2<f64>,
        pivot: na::Vector2<f64>,
    ) {
        self.translate_strokes(keys, -pivot);
        self.scale_strokes(keys, scale);
        self.translate_strokes(keys, pivot);
    }

    /// Scale the stroke rendering images with a pivot.
    ///
    /// The strokes then need to update their rendering.
    pub(crate) fn scale_strokes_images_with_pivot(
        &mut self,
        strokes: &[StrokeKey],
        scale: na::Vector2<f64>,
        pivot: na::Vector2<f64>,
    ) {
        self.translate_strokes_images(strokes, -pivot);
        self.scale_strokes_images(strokes, scale);
        self.translate_strokes_images(strokes, pivot);
    }

    /// Return the keys for stroke whose hitboxes are contained in the given polygon path.
    pub(crate) fn strokes_hitboxes_contained_in_path_polygon(
        &mut self,
        path: &[Element],
        viewport: Aabb,
    ) -> Vec<StrokeKey> {
        let mut bounds = viewport;
        for p in path {
            bounds.take_point(p.pos.into());
        }

        let path_polygon = {
            let selector_path_points = path
                .iter()
                .map(|element| geo::Coord {
                    x: element.pos[0],
                    y: element.pos[1],
                })
                .collect::<Vec<geo::Coord<f64>>>();

            geo::Polygon::new(selector_path_points.into(), vec![])
        };

        self.keys_sorted_chrono_intersecting_bounds(bounds)
            .into_iter()
            .filter_map(|key| {
                // skip if stroke is trashed
                if self.trashed(key)? {
                    return None;
                }

                let stroke = self.stroke_components.get(key)?;
                let stroke_bounds = stroke.bounds();

                if path_polygon.contains(&crate::utils::p2d_aabb_to_geo_polygon(stroke_bounds)) {
                    return Some(key);
                } else if path_polygon
                    .intersects(&crate::utils::p2d_aabb_to_geo_polygon(stroke_bounds))
                {
                    for &hitbox_elem in stroke.hitboxes().iter() {
                        if !path_polygon
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

    /// Return the keys for strokes whose hitboxes intersect in the given path.
    pub(crate) fn strokes_hitboxes_intersect_path(
        &mut self,
        path: &[Element],
        viewport: Aabb,
    ) -> Vec<StrokeKey> {
        let mut bounds = viewport;
        for p in path {
            bounds.take_point(p.pos.into());
        }

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

        self.keys_sorted_chrono_intersecting_bounds(bounds)
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

    /// Return the keys for strokes whose hitboxes are contained in the given Aabb.
    pub(crate) fn strokes_hitboxes_contained_in_aabb(
        &mut self,
        aabb: Aabb,
        viewport: Aabb,
    ) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(viewport.merged(&aabb))
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

    /// Return the keys for strokes where the given coord is inside at least one of their hitboxes.
    pub(crate) fn stroke_hitboxes_contain_coord(
        &self,
        viewport: Aabb,
        coord: na::Vector2<f64>,
    ) -> Vec<StrokeKey> {
        let mut bounds = viewport;
        bounds.take_point(coord.into());

        self.stroke_keys_as_rendered_intersecting_bounds(bounds)
            .into_iter()
            .filter(|&key| {
                if let Some(stroke) = self.stroke_components.get(key) {
                    stroke
                        .hitboxes()
                        .into_iter()
                        .any(|hitbox| hitbox.contains_local_point(&coord.into()))
                } else {
                    false
                }
            })
            .collect()
    }

    /// Return all keys below the given `y`.
    pub(crate) fn keys_below_y(&self, y: f64) -> Vec<StrokeKey> {
        self.stroke_components
            .iter()
            .filter_map(|(key, stroke)| {
                if stroke.bounds().mins[1] > y {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub(crate) fn filter_keys_intersecting_bounds<'a, I: IntoIterator<Item = &'a StrokeKey>>(
        &'a self,
        keys: I,
        bounds: Aabb,
    ) -> impl Iterator<Item = &'a StrokeKey> {
        keys.into_iter().filter(move |key| {
            self.stroke_components
                .get(**key)
                .map(|s| s.bounds().intersects(&bounds))
                .unwrap_or(false)
        })
    }

    pub(crate) fn fetch_stroke_content(&self, keys: &[StrokeKey]) -> StrokeContent {
        let strokes = keys
            .iter()
            .filter_map(|k| self.stroke_components.get(*k).cloned())
            .collect();

        StrokeContent::default().with_strokes(strokes)
    }

    /// Cut the strokes for the given keys and return them as stroke content.
    pub(crate) fn cut_stroke_content(&mut self, keys: &[StrokeKey]) -> StrokeContent {
        let strokes = keys
            .iter()
            .filter_map(|k| {
                self.set_selected(*k, false);
                self.set_trashed(*k, true);
                self.stroke_components.get(*k).cloned()
            })
            .collect();

        StrokeContent::default().with_strokes(strokes)
    }

    /// Paste the clipboard content as a selection.
    ///
    /// Returns the keys for the inserted strokes.
    ///
    /// The inserted strokes then need to update their geometry and rendering.
    pub(crate) fn insert_stroke_content(
        &mut self,
        clipboard_content: StrokeContent,
        ratio: f64,
        pos: na::Vector2<f64>,
    ) -> Vec<StrokeKey> {
        if clipboard_content.strokes.is_empty() {
            return vec![];
        }
        let clipboard_bounds = clipboard_content
            .strokes
            .iter()
            .fold(Aabb::new_invalid(), |acc, s| acc.merged(&s.bounds()));

        clipboard_content
            .strokes
            .into_iter()
            .map(|s| {
                let offset = s.bounds().mins.coords - clipboard_bounds.mins.coords;
                let key = self.insert_stroke((*s).clone(), None);
                // position strokes without resizing
                self.set_stroke_pos(key, pos);
                self.translate_strokes(&[key], offset);

                // apply a rescale around a pivot
                self.scale_strokes_with_pivot(&[key], na::Vector2::new(ratio, ratio), pos);
                self.scale_strokes_images_with_pivot(&[key], na::Vector2::new(ratio, ratio), pos);

                // select keys
                self.set_selected(key, true);
                key
            })
            .collect()
    }
}
