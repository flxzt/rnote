// Imports
use p2d::glamx::DAffine2;
use p2d::math::Vector2;
use p2d::shape::Cuboid;
use rnote_compose::Transformable;
use rnote_compose::shapes::{Ellipse, Line, Rectangle, Shape, Shapeable};

/// The extent a shape keeps at least while it is adjusted, so that it can't collapse into nothing.
const MIN_EXTENT: f64 = 1.0;

/// Adjusting a recognized shape with the pen that drew it, while that pen is still down.
///
/// The pen keeps changing the shape that replaced its stroke, like when drawing one with the shaper pen:
/// a line follows the pen with its end, a rectangle with the corner closest to the pen and an ellipse with
/// its radii. Any other shape is scaled between the pen and the corner of its bounds opposite to it.
///
/// The shape follows the movement of the pen since the recognition, so that it does not jump when the pen
/// does not rest exactly on the part of the shape that follows it.
#[derive(Debug, Clone)]
pub(crate) struct ShapeAdjust {
    /// The shape as it was recognized.
    recognized: Shape,
    /// The position of the pen when the recognized shape replaced the drawn stroke.
    start_pos: Vector2,
    kind: AdjustKind,
}

#[derive(Debug, Clone)]
enum AdjustKind {
    /// The end of the line follows the pen, its start stays fixed.
    LineEnd,
    /// The corner of the rectangle closest to the pen follows it, the opposite corner stays fixed.
    ///
    /// Holds the signs of that corner in the local frame of the rectangle.
    RectangleCorner { corner_signs: Vector2 },
    /// The radii of the ellipse follow the pen, its center stays fixed.
    ///
    /// Holds the signs of the pen position in the local frame of the ellipse.
    EllipseRadii { radii_signs: Vector2 },
    /// The shape is scaled between the pen and the corner of its bounds opposite to it.
    ScaleFromPivot {
        pivot: Vector2,
        start_extents: Vector2,
    },
}

impl ShapeAdjust {
    /// Start adjusting the given recognized shape, with the pen at `start_pos`.
    pub(crate) fn new(recognized: Shape, start_pos: Vector2) -> Self {
        let kind = match &recognized {
            Shape::Line(_) => AdjustKind::LineEnd,
            Shape::Rectangle(rectangle) => AdjustKind::RectangleCorner {
                corner_signs: signs(rectangle.affine.inverse().transform_point2(start_pos)),
            },
            Shape::Ellipse(ellipse) => AdjustKind::EllipseRadii {
                radii_signs: signs(ellipse.affine.inverse().transform_point2(start_pos)),
            },
            shape => {
                let bounds = shape.bounds();
                let center = bounds.center();
                // The corner of the bounds that lies opposite to the pen stays fixed.
                let pivot = Vector2::new(
                    if start_pos.x >= center.x {
                        bounds.mins.x
                    } else {
                        bounds.maxs.x
                    },
                    if start_pos.y >= center.y {
                        bounds.mins.y
                    } else {
                        bounds.maxs.y
                    },
                );

                AdjustKind::ScaleFromPivot {
                    pivot,
                    start_extents: bounds.extents(),
                }
            }
        };

        Self {
            recognized,
            start_pos,
            kind,
        }
    }

    /// The recognized shape, adjusted for the given pen position.
    pub(crate) fn adjusted(&self, pen_pos: Vector2) -> Shape {
        let offset = pen_pos - self.start_pos;

        match (&self.recognized, &self.kind) {
            (Shape::Line(line), AdjustKind::LineEnd) => Shape::Line(Line {
                start: line.start,
                end: line.end + offset,
            }),
            (Shape::Rectangle(rectangle), AdjustKind::RectangleCorner { corner_signs }) => {
                let local_offset = rectangle.affine.inverse().transform_vector2(offset);
                let fixed_corner = -*corner_signs * rectangle.cuboid.half_extents;
                let moved_corner = *corner_signs * rectangle.cuboid.half_extents + local_offset;
                let half_extents = ((moved_corner - fixed_corner) * 0.5)
                    .abs()
                    .max(Vector2::splat(MIN_EXTENT * 0.5));
                let center = (fixed_corner + moved_corner) * 0.5;

                Shape::Rectangle(Rectangle {
                    cuboid: Cuboid::new(half_extents),
                    affine: rectangle.affine * DAffine2::from_translation(center),
                })
            }
            (Shape::Ellipse(ellipse), AdjustKind::EllipseRadii { radii_signs }) => {
                let local_offset = ellipse.affine.inverse().transform_vector2(offset);
                let radii = (ellipse.radii + *radii_signs * local_offset)
                    .abs()
                    .max(Vector2::splat(MIN_EXTENT * 0.5));

                Shape::Ellipse(Ellipse {
                    radii,
                    affine: ellipse.affine,
                })
            }
            (
                shape,
                AdjustKind::ScaleFromPivot {
                    pivot,
                    start_extents,
                },
            ) => {
                // Moving away from the pivot grows the shape, no matter on which side of it the pen is.
                let signed_offset = Vector2::new(
                    if self.start_pos.x >= pivot.x {
                        offset.x
                    } else {
                        -offset.x
                    },
                    if self.start_pos.y >= pivot.y {
                        offset.y
                    } else {
                        -offset.y
                    },
                );
                let scale = Vector2::new(
                    axis_scale(start_extents.x, signed_offset.x),
                    axis_scale(start_extents.y, signed_offset.y),
                );

                let mut shape = shape.clone();
                shape.translate(-*pivot);
                shape.scale(scale);
                shape.translate(*pivot);
                shape
            }
            // The kind is always derived from the shape it is created with, so this can't be reached.
            (shape, _) => shape.clone(),
        }
    }
}

