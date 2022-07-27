use super::{StrokeKey, StrokeStore};
use crate::strokes::{BrushStroke, Stroke};
use crate::WidgetFlags;

use p2d::bounding_volume::{BoundingVolume, AABB};
use rnote_compose::penpath::Segment;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::PenPath;
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

/// Systems that are related trashing
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
    pub fn trash_colliding_strokes(&mut self, eraser_bounds: AABB, viewport: AABB) -> WidgetFlags {
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
                        Stroke::TextStroke(_textstroke) => {
                            // Ignore text strokes when trashing with the Eraser
                        }
                        Stroke::VectorImage(_vectorimage) => {
                            // Ignore vector images when trashing with the Eraser
                        }
                        Stroke::BitmapImage(_bitmapimage) => {
                            // Ignore bitmap images when trashing with the Eraser
                        }
                    }
                }

                if trash_current_stroke {
                    widget_flags.merge_with_other(self.record());
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
        eraser_bounds: AABB,
        viewport: AABB,
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
                            let stroke_width = brushstroke.style.stroke_width();
                            brushstroke.path.make_contiguous();

                            let split_segments = brushstroke
                                .path
                                .as_slices()
                                .0
                                .split(|segment| {
                                    segment.hitboxes().iter().any(|hitbox| {
                                        // The hitboxes of the individual segments need to be loosened with the style stroke width
                                        hitbox
                                            .loosened(stroke_width * 0.5)
                                            .intersects(&eraser_bounds)
                                    })
                                })
                                .collect::<Vec<&[Segment]>>();

                            // If this is met, we intersect with the stroke but we cant form any new split strokes, so we trash it ( e.g. strokes that only have a single element )
                            if split_segments.iter().all(|segments| segments.is_empty()) {
                                trash_current_stroke = true;
                            } else {
                                // Filter out all empty paths
                                let mut split_penpaths = split_segments
                                    .into_iter()
                                    .filter_map(|segments| {
                                        let split_penpath =
                                            PenPath::from_iter(segments.iter().cloned());

                                        if split_penpath.is_empty() {
                                            None
                                        } else {
                                            Some(split_penpath)
                                        }
                                    })
                                    .collect::<Vec<PenPath>>();

                                if let Some(last_penpath) = split_penpaths.pop() {
                                    for split_penpath in split_penpaths {
                                        if let Some(new_brushstroke) = BrushStroke::from_penpath(
                                            split_penpath,
                                            brushstroke.style.clone(),
                                        ) {
                                            new_strokes.push(Stroke::BrushStroke(new_brushstroke));
                                        }
                                    }

                                    // reusing the current brushstroke by replacing its path with the last new path
                                    brushstroke.replace_path(last_penpath);
                                    modified_keys.push(key);
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
                    Stroke::TextStroke(_textstroke) => {
                        // Ignore text strokes when trashing with the Eraser
                    }
                    Stroke::VectorImage(_vectorimage) => {
                        // Ignore vector images when trashing with the Eraser
                    }
                    Stroke::BitmapImage(_bitmapimage) => {
                        // Ignore bitmap images when trashing with the Eraser
                    }
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
