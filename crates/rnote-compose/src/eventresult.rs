// Imports
use std::fmt::Debug;

/// The event result.
#[derive(Debug, Clone)]
pub struct EventResult<T>
where
    T: Debug,
{
    /// Whether the event was handled.
    pub handled: bool,
    /// Whether the event should be propagated further.
    pub propagate: EventPropagation,
    /// The pen progress.
    pub progress: T,
}

/// Whether the event should be propagated further.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPropagation {
    /// Proceed with the propagation.
    Proceed,
    /// Stop the propagation.
    Stop,
}

impl core::ops::BitOr for EventPropagation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Proceed, Self::Proceed) => Self::Proceed,
            _ => Self::Stop,
        }
    }
}

impl core::ops::BitOrAssign for EventPropagation {
    fn bitor_assign(&mut self, rhs: Self) {
        if rhs == Self::Stop {
            *self = Self::Stop;
        }
    }
}
