// Imports
use super::{StrokeKey, StrokeStore};
use p2d::bounding_volume::Aabb;
use rayon::slice::ParallelSliceMut;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Clone, Copy, Serialize, Deserialize, Eq)]
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

impl std::fmt::Debug for StrokeLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserLayer(arg0) => f.debug_tuple("UL").field(arg0).finish(),
            Self::Highlighter => write!(f, "HL"),
            Self::Image => write!(f, "IMG"),
            Self::Document => write!(f, "DOC"),
        }
    }
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
            (StrokeLayer::Image, _) => Ordering::Greater,
            (StrokeLayer::Document, StrokeLayer::Document) => Ordering::Equal,
            (StrokeLayer::Document, _) => Ordering::Less,
        }
    }
}

impl StrokeLayer {
    fn user_up(self) -> Self {
        match self {
            Self::UserLayer(ul) => Self::UserLayer(ul.saturating_add(1)),
            _ => Self::UserLayer(0),
        }
    }

    fn user_down(self) -> Self {
        match self {
            // Only apply in user layers, never go to the predetermined layers
            Self::UserLayer(ul) => Self::UserLayer(ul.saturating_sub(1)),
            _ => self,
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

    pub(crate) fn keys_sorted_chrono_intersecting_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        let chrono_components = &self.chrono_components;

        let mut keys = self.key_tree.keys_intersecting_bounds(bounds);

        keys.par_sort_unstable_by(|&first, &second| {
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

    pub(crate) fn highest_layer(
        &self,
        keys: &[StrokeKey],
    ) -> Option<(Vec<StrokeKey>, StrokeLayer)> {
        let chrono_components = &self.chrono_components;

        let highest_layer_first_key = keys.into_iter().reduce(|first, second| {
            let (Some(first_chrono), Some(second_chrono)) = (
                chrono_components.get(*first),
                chrono_components.get(*second),
            ) else {
                return first;
            };

            match first_chrono.layer.cmp(&second_chrono.layer) {
                Ordering::Less => return second,
                Ordering::Equal => match first_chrono.t.cmp(&second_chrono.t) {
                    Ordering::Less => return second,
                    Ordering::Equal => return second,
                    Ordering::Greater => return first,
                },
                Ordering::Greater => return first,
            }
        })?;
        let highest_layer = chrono_components
            .get(*highest_layer_first_key)
            .map(|comp| comp.layer)?;

        let highest_layer_keys = keys
            .into_iter()
            .filter_map(|&key| {
                let chrono_comp = chrono_components.get(key)?;

                if chrono_comp.layer < highest_layer {
                    return None;
                }
                Some(key)
            })
            .collect();

        Some((highest_layer_keys, highest_layer))
    }

    pub(crate) fn lowest_layer(&self, keys: &[StrokeKey]) -> Option<(Vec<StrokeKey>, StrokeLayer)> {
        let chrono_components = &self.chrono_components;

        let lowest_layer_first_key = keys.into_iter().reduce(|first, second| {
            let (Some(first_chrono), Some(second_chrono)) = (
                chrono_components.get(*first),
                chrono_components.get(*second),
            ) else {
                return first;
            };

            match first_chrono.layer.cmp(&second_chrono.layer) {
                Ordering::Less => return first,
                Ordering::Equal => match first_chrono.t.cmp(&second_chrono.t) {
                    Ordering::Less => return first,
                    Ordering::Equal => return first,
                    Ordering::Greater => return second,
                },
                Ordering::Greater => return second,
            }
        })?;
        let lowest_layer = chrono_components
            .get(*lowest_layer_first_key)
            .map(|comp| comp.layer)?;

        let lowest_layer_keys = keys
            .into_iter()
            .filter_map(|&key| {
                let chrono_comp = chrono_components.get(key)?;

                if chrono_comp.layer > lowest_layer {
                    return None;
                }
                Some(key)
            })
            .collect();

        Some((lowest_layer_keys, lowest_layer))
    }

    pub(crate) fn move_layer_up(&mut self, keys: &[StrokeKey]) {
        let Some((highest_layer_keys, highest_layer)) = self.highest_layer(&self.keys_unordered())
        else {
            return;
        };

        if crate::utils::iterators_contain_same_items(highest_layer_keys.iter(), keys.iter()) {
            return;
        }

        keys.iter().for_each(|&key| {
            let Some(chrono_comp) = Arc::make_mut(&mut self.chrono_components).get_mut(key) else {
                return;
            };
            let layer = chrono_comp.layer;

            if layer <= highest_layer {
                Arc::make_mut(chrono_comp).layer = layer.user_up();
            }
        });
    }

    pub(crate) fn move_layer_down(&mut self, keys: &[StrokeKey]) {
        keys.iter().for_each(|&key| {
            let Some(chrono_comp) = Arc::make_mut(&mut self.chrono_components).get_mut(key) else {
                return;
            };
            let layer = chrono_comp.layer;
            Arc::make_mut(chrono_comp).layer = layer.user_down();
        });
    }

    pub(crate) fn move_layer_highest(&mut self, keys: &[StrokeKey]) {
        let Some((highest_layer_keys, highest_layer)) = self.highest_layer(&self.keys_unordered())
        else {
            return;
        };

        if crate::utils::iterators_contain_same_items(highest_layer_keys.iter(), keys.iter()) {
            return;
        }

        keys.iter().for_each(|&key| {
            let Some(chrono_comp) = Arc::make_mut(&mut self.chrono_components).get_mut(key) else {
                return;
            };
            let layer = chrono_comp.layer;

            if layer <= highest_layer {
                Arc::make_mut(chrono_comp).layer = highest_layer.user_up();
            }
        });
    }

    pub(crate) fn move_layer_lowest(&mut self, keys: &[StrokeKey]) {
        let Some((lowest_layer_keys, lowest_layer)) = self.lowest_layer(&self.keys_unordered())
        else {
            return;
        };

        if crate::utils::iterators_contain_same_items(lowest_layer_keys.iter(), keys.iter()) {
            return;
        }

        keys.iter().for_each(|&key| {
            let chrono_components = Arc::make_mut(&mut self.chrono_components);
            let Some(layer) = chrono_components.get(key).map(|c| c.layer) else {
                return;
            };
            if layer <= lowest_layer {
                // Move layers of all other strokes up
                keys.iter().filter(|&&k| k != key).for_each(|key| {
                    let Some(chrono_comp) = chrono_components.get_mut(*key) else {
                        return;
                    };
                    Arc::make_mut(chrono_comp).layer = chrono_comp.layer.user_up();
                });
            }
            let Some(chrono_comp) = chrono_components.get_mut(key) else {
                return;
            };

            Arc::make_mut(chrono_comp).layer = lowest_layer.user_down();
        });
    }

    #[cfg(feature = "ui")]
    pub(crate) fn debug_layers(&self, keys: &[StrokeKey]) -> Vec<String> {
        keys.iter()
            .filter_map(|&key| {
                let layer = self.chrono_components.get(key)?.layer;
                Some(format!("{layer:?}"))
            })
            .collect()
    }
}
