use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use gtk4::Snapshot;
use p2d::bounding_volume::AABB;

use crate::render::Renderer;
use crate::sheet::Sheet;
use crate::strokes::inputdata::InputData;

pub trait PenBehaviour {
    fn begin(
        &mut self,
        data_entries: VecDeque<InputData>,
        sheet: &mut Sheet,
        viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    );
    fn motion(
        &mut self,
        data_entries: VecDeque<InputData>,
        sheet: &mut Sheet,
        viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    );
    fn end(
        &mut self,
        data_entries: VecDeque<InputData>,
        sheet: &mut Sheet,
        viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    );
    fn draw(
        &self,
        _snapshot: &Snapshot,
        _sheet: &Sheet,
        _viewport: Option<AABB>,
        _zoom: f64,
        _renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
