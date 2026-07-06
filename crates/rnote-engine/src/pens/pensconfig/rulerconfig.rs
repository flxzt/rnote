// Imports
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

/// Configuration and runtime state for the on-canvas ruler.
///
/// The ruler is a translucent straight-edge spanning the viewport. Position is
/// stored in **scroller (window-relative) coordinates** — i.e. pixel offsets
/// inside the visible viewport — so panning or zooming the canvas does not
/// move the ruler on screen. Conversions to document coordinates are performed
/// on demand via [`Self::pos_to_doc`] / [`Self::pos_from_doc`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "ruler_config")]
pub struct RulerConfig {
    /// Whether the ruler is currently shown and active for snapping.
    /// In-session only — not persisted.
    #[serde(skip)]
    pub visible: bool,
    /// Angle of the ruler's long axis in radians (0 = horizontal). In-session only.
    #[serde(skip)]
    pub angle: f64,
    /// A point in scroller (window-relative) pixel coordinates the ruler
    /// centerline passes through. Defines the origin for tick marks. The
    /// ruler is fixed relative to the window — this does not change as the
    /// canvas is panned or zoomed. In-session only.
    #[serde(skip)]
    pub anchor: Vector2,
    /// Where the angle dial / rotation pivot is rendered, in scroller
    /// coordinates. Always lies on the centerline. Distinct from `anchor` so
    /// the dial can move (e.g., to the finger centroid) without shifting the
    /// tick origin. In-session only.
    #[serde(skip)]
    pub dial_pos: Vector2,
    /// Snap distance beyond the ruler's edge, as a percentage of the ruler's
    /// full width (`2 × BODY_HALF_WIDTH_PX`). Stored as a preference, e.g.
    /// `25.0` means 25%, i.e. half a body-width beyond each edge.
    pub snap_distance: f64,
    /// Whether to snap the ruler's angle to common values (0°, ±45°, ±90°)
    /// during two-finger rotation. Persisted.
    pub angle_snap_enabled: bool,
    /// Whether to render the angle dial in the center of rotation. Persisted.
    pub show_dial: bool,
    /// Half-width of the ruler body in surface pixels. Persisted.
    pub body_half_width: f64,
    /// On-screen spacing between minor long-edge ticks, in surface pixels.
    /// Persisted.
    pub tick_spacing: f64,
    /// Opacity of the ruler body fill, in percent (`0.0` = fully transparent,
    /// `100.0` = fully opaque). Persisted.
    pub body_opacity: f64,
    /// Degrees of ruler rotation per scroll-wheel unit (`dy`). Persisted.
    pub scroll_rotation_step_deg: f64,
}

impl Default for RulerConfig {
    fn default() -> Self {
        Self {
            visible: false,
            angle: 0.0,
            anchor: Vector2::ZERO,
            dial_pos: Vector2::ZERO,
            snap_distance: Self::SNAP_DISTANCE_DEFAULT,
            angle_snap_enabled: true,
            show_dial: true,
            body_half_width: Self::BODY_HALF_WIDTH_DEFAULT,
            tick_spacing: Self::TICK_SPACING_DEFAULT,
            body_opacity: Self::BODY_OPACITY_DEFAULT,
            scroll_rotation_step_deg: Self::SCROLL_ROTATION_STEP_DEG_DEFAULT,
        }
    }
}

