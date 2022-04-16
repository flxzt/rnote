use crate::sheet::Sheet;
use crate::{Camera, DrawOnSheetBehaviour, StrokeStore, SurfaceFlags};

use super::AudioPlayer;

/// types that are pens and can handle pen events
pub trait PenBehaviour: DrawOnSheetBehaviour {
    #[must_use]
    fn handle_event(
        &mut self,
        event: rnote_compose::PenEvent,
        sheet: &mut Sheet,
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