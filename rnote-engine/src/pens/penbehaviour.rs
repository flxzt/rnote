use rnote_compose::penhelpers::PenEvent;

use crate::engine::EngineTaskSender;
use crate::document::Document;
use crate::{Camera, DrawOnDocBehaviour, StrokeStore, SurfaceFlags};

use super::AudioPlayer;

/// types that are pens and can handle pen events
pub trait PenBehaviour: DrawOnDocBehaviour {
    /// Handles a pen event
    #[must_use]
    fn handle_event(
        &mut self,
        event: PenEvent,
        tasks_tx: EngineTaskSender,
        doc: &mut Document,
        store: &mut StrokeStore,
        camera: &mut Camera,
        audioplayer: Option<&mut AudioPlayer>,
    ) -> (PenProgress, SurfaceFlags);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenProgress {
    Idle,
    InProgress,
    Finished,
}
