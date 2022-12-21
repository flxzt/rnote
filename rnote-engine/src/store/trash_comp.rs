use super::{StrokeKey, StrokeStore};
use crate::strokes::{BrushStroke, Stroke};
use crate::WidgetFlags;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::PenPath;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

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

/// Systems that are related trashing
impl StrokeStore {
    /// Rebuilds the slotmap with default trash components from the keys returned from the primary map, stroke_components.
    pub fn rebuild_trash_components_slotmap(&mut self) {
        self.trash_components = Arc::new(slotmap::SecondaryMap::new());
        self.stroke_components.keys().for_each(|key| {
            Arc::make_mut(&mut self.trash_components)
                .insert(key, Arc::new(TrashComponent::default()));
        });
    }

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
        if let Some(trash_comp) = Arc::make_mut(&mut self.trash_components)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            trash_comp.trashed = trash;

            self.update_chrono_to_last(key);
        } else {
            log::debug!(
                "get trash_comp in set_trashed() returned None for stroke with key {:?}",
                key
            );
        }
    }

    pub fn set_trashed_keys(&mut self, keys: &[StrokeKey], trash: bool) {
        keys.iter().for_each(|&key| {
            self.set_selected(key, false);
            self.set_trashed(key, trash);
            self.update_chrono_to_last(key);
        });
    }

    pub fn trashed_keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components
            .keys()
            .filter(|&key| self.trashed(key).unwrap_or(false))
            .collect()
    }

    pub fn remove_trashed_strokes(&mut self) {
        for key in self.trashed_keys_unordered() {
            self.remove_stroke(key);
        }
    }

    /// trash strokes that collide with the given bounds
    pub fn trash_colliding_strokes(&mut self, eraser_bounds: Aabb, viewport: Aabb) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        self.stroke_keys_as_rendered_intersecting_bounds(viewport)
            .into_iter()
            .for_each(|key| {
                let mut trash_current_stroke = false;

                if let Some(stroke) = self.stroke_components.get(key) {
                    match stroke.as_ref() {
                        Stroke::BrushStroke(_) | Stroke::ShapeStroke(_) => {
                            // First check if eraser even intersects stroke bounds, avoiding unnecessary work
                            if eraser_bounds.intersects(&stroke.bounds()) {
                                for hitbox in stroke.hitboxes().into_iter() {
                                    if eraser_bounds.intersects(&hitbox) {
                                        trash_current_stroke = true;

                                        break;
                                    }
                                }
                            }
                        }
                        // Ignore other strokes when trashing with the Eraser
                        Stroke::TextStroke(_) | Stroke::VectorImage(_) | Stroke::BitmapImage(_) => {
                        }
                    }
                }

                if trash_current_stroke {
                    widget_flags.merge(self.record(Instant::now()));
                    self.set_trashed(key, true);
                }
            });

        widget_flags
    }

    /// remove colliding stroke segments with the given bounds. The stroke is then split. For strokes that don't have segments, trash the entire stroke.
    /// Returns the keys of all created or modified strokes.
    /// returned strokes need to update their rendering.
    pub fn split_colliding_strokes(
        &mut self,
        eraser_bounds: Aabb,
        viewport: Aabb,
    ) -> Vec<StrokeKey> {
        let mut modified_keys = vec![];

        let new_strokes = self
            .stroke_keys_as_rendered_intersecting_bounds(viewport)
            .into_iter()
            .flat_map(|key| {
                let stroke = match Arc::make_mut(&mut self.stroke_components)
                    .get_mut(key)
                    .map(Arc::make_mut)
                {
                    Some(stroke) => stroke,
                    None => return vec![],
                };

                let mut new_strokes = vec![];
                let mut trash_current_stroke = false;
                let stroke_bounds = stroke.bounds();

                match stroke {
                    Stroke::BrushStroke(brushstroke) => {
                        if eraser_bounds.intersects(&stroke_bounds) {
                            if let Some(split_at) = brushstroke
                                .path
                                .hittest(&eraser_bounds, brushstroke.style.stroke_width() * 0.5)
                            {
                                let (first_split, second_split) =
                                    brushstroke.path.segments[..].split_at(split_at);
                                let first_split = first_split.to_vec();
                                // We want to exclude the colliding segment, so +1
                                let second_split =
                                    second_split[1.min(second_split.len())..].to_vec();

                                let first_empty = first_split.is_empty();
                                let second_empty = second_split.is_empty()
                                    || split_at == second_split.len().saturating_sub(1);

                                //log::debug!("split stroke, first_empty: {first_empty}, second_empty: {second_empty}, split_i: {split_at}");

                                match (first_empty, second_empty) {
                                    (false, false) => {
                                        // the first split is the original path until the hit, so we only need to replace the segments and can keep start
                                        let first_start = brushstroke.path.start;
                                        let second_start = first_split.last().unwrap().end();

                                        let first_split = first_split.to_vec();
                                        brushstroke.replace_path(PenPath::new_w_segments(
                                            first_start,
                                            first_split,
                                        ));
                                        modified_keys.push(key);

                                        new_strokes.push(Stroke::BrushStroke(
                                            BrushStroke::from_penpath(
                                                PenPath::new_w_segments(
                                                    second_start,
                                                    second_split.to_vec(),
                                                ),
                                                brushstroke.style.clone(),
                                            ),
                                        ));
                                    }
                                    (false, true) => {
                                        brushstroke.replace_path(PenPath::new_w_segments(
                                            brushstroke.path.start,
                                            first_split.to_vec(),
                                        ));
                                        modified_keys.push(key);
                                    }
                                    (true, false) => {
                                        let new_start = second_split.first().unwrap().end();
                                        brushstroke.replace_path(PenPath::new_w_segments(
                                            new_start,
                                            second_split.to_vec(),
                                        ));
                                        modified_keys.push(key);
                                    }
                                    (true, true) => {
                                        trash_current_stroke = true;
                                    }
                                }
                            }
                        }
                    }
                    Stroke::ShapeStroke(_) => {
                        if eraser_bounds.intersects(&stroke_bounds) {
                            for hitbox_elem in stroke.hitboxes().iter() {
                                if eraser_bounds.intersects(hitbox_elem) {
                                    trash_current_stroke = true;
                                }
                            }
                        }
                    }
                    // Ignore other strokes when trashing with the Eraser
                    Stroke::TextStroke(_) | Stroke::VectorImage(_) | Stroke::BitmapImage(_) => {}
                }

                if trash_current_stroke {
                    self.set_trashed(key, true);
                }

                new_strokes
            })
            .collect::<Vec<Stroke>>();

        modified_keys.append(
            &mut new_strokes
                .into_iter()
                .map(|new_stroke| self.insert_stroke(new_stroke, None))
                .collect(),
        );

        modified_keys
    }
}
