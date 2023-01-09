use super::{StrokeKey, StrokeStore};

use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    const SELECTION_DUPLICATION_OFFSET: na::Vector2<f64> = na::vector![20.0, 20.0];

    pub fn new(selected: bool) -> Self {
        Self { selected }
    }
}

impl StrokeStore {
    /// Reloads the slotmap with empty selection components from the keys returned from the primary map, stroke_components.
    pub fn rebuild_selection_components_slotmap(&mut self) {
        self.selection_components = Arc::new(slotmap::SecondaryMap::new());
        self.stroke_components.keys().for_each(|key| {
            Arc::make_mut(&mut self.selection_components)
                .insert(key, Arc::new(SelectionComponent::default()));
        });
    }

    /// Returns false if selecting is unsupported
    pub fn can_select(&self, key: StrokeKey) -> bool {
        self.selection_components.get(key).is_some()
    }

    pub fn selected(&self, key: StrokeKey) -> Option<bool> {
        self.selection_components
            .get(key)
            .map(|selection_comp| selection_comp.selected)
    }

    /// Sets if the stroke is currently selected
    pub fn set_selected(&mut self, key: StrokeKey, selected: bool) {
        if let Some(selection_comp) = Arc::make_mut(&mut self.selection_components)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            selection_comp.selected = selected;

            self.update_chrono_to_last(key);
        }
    }

    pub fn set_selected_keys(&mut self, keys: &[StrokeKey], selected: bool) {
        keys.iter().for_each(|&key| {
            self.set_selected(key, selected);
        })
    }

    pub fn selection_keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components
            .keys()
            .filter(|&key| {
                !(self.trashed(key).unwrap_or(false)) && (self.selected(key).unwrap_or(false))
            })
            .collect()
    }

    /// Returns the selection keys in the order that they should be rendered.
    /// Does not return the not-selected stroke keys.
    pub fn selection_keys_as_rendered(&self) -> Vec<StrokeKey> {
        let keys_sorted_chrono = self.keys_sorted_chrono();

        keys_sorted_chrono
            .into_iter()
            .filter(|&key| {
                !(self.trashed(key).unwrap_or(false)) && (self.selected(key).unwrap_or(false))
            })
            .collect::<Vec<StrokeKey>>()
    }

    /// Returns the selection keys in the order that they should be rendered that intersect the given bounds.
    /// Does not return the not-selected stroke keys.
    pub fn selection_keys_as_rendered_intersecting_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        self.keys_sorted_chrono_intersecting_bounds(bounds)
            .into_iter()
            .filter(|&key| {
                !(self.trashed(key).unwrap_or(false)) && (self.selected(key).unwrap_or(false))
            })
            .collect::<Vec<StrokeKey>>()
    }

    /// Generates the bounds that include all selected strokes.
    /// None if no strokes are selected
    pub fn gen_selection_bounds(&self) -> Option<Aabb> {
        self.bounds_for_strokes(&self.selection_keys_unordered())
    }

    /// Duplicates the selected keys
    /// the returned, duplicated strokes then need to update their geometry and rendering
    pub fn duplicate_selection(&mut self) -> Vec<StrokeKey> {
        let old_selected = self.selection_keys_as_rendered();
        self.set_selected_keys(&old_selected, false);

        let new_selected = old_selected
            .iter()
            .filter_map(|&key| {
                let new_key =
                    self.insert_stroke((**self.stroke_components.get(key)?).clone(), None);
                self.set_selected(new_key, true);
                Some(new_key)
            })
            .collect::<Vec<StrokeKey>>();

        // Offsetting the new selected stroke to make the duplication apparent
        self.translate_strokes(
            &new_selected,
            SelectionComponent::SELECTION_DUPLICATION_OFFSET,
        );

        new_selected
    }
}
