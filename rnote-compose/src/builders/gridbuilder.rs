use std::time::Instant;

use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;

use crate::helpers::AABBHelpers;
use crate::penhelpers::PenEvent;
use crate::penpath::Element;
use crate::shapes::{Line, Rectangle};
use crate::style::Composer;
use crate::{Shape, Style};

use super::shapebuilderbehaviour::{ShapeBuilderCreator, ShapeBuilderProgress};
use super::{Constraints, ShapeBuilderBehaviour};

#[derive(Debug, Clone, Copy)]
enum GridBuilderState {
    Start(na::Vector2<f64>),
    FirstCell {
        start: na::Vector2<f64>,
        current: na::Vector2<f64>,
    },
    Grids {
        start: na::Vector2<f64>,
        cell_size: na::Vector2<f64>,
        current: na::Vector2<f64>,
    },
}

/// rect builder
#[derive(Debug, Clone)]
pub struct GridBuilder {
    state: GridBuilderState,
}

impl ShapeBuilderCreator for GridBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            state: GridBuilderState::Start(element.pos),
        }
    }
}

impl ShapeBuilderBehaviour for GridBuilder {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        constraints: Constraints,
    ) -> ShapeBuilderProgress {
        //log::debug!("state: {:?}, event: {:?}", &self.state, &event);

        match (&mut self.state, event) {
            (GridBuilderState::Start(start), PenEvent::Down { element, .. }) => {
                self.state = GridBuilderState::FirstCell {
                    start: *start,
                    current: element.pos,
                };
            }
            (GridBuilderState::Start(_), ..) => {}
            (GridBuilderState::FirstCell { start, current }, PenEvent::Down { element, .. }) => {
                *current = constraints.constrain(element.pos - *start) + *start;
            }
            (GridBuilderState::FirstCell { start, current }, PenEvent::Up { element, .. }) => {
                self.state = GridBuilderState::Grids {
                    start: *start,
                    cell_size: (*current - *start),
                    current: constraints.constrain(element.pos - *start) + *start,
                };
            }
            (GridBuilderState::FirstCell { .. }, ..) => {}
            (GridBuilderState::Grids { current, .. }, PenEvent::Down { element, .. }) => {
                // The grid is already constrained by the cell size
                *current = element.pos;
            }
            (GridBuilderState::Grids { .. }, PenEvent::Up { .. }) => {
                return ShapeBuilderProgress::Finished(
                    self.state_as_lines().into_iter().map(Shape::Line).collect(),
                );
            }
            (GridBuilderState::Grids { .. }, ..) => {}
        }

        ShapeBuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, _zoom: f64) -> Option<AABB> {
        let bounds_margin = style.bounds_margin();

        match &self.state {
            GridBuilderState::Start(start) => Some(AABB::from_half_extents(
                na::Point2::from(*start),
                na::Vector2::repeat(bounds_margin),
            )),
            GridBuilderState::FirstCell { start, current }
            | GridBuilderState::Grids { start, current, .. } => Some(
                AABB::new_positive(na::Point2::from(*start), na::Point2::from(*current))
                    .loosened(bounds_margin),
            ),
        }
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, _zoom: f64) {
        cx.save().unwrap();

        let mut style = style.clone();

        for line in self.state_as_lines() {
            line.draw_composed(cx, &style);

            style.advance_seed();
        }

        cx.restore().unwrap();
    }
}

impl GridBuilder {
    fn state_as_lines(&self) -> Vec<Line> {
        match &self.state {
            GridBuilderState::Start(_) => vec![],
            GridBuilderState::FirstCell { start, current } => {
                Rectangle::from_corners(*start, *current)
                    .outline_lines()
                    .into_iter()
                    .collect()
            }
            GridBuilderState::Grids {
                start,
                cell_size,
                current,
            } => {
                let (cols, rows) = {
                    let cols = (current - start)[0] / cell_size[0];
                    let rows = (current - start)[1] / cell_size[1];

                    // is only met when having a positive initial cell size, but want to span in negative direction, or the other way around
                    if cols.is_sign_negative() || rows.is_sign_negative() {
                        return vec![];
                    }

                    (cols.abs().floor() as u32, rows.abs().floor() as u32)
                };

                let mut lines = (0..cols)
                    .map(|col| Line {
                        start: na::vector![start[0] + cell_size[0] * col as f64, start[1]],
                        end: na::vector![start[0] + cell_size[0] * (col + 1) as f64, start[1]],
                    })
                    .collect::<Vec<Line>>();

                lines.extend((0..rows).map(|row| Line {
                    start: na::vector![start[0], start[1] + cell_size[1] * row as f64],
                    end: na::vector![start[0], start[1] + cell_size[1] * (row + 1) as f64],
                }));

                lines.extend((0..rows).flat_map(move |row| {
                    (0..cols).flat_map(move |col| {
                        let corner =
                            start + cell_size.component_mul(&na::vector![col as f64, row as f64]);

                        [
                            Line {
                                start: na::vector![corner[0] + cell_size[0], corner[1]],
                                end: na::vector![
                                    corner[0] + cell_size[0],
                                    corner[1] + cell_size[1]
                                ],
                            },
                            Line {
                                start: na::vector![corner[0], corner[1] + cell_size[1]],
                                end: na::vector![
                                    corner[0] + cell_size[0],
                                    corner[1] + cell_size[1]
                                ],
                            },
                        ]
                    })
                }));

                lines
            }
        }
    }
}
