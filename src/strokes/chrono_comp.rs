use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChronoComponent {
    pub t: u64,
}

impl Default for ChronoComponent {
    fn default() -> Self {
        Self { t: 0 }
    }
}

impl ChronoComponent {
    pub fn new(t: u64) -> Self {
        Self {
            t,
        }
    }
}