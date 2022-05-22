use rnote_compose::penhelpers::PenEvent;

use crate::document::Document;
use crate::engine::EngineTaskSender;
use crate::AudioPlayer;
use crate::{Camera, DrawOnDocBehaviour, StrokeStore, SurfaceFlags};

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
        audioplayer: &mut Option<AudioPlayer>,
    ) -> (PenProgress, SurfaceFlags);

    /// fetches clipboard content from the pen
    fn fetch_clipboard_content(
        &self,
        _doc: &Document,
        _store: &StrokeStore,
        _camera: &Camera,
    ) -> (Vec<u8>, String) {
        (vec![], String::from(""))
    }

    /// Pasts the clipboard content into the pen
    fn paste_clipboard_content(
        &mut self,
        _clipboard_content: &[u8],
        _mime_types: Vec<String>,
        _tasks_tx: EngineTaskSender,
        _doc: &mut Document,
        _store: &mut StrokeStore,
        _camera: &mut Camera,
        _audioplayer: &mut Option<AudioPlayer>,
    ) -> (PenProgress, SurfaceFlags) {
        (PenProgress::Idle, SurfaceFlags::default())
    }

    /// Updates the internal state of the pen ( called for example when the engine state has changed outside of pen events )
    fn update_internal_state(
        &mut self,
        _doc: &Document,
        _store: &StrokeStore,
        _camera: &Camera,
        _audioplayer: &Option<AudioPlayer>,
    ) {
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenProgress {
    Idle,
    InProgress,
    Finished,
}