impl RulerConfig {
    pub const SNAP_DISTANCE_DEFAULT: f64 = 25.0;
    pub const SNAP_DISTANCE_MIN: f64 = 0.0;
    pub const SNAP_DISTANCE_MAX: f64 = 200.0;
    /// Default / range for `body_half_width`, in surface pixels (constant on-screen size).
    pub const BODY_HALF_WIDTH_DEFAULT: f64 = 60.0;
    pub const BODY_HALF_WIDTH_MIN: f64 = 20.0;
    pub const BODY_HALF_WIDTH_MAX: f64 = 120.0;
    /// Default / range for `tick_spacing`, in surface pixels.
    pub const TICK_SPACING_DEFAULT: f64 = 5.0;
    pub const TICK_SPACING_MIN: f64 = 2.0;
    pub const TICK_SPACING_MAX: f64 = 20.0;
    /// Default / range for `body_opacity`, in percent.
    pub const BODY_OPACITY_DEFAULT: f64 = 35.0;
    pub const BODY_OPACITY_MIN: f64 = 0.0;
    pub const BODY_OPACITY_MAX: f64 = 100.0;
    /// Default / range for `scroll_rotation_step_deg`, in degrees per scroll-wheel unit.
    pub const SCROLL_ROTATION_STEP_DEG_DEFAULT: f64 = 2.0;
    pub const SCROLL_ROTATION_STEP_DEG_MIN: f64 = 0.1;
    pub const SCROLL_ROTATION_STEP_DEG_MAX: f64 = 15.0;
    /// Length of major tick marks in surface pixels.
    pub const TICK_MAJOR_LEN_PX: f64 = 14.0;
    /// Length of minor tick marks in surface pixels.
    pub const TICK_MINOR_LEN_PX: f64 = 7.0;
    /// Half-width of the snap-to-angle window, in degrees, when *approaching*
    /// a target. Within this many degrees of `0`, `±45`, or `±90`, the angle
    /// gets pulled in to the target.
    pub const ANGLE_SNAP_THRESHOLD_DEG: f64 = 3.0;
    /// Half-width of the snap-to-angle window when the ruler is **already
    /// locked** to a target, in degrees. Smaller than the enter threshold so
    /// a single small scroll-wheel click is enough to break out of a snap.
    pub const ANGLE_SNAP_LEAVE_THRESHOLD_DEG: f64 = 0.5;

    /// Unit direction vector along the ruler's long axis.
    pub fn direction(&self) -> Vector2 {
        Vector2::new(self.angle.cos(), self.angle.sin())
    }

    /// Unit normal vector perpendicular to the ruler's long axis (left-hand side).
    pub fn normal(&self) -> Vector2 {
        Vector2::new(-self.angle.sin(), self.angle.cos())
    }

    /// Half-width of the ruler body in document coordinates at the given zoom.
    pub fn body_half_width_doc(&self, total_zoom: f64) -> f64 {
        self.body_half_width / total_zoom
    }

    /// Whether a dark-on-light palette should be used, given the page's
    /// background color. Uses BT.601 relative luminance with a 0.5 threshold:
    /// dark backgrounds (luminance < 0.5) get a light ruler palette and vice
    /// versa.
    pub fn dark_mode_for_background(bg: &rnote_compose::Color) -> bool {
        let luminance = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
        luminance < 0.5
    }

    /// Body fill color computed from `body_opacity` and the dark-mode decision.
    ///
    /// The body's brightness is interpolated against `body_opacity`: at low
    /// opacity we want the body to contrast with the canvas background (so the
    /// ruler is visible at all), and at high opacity we want it to contrast
    /// with the marking colors (so the ticks / text stay readable).
    pub fn body_fill_color(&self, dark_mode: bool) -> piet::Color {
        let opacity = (self.body_opacity / 100.0).clamp(0.0, 1.0);
        let alpha = (opacity * 255.0).round() as u8;
        let (low, high) = if dark_mode {
            // Dark mode: low opacity ≈ near-white (contrast against dark canvas);
            // high opacity ≈ dark gray (contrast against white markings/text).
            (220.0, 60.0)
        } else {
            // Light mode: low opacity ≈ dark gray (contrast against light canvas);
            // high opacity ≈ light gray (contrast against black markings/text).
            (80.0, 200.0)
        };
        let brightness = (low + (high - low) * opacity).round().clamp(0.0, 255.0) as u8;
        piet::Color::rgba8(brightness, brightness, brightness, alpha)
    }

    /// Color used for the body outline.
    pub fn body_stroke_color(dark_mode: bool) -> piet::Color {
        if dark_mode {
            piet::Color::rgba8(255, 255, 255, 220)
        } else {
            piet::Color::rgba8(0, 0, 0, 220)
        }
    }

