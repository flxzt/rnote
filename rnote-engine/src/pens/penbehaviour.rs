use rnote_compose::penhelpers::PenEvent;

use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawOnDocBehaviour, WidgetFlags};

/// types that are pens and can handle pen events
pub trait PenBehaviour: DrawOnDocBehaviour {
    /// Handles a pen event
    fn handle_event(
        &mut self,
        event: PenEvent,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags);

    /// fetches clipboard content from the pen
    fn fetch_clipboard_content(
        &self,
        _engine_view: &EngineView,
    ) -> anyhow::Result<Option<(Vec<u8>, String)>> {
        Ok(None)
    }

    /// Pasts the clipboard content into the pen
    fn paste_clipboard_content(
        &mut self,
        _clipboard_content: &[u8],
        _mime_types: Vec<String>,
        _engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        (PenProgress::Idle, WidgetFlags::default())
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
