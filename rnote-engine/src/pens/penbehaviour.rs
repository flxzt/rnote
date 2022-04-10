use crate::sheet::Sheet;
use crate::{Camera, DrawOnSheetBehaviour, StrokesState};

use super::AudioPlayer;

pub trait PenBehaviour: DrawOnSheetBehaviour {
    fn handle_event(
        &mut self,
        event: rnote_compose::PenEvent,
        sheet: &mut Sheet,
        strokes_state: &mut StrokesState,
        camera: &Camera,
        audioplayer: Option<&mut AudioPlayer>,
    );
}
