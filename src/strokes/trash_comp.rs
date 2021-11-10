use crate::pens::eraser::Eraser;
use crate::strokes::{StrokeBehaviour, StrokeStyle};

use super::{StrokeKey, StrokesState};

use p2d::bounding_volume::BoundingVolume;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrashComponent {
    pub trashed: bool,
}

impl Default for TrashComponent {
    fn default() -> Self {
        Self { trashed: false }
    }
}

/// Systems that are related to the trash.
impl StrokesState {
    pub fn can_trash(&self, key: StrokeKey) -> Option<bool> {
        if let Some(trash_comp) = self.trash_components.get(key) {
            return Some(trash_comp.is_some());
        } else {
            log::warn!(
                "failed to get trash_component for stroke with key {:?}, invalid key used",
                key
            );
            return None;
        }
    }

    pub fn trashed(&self, key: StrokeKey) -> Option<bool> {
        if let Some(Some(trash_comp)) = self.trash_components.get(key) {
            Some(trash_comp.trashed)
        } else {
            log::warn!(
                "failed to get trash_comp of stroke with key {:?}, invalid key used or stroke does not support trashing",
                key
            );
            None
        }
    }

    pub fn set_trashed(&mut self, key: StrokeKey, trash: bool) {
        if let Some(Some(trash_comp)) = self.trash_components.get_mut(key) {
            trash_comp.trashed = trash;

            if let Some(Some(chrono_comp)) = self.chrono_components.get_mut(key) {
                self.chrono_counter += 1;
                chrono_comp.t = self.chrono_counter;
            }
        } else {
            log::warn!(
                "failed to get trash_comp of stroke with key {:?}, invalid key used or stroke does not support trashing",
                key
            );
        }
    }

    pub fn last_trashed_key(&self) -> Option<StrokeKey> {
        let mut sorted = self
            .chrono_components
            .iter()
            .filter_map(|(key, chrono_comp)| {
                if let (Some(Some(trash_comp)), Some(chrono_comp)) =
                    (self.trash_components.get(key), chrono_comp)
                {
                    if trash_comp.trashed {
                        return Some((key, chrono_comp.t));
                    }
                }
                None
            })
            .collect::<Vec<(StrokeKey, u64)>>();
        sorted.par_sort_unstable_by(|first, second| first.1.cmp(&second.1));

        let last_trashed_key = sorted.last().copied();
        if let Some(last_trashed_key) = last_trashed_key {
            Some(last_trashed_key.0)
        } else {
            None
        }
    }

    /// Resize needed after calling this
    pub fn undo_last_stroke(&mut self) -> Option<StrokeKey> {
        let last_stroke_key = self.last_stroke_key();
        if let Some(last_stroke_key) = last_stroke_key {
            self.set_trashed(last_stroke_key, true);
            self.update_rendering_for_stroke(last_stroke_key);

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
            self.update_rendering_for_stroke(last_trashed_key);

            Some(last_trashed_key)
        } else {
            None
        }
    }

    /// trash strokes that collide with the eraser
    pub fn trash_colliding_strokes(
        &mut self,
        eraser: &Eraser,
        viewport: Option<p2d::bounding_volume::AABB>,
    ) {
        if let Some(ref eraser_current_input) = eraser.current_input {
            let eraser_bounds = p2d::bounding_volume::AABB::new(
                na::Point2::from(
                    eraser_current_input.pos()
                        - na::vector![eraser.width() / 2.0, eraser.width() / 2.0],
                ),
                na::Point2::from(
                    eraser_current_input.pos()
                        + na::vector![eraser.width() / 2.0, eraser.width() / 2.0],
                ),
            );

            self.strokes.iter().for_each(|(key, stroke)| {
                if let Some(viewport) = viewport {
                    if !viewport.intersects(&stroke.bounds()) {
                        return;
                    }
                }
                match stroke {
                    StrokeStyle::MarkerStroke(markerstroke) => {
                        // First check markerstroke bounds, then conditionally check hitbox
                        if eraser_bounds.intersects(&markerstroke.bounds) {
                            for hitbox_elem in markerstroke.hitbox.iter() {
                                if eraser_bounds.intersects(hitbox_elem) {
                                    if let Some(Some(trash_comp)) =
                                        self.trash_components.get_mut(key)
                                    {
                                        trash_comp.trashed = true;

                                        if let Some(Some(chrono_comp)) =
                                            self.chrono_components.get_mut(key)
                                        {
                                            self.chrono_counter += 1;
                                            chrono_comp.t = self.chrono_counter;
                                        }
                                    }

                                    return;
                                }
                            }
                        }
                    }
                    StrokeStyle::BrushStroke(brushstroke) => {
                        // First check markerstroke bounds, then conditionally check hitbox
                        if eraser_bounds.intersects(&brushstroke.bounds) {
                            for hitbox_elem in brushstroke.hitbox.iter() {
                                if eraser_bounds.intersects(hitbox_elem) {
                                    if let Some(Some(trash_comp)) =
                                        self.trash_components.get_mut(key)
                                    {
                                        trash_comp.trashed = true;

                                        if let Some(Some(chrono_comp)) =
                                            self.chrono_components.get_mut(key)
                                        {
                                            self.chrono_counter += 1;
                                            chrono_comp.t = self.chrono_counter;
                                        }
                                    }

                                    return;
                                }
                            }
                        }
                    }
                    StrokeStyle::ShapeStroke(shapestroke) => {
                        if eraser_bounds.intersects(&shapestroke.bounds) {
                            if let Some(Some(trash_comp)) = self.trash_components.get_mut(key) {
                                trash_comp.trashed = true;

                                if let Some(Some(chrono_comp)) = self.chrono_components.get_mut(key)
                                {
                                    self.chrono_counter += 1;
                                    chrono_comp.t = self.chrono_counter;
                                }
                            }

                            return;
                        }
                    }
                    StrokeStyle::VectorImage(vectorimage) => {
                        if eraser_bounds.intersects(&vectorimage.bounds) {
                            if let Some(Some(trash_comp)) = self.trash_components.get_mut(key) {
                                trash_comp.trashed = true;

                                if let Some(Some(chrono_comp)) = self.chrono_components.get_mut(key)
                                {
                                    self.chrono_counter += 1;
                                    chrono_comp.t = self.chrono_counter;
                                }
                            }

                            return;
                        }
                    }
                    StrokeStyle::BitmapImage(bitmapimage) => {
                        if eraser_bounds.intersects(&bitmapimage.bounds) {
                            if let Some(Some(trash_comp)) = self.trash_components.get_mut(key) {
                                trash_comp.trashed = true;

                                if let Some(Some(chrono_comp)) = self.chrono_components.get_mut(key)
                                {
                                    self.chrono_counter += 1;
                                    chrono_comp.t = self.chrono_counter;
                                }
                            }

                            return;
                        }
                    }
                }
            });
        }
    }
}
