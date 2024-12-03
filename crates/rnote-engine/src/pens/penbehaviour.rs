// Imports
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawableOnDoc, WidgetFlags};
use futures::channel::oneshot;
use rnote_compose::penevent::{PenEvent, PenProgress};
use rnote_compose::EventResult;
use std::time::Instant;
use tracing::error;

/// Types that are pens.
pub trait PenBehaviour: DrawableOnDoc {
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
    ) -> (EventResult<PenProgress>, WidgetFlags);

    /// Handle a requested animation frame.
    ///
    /// Can request another frame using `ÈngineViewMut#animation.claim_frame()`.
    fn handle_animation_frame(&mut self, _engine_view: &mut EngineViewMut, _optimize_epd: bool) {}

    /// Fetch clipboard content from the pen.
    ///
    /// The fetched content can be available in multiple formats,
    /// so it is returned as: `Vec<(data, MIME-Type)>`.
    #[allow(clippy::type_complexity)]
    fn fetch_clipboard_content(
        &self,
        _engine_view: &EngineView,
    ) -> oneshot::Receiver<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>> {
        let (sender, receiver) =
            oneshot::channel::<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>>();
        rayon::spawn(move || {
            if sender.send(Ok((vec![], WidgetFlags::default()))).is_err() {
                error!("Sending (empty) clipboard content in `fetch_clipboard_content()` default impl failed, receiver already dropped.")
            }
        });
        receiver
    }

    /// Cut clipboard content from the pen.
    ///
    /// The cut content can be available in multiple formats,
    /// so it is returned as: `Vec<(data, MIME-Type)>`.
    #[allow(clippy::type_complexity)]
    fn cut_clipboard_content(
        &mut self,
        _engine_view: &mut EngineViewMut,
    ) -> oneshot::Receiver<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>> {
        let (sender, receiver) =
            oneshot::channel::<anyhow::Result<(Vec<(Vec<u8>, String)>, WidgetFlags)>>();
        rayon::spawn(move || {
            if sender.send(Ok((vec![], WidgetFlags::default()))).is_err() {
                error!("Sending (empty) clipboard content in `cut_clipboard_content()` default impl failed, receiver already dropped")
            }
        });
        receiver
    }
}
