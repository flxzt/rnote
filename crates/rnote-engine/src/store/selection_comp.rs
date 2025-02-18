// Imports
use super::render_comp::RenderCompState;
use super::{StrokeKey, StrokeStore};
use crate::strokes::content::GeneratedContentImages;
use crate::strokes::Stroke;
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

/// Systems that are related to selecting.
impl StrokeStore {
    /// Rebuild the slotmap with empty selection components with the keys returned from the stroke components.
    pub(crate) fn rebuild_selection_components_slotmap(&mut self) {
        self.selection_components = Arc::new(slotmap::SecondaryMap::new());
        self.stroke_components.keys().for_each(|key| {
            Arc::make_mut(&mut self.selection_components)
                .insert(key, Arc::new(SelectionComponent::default()));
        });
    }

    /// Ability if selecting is supported.
    #[allow(unused)]
    pub(crate) fn can_select(&self, key: StrokeKey) -> bool {
        self.selection_components.get(key).is_some()
    }

    pub(crate) fn selected(&self, key: StrokeKey) -> Option<bool> {
        self.selection_components
            .get(key)
            .map(|selection_comp| selection_comp.selected)
    }

    /// Set if the stroke is currently selected.
    pub(crate) fn set_selected(&mut self, key: StrokeKey, selected: bool) {
        if let Some(selection_comp) = Arc::make_mut(&mut self.selection_components)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            selection_comp.selected = selected;

            self.update_chrono_to_last(key);
        }
    }

    pub(crate) fn set_selected_keys(&mut self, keys: &[StrokeKey], selected: bool) {
        keys.iter().for_each(|&key| {
            self.set_selected(key, selected);
        })
    }

    pub(crate) fn selection_keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components
            .keys()
            .filter(|&key| {
                !(self.trashed(key).unwrap_or(false)) && (self.selected(key).unwrap_or(false))
            })
            .collect()
    }

    /// Return the selection keys in the order that they should be rendered.
    ///
    /// Does not return the non-selected stroke keys.
    pub(crate) fn selection_keys_as_rendered(&self) -> Vec<StrokeKey> {
        let keys_sorted_chrono = self.keys_sorted_chrono();

        keys_sorted_chrono
            .into_iter()
            .filter(|&key| {
                !(self.trashed(key).unwrap_or(false)) && (self.selected(key).unwrap_or(false))
            })
            .collect::<Vec<StrokeKey>>()
    }

    /// Generate the bounds that include all selected strokes.
    ///
    /// None if no strokes are selected
    #[allow(unused)]
    pub(crate) fn selection_bounds(&self) -> Option<Aabb> {
        self.bounds_for_strokes(&self.selection_keys_unordered())
    }

    /// Duplicate the selected keys.
    ///
    /// The returned, duplicated strokes then need to update their geometry and rendering.
    pub(crate) fn duplicate_selection(&mut self) -> Vec<StrokeKey> {
        let old_selected = self.selection_keys_as_rendered();
        self.set_selected_keys(&old_selected, false);

        let new_selected = old_selected
            .iter()
            .filter_map(|&old_key| {
                let new_key =
                    self.insert_stroke((**self.stroke_components.get(old_key)?).clone(), None);

                // duplicate and insert the render images of the old stroke to avoid flickering
                if let Some(render_comp) = self.render_components.get(old_key) {
                    let images = render_comp.images.clone();
                    if let RenderCompState::ForViewport(viewport) = render_comp.state {
                        self.replace_rendering_with_images(
                            new_key,
                            GeneratedContentImages::Partial { images, viewport },
                        );
                    } else if render_comp.state == RenderCompState::Complete {
                        self.replace_rendering_with_images(
                            new_key,
                            GeneratedContentImages::Full(images),
                        );
                    }
                }

                self.set_selected(new_key, true);

                Some(new_key)
            })
            .collect::<Vec<StrokeKey>>();

        // Offsetting the new selected stroke to make the duplication apparent
        self.translate_strokes(&new_selected, Stroke::IMPORT_OFFSET_DEFAULT);
        self.translate_strokes_images(&new_selected, Stroke::IMPORT_OFFSET_DEFAULT);

        new_selected
    }
}
