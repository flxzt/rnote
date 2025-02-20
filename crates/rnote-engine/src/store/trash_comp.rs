// Imports
use super::chrono_comp::StrokeLayer;
use super::{StrokeKey, StrokeStore};
use crate::WidgetFlags;
use crate::strokes::{BrushStroke, Stroke};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::PenPath;
use rnote_compose::shapes::Shapeable;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

/// Systems that are related to trashing.
impl StrokeStore {
    /// Rebuild the slotmap with empty trash components with the keys returned from the stroke components.
    pub(crate) fn rebuild_trash_components_slotmap(&mut self) {
        self.trash_components = Arc::new(slotmap::SecondaryMap::new());
        self.stroke_components.keys().for_each(|key| {
            Arc::make_mut(&mut self.trash_components)
                .insert(key, Arc::new(TrashComponent::default()));
        });
    }

    /// Ability if trashing is supported.
    #[allow(unused)]
    pub(crate) fn can_trash(&self, key: StrokeKey) -> bool {
        self.trash_components.get(key).is_some()
    }

    pub(crate) fn trashed(&self, key: StrokeKey) -> Option<bool> {
        self.trash_components.get(key).map(|t| t.trashed)
    }

    pub(crate) fn set_trashed(&mut self, key: StrokeKey, trash: bool) {
        if let Some(trash_comp) = Arc::make_mut(&mut self.trash_components)
            .get_mut(key)
            .map(Arc::make_mut)
        {
            trash_comp.trashed = trash;
            self.update_chrono_to_last(key);
        }
    }

    pub(crate) fn set_trashed_keys(&mut self, keys: &[StrokeKey], trash: bool) {
        keys.iter().for_each(|&key| {
            self.set_selected(key, false);
            self.set_trashed(key, trash);
            self.update_chrono_to_last(key);
        });
    }

    pub(crate) fn trashed_keys_unordered(&self) -> Vec<StrokeKey> {
        self.stroke_components
            .keys()
            .filter(|&key| self.trashed(key).unwrap_or(false))
            .collect()
    }

    /// Removes all trashed strokes permanently from the store.
    #[allow(unused)]
    pub(crate) fn remove_trashed_strokes(&mut self) -> Vec<Stroke> {
        self.trashed_keys_unordered()
            .into_iter()
            .filter_map(|k| self.remove_stroke(k))
            .collect()
    }

    /// Trash strokes that collide with the given bounds.
    pub(crate) fn trash_colliding_strokes(
        &mut self,
        eraser_bounds: Aabb,
        viewport: Aabb,
    ) -> WidgetFlags {
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
                    self.set_trashed(key, true);
                    widget_flags.store_modified = true;
                    widget_flags.resize = true;
                }
            });

        widget_flags
    }

    /// Remove colliding stroke segments with the given bounds.
    /// The stroke is then split. Strokes that don't have segments are trashed completely.
    ///
    /// Returns the keys of all created or modified strokes.
    ///
    /// The returned strokes need to update their rendering.
    pub(crate) fn split_colliding_strokes(
        &mut self,
        eraser_bounds: Aabb,
        viewport: Aabb,
    ) -> (Vec<StrokeKey>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();
        let mut modified_keys = vec![];

        let new_strokes = self
            .stroke_keys_as_rendered_intersecting_bounds(viewport)
            .into_iter()
            .flat_map(|key| {
                let Some(stroke) = Arc::make_mut(&mut self.stroke_components)
                    .get_mut(key)
                    .map(Arc::make_mut)
                else {
                    return vec![];
                };

                let Some(chrono_comp) = self.chrono_components.get(key) else {
                    return vec![];
                };

                let mut new_strokes = vec![];
                let mut trash_current_stroke = false;
                let stroke_bounds = stroke.bounds();

                match stroke {
                    Stroke::BrushStroke(brushstroke) => {
                        if eraser_bounds.intersects(&stroke_bounds) {
                            let mut split = Vec::new();

                            let mut hits = brushstroke
                                .path
                                .hittest(&eraser_bounds, brushstroke.style.stroke_width() * 0.5)
                                .into_iter();

                            if let Some(first_hit) = hits.next() {
                                let mut prev = first_hit;
                                for hit in hits {
                                    let split_slice = &brushstroke.path.segments[prev..hit];

                                    // skip splits that don't have at least two segments (one's end as path start, one additional)
                                    if split_slice.len() > 1 {
                                        split.push(split_slice.to_vec());
                                    }

                                    prev = hit;
                                }

                                // Catch the last
                                let last_split = &brushstroke.path.segments[prev..];
                                if last_split.len() > 1 {
                                    split.push(last_split.to_vec());
                                }

                                for next_split in split {
                                    let mut next_split_iter = next_split.into_iter();
                                    let next_start = next_split_iter.next().unwrap().end();

                                    new_strokes.push((
                                        Stroke::BrushStroke(BrushStroke::from_penpath(
                                            PenPath::new_w_segments(next_start, next_split_iter),
                                            brushstroke.style.clone(),
                                        )),
                                        chrono_comp.layer,
                                    ));
                                }

                                let first_split = &brushstroke.path.segments[..first_hit];
                                // Modify the original stroke at the end.
                                // We keep the start, so we only need at least one segment
                                if !first_split.is_empty() {
                                    brushstroke.replace_path(PenPath::new_w_segments(
                                        brushstroke.path.start,
                                        first_split.to_vec(),
                                    ));
                                } else {
                                    trash_current_stroke = true;
                                }

                                modified_keys.push(key);
                            }
                        }
                    }
                    Stroke::ShapeStroke(_) => {
                        if eraser_bounds.intersects(&stroke_bounds) {
                            for hitbox_elem in stroke.hitboxes().iter() {
                                if eraser_bounds.intersects(hitbox_elem) {
                                    trash_current_stroke = true;
                                    modified_keys.push(key);
                                }
                            }
                        }
                    }
                    // Ignore other strokes when trashing with the Eraser
                    Stroke::TextStroke(_) | Stroke::VectorImage(_) | Stroke::BitmapImage(_) => {}
                }

                if trash_current_stroke {
                    self.set_trashed(key, true);
                    widget_flags.store_modified = true;
                    widget_flags.resize = true;
                }

                new_strokes
            })
            .collect::<Vec<(Stroke, StrokeLayer)>>();

        modified_keys.append(
            &mut new_strokes
                .into_iter()
                .map(|(new_stroke, layer)| self.insert_stroke(new_stroke, Some(layer)))
                .collect(),
        );

        if !modified_keys.is_empty() {
            widget_flags.store_modified = true;
            widget_flags.resize = true;
        }

        (modified_keys, widget_flags)
    }
}
