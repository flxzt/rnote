use gtk4::Snapshot;

use crate::render::Renderer;
use crate::strokes::strokestyle::InputData;
use crate::ui::appwindow::RnoteAppWindow;

pub trait PenBehaviour {
    fn begin(&mut self, inputdata: InputData, appwindow: &RnoteAppWindow);
    fn motion(&mut self, inputdata: InputData, appwindow: &RnoteAppWindow);
    fn end(&mut self, inputdata: InputData, appwindow: &RnoteAppWindow);
    fn draw(
        &self,
        sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &Renderer,
        zoom: f64,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error>;
}
