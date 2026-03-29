// Imports
use crate::document::Layout;
use crate::engine::snapshot::Snapshotable;
use crate::engine::{EngineTask, EngineTaskSender};
use crate::tasks::{OneOffTaskError, OneOffTaskHandle};
use crate::{Document, WidgetFlags};
use p2d::bounding_volume::Aabb;
use p2d::glamx::DAffine2;
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NudgeDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "camera")]
pub struct Camera {
    /// The offset in surface coordinates.
    #[serde(rename = "offset")]
    offset: Vector2,
    /// The dimensions in surface coordinates.
    #[serde(rename = "size")]
    size: Vector2,
    /// The camera zoom, origin at (0.0, 0.0).
    #[serde(rename = "zoom")]
    zoom: f64,
    /// The camera rotation in radians.
    #[serde(skip)]
    rotation: f64,
    /// The temporary zoom. Is used to overlay the "permanent" zoom.
    #[serde(skip)]
    temporary_zoom: f64,

    /// The scale factor of the surface, usually 1.0 or 2.0 for high-dpi screens.
    ///
    /// This value could become a non-integer value in the future, so it is stored as float.
    #[serde(skip)]
    scale_factor: f64,

    #[serde(skip)]
    zoom_task_handle: Option<crate::tasks::OneOffTaskHandle>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            offset: Vector2::new(-Self::OVERSHOOT_HORIZONTAL, -Self::OVERSHOOT_VERTICAL),
            size: Vector2::new(800.0, 600.0),
            zoom: 1.0,
            rotation: 0.0,
            temporary_zoom: 1.0,
            scale_factor: 1.0,
            zoom_task_handle: None,
        }
    }
}

impl Snapshotable for Camera {
    fn extract_snapshot_data(&self) -> Self {
        Self {
            offset: self.offset,
            size: self.size,
            zoom: self.zoom,
            rotation: self.rotation,
            ..Default::default()
        }
    }
}

