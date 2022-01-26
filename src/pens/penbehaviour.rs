use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use gtk4::Snapshot;
use p2d::bounding_volume::AABB;

use crate::render::Renderer;
use crate::strokes::strokestyle::InputData;
use crate::ui::appwindow::RnoteAppWindow;

pub trait PenBehaviour {
    fn begin(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow);
    fn motion(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow);
    fn end(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow);
    fn draw(
        &self,
        _sheet_bounds: AABB,
        _zoom: f64,
        _snapshot: &Snapshot,
        _renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
