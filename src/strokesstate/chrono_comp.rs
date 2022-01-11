use rayon::iter::{ParallelBridge, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use serde::{Deserialize, Serialize};

use super::{StrokeKey, StrokesState};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(default)]
pub struct ChronoComponent {
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

/// Systems that are related to their Chronology.
impl StrokesState {
    pub fn set_chrono_to_last(&mut self, key: StrokeKey) {
        if let Some(chrono_comp) = self.chrono_components.get_mut(key) {
            self.chrono_counter += 1;
            chrono_comp.t = self.chrono_counter;
        } else {
            log::debug!(
                "get chrono_comp in set_chrono_to_last() returned None for stroke with key {:?}",
                key
            );
        }
    }

    pub fn last_stroke_key(&self) -> Option<StrokeKey> {
        let chrono_components = &self.chrono_components;
        let trash_components = &self.trash_components;

        let mut sorted: Vec<(StrokeKey, u32)> = chrono_components
            .iter()
            .par_bridge()
            .filter_map(|(key, chrono_comp)| {
                if let (Some(trash_comp), chrono_comp) = (trash_components.get(key), chrono_comp) {
                    if !trash_comp.trashed {
                        return Some((key, chrono_comp.t));
                    }
                }
                None
            })
            .collect();
        sorted.sort_unstable_by(|first, second| first.1.cmp(&second.1));

        let last_stroke_key = sorted.last().copied();

        last_stroke_key.map(|(last_stroke_key, _i)| last_stroke_key)
    }

    pub fn last_selection_key(&self) -> Option<StrokeKey> {
        let chrono_components = &self.chrono_components;
        let trash_components = &self.trash_components;
        let selection_components = &self.selection_components;

        let mut sorted: Vec<(StrokeKey, u32)> = chrono_components
            .iter()
            .par_bridge()
            .filter_map(|(key, chrono_comp)| {
                if let (Some(trash_comp), Some(selection_comp)) =
                    (trash_components.get(key), selection_components.get(key))
                {
                    if !trash_comp.trashed && selection_comp.selected {
                        return Some((key, chrono_comp.t));
                    }
                }
                None
            })
            .collect();
        sorted.sort_unstable_by(|first, second| first.1.cmp(&second.1));

        let last_selection_key = sorted.last().copied();

        last_selection_key.map(|(last_selection_key, _i)| last_selection_key)
    }

    pub fn last_trashed_key(&self) -> Option<StrokeKey> {
        let chrono_components = &self.chrono_components;
        let trash_components = &self.trash_components;

        let mut sorted = chrono_components
            .iter()
            .par_bridge()
            .filter_map(|(key, chrono_comp)| {
                if let (Some(trash_comp), chrono_comp) = (trash_components.get(key), chrono_comp) {
                    if trash_comp.trashed {
                        return Some((key, chrono_comp.t));
                    }
                }
                None
            })
            .collect::<Vec<(StrokeKey, u32)>>();
        sorted.par_sort_unstable_by(|first, second| first.1.cmp(&second.1));

        let last_trashed_key = sorted.last().copied();

        last_trashed_key.map(|(last_trashed_key, _i)| last_trashed_key)
    }

    pub fn keys_sorted_chrono(&self) -> Vec<StrokeKey> {
        let chrono_components = &self.chrono_components;

        let mut sorted = chrono_components
            .iter()
            .par_bridge()
            .map(|(key, chrono_comp)| (key, chrono_comp.t))
            .collect::<Vec<(StrokeKey, u32)>>();
        sorted.par_sort_unstable_by(|first, second| first.1.cmp(&second.1));

        let (keys, _ts): (Vec<StrokeKey>, Vec<u32>) = sorted.into_iter().unzip();
        keys
    }
}
