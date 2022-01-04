use crate::pens::selector::{self, Selector};
use crate::{compose, geometry, render};

use super::{StrokeKey, StrokeStyle, StrokesState};
use crate::strokes::strokestyle::StrokeBehaviour;

use geo::line_string;
use geo::prelude::*;
use gtk4::{gio, prelude::*};
use p2d::bounding_volume::BoundingVolume;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SelectionComponent {
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

            if selected {
                if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                    self.chrono_counter += 1;
                    chrono_comp.t = self.chrono_counter;
                }
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
    pub fn keys_selection(&self) -> Vec<StrokeKey> {
        self.selection_components
            .iter()
            .par_bridge()
            .filter_map(|(key, selection_comp)| {
                if selection_comp.selected {
                    Some(key)
                } else {
                    None
                }
            })
            .collect::<Vec<StrokeKey>>()
    }

    pub fn selection_len(&self) -> usize {
        self.keys_selection().len()
    }

    pub fn update_selection_bounds(&mut self) {
        self.selection_bounds = self.gen_bounds(&self.keys_selection());
    }

    pub fn deselect_all_strokes(&mut self) {
        self.selection_components
            .iter_mut()
            .for_each(|(key, selection_comp)| {
                selection_comp.selected = false;

                if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
                    self.chrono_counter += 1;
                    chrono_comp.t = self.chrono_counter;
                }
            });

        self.selection_bounds = None;
    }

    pub fn duplicate_selection(&mut self) {
        let offset = na::vector![
            SelectionComponent::SELECTION_DUPLICATION_OFFSET_X,
            SelectionComponent::SELECTION_DUPLICATION_OFFSET_Y
        ];

        let selected = self.keys_selection();
        self.deselect_all_strokes();

        selected.iter().for_each(|&key| {
            let new_key = self.insert_stroke(self.strokes.get(key).unwrap().clone());
            self.set_selected(new_key, true);
        });

        // Offsetting the new selected stroke to make the duplication apparent to the user
        self.translate_selection(offset);
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

        self.strokes.iter_mut().for_each(|(key, stroke)| {
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
                            for &hitbox_elem in brushstroke.hitbox.iter() {
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

    /// Resizing the selection with its contents to the new bounds. Stroke rendering regeneration is needed when resizing is finished.
    pub fn resize_selection(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        if let Some(selection_bounds) = self.selection_bounds {
            self.strokes.iter_mut().for_each(|(key, stroke)| {
                if let Some(selection_comp) = self.selection_components.get(key) {
                    if selection_comp.selected {
                        let old_stroke_bounds = stroke.bounds();
                        let new_stroke_bounds = geometry::scale_inner_bounds_to_new_outer_bounds(
                            stroke.bounds(),
                            selection_bounds,
                            new_bounds,
                        );
                        stroke.resize(new_stroke_bounds);

                        if let Some(render_comp) = self.render_components.get_mut(key) {
                            for image in render_comp.images.iter_mut() {
                                image.bounds = geometry::scale_inner_bounds_to_new_outer_bounds(
                                    image.bounds,
                                    old_stroke_bounds,
                                    new_stroke_bounds,
                                )
                            }

                            if let Some(new_rendernode) =
                                render::images_to_rendernode(&render_comp.images, self.zoom)
                            {
                                render_comp.rendernode = new_rendernode;
                            }
                            render_comp.regenerate_flag = true;
                        }
                    }
                }
            });

            self.selection_bounds = Some(new_bounds);
        }
    }

    /// Translate the selection with its contents with an offset relative to the current position
    pub fn translate_selection(&mut self, offset: na::Vector2<f64>) {
        self.strokes.iter_mut().for_each(|(key, stroke)| {
            if let Some(selection_comp) = self.selection_components.get(key) {
                if selection_comp.selected {
                    stroke.translate(offset);

                    if let Some(render_comp) = self.render_components.get_mut(key) {
                        for image in render_comp.images.iter_mut() {
                            image.bounds = geometry::aabb_translate(image.bounds, offset);
                        }

                        if let Some(new_rendernode) =
                            render::images_to_rendernode(&render_comp.images, self.zoom)
                        {
                            render_comp.rendernode = new_rendernode;
                        }
                    }
                }
            }
        });

        self.selection_bounds = self
            .selection_bounds
            .map(|selection_bounds| geometry::aabb_translate(selection_bounds, offset));
    }

    pub fn gen_svg_selection(&self) -> Result<Option<String>, anyhow::Error> {
        if let Some(selection_bounds) = self.selection_bounds {
            let mut data = self
                .keys_selection()
                .iter()
                .filter_map(|key| self.strokes.get(*key))
                .filter_map(|stroke| {
                    stroke
                        .gen_svgs(na::vector![
                            -selection_bounds.mins[0],
                            -selection_bounds.mins[1]
                        ])
                        .ok()
                })
                .flatten()
                .map(|svg| svg.svg_data)
                .collect::<Vec<String>>()
                .join("\n");

            let wrapper_bounds = p2d::bounding_volume::AABB::new(
                na::point![0.0, 0.0],
                na::point![
                    selection_bounds.maxs[0] - selection_bounds.mins[0],
                    selection_bounds.maxs[1] - selection_bounds.mins[1]
                ],
            );
            data = compose::wrap_svg(
                data.as_str(),
                Some(wrapper_bounds),
                Some(wrapper_bounds),
                true,
                false,
            );

            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub fn export_selection_as_svg(&self, file: gio::File) -> Result<(), anyhow::Error> {
        if let Some(data) = self.gen_svg_selection()? {
            let output_stream = file.replace::<gio::Cancellable>(
                None,
                false,
                gio::FileCreateFlags::REPLACE_DESTINATION,
                None,
            )?;
            output_stream.write::<gio::Cancellable>(data.as_bytes(), None)?;
            output_stream.close::<gio::Cancellable>(None)?;
        }

        Ok(())
    }
}