/// The scale for one axis that grows the given extent by the offset, keeping the minimum extent.
fn axis_scale(start_extent: f64, signed_offset: f64) -> f64 {
    if start_extent < MIN_EXTENT {
        return 1.0;
    }

    (start_extent + signed_offset).max(MIN_EXTENT) / start_extent
}

/// The signs of the components of the given vector, with zero counting as positive.
fn signs(v: Vector2) -> Vector2 {
    Vector2::new(
        if v.x < 0.0 { -1.0 } else { 1.0 },
        if v.y < 0.0 { -1.0 } else { 1.0 },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rnote_compose::shapes::Polygon;

    #[test]
    fn line_end_follows_pen() {
        let line = Line {
            start: Vector2::new(100.0, 100.0),
            end: Vector2::new(140.0, 100.0),
        };
        let adjust = ShapeAdjust::new(Shape::Line(line), Vector2::new(140.0, 100.0));

        match adjust.adjusted(Vector2::new(160.0, 120.0)) {
            Shape::Line(adjusted) => {
                assert_eq!(adjusted.start, line.start);
                assert_eq!(adjusted.end, Vector2::new(160.0, 120.0));
            }
            other => panic!("expected line, got {other:?}"),
        }
    }

    #[test]
    fn rectangle_corner_follows_pen() {
        // A rectangle from (80, 90) to (120, 110), with the pen at its lower right corner.
        let rectangle = Rectangle {
            cuboid: Cuboid::new(Vector2::new(20.0, 10.0)),
            affine: DAffine2::from_translation(Vector2::new(100.0, 100.0)),
        };
        let adjust = ShapeAdjust::new(Shape::Rectangle(rectangle), Vector2::new(120.0, 110.0));

        match adjust.adjusted(Vector2::new(130.0, 110.0)) {
            Shape::Rectangle(adjusted) => {
                // The upper left corner stays fixed, the lower right one moved with the pen.
                let bounds = Shape::Rectangle(adjusted).bounds();
                assert!((bounds.mins - Vector2::new(80.0, 90.0)).length() < 1e-6);
                assert!((bounds.maxs - Vector2::new(130.0, 110.0)).length() < 1e-6);
            }
            other => panic!("expected rectangle, got {other:?}"),
        }
    }

    #[test]
    fn rotated_rectangle_grows_along_its_own_axes() {
        // Rotated by 90 degrees, so moving the pen along x grows the rectangle along its local y axis.
        let rectangle = Rectangle {
            cuboid: Cuboid::new(Vector2::new(20.0, 10.0)),
            affine: DAffine2::from_translation(Vector2::new(100.0, 100.0))
                * DAffine2::from_angle(std::f64::consts::FRAC_PI_2),
        };
        let start_pos = rectangle.affine.transform_point2(Vector2::new(20.0, 10.0));
        let adjust = ShapeAdjust::new(Shape::Rectangle(rectangle), start_pos);

        match adjust.adjusted(start_pos + Vector2::new(0.0, 10.0)) {
            Shape::Rectangle(adjusted) => {
                assert!((adjusted.cuboid.half_extents.x - 25.0).abs() < 1e-6);
                assert!((adjusted.cuboid.half_extents.y - 10.0).abs() < 1e-6);
            }
            other => panic!("expected rectangle, got {other:?}"),
        }
    }

    #[test]
    fn ellipse_radii_follow_pen_around_its_center() {
        let ellipse = Ellipse {
            radii: Vector2::new(30.0, 20.0),
            affine: DAffine2::from_translation(Vector2::new(100.0, 100.0)),
        };
        let adjust = ShapeAdjust::new(Shape::Ellipse(ellipse), Vector2::new(130.0, 100.0));

        match adjust.adjusted(Vector2::new(140.0, 105.0)) {
            Shape::Ellipse(adjusted) => {
                assert!((adjusted.radii - Vector2::new(40.0, 25.0)).length() < 1e-6);
                // The center stays where it was.
                assert!((adjusted.affine.translation - Vector2::new(100.0, 100.0)).length() < 1e-6);
            }
            other => panic!("expected ellipse, got {other:?}"),
        }
    }

    #[test]
    fn polygon_scales_from_the_opposite_corner() {
        // A triangle with bounds from (100, 100) to (140, 140), with the pen at its lower right.
        let polygon = Polygon {
            start: Vector2::new(100.0, 140.0),
            path: vec![Vector2::new(140.0, 140.0), Vector2::new(120.0, 100.0)],
        };
        let adjust = ShapeAdjust::new(Shape::Polygon(polygon), Vector2::new(140.0, 140.0));

        // Dragging away from the upper left corner of the bounds grows the triangle.
        let adjusted = adjust.adjusted(Vector2::new(160.0, 160.0));
        let bounds = adjusted.bounds();
        assert!((bounds.mins - Vector2::new(100.0, 100.0)).length() < 1e-6);
        assert!((bounds.maxs - Vector2::new(160.0, 160.0)).length() < 1e-6);
    }
}
