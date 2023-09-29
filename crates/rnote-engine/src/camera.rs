// Imports
use crate::document::Layout;
use crate::engine::{EngineTask, EngineTaskSender};
use crate::tasks::{OneOffTaskError, OneOffTaskHandle};
use crate::{Document, WidgetFlags};
use p2d::bounding_volume::Aabb;
use rnote_compose::ext::AabbExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    offset: na::Vector2<f64>,
    /// The dimensions in surface coordinates.
    #[serde(rename = "size")]
    size: na::Vector2<f64>,
    /// The camera zoom, origin at (0.0, 0.0).
    #[serde(rename = "zoom")]
    zoom: f64,
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
            offset: na::vector![-Self::OVERSHOOT_HORIZONTAL, -Self::OVERSHOOT_VERTICAL],
            size: na::vector![800.0, 600.0],
            zoom: 1.0,
            temporary_zoom: 1.0,
            scale_factor: 1.0,
            zoom_task_handle: None,
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

    pub fn clone_config(&self) -> Self {
        Self {
            offset: self.offset,
            size: self.size,
            zoom: self.zoom,
            ..Default::default()
        }
    }

    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.zoom = zoom.clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
        self
    }

    pub fn with_offset(mut self, offset: na::Vector2<f64>) -> Self {
        self.offset = offset;
        self
    }
    pub fn with_size(mut self, size: na::Vector2<f64>) -> Self {
        self.size = size;
        self
    }

    /// The current viewport offset in surface coordinate space.
    pub fn offset(&self) -> na::Vector2<f64> {
        self.offset
    }

    pub fn set_offset(&mut self, offset: na::Vector2<f64>, doc: &Document) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        let (lower, upper) = self.offset_lower_upper(doc);
        self.offset = na::vector![
            offset[0].clamp(lower[0], upper[0]),
            offset[1].clamp(lower[1], upper[1])
        ];

        widget_flags.update_view = true;
        widget_flags.resize = true;
        widget_flags
    }

    /// The offset minimum and maximum values in surface coordinate space.
    pub fn offset_lower_upper(&self, doc: &Document) -> (na::Vector2<f64>, na::Vector2<f64>) {
        let total_zoom = self.total_zoom();

        let (h_lower, h_upper) = match doc.layout {
            Layout::FixedSize | Layout::ContinuousVertical => (
                doc.x * total_zoom - Self::OVERSHOOT_HORIZONTAL,
                (doc.x + doc.width) * total_zoom + Self::OVERSHOOT_HORIZONTAL,
            ),
            Layout::SemiInfinite => (
                doc.x * total_zoom - Self::OVERSHOOT_HORIZONTAL,
                (doc.x + doc.width) * total_zoom,
            ),
            Layout::Infinite => (doc.x * total_zoom, (doc.x + doc.width) * total_zoom),
        };
        let (v_lower, v_upper) = match doc.layout {
            Layout::FixedSize | Layout::ContinuousVertical => (
                doc.y * total_zoom - Self::OVERSHOOT_VERTICAL,
                (doc.y + doc.height) * total_zoom + Self::OVERSHOOT_VERTICAL,
            ),
            Layout::SemiInfinite => (
                doc.y * total_zoom - Self::OVERSHOOT_VERTICAL,
                (doc.y + doc.height) * total_zoom,
            ),
            Layout::Infinite => (doc.y * total_zoom, (doc.y + doc.height) * total_zoom),
        };

        (na::vector![h_lower, v_lower], na::vector![h_upper, v_upper])
    }

    /// The current viewport size in surface coordinate space.
    pub fn size(&self) -> na::Vector2<f64> {
        self.size
    }

    pub fn set_size(&mut self, size: na::Vector2<f64>) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.size = size;

        widget_flags.update_view = true;
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
                    log::error!("Could not replace task for one off zoom task, Err: {e:?}");
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

    pub fn set_scale_factor(&mut self, scale_factor: f64) -> WidgetFlags {
        self.scale_factor = scale_factor;
        let mut widget_flags = WidgetFlags::default();
        widget_flags.redraw = true;
        widget_flags
    }

    /// The viewport in document coordinate space.
    pub fn viewport(&self) -> Aabb {
        let total_zoom = self.total_zoom();

        Aabb::new_positive(
            (self.offset / total_zoom).into(),
            ((self.offset + self.size) / total_zoom).into(),
        )
    }

    /// The current viewport center in document coordinate space.
    pub fn viewport_center(&self) -> na::Vector2<f64> {
        (self.offset + self.size * 0.5) / self.total_zoom()
    }

    /// Set the viewport center.
    ///
    /// `center` must be in document coordinate space.
    pub fn set_viewport_center(&mut self, center: na::Vector2<f64>) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        self.offset = center * self.total_zoom() - self.size * 0.5;
        widget_flags.update_view = true;
        widget_flags.resize = true;
        widget_flags
    }

    /// Transform Aabb from document coords to surface coords.
    pub fn transform_bounds(&self, bounds: Aabb) -> Aabb {
        bounds.scale(self.total_zoom()).translate(-self.offset)
    }

    /// Transform Aabb from surface coords to document coords.
    pub fn transform_inv_bounds(&self, bounds: Aabb) -> Aabb {
        bounds.translate(self.offset).scale(1.0 / self.total_zoom())
    }

    /// The transform from document coords to surface coords.
    ///
    /// To get the inverse, call `.inverse()`.
    pub fn transform(&self) -> na::Affine2<f64> {
        let total_zoom = self.total_zoom();

        na::try_convert(
            // LHS is applied onto RHS, so the order is scaling by zoom -> Translation by offset
            na::Translation2::from(-self.offset).to_homogeneous()
                * na::Scale2::from(na::Vector2::from_element(total_zoom)).to_homogeneous(),
        )
        .unwrap()
    }

    /// The gsk transform for the GTK snapshot function.
    ///
    /// GTKs transformations are applied on its coordinate system,
    /// so we need to reverse the transformation order (translate, then scale).
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
    }

    /// Detects if a nudge is needed, meaning: the position is close to an edge of the current viewport.
    pub fn detect_nudge_needed(&self, pos: na::Vector2<f64>) -> Option<NudgeDirection> {
        const NUDGE_VIEWPORT_DIST: f64 = 10.0;
        let viewport = self.viewport();
        let nudge_north = pos[1] <= viewport.mins[1] + NUDGE_VIEWPORT_DIST;
        let nudge_east = pos[0] >= viewport.maxs[0] - NUDGE_VIEWPORT_DIST;
        let nudge_south = pos[1] >= viewport.maxs[1] - NUDGE_VIEWPORT_DIST;
        let nudge_west = pos[0] <= viewport.mins[0] + NUDGE_VIEWPORT_DIST;

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
            NudgeDirection::North => na::vector![0., -amount],
            NudgeDirection::NorthEast => na::vector![amount, -amount],
            NudgeDirection::East => na::vector![amount, 0.],
            NudgeDirection::SouthEast => na::vector![amount, amount],
            NudgeDirection::South => na::vector![0., amount],
            NudgeDirection::SouthWest => na::vector![-amount, amount],
            NudgeDirection::West => na::vector![-amount, 0.],
            NudgeDirection::NorthWest => na::vector![-amount, -amount],
        };
        self.set_offset(self.offset() + nudge_offset, doc)
    }

    pub fn nudge(&mut self, direction: NudgeDirection, doc: &Document) -> WidgetFlags {
        const NUDGE_AMOUNT: f64 = 20.0;
        self.nudge_by(NUDGE_AMOUNT, direction, doc)
    }

    pub fn nudge_w_pos(&mut self, pos: na::Vector2<f64>, doc: &Document) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if let Some(nudge_direction) = self.detect_nudge_needed(pos) {
            widget_flags |= self.nudge(nudge_direction, doc);
        }
        widget_flags
    }
}

#[cfg(test)]
mod tests {
    use crate::Camera;
    use approx::assert_relative_eq;

    #[test]
    fn transform_vec() {
        let offset = na::vector![4.0, 2.0];
        let zoom = 1.5;
        let camera = Camera::default().with_zoom(zoom).with_offset(offset);

        // Point in document coordinates
        let p0 = na::point![10.0, 2.0];

        // first zoom, then scale
        assert_relative_eq!(
            camera.transform().transform_point(&p0).coords,
            (p0.coords * zoom) - offset
        );
    }

    #[test]
    fn viewport() {
        let zoom = 2.0;
        let offset = na::vector![10.0, 10.0];
        let size = na::vector![20.0, 30.0];
        let camera = Camera::default()
            .with_zoom(zoom)
            .with_offset(offset)
            .with_size(size);

        let mins = na::Point2::from(offset / zoom);
        let maxs = na::Point2::from((offset + size) / zoom);

        let viewport = camera.viewport();

        assert_relative_eq!(viewport.mins, mins);
        assert_relative_eq!(viewport.maxs, maxs);
    }
}
