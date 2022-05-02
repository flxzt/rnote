use std::sync::Arc;

use p2d::bounding_volume::AABB;
use rayon::slice::ParallelSliceMut;
use serde::{Deserialize, Serialize};

use super::{StrokeKey, StrokeStore};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(default, rename = "chrono_component")]
pub struct ChronoComponent {
    #[serde(rename = "t")]
    pub t: u32,
}

impl Default for ChronoComponent {
    fn default() -> Self {
        Self { t: 0 }
    }
}

impl ChronoComponent {
    pub fn new(t: u32) -> Self {
        Self { t }
    }
}

/// Systems that are related to their chronological ordering.
impl StrokeStore {
    pub fn update_chrono_to_last(&mut self, key: StrokeKey) {
        if let Some(chrono_comp) = Arc::make_mut(&mut self.chrono_components).get_mut(key) {
            self.chrono_counter += 1;
            Arc::make_mut(chrono_comp).t = self.chrono_counter;
        } else {
            log::debug!(
                "get chrono_comp in set_chrono_to_last() returned None for stroke with key {:?}",
                key
            );
        }
    }

    /// Returns the keys in chronological order, as in first: gets drawn first, last: gets drawn last
    pub fn keys_sorted_chrono(&self) -> Vec<StrokeKey> {
        let chrono_components = &self.chrono_components;

        let mut keys = self.stroke_components.keys().collect::<Vec<StrokeKey>>();

        keys.par_sort_unstable_by(|&first, &second| {
            if let (Some(first_chrono), Some(second_chrono)) =
                (chrono_components.get(first), chrono_components.get(second))
            {
                first_chrono.t.cmp(&second_chrono.t)
            } else {
                std::cmp::Ordering::Equal
            }
        });

        keys
    }

    pub fn keys_sorted_chrono_intersecting_bounds(&self, bounds: AABB) -> Vec<StrokeKey> {
        let chrono_components = &self.chrono_components;

        let mut keys = self.key_tree.keys_intersecting_bounds(bounds);

        keys.par_sort_unstable_by(|&first, &second| {
            if let (Some(first_chrono), Some(second_chrono)) =
                (chrono_components.get(first), chrono_components.get(second))
            {
                first_chrono.t.cmp(&second_chrono.t)
            } else {
                std::cmp::Ordering::Equal
            }
        });

        keys
    }
}
