use super::{StrokeKey, StrokeStore};

use geo::prelude::*;
use p2d::bounding_volume::{BoundingVolume, AABB};
use rayon::prelude::*;
use rnote_compose::penpath::Element;
use rnote_compose::shapes::ShapeBehaviour;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "selection_component")]
pub struct SelectionComponent {
    #[serde(default, rename = "selected")]
    pub selected: bool,
}

impl Default for SelectionComponent {
    fn default() -> Self {
        Self { selected: false }
    }
}

impl SelectionComponent {
    pub const SELECTION_DUPLICATION_OFFSET_X: f64 = 20.0;
    pub const SELECTION_DUPLICATION_OFFSET_Y: f64 = 20.0;

    pub fn new(selected: bool) -> Self {
        Self { selected }
    }
}

impl StrokeStore {
    /// Returns false if selecting is unsupported
    pub fn can_select(&self, key: StrokeKey) -> bool {
        self.selection_components.get(key).is_some()
    }

    pub fn selected(&self, key: StrokeKey) -> Option<bool> {
        if let Some(selection_comp) = self.selection_components.get(key) {
            Some(selection_comp.selected)
        } else {
            log::debug!(
                "get selection_comp in selected() returned None for stroke with key {:?}",
                key
            );
            None
        }
    }

    /// Sets if the stroke is currently selected
    pub fn set_selected(&mut self, key: StrokeKey, selected: bool) {
        if let Some(selection_comp) = self.selection_components.get_mut(key) {
            selection_comp.selected = selected;

            if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                self.chrono_counter += 1;
                chrono_comp.t = self.chrono_counter;
            }
        } else {
            log::debug!(
                "get selection_comp in set_selected() returned None for stroke with key {:?}",
                key
            );
        }
    }

    pub fn set_selected_keys(&mut self, keys: &[StrokeKey], selected: bool) {
        keys.iter().for_each(|&key| {
            self.set_selected(key, selected);
        })
    }

    pub fn selection_keys_unordered(&self) -> Vec<StrokeKey> {
        self.strokes
            .keys()
            .filter_map(|key| {
                if self.does_render(key).unwrap_or(false)
                    && !(self.trashed(key).unwrap_or(false))
                    && (self.selected(key).unwrap_or(false))
                {
                    Some(key)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the selection keys in the order that they should be rendered. Does not return the stroke keys!
    pub fn selection_keys_as_rendered(&self) -> Vec<StrokeKey> {
        let keys_sorted_chrono = self.keys_sorted_chrono();

        keys_sorted_chrono
            .iter()
            .filter_map(|&key| {
                if self.does_render(key).unwrap_or(false)
                    && !(self.trashed(key).unwrap_or(false))
                    && (self.selected(key).unwrap_or(false))
                {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub fn selection_keys_as_rendered_intersecting_bounds(&self, bounds: AABB) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(bounds)
            .into_iter()
            .filter(|&key| {
                self.does_render(key).unwrap_or(false)
                    && !(self.trashed(key).unwrap_or(false))
                    && (self.selected(key).unwrap_or(false))
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub fn selection_len(&self) -> usize {
        self.selection_keys_unordered().len()
    }

    pub fn gen_selection_bounds(&self) -> Option<AABB> {
        self.gen_bounds(&self.selection_keys_unordered())
    }

    pub fn duplicate_selection(&mut self) {
        let offset = na::vector![
            SelectionComponent::SELECTION_DUPLICATION_OFFSET_X,
            SelectionComponent::SELECTION_DUPLICATION_OFFSET_Y
        ];

        let old_selected = self.selection_keys_as_rendered();
        self.set_selected_keys(&old_selected, false);

        let new_selected = old_selected
            .iter()
            .map(|&key| {
                let new_key = self.insert_stroke(self.strokes.get(key).unwrap().clone());
                self.set_selected(new_key, true);
                new_key
            })
            .collect::<Vec<StrokeKey>>();

        // Offsetting the new selected stroke to make the duplication apparent to the user
        self.translate_strokes(&new_selected, offset);
    }

    /// Updates the selected strokes for a given polygon path. Returns the selected keys
    pub fn update_selection_for_polygon_path(
        &mut self,
        path: &[Element],
        viewport: AABB,
    ) -> Vec<StrokeKey> {
        let selector_polygon = {
            let selector_path_points = path
                .par_iter()
                .map(|element| geo::Coordinate {
                    x: element.pos[0],
                    y: element.pos[1],
                })
                .collect::<Vec<geo::Coordinate<f64>>>();

            geo::Polygon::new(selector_path_points.into(), vec![])
        };

        self.keys_sorted_chrono_intersecting_bounds(viewport)
            .iter()
            .filter_map(|&key| {
                let stroke = self.strokes.get(key)?;
                let selection_comp = self.selection_components.get_mut(key)?;

                // skip if stroke is trashed
                if let Some(trash_comp) = self.trash_components.get(key) {
                    if trash_comp.trashed {
                        return None;
                    }
                }
                selection_comp.selected = false;

                let stroke_bounds = stroke.bounds();

                if selector_polygon.contains(&crate::utils::p2d_aabb_to_geo_polygon(stroke_bounds))
                {
                    if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                        self.chrono_counter += 1;
                        chrono_comp.t = self.chrono_counter;
                    }
                    selection_comp.selected = true;
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

                    if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                        self.chrono_counter += 1;
                        chrono_comp.t = self.chrono_counter;
                    }
                    selection_comp.selected = true;
                    return Some(key);
                }

                None
            })
            .collect()
    }

    /// Updates the selected strokes for a given aabb. Returns the selected keys
    pub fn update_selection_for_aabb(&mut self, aabb: AABB, viewport: AABB) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(viewport)
            .iter()
            .filter_map(|&key| {
                let stroke = self.strokes.get(key)?;
                let selection_comp = self.selection_components.get_mut(key)?;

                // skip if stroke is trashed
                if let Some(trash_comp) = self.trash_components.get(key) {
                    if trash_comp.trashed {
                        return None;
                    }
                }

                selection_comp.selected = false;

                let stroke_bounds = stroke.bounds();

                if aabb.contains(&stroke_bounds) {
                    if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                        self.chrono_counter += 1;
                        chrono_comp.t = self.chrono_counter;
                    }
                    selection_comp.selected = true;
                    return Some(key);
                } else if aabb.intersects(&stroke_bounds) {
                    for &hitbox_elem in stroke.hitboxes().iter() {
                        if !aabb.contains(&hitbox_elem) {
                            return None;
                        }
                    }

                    if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                        self.chrono_counter += 1;
                        chrono_comp.t = self.chrono_counter;
                    }
                    selection_comp.selected = true;
                    return Some(key);
                }

                None
            })
            .collect()
    }
}