    /// Color used for the tick marks (long edges + dial ticks).
    pub fn tick_color(dark_mode: bool) -> piet::Color {
        if dark_mode {
            piet::Color::rgba8(255, 255, 255, 240)
        } else {
            piet::Color::rgba8(0, 0, 0, 240)
        }
    }

    /// Color used for the angle text in the dial.
    pub fn angle_text_color(dark_mode: bool) -> piet::Color {
        if dark_mode {
            piet::Color::rgba8(255, 255, 255, 255)
        } else {
            piet::Color::rgba8(0, 0, 0, 255)
        }
    }

    /// Convert a position from window-relative surface pixels to document coordinates.
    pub fn pos_to_doc(surface_pos: Vector2, camera_offset: Vector2, total_zoom: f64) -> Vector2 {
        // Surface here is the canvas widget's local coord system, which has its
        // origin at the top-left of the visible viewport (the canvas widget
        // implements Scrollable internally — there's no scroll-offset between
        // the scroller widget and the canvas widget). So this is just the
        // standard inverse of the camera transform.
        (surface_pos + camera_offset) / total_zoom
    }

    /// Convert a position from document coordinates to window-relative surface pixels.
    pub fn pos_from_doc(doc_pos: Vector2, camera_offset: Vector2, total_zoom: f64) -> Vector2 {
        doc_pos * total_zoom - camera_offset
    }

    /// Ruler centerline anchor in document coordinates.
    pub fn anchor_doc(&self, camera_offset: Vector2, total_zoom: f64) -> Vector2 {
        Self::pos_to_doc(self.anchor, camera_offset, total_zoom)
    }

    /// Dial position in document coordinates.
    pub fn dial_pos_doc(&self, camera_offset: Vector2, total_zoom: f64) -> Vector2 {
        Self::pos_to_doc(self.dial_pos, camera_offset, total_zoom)
    }

    /// If `pos_doc` lies within the snap zone of one of the ruler's long
    /// edges, return the sign of the perpendicular (`+1.0` or `-1.0`) that
    /// identifies that edge. `None` means no snap.
    pub fn snap_side(
        &self,
        pos_doc: Vector2,
        camera_offset: Vector2,
        total_zoom: f64,
    ) -> Option<f64> {
        if !self.visible {
            return None;
        }
        let pos_scroller = Self::pos_from_doc(pos_doc, camera_offset, total_zoom);
        let half_w = self.body_half_width;
        let snap_dist_px = (self.snap_distance / 100.0) * 2.0 * half_w;
        let rel = pos_scroller - self.anchor;
        let perp = rel.dot(self.normal());
        if perp.abs() - half_w > snap_dist_px {
            None
        } else {
            Some(if perp >= 0.0 { 1.0 } else { -1.0 })
        }
    }

    /// Project `pos_doc` onto the long edge identified by `side` (`+1.0` or
    /// `-1.0`), regardless of distance. Used to keep a stroke locked to the
    /// ruler once it has snapped.
    pub fn project_to_edge(
        &self,
        pos_doc: Vector2,
        side: f64,
        camera_offset: Vector2,
        total_zoom: f64,
    ) -> Vector2 {
        let pos_scroller = Self::pos_from_doc(pos_doc, camera_offset, total_zoom);
        let dir = self.direction();
        let normal = self.normal();
        let along = (pos_scroller - self.anchor).dot(dir);
        let half_w = self.body_half_width;
        let snapped_scroller = self.anchor + along * dir + side * half_w * normal;
        Self::pos_to_doc(snapped_scroller, camera_offset, total_zoom)
    }

    /// Normalize an angle (radians) to the displayable principal angle in
    /// `[-π/2, π/2)`, with sign flipped so positive is CCW as the user sees
    /// it on screen (the stored `angle` uses screen-y-down radians where
    /// positive rotation is visually clockwise).
    pub fn normalize_angle(angle_rad: f64) -> f64 {
        let half_pi = std::f64::consts::FRAC_PI_2;
        let pi = std::f64::consts::PI;
        -(((angle_rad + half_pi).rem_euclid(pi)) - half_pi)
    }

