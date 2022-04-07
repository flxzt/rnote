use crate::sheet::Sheet;
use crate::{Camera, DrawOnSheetBehaviour, StrokesState};

pub trait PenBehaviour: DrawOnSheetBehaviour {
    fn handle_event(
        &mut self,
        event: rnote_compose::PenEvent,
        sheet: &mut Sheet,
        strokes_state: &mut StrokesState,
        camera: &Camera,
    );
}
