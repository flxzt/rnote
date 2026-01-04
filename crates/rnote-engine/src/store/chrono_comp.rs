// Imports
use super::{StrokeKey, StrokeStore};
use p2d::bounding_volume::Aabb;
use rayon::slice::ParallelSliceMut;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq)]
#[serde(rename = "stroke_layer")]
pub enum StrokeLayer {
    #[serde(rename = "user_layer", alias = "UserLayer")]
    UserLayer(u32),
    #[serde(rename = "highlighter", alias = "Highlighter")]
    Highlighter,
    #[serde(rename = "image", alias = "Image")]
    Image,
    #[serde(rename = "document", alias = "Document")]
    Document,
}

impl Default for StrokeLayer {
    fn default() -> Self {
        Self::UserLayer(0)
    }
}

impl PartialEq for StrokeLayer {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UserLayer(l0), Self::UserLayer(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialOrd for StrokeLayer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrokeLayer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (StrokeLayer::UserLayer(this_ul), StrokeLayer::UserLayer(other_ul)) => {
                this_ul.cmp(other_ul)
            }
            (StrokeLayer::UserLayer(_), _) => Ordering::Greater,
            (StrokeLayer::Highlighter, StrokeLayer::UserLayer(_)) => Ordering::Less,
            (StrokeLayer::Highlighter, StrokeLayer::Highlighter) => Ordering::Equal,
            (StrokeLayer::Highlighter, _) => Ordering::Greater,
            (StrokeLayer::Image, StrokeLayer::UserLayer(_) | StrokeLayer::Highlighter) => {
                Ordering::Less
            }
            (StrokeLayer::Image, StrokeLayer::Image) => Ordering::Equal,
            (StrokeLayer::Image, StrokeLayer::Document) => Ordering::Greater,
            (StrokeLayer::Document, StrokeLayer::Document) => Ordering::Equal,
            (StrokeLayer::Document, _) => Ordering::Less,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(default, rename = "chrono_component")]
pub struct ChronoComponent {
    #[serde(rename = "t")]
    t: u32,
    #[serde(rename = "layer")]
    pub layer: StrokeLayer,
}

impl Default for ChronoComponent {
    fn default() -> Self {
        Self {
            t: 0,
            layer: StrokeLayer::default(),
        }
    }
}

impl ChronoComponent {
    pub(crate) fn new(t: u32, layer: StrokeLayer) -> Self {
        Self { t, layer }
    }
}

/// Systems that are related to their chronological ordering.
impl StrokeStore {
    pub(crate) fn update_chrono_to_last(&mut self, key: StrokeKey) {
        if let Some(chrono_comp) = Arc::make_mut(&mut self.chrono_components).get_mut(key) {
            self.chrono_counter += 1;
            Arc::make_mut(chrono_comp).t = self.chrono_counter;
        }
    }

    /// Returns the keys in chronological order, as in first: gets drawn first, last: gets drawn last.
    pub(crate) fn keys_sorted_chrono(&self) -> Vec<StrokeKey> {
        let chrono_components = &self.chrono_components;

        let mut keys = self.stroke_components.keys().collect::<Vec<StrokeKey>>();

        keys.par_sort_unstable_by(|&first, &second| {
            if let Some(first_chrono) = chrono_components.get(first)
                && let Some(second_chrono) = chrono_components.get(second)
            {
                let layer_order = first_chrono.layer.cmp(&second_chrono.layer);

                if layer_order != std::cmp::Ordering::Equal {
                    layer_order
                } else {
                    first_chrono.t.cmp(&second_chrono.t)
                }
            } else {
                std::cmp::Ordering::Equal
            }
        });

        keys
    }

    pub(crate) fn keys_sorted_chrono_intersecting_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        let chrono_components = &self.chrono_components;

        let mut keys = self.key_tree.keys_intersecting_bounds(bounds);

        keys.par_sort_unstable_by(|&first, &second| {
            if let Some(first_chrono) = chrono_components.get(first)
                && let Some(second_chrono) = chrono_components.get(second)
            {
                let layer_order = first_chrono.layer.cmp(&second_chrono.layer);

                if layer_order != std::cmp::Ordering::Equal {
                    layer_order
                } else {
                    first_chrono.t.cmp(&second_chrono.t)
                }
            } else {
                std::cmp::Ordering::Equal
            }
        });

        keys
    }

    pub(crate) fn keys_bounds_sorted_chrono_intersecting_bounds(
        &self,
        bounds: Aabb,
    ) -> Vec<(StrokeKey, Aabb)> {
        let chrono_components = &self.chrono_components;

        let mut keys = self.key_tree.keys_bounds_intersecting_bounds(bounds);

        keys.par_sort_unstable_by(|&(first, ..), &(second, ..)| {
            if let (Some(first_chrono), Some(second_chrono)) =
                (chrono_components.get(first), chrono_components.get(second))
            {
                let layer_order = first_chrono.layer.cmp(&second_chrono.layer);

                if layer_order != std::cmp::Ordering::Equal {
                    layer_order
                } else {
                    first_chrono.t.cmp(&second_chrono.t)
                }
            } else {
                std::cmp::Ordering::Equal
            }
        });

        keys
    }

    pub(crate) fn keys_sorted_chrono_in_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        let chrono_components = &self.chrono_components;

        let mut keys = self.key_tree.keys_in_bounds(bounds);

        keys.par_sort_unstable_by(|&first, &second| {
            if let Some(first_chrono) = chrono_components.get(first)
                && let Some(second_chrono) = chrono_components.get(second)
            {
                let layer_order = first_chrono.layer.cmp(&second_chrono.layer);

                if layer_order != std::cmp::Ordering::Equal {
                    layer_order
                } else {
                    first_chrono.t.cmp(&second_chrono.t)
                }
            } else {
                std::cmp::Ordering::Equal
            }
        });

        keys
    }
}
