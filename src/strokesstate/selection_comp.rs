use super::{StrokeKey, StrokeStyle, StrokesState};
use crate::compose::geometry;
use crate::drawbehaviour::DrawBehaviour;
use crate::pens::selector::{self, Selector};
use crate::{compose, render};

use geo::line_string;
use geo::prelude::*;
use gtk4::{gio, glib, prelude::*};
use p2d::bounding_volume::BoundingVolume;
use rayon::prelude::*;
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

impl StrokesState {
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

            self.update_selection_bounds();
        } else {
            log::debug!(
                "get selection_comp in set_selected() returned None for stroke with key {:?}",
                key
            );
        }
    }

    /// Returns all keys for the selection
    pub fn selection_keys(&self) -> Vec<StrokeKey> {
        self.keys_sorted_chrono()
            .iter()
            .filter_map(|&key| {
                let selection_comp = self.selection_components.get(key)?;

                if selection_comp.selected {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub fn selection_len(&self) -> usize {
        self.selection_keys().len()
    }

    pub fn update_selection_bounds(&mut self) {
        self.selection_bounds = self.gen_bounds(&self.selection_keys());
    }

    pub fn deselect_all_strokes(&mut self) {
        self.keys_sorted_chrono().iter().for_each(|&key| {
            if let Some(selection_comp) = self.selection_components.get_mut(key) {
                if selection_comp.selected {
                    if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                        self.chrono_counter += 1;
                        chrono_comp.t = self.chrono_counter;
                    }
                    selection_comp.selected = false;
                }
            }
        });

        self.selection_bounds = None;
    }

    pub fn duplicate_selection(&mut self) {
        let offset = na::vector![
            SelectionComponent::SELECTION_DUPLICATION_OFFSET_X,
            SelectionComponent::SELECTION_DUPLICATION_OFFSET_Y
        ];

        let selected = self.selection_keys();
        self.deselect_all_strokes();

        selected.iter().for_each(|&key| {
            let new_key = self.insert_stroke(self.strokes.get(key).unwrap().clone());
            self.set_selected(new_key, true);
        });

        // Offsetting the new selected stroke to make the duplication apparent to the user
        self.translate_strokes(&selected, offset);
        self.update_selection_bounds();
    }

    /// Returns true if selection has changed
    pub fn update_selection_for_selector(
        &mut self,
        selector: &Selector,
        viewport: Option<p2d::bounding_volume::AABB>,
    ) -> bool {
        let selection_len_prev = self.selection_len();

        let selector_polygon = match selector.style() {
            selector::SelectorStyle::Polygon => {
                let selector_path_points = selector
                    .path
                    .par_iter()
                    .map(|inputdata| geo::Coordinate {
                        x: inputdata.pos()[0],
                        y: inputdata.pos()[1],
                    })
                    .collect::<Vec<geo::Coordinate<f64>>>();

                geo::Polygon::new(selector_path_points.into(), vec![])
            }
            selector::SelectorStyle::Rectangle => {
                if let (Some(first), Some(last)) = (selector.path.first(), selector.path.last()) {
                    let selector_path_points = line_string![
                        (x: first.pos()[0], y: first.pos()[1]),
                        (x: first.pos()[0], y: last.pos()[1]),
                        (x: last.pos()[0], y: last.pos()[1]),
                        (x: last.pos()[0], y: first.pos()[1]),
                        (x: first.pos()[0], y: first.pos()[1]),
                    ];

                    geo::Polygon::new(selector_path_points, vec![])
                } else {
                    return false;
                }
            }
        };

        self.keys_sorted_chrono().iter().for_each(|&key| {
            let stroke = if let Some(stroke) = self.strokes.get(key) {
                stroke
            } else {
                return;
            };
            // skip if stroke is trashed
            if let Some(trash_comp) = self.trash_components.get(key) {
                if trash_comp.trashed {
                    return;
                }
            }
            // skip if stroke is not in viewport
            if let Some(viewport) = viewport {
                if !viewport.intersects(&stroke.bounds()) {
                    return;
                }
            }
            if let Some(selection_comp) = self.selection_components.get_mut(key) {
                // default to not selected, check for if selected
                selection_comp.selected = false;

                match stroke {
                    StrokeStyle::MarkerStroke(markerstroke) => {
                        if selector_polygon
                            .contains(&geometry::p2d_aabb_to_geo_polygon(markerstroke.bounds))
                        {
                            selection_comp.selected = true;
                        } else if selector_polygon
                            .contains(&geometry::p2d_aabb_to_geo_polygon(markerstroke.bounds))
                        {
                            for &hitbox_elem in markerstroke.hitbox.iter() {
                                if !selector_polygon
                                    .contains(&geometry::p2d_aabb_to_geo_polygon(hitbox_elem))
                                {
                                    return;
                                }
                            }
                            selection_comp.selected = true;

                            if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                                self.chrono_counter += 1;
                                chrono_comp.t = self.chrono_counter;
                            }
                        }
                    }
                    StrokeStyle::BrushStroke(brushstroke) => {
                        if selector_polygon
                            .contains(&geometry::p2d_aabb_to_geo_polygon(brushstroke.bounds))
                        {
                            selection_comp.selected = true;
                        } else if selector_polygon
                            .contains(&geometry::p2d_aabb_to_geo_polygon(brushstroke.bounds))
                        {
                            for &hitbox_elem in brushstroke.hitboxes.iter() {
                                if !selector_polygon
                                    .contains(&geometry::p2d_aabb_to_geo_polygon(hitbox_elem))
                                {
                                    return;
                                }
                            }
                            selection_comp.selected = true;

                            if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                                self.chrono_counter += 1;
                                chrono_comp.t = self.chrono_counter;
                            }
                        }
                    }
                    StrokeStyle::ShapeStroke(shapestroke) => {
                        if selector_polygon
                            .contains(&geometry::p2d_aabb_to_geo_polygon(shapestroke.bounds))
                        {
                            selection_comp.selected = true;

                            if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                                self.chrono_counter += 1;
                                chrono_comp.t = self.chrono_counter;
                            }
                        }
                    }
                    StrokeStyle::VectorImage(vectorimage) => {
                        if selector_polygon
                            .contains(&geometry::p2d_aabb_to_geo_polygon(vectorimage.bounds))
                        {
                            selection_comp.selected = true;

                            if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                                self.chrono_counter += 1;
                                chrono_comp.t = self.chrono_counter;
                            }
                        }
                    }
                    StrokeStyle::BitmapImage(bitmapimage) => {
                        if selector_polygon
                            .contains(&geometry::p2d_aabb_to_geo_polygon(bitmapimage.bounds))
                        {
                            selection_comp.selected = true;

                            if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                                self.chrono_counter += 1;
                                chrono_comp.t = self.chrono_counter;
                            }
                        }
                    }
                }
            }
        });

        if self.selection_len() != selection_len_prev {
            self.update_selection_bounds();
            self.regenerate_rendering_for_selection_threaded();
            true
        } else {
            false
        }
    }

    /// the svgs of the current selection, without xml header or svg root
    pub fn gen_svgs_selection(&self) -> Result<Vec<render::Svg>, anyhow::Error> {
        if self.selection_bounds.is_none() {
            return Ok(vec![]);
        }

        Ok(self
            .keys_sorted_chrono()
            .iter()
            .filter(|&&key| {
                self.does_render(key).unwrap_or(false)
                    && !(self.trashed(key).unwrap_or(false))
                    && (self.selected(key).unwrap_or(false))
                    && (self.does_render(key).unwrap_or(false))
            })
            .filter_map(|&key| {
                let stroke = self.strokes.get(key)?;

                stroke.gen_svgs(na::vector![0.0, 0.0]).ok()
            })
            .flatten()
            .collect::<Vec<render::Svg>>())
    }

    pub fn export_selection_as_svg(&self, file: gio::File) -> Result<(), anyhow::Error> {
        let selection_svgs = self.gen_svgs_selection()?;

        let mut svg_data = selection_svgs
            .iter()
            .map(|svg| svg.svg_data.as_str())
            .collect::<Vec<&str>>()
            .join("\n");

        let selection_bounds = if let Some(selection_bounds) = self.selection_bounds {
            selection_bounds
        } else {
            return Ok(());
        };
        svg_data = compose::wrap_svg_root(
            svg_data.as_str(),
            Some(selection_bounds),
            Some(selection_bounds),
            true,
        );

        file.replace_async(
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
            glib::PRIORITY_HIGH_IDLE,
            None::<&gio::Cancellable>,
            move |result| {
                let output_stream = match result {
                    Ok(output_stream) => output_stream,
                    Err(e) => {
                        log::error!(
                            "replace_async() failed in export_selection_as_svg() with Err {}",
                            e
                        );
                        return;
                    }
                };

                if let Err(e) = output_stream.write(svg_data.as_bytes(), None::<&gio::Cancellable>)
                {
                    log::error!(
                        "output_stream().write() failed in export_selection_as_svg() with Err {}",
                        e
                    );
                };
                if let Err(e) = output_stream.close(None::<&gio::Cancellable>) {
                    log::error!(
                        "output_stream().close() failed in export_selection_as_svg() with Err {}",
                        e
                    );
                };
            },
        );

        Ok(())
    }
}
