use rnote_compose::penhelpers::PenEvent;

use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawOnDocBehaviour, SurfaceFlags};

/// types that are pens and can handle pen events
pub trait PenBehaviour: DrawOnDocBehaviour {
    /// Handles a pen event
    #[must_use]
    fn handle_event(
        &mut self,
        event: PenEvent,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, SurfaceFlags);

    /// fetches clipboard content from the pen
    fn fetch_clipboard_content(&self, _engine_view: &EngineView) -> (Vec<u8>, String) {
        (vec![], String::from(""))
    }

    /// Pasts the clipboard content into the pen
    fn paste_clipboard_content(
        &mut self,
        _clipboard_content: &[u8],
        _mime_types: Vec<String>,
        _engine_view: &mut EngineViewMut,
    ) -> (PenProgress, SurfaceFlags) {
        (PenProgress::Idle, SurfaceFlags::default())
    }

    /// Updates the internal state of the pen ( called for example when the engine state has changed outside of pen events )
    fn update_internal_state(&mut self, _engine_view: &EngineView) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenProgress {
    Idle,
    InProgress,
    Finished,
}
