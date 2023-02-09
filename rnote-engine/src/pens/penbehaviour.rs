use std::time::Instant;

use rnote_compose::penevents::PenEvent;

use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawOnDocBehaviour, WidgetFlags};

use super::PenStyle;

/// types that are pens and can handle pen events
pub trait PenBehaviour: DrawOnDocBehaviour {
    // gives what pen style it is
    fn style(&self) -> PenStyle;

    /// init the pen with initial state with the engine view
    fn update_state(&mut self, engine_view: &mut EngineViewMut) -> WidgetFlags;

    /// Handles a pen event
    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags);

    /// fetches clipboard content from the pen
    #[allow(clippy::type_complexity)]
    fn fetch_clipboard_content(
        &self,
        _engine_view: &EngineView,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        Ok((None, WidgetFlags::default()))
    }

    /// cut clipboard content from the pen
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
