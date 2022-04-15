use crate::sheet::Sheet;
use crate::{Camera, DrawOnSheetBehaviour, StrokeStore, SurfaceFlags};

use super::AudioPlayer;

/// types that are pens and can handle pen events
pub trait PenBehaviour: DrawOnSheetBehaviour {
    fn handle_event(
        &mut self,
        event: rnote_compose::PenEvent,
        sheet: &mut Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
        audioplayer: Option<&mut AudioPlayer>,
    ) -> SurfaceFlags;
}
