use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
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