    /// Whether `angle_rad` is essentially equal to one of the snap targets
    /// (0°, ±45°, ±90° — modulo π for line symmetry). Used by the
    /// hysteretic snap to know whether to use the "enter" or "leave" window.
    pub fn is_at_snap_target(angle_rad: f64) -> bool {
        const EPS_DEG: f64 = 0.001;
        const TARGETS_DEG: [f64; 5] = [-90.0, -45.0, 0.0, 45.0, 90.0];
        let normalized_deg = Self::normalize_angle(angle_rad).to_degrees();
        TARGETS_DEG
            .iter()
            .any(|t| (normalized_deg - t).abs() < EPS_DEG)
    }

    /// If `angle_rad` is within `threshold_deg` of one of the target angles
    /// (0°, ±45°, ±90°), return the snapped angle in the same
    /// (screen-radian) convention as the input. Otherwise return the input
    /// unchanged.
    pub fn snap_angle_with_threshold(angle_rad: f64, threshold_deg: f64) -> f64 {
        const TARGETS_DEG: [f64; 5] = [-90.0, -45.0, 0.0, 45.0, 90.0];
        let normalized_deg = Self::normalize_angle(angle_rad).to_degrees();
        for t in TARGETS_DEG {
            if (normalized_deg - t).abs() < threshold_deg {
                // The ruler line is symmetric: `base + k*π` for any integer k
                // describes the same physical line direction. We pick the k
                // that keeps the returned value closest to the input — this
                // avoids a π-jump in `ruler.angle` when the user rotates
                // across a snap boundary (which would rotate the anchor
                // around the pivot by π and visibly shift the tick pattern
                // along the ruler's axis).
                let base = -t.to_radians();
                let pi = std::f64::consts::PI;
                let k = ((angle_rad - base) / pi).round();
                return base + k * pi;
            }
        }
        angle_rad
    }

    /// Hysteretic snap: the narrow "leave" window applies only to the target
    /// the ruler is *currently* locked on, so it's easy to escape that one
    /// without making it hard to engage any of the others.
    pub fn snap_angle_hysteretic(raw_angle: f64, prev_angle: f64) -> f64 {
        const TARGETS_DEG: [f64; 5] = [-90.0, -45.0, 0.0, 45.0, 90.0];
        const EPS_DEG: f64 = 0.001;
        let normalized_deg = Self::normalize_angle(raw_angle).to_degrees();
        let prev_normalized_deg = Self::normalize_angle(prev_angle).to_degrees();
        for t in TARGETS_DEG {
            let was_at_this_target = (prev_normalized_deg - t).abs() < EPS_DEG;
            let threshold = if was_at_this_target {
                Self::ANGLE_SNAP_LEAVE_THRESHOLD_DEG
            } else {
                Self::ANGLE_SNAP_THRESHOLD_DEG
            };
            if (normalized_deg - t).abs() < threshold {
                let base = -t.to_radians();
                let pi = std::f64::consts::PI;
                let k = ((raw_angle - base) / pi).round();
                return base + k * pi;
            }
        }
        raw_angle
    }

    /// Back-compat wrapper using the enter-threshold only.
    pub fn snap_angle(angle_rad: f64) -> f64 {
        Self::snap_angle_with_threshold(angle_rad, Self::ANGLE_SNAP_THRESHOLD_DEG)
    }

    /// Whether `pos_doc` (in document coordinates) lies within the ruler body
    /// strip (infinite along the long axis, finite across).
    pub fn hit_body(&self, pos_doc: Vector2, camera_offset: Vector2, total_zoom: f64) -> bool {
        if !self.visible {
            return false;
        }
        let pos_scroller = Self::pos_from_doc(pos_doc, camera_offset, total_zoom);
        let rel = pos_scroller - self.anchor;
        rel.dot(self.normal()).abs() <= self.body_half_width
    }
}
