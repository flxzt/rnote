use crate::strokes::{BrushStroke, Stroke};

use super::{StrokeKey, StrokeStore};

use p2d::bounding_volume::{BoundingVolume, AABB};
use rnote_compose::penpath::Segment;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::PenPath;
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
                        Stroke::BrushStroke(_) | Stroke::ShapeStroke(_) => {
                            // First check if eraser even intersects stroke bounds, avoiding unnecessary work
                            if eraser_bounds.intersects(&stroke.bounds()) {
                                for hitbox_elem in stroke.hitboxes().iter() {
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

    /// remove colliding stroke segments with the given bounds. The stroke is then split. For strokes that don't have segments, trash the entire stroke.
    /// Returns the keys to newly created strokes.
    /// Needs rendering regeneration
    pub fn split_colliding_strokes(
        &mut self,
        eraser_bounds: AABB,
        viewport: AABB,
    ) -> Vec<StrokeKey> {
        self.stroke_keys_as_rendered_intersecting_bounds(viewport)
            .into_iter()
            .filter_map(|key| {
                let mut new_strokes = vec![];

                let stroke = self.strokes.get_mut(key)?;
                let trash_comp = self.trash_components.get_mut(key)?;
                let mut remove_current_stroke = false;
                let stroke_bounds = stroke.bounds();

                match stroke {
                    Stroke::BrushStroke(brushstroke) => {
                        if eraser_bounds.intersects(&stroke_bounds) {
                            brushstroke.path.make_contiguous();

                            let split_penpaths = brushstroke
                                .path
                                .as_slices()
                                .0
                                .split(|segment| {
                                    segment
                                        .hitboxes()
                                        .iter()
                                        .find(|bounds| bounds.intersects(&eraser_bounds))
                                        .is_some()
                                })
                                .collect::<Vec<&[Segment]>>();

                            if split_penpaths.len() > 1 {
                                split_penpaths
                                    .into_iter()
                                    .filter_map(|penpath| {
                                        let split_penpath =
                                            PenPath::from_iter(penpath.to_owned().into_iter());

                                        BrushStroke::from_penpath(
                                            split_penpath,
                                            brushstroke.style.clone(),
                                        )
                                    })
                                    .collect::<Vec<BrushStroke>>()
                                    .into_iter()
                                    .for_each(|split_brushstroke| {
                                        new_strokes.push(
                                            self.insert_stroke(Stroke::BrushStroke(
                                                split_brushstroke,
                                            )),
                                        );
                                    });

                                remove_current_stroke = true;
                            }
                        }
                    }
                    Stroke::ShapeStroke(_) => {
                        if eraser_bounds.intersects(&stroke_bounds) {
                            for hitbox_elem in stroke.hitboxes().iter() {
                                if eraser_bounds.intersects(hitbox_elem) {
                                    trash_comp.trashed = true;

                                    if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                                        self.chrono_counter += 1;
                                        chrono_comp.t = self.chrono_counter;
                                    }

                                    return None;
                                }
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

                if remove_current_stroke {
                    self.remove_stroke(key);
                }

                Some(new_strokes)
            })
            .flatten()
            .collect::<Vec<StrokeKey>>()
    }
}
