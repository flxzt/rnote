use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::inputdata::InputData;

// Represents a single Stroke Element
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename = "element")]
pub struct Element {
    #[serde(rename = "inputdata")]
    pub inputdata: InputData,
    #[serde(rename = "timestamp")]
    pub timestamp: Option<chrono::DateTime<Utc>>,
}

impl Element {
    pub fn new(inputdata: InputData) -> Self {
        let timestamp = Utc::now();

        Self {
            inputdata,
            timestamp: Some(timestamp),
        }
    }
}
