use crate::strokes::Stroke;

use super::{StrokeKey, StrokeStore};

use p2d::bounding_volume::{BoundingVolume, AABB};
use rnote_compose::shapes::ShapeBehaviour;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "trash_component")]
pub struct TrashComponent {
    #[serde(rename = "trashed")]
    pub trashed: bool,
}

impl Default for TrashComponent {
    fn default() -> Self {
        Self { trashed: false }
    }
}

/// Systems that are related to the trash.
impl StrokeStore {
    pub fn can_trash(&self, key: StrokeKey) -> bool {
        self.trash_components.get(key).is_some()
    }

    pub fn trashed(&self, key: StrokeKey) -> Option<bool> {
        if let Some(trash_comp) = self.trash_components.get(key) {
            Some(trash_comp.trashed)
        } else {
            log::debug!(
                "get trash_comp in trashed() returned None for stroke with key {:?}",
                key
            );
            None
        }
    }

    pub fn set_trashed(&mut self, key: StrokeKey, trash: bool) {
        if let Some(trash_comp) = self.trash_components.get_mut(key) {
            trash_comp.trashed = trash;

            if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                self.chrono_counter += 1;
                chrono_comp.t = self.chrono_counter;
            }
        } else {
            log::debug!(
                "get trash_comp in set_trashed() returned None for stroke with key {:?}",
                key
            );
        }
    }

    /// Resize needed after calling this
    pub fn undo_last_stroke(&mut self) -> Option<StrokeKey> {
        let last_stroke_key = self.last_stroke_key();
        if let Some(last_stroke_key) = last_stroke_key {
            self.set_trashed(last_stroke_key, true);

            Some(last_stroke_key)
        } else {
            None
        }
    }

    /// Resize needed after calling this
    pub fn redo_last_stroke(&mut self) -> Option<StrokeKey> {
        let last_trashed_key = self.last_trashed_key();
        if let Some(last_trashed_key) = last_trashed_key {
            self.set_trashed(last_trashed_key, false);

            Some(last_trashed_key)
        } else {
            None
        }
    }

    pub fn trash_selection(&mut self) {
        // have to be in rendered order, to ensure consistent chrono_comp t value
        self.selection_keys_as_rendered().iter().for_each(|&key| {
            if let Some(selection_comp) = self.selection_components.get_mut(key) {
                if selection_comp.selected {
                    selection_comp.selected = false;

                    if let Some(trash_comp) = self.trash_components.get_mut(key) {
                        trash_comp.trashed = true;

                        if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                            self.chrono_counter += 1;
                            chrono_comp.t = self.chrono_counter;
                        }
                    }
                }
            }
        });
    }

    /// trash strokes that collide with the given bounds
    pub fn trash_colliding_strokes(&mut self, eraser_bounds: AABB, viewport: AABB) {
        self.stroke_keys_as_rendered_intersecting_bounds(viewport)
            .into_iter()
            .for_each(|key| {
                if let (Some(stroke), Some(trash_comp)) =
                    (self.strokes.get(key), self.trash_components.get_mut(key))
                {
                    match stroke {
                        Stroke::BrushStroke(brushstroke) => {
                            // First check if eraser even intersects stroke bounds, avoiding unnecessary work
                            if eraser_bounds.intersects(&brushstroke.bounds()) {
                                for hitbox_elem in brushstroke.hitboxes.iter() {
                                    if eraser_bounds.intersects(hitbox_elem) {
                                        trash_comp.trashed = true;

                                        if let Some(chrono_comp) =
                                            self.chrono_components.get_mut(key)
                                        {
                                            self.chrono_counter += 1;
                                            chrono_comp.t = self.chrono_counter;
                                        }

                                        return;
                                    }
                                }
                            }
                        }
                        Stroke::ShapeStroke(shapestroke) => {
                            if eraser_bounds.intersects(&shapestroke.bounds()) {
                                trash_comp.trashed = true;

                                if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                                    self.chrono_counter += 1;
                                    chrono_comp.t = self.chrono_counter;
                                }
                            }
                        }
                        Stroke::VectorImage(_vectorimage) => {
                            // Ignore vector images when trashing with the Eraser
                        }
                        Stroke::BitmapImage(_bitmapimage) => {
                            // Ignore bitmap images when trashing with the Eraser
                        }
                    }
                }
            });
    }
}
