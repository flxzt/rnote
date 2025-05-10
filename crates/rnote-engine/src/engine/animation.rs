// Imports
use tracing::debug;

#[derive(Debug, Clone, Default)]
pub struct Animation {
    frame_in_flight: bool,
}

impl Animation {
    /// Claim an animation frame.
    ///
    /// Returns whether an animation frame was already claimed.
    pub fn claim_frame(&mut self) -> bool {
        if self.frame_in_flight {
            debug!("Animation frame already in flight, skipping");
            true
        } else {
            self.frame_in_flight = true;
            false
        }
    }

    pub fn frame_in_flight(&self) -> bool {
        self.frame_in_flight
    }

    pub fn process_frame(&mut self) -> bool {
        if self.frame_in_flight {
            self.frame_in_flight = false;
            true
        } else {
            false
        }
    }
}
