// Imports
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawOnDocBehaviour, WidgetFlags};
use rnote_compose::penevents::PenEvent;
use std::time::Instant;

/// Types that are pens.
pub trait PenBehaviour: DrawOnDocBehaviour {
    // The pen style.
    fn style(&self) -> PenStyle;

    /// Update the pen and pen config state with the state from the engine.
    fn update_state(&mut self, engine_view: &mut EngineViewMut) -> WidgetFlags;

    /// Handle a pen event.
    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags);

    /// Fetch clipboard content from the pen.
    #[allow(clippy::type_complexity)]
    fn fetch_clipboard_content(
        &self,
        _engine_view: &EngineView,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        Ok((None, WidgetFlags::default()))
    }

    /// Cut clipboard content from the pen.
    #[allow(clippy::type_complexity)]
    fn cut_clipboard_content(
        &mut self,
        _engine_view: &mut EngineViewMut,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        Ok((None, WidgetFlags::default()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenProgress {
    Idle,
    InProgress,
    Finished,
}
