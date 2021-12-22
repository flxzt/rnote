use crate::pens::eraser::Eraser;
use crate::strokes::strokestyle::{StrokeBehaviour, StrokeStyle};

use super::{StrokeKey, StrokesState};

use p2d::bounding_volume::BoundingVolume;
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
            self.regenerate_rendering_for_stroke_threaded(last_stroke_key);

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
            self.regenerate_rendering_for_stroke_threaded(last_trashed_key);

            Some(last_trashed_key)
        } else {
            None
        }
    }

    pub fn trash_selection(&mut self) {
        self.selection_components
            .iter_mut()
            .for_each(|(key, selection_comp)| {
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
            });
        self.selection_bounds = None;
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

            self.strokes.iter_mut().for_each(|(key, stroke)| {
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
                                    if let Some(trash_comp) = self.trash_components.get_mut(key) {
                                        trash_comp.trashed = true;

                                        if let Some(chrono_comp) =
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
                                    if let Some(trash_comp) = self.trash_components.get_mut(key) {
                                        trash_comp.trashed = true;

                                        if let Some(chrono_comp) =
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
                            if let Some(trash_comp) = self.trash_components.get_mut(key) {
                                trash_comp.trashed = true;

                                if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                                    self.chrono_counter += 1;
                                    chrono_comp.t = self.chrono_counter;
                                }
                            }

                            return;
                        }
                    }
                    StrokeStyle::VectorImage(_vectorimage) => {
                        // Ignore VectorImage when trashing with the Eraser
                    }
                    StrokeStyle::BitmapImage(_bitmapimage) => {
                        // Ignore BitmapImage when trashing with the Eraser
                    }
                }
            });
        }
    }
}
