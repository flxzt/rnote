use p2d::bounding_volume::AABB;

use crate::sheet::Sheet;
use crate::{DrawOnSheetBehaviour, StrokesState};

pub trait PenBehaviour: DrawOnSheetBehaviour {
    fn handle_event(
        &mut self,
        event: rnote_compose::PenEvent,
        sheet: &mut Sheet,
        strokes_state: &mut StrokesState,
        viewport: Option<AABB>,
        zoom: f64,
    );
}