impl Camera {
    pub const ZOOM_MIN: f64 = 0.2;
    pub const ZOOM_MAX: f64 = 6.0;
    pub const ZOOM_DEFAULT: f64 = 1.0;
    // The zoom timeout time.
    pub const ZOOM_TIMEOUT: Duration = Duration::from_millis(400);
    // when performing a drag - zoom 0.5% zoom for every pixel in y dir
    pub const DRAG_ZOOM_MAGN_ZOOM_FACTOR: f64 = 0.005;
    pub const OVERSHOOT_HORIZONTAL: f64 = 96.0;
    pub const OVERSHOOT_VERTICAL: f64 = 96.0;

    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.zoom = zoom.clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
        self
    }

    pub fn with_offset(mut self, offset: Vector2) -> Self {
        self.offset = offset;
        self
    }
    pub fn with_size(mut self, size: Vector2) -> Self {
        self.size = size;
        self
    }

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// The current viewport offset in surface coordinate space.
    pub fn offset(&self) -> Vector2 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vector2, doc: &Document) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        let (mins, maxs) = self.surface_mins_maxs(doc);
        let offset_maxs = Vector2::new(
            (maxs.x - self.size.x).max(mins.x),
            (maxs.y - self.size.y).max(mins.y),
        );

        self.offset = Vector2::new(
            offset.x.clamp(mins.x, offset_maxs.x),
            offset.y.clamp(mins.y, offset_maxs.y),
        );

        widget_flags.view_modified = true;
        widget_flags
    }

    /// The minimum and maximum surface bounds (document including overshoot) in surface coordinate space.
    pub fn surface_mins_maxs(&self, doc: &Document) -> (Vector2, Vector2) {
        let transform = self.transform();

        let corners = [
            Vector2::new(doc.x, doc.y),
            Vector2::new(doc.x + doc.width, doc.y),
            Vector2::new(doc.x + doc.width, doc.y + doc.height),
            Vector2::new(doc.x, doc.y + doc.height),
        ]
        .map(|p| transform.transform_vector2(p));

        let bounds = Aabb::from_points(corners);

        let (h_lower, h_upper) = match doc.config.layout {
            Layout::FixedSize | Layout::ContinuousVertical => (
                bounds.mins.x - Self::OVERSHOOT_HORIZONTAL,
                bounds.maxs.x + Self::OVERSHOOT_HORIZONTAL,
            ),
            Layout::SemiInfinite => (bounds.mins.x - Self::OVERSHOOT_HORIZONTAL, bounds.maxs.x),
            Layout::Infinite => (bounds.mins.x, bounds.maxs.x),
        };
        let (v_lower, v_upper) = match doc.config.layout {
            Layout::FixedSize | Layout::ContinuousVertical => (
                bounds.mins.y - Self::OVERSHOOT_VERTICAL,
                bounds.maxs.y + Self::OVERSHOOT_VERTICAL,
            ),
            Layout::SemiInfinite => (bounds.mins.y - Self::OVERSHOOT_VERTICAL, bounds.maxs.y),
            Layout::Infinite => (bounds.mins.y, bounds.maxs.y),
        };

        (
            Vector2::new(h_lower, v_lower),
            Vector2::new(h_upper, v_upper),
        )
    }

    /// The current viewport size in surface coordinate space.
    pub fn size(&self) -> Vector2 {
        self.size
    }

    pub fn set_size(&mut self, size: Vector2) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.size = size;

        widget_flags.view_modified = true;
        widget_flags.resize = true;
        widget_flags
    }

    /// The permanent zoom.
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Set the permanent zoom.
    pub fn zoom_to(&mut self, zoom: f64) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.zoom = zoom.clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
        widget_flags.zoomed = true;
        widget_flags
    }

    /// The camera rotation in radians.
    pub fn rotation(&self) -> f64 {
        self.rotation
    }

    fn snap_angle(angle: f64, step: f64) -> f64 {
        const SNAP_EPS: f64 = 1_f64.to_radians();

        let k = (angle / step).round();
        let snapped_angle = k * step;

        if (angle - snapped_angle).abs() <= SNAP_EPS {
            snapped_angle
        } else {
            angle
        }
    }

    pub fn set_rotation(&mut self, rotation: f64) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        // snap angle to nearest 90 degrees and normalize it to [0, 2pi)
        // angle must not be close to zero, because it causes major rendering issues in GTK
        self.rotation = Self::snap_angle(rotation, std::f64::consts::FRAC_PI_2)
            .rem_euclid(std::f64::consts::TAU);

        widget_flags.view_modified = true;
        widget_flags.resize = true;
        widget_flags
    }

    /// The temporary zoom, to be overlaid on the surface when zooming with a timeout.
    pub fn temporary_zoom(&self) -> f64 {
        self.temporary_zoom
    }

    /// Set the temporary zoom.
    pub fn zoom_temporarily_to(&mut self, temporary_zoom: f64) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.temporary_zoom =
            temporary_zoom.clamp(Camera::ZOOM_MIN / self.zoom, Camera::ZOOM_MAX / self.zoom);
        widget_flags.zoomed_temporarily = true;
        widget_flags
    }

    /// First zoom temporarily and then permanently after a timeout.
    ///
    /// Repeated calls to this function reset the timeout.
    pub(crate) fn zoom_w_timeout(&mut self, zoom: f64, tasks_tx: EngineTaskSender) -> WidgetFlags {
        let old_zoom = self.zoom();
        let mut reinstall_zoom_task = false;

        // zoom temporarily immediately
        let new_temporary_zoom = zoom / old_zoom;
        let widget_flags = self.zoom_temporarily_to(new_temporary_zoom);

        let zoom_task = move || {
            tasks_tx.send(EngineTask::Zoom(zoom));
        };
        if let Some(handle) = self.zoom_task_handle.as_mut() {
            match handle.replace_task(zoom_task.clone()) {
                Ok(()) => {}
                Err(OneOffTaskError::TimeoutReached) => {
                    reinstall_zoom_task = true;
                }
                Err(e) => {
                    error!("Could not replace task for one off zoom task, Err: {e:?}");
                    reinstall_zoom_task = true;
                }
            }
        } else {
            reinstall_zoom_task = true;
        }

        if reinstall_zoom_task {
            self.zoom_task_handle = Some(OneOffTaskHandle::new(zoom_task, Self::ZOOM_TIMEOUT));
        }

        widget_flags
    }

    /// The total zoom of the camera, including the temporary zoom.
    pub fn total_zoom(&self) -> f64 {
        self.zoom * self.temporary_zoom
    }

    /// The scaling factor for generating bitmap images with the current permanent zoom.
    ///
    /// Takes the scale factor in account
    pub fn image_scale(&self) -> f64 {
        self.zoom * self.scale_factor
    }

    /// The scale factor that gets set according to the toolkit hi-dpi settings.
    ///
    /// For Gtk it currently is 1.0 for scaling < 150%, 2.0 for >= 150% and < 250%, ..
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) -> WidgetFlags {
        self.scale_factor = scale_factor;
        let mut widget_flags = WidgetFlags::default();
        widget_flags.redraw = true;
        widget_flags
    }

    /// The viewport in document coordinate space.
    ///
    /// Returns the Aabb enclosing the (potentially rotated) viewport.
    pub fn viewport(&self) -> Aabb {
        let transform_inv = self.transform().inverse();

        let corners = [
            Vector2::new(0.0, 0.0),
            Vector2::new(self.size[0], 0.0),
            Vector2::new(self.size[0], self.size[1]),
            Vector2::new(0.0, self.size[1]),
        ]
        .map(|p| transform_inv.transform_point2(p));

        Aabb::from_points(corners)
    }

    /// The current viewport center in document coordinate space.
    pub fn viewport_center(&self) -> Vector2 {
        self.transform().inverse().transform_point2(self.size * 0.5)
    }

    /// Set the viewport center.
    ///
    /// `center` must be in document coordinate space.
    pub fn set_viewport_center(&mut self, center: Vector2) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.offset = self.transform().transform_vector2(center) - self.size * 0.5;
        widget_flags.view_modified = true;
        widget_flags
    }

    /// Transform Aabb from document coords to surface coords.
    ///
    /// Returns the Aabb enclosing the (potentially rotated) transformed bounds.
    pub fn transform_bounds(&self, bounds: Aabb) -> Aabb {
        let transform = self.transform();

        let corners = [
            Vector2::new(bounds.mins[0], bounds.mins[1]),
            Vector2::new(bounds.maxs[0], bounds.mins[1]),
            Vector2::new(bounds.maxs[0], bounds.maxs[1]),
            Vector2::new(bounds.mins[0], bounds.maxs[1]),
        ]
        .map(|p| transform.transform_point2(p));

        Aabb::from_points(corners)
    }

    /// Transform Aabb from surface coords to document coords.
    ///
    /// Returns the Aabb enclosing the (potentially rotated) transformed bounds.
    pub fn transform_inv_bounds(&self, bounds: Aabb) -> Aabb {
        let transform_inv = self.transform().inverse();

        let corners = [
            Vector2::new(bounds.mins[0], bounds.mins[1]),
            Vector2::new(bounds.maxs[0], bounds.mins[1]),
            Vector2::new(bounds.maxs[0], bounds.maxs[1]),
            Vector2::new(bounds.mins[0], bounds.maxs[1]),
        ]
        .map(|p| transform_inv.transform_point2(p));

        Aabb::from_points(corners)
    }

    /// The transform from document coords to surface coords.
    ///
    /// To get the inverse, call `.inverse()`.
    pub fn transform(&self) -> DAffine2 {
        // LHS is applied onto RHS: rotate -> scale -> translate
        DAffine2::from_translation(-self.offset)
            * DAffine2::from_scale(Vector2::splat(self.total_zoom()))
            * DAffine2::from_angle(self.rotation)
    }

    /// The gsk transform for the GTK snapshot function.
    ///
    /// GTKs transformations are applied on its coordinate system,
    /// so we need to reverse the transformation order (translate, then scale, then rotate).
    /// Small rotation angles seem to cause major rendering issues.
    ///
    /// To get the inverse, call .invert().
    #[cfg(feature = "ui")]
    pub fn transform_for_gtk_snapshot(&self) -> gtk4::gsk::Transform {
        let total_zoom = self.total_zoom();

        gtk4::gsk::Transform::new()
            .translate(&gtk4::graphene::Point::new(
                -self.offset[0] as f32,
                -self.offset[1] as f32,
            ))
            .scale(total_zoom as f32, total_zoom as f32)
            .rotate(self.rotation.to_degrees() as f32)
    }

    /// Detects if a nudge is needed, meaning: the position is close to an edge of the current viewport.
    pub fn detect_nudge_needed(&self, pos: Vector2) -> Option<NudgeDirection> {
        const NUDGE_VIEWPORT_DIST: f64 = 10.0;

        // Transform position into surface coordinates and compare against the surface viewport.
        let pos_surface = self.transform().transform_point2(pos);

        let nudge_north = pos_surface.y <= NUDGE_VIEWPORT_DIST;
        let nudge_east = pos_surface.x >= self.size[0] - NUDGE_VIEWPORT_DIST;
        let nudge_south = pos_surface.y >= self.size[1] - NUDGE_VIEWPORT_DIST;
        let nudge_west = pos_surface.x <= NUDGE_VIEWPORT_DIST;

        match (nudge_north, nudge_east, nudge_south, nudge_west) {
            (true, false, _, false) => Some(NudgeDirection::North),
            (true, true, _, _) => Some(NudgeDirection::NorthEast),
            (false, true, false, _) => Some(NudgeDirection::East),
            (_, true, true, _) => Some(NudgeDirection::SouthEast),
            (_, false, true, false) => Some(NudgeDirection::South),
            (_, _, true, true) => Some(NudgeDirection::SouthWest),
            (false, _, false, true) => Some(NudgeDirection::West),
            (true, _, _, true) => Some(NudgeDirection::NorthWest),
            (false, false, false, false) => None,
        }
    }

    pub fn nudge_by(
        &mut self,
        amount: f64,
        direction: NudgeDirection,
        doc: &Document,
    ) -> WidgetFlags {
        let nudge_offset = match direction {
            NudgeDirection::North => Vector2::new(0., -amount),
            NudgeDirection::NorthEast => Vector2::new(amount, -amount),
            NudgeDirection::East => Vector2::new(amount, 0.),
            NudgeDirection::SouthEast => Vector2::new(amount, amount),
            NudgeDirection::South => Vector2::new(0., amount),
            NudgeDirection::SouthWest => Vector2::new(-amount, amount),
            NudgeDirection::West => Vector2::new(-amount, 0.),
            NudgeDirection::NorthWest => Vector2::new(-amount, -amount),
        };
        self.set_offset(self.offset() + nudge_offset, doc)
    }

    pub fn nudge(&mut self, direction: NudgeDirection, doc: &Document) -> WidgetFlags {
        const NUDGE_AMOUNT: f64 = 20.0;
        self.nudge_by(NUDGE_AMOUNT, direction, doc)
    }

    pub fn nudge_w_pos(&mut self, pos: Vector2, doc: &Document) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if let Some(nudge_direction) = self.detect_nudge_needed(pos) {
            widget_flags |= self.nudge(nudge_direction, doc);
        }
        widget_flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn transform_vec() {
        let offset = Vector2::new(4.0, 2.0);
        let zoom = 1.5;
        let camera = Camera::default().with_zoom(zoom).with_offset(offset);

        // Point in document coordinates
        let p0 = Vector2::new(10.0, 2.0);

        // first zoom, then scale
        assert_relative_eq!(
            camera.transform().transform_point2(p0),
            (p0 * zoom) - offset
        );
    }

    #[test]
    fn viewport() {
        let zoom = 2.0;
        let offset = Vector2::splat(10.0);
        let size = Vector2::new(20.0, 30.0);

        let camera = Camera::default()
            .with_zoom(zoom)
            .with_offset(offset)
            .with_size(size);

        let mins = offset / zoom;
        let maxs = (offset + size) / zoom;
        let viewport = camera.viewport();

        assert_relative_eq!(viewport.mins, mins);
        assert_relative_eq!(viewport.maxs, maxs);
    }
}
