// Imports
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawOnDocBehaviour, WidgetFlags};
use rnote_compose::penevents::PenEvent;
use std::time::Instant;

/// Types that are pens.
pub trait PenBehaviour: DrawOnDocBehaviour {
    /// Init the pen.
    ///
    /// Should be called right after creating a new pen instance.
    fn init(&mut self, _engine_view: &EngineView) -> WidgetFlags;

    /// Deinit the pen.
    fn deinit(&mut self) -> WidgetFlags;

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
    ///
    /// The fetched content can be available in multiple formats,
    /// so it is returned as: `Vec<(data, MIME-Type)>`.
    #[allow(clippy::type_complexity)]
    fn fetch_clipboard_content(
        &self,
        _engine_view: &EngineView,
    ) -> anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)> {
        Ok((vec![], WidgetFlags::default()))
    }

    /// Cut clipboard content from the pen.
    ///
    /// The cut content can be available in multiple formats,
    /// so it is returned as: `Vec<(data, MIME-Type)>`.
    #[allow(clippy::type_complexity)]
    fn cut_clipboard_content(
        &mut self,
        _engine_view: &mut EngineViewMut,
    ) -> anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)> {
        Ok((vec![], WidgetFlags::default()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenProgress {
    Idle,
    InProgress,
    Finished,
}
