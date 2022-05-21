use rnote_compose::penhelpers::PenEvent;

use crate::document::Document;
use crate::engine::EngineTaskSender;
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

    /// Updates the internal state of the pen ( called for example when the engine state has changed outside of pen events )
    fn update_internal_state(
        &mut self,
        _doc: &Document,
        _store: &StrokeStore,
        _camera: &Camera,
        _audioplayer: Option<&AudioPlayer>,
    ) {
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenProgress {
    Idle,
    InProgress,
    Finished,
}
