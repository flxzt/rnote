// Imports
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::eventresult::EventPropagation;
use crate::ext::AabbExt;
use crate::penevent::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::{Line, Rectangle};
use crate::style::{indicators, Composer};
use crate::{Constraints, EventResult};
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
enum GridBuilderState {
    FirstCell {
        start: na::Vector2<f64>,
        current: na::Vector2<f64>,
    },
    FirstCellFinished {
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

impl BuilderCreator for GridBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            state: GridBuilderState::FirstCell {
                start: element.pos,
                current: element.pos,
            },
        }
    }
}

impl Buildable for GridBuilder {
    type Emit = Shape;

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        constraints: Constraints,
    ) -> EventResult<BuilderProgress<Self::Emit>> {
        let progress = match (&mut self.state, event) {
            (GridBuilderState::FirstCell { start, current }, PenEvent::Down { element, .. }) => {
                *current = constraints.constrain(element.pos - *start) + *start;
                BuilderProgress::InProgress
            }
            (GridBuilderState::FirstCell { start, .. }, PenEvent::Up { element, .. }) => {
                let cell_size = constraints.constrain(element.pos - *start);

                if cell_size.x.abs() < Self::FIRST_CELL_DIMENSIONS_MIN
                    || cell_size.y.abs() < Self::FIRST_CELL_DIMENSIONS_MIN
                {
                    BuilderProgress::Finished(vec![])
                } else {
                    self.state = GridBuilderState::FirstCellFinished {
                        start: *start,
                        current: cell_size + *start,
                    };
                    BuilderProgress::InProgress
                }
            }
            (GridBuilderState::FirstCell { .. }, ..) => BuilderProgress::InProgress,
            (
                GridBuilderState::FirstCellFinished { start, current },
                PenEvent::Down { element, .. },
            ) => {
                self.state = GridBuilderState::Grids {
                    start: *start,
                    cell_size: (*current - *start),
                    current: constraints.constrain(element.pos - *start) + *start,
                };
                BuilderProgress::InProgress
            }
            (GridBuilderState::FirstCellFinished { .. }, ..) => BuilderProgress::InProgress,
            (GridBuilderState::Grids { current, .. }, PenEvent::Down { element, .. }) => {
                // The grid is already constrained by the cell size
                *current = element.pos;
                BuilderProgress::InProgress
            }
            (GridBuilderState::Grids { .. }, PenEvent::Up { .. }) => BuilderProgress::Finished(
                self.state_as_lines().into_iter().map(Shape::Line).collect(),
            ),
            (GridBuilderState::Grids { .. }, ..) => BuilderProgress::InProgress,
        };

        EventResult {
            handled: true,
            propagate: EventPropagation::Stop,
            progress,
        }
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        let bounds_margin = style.bounds_margin().max(indicators::POS_INDICATOR_RADIUS) / zoom;

        match &self.state {
            GridBuilderState::FirstCell { start, current }
            | GridBuilderState::FirstCellFinished { start, current }
            | GridBuilderState::Grids { start, current, .. } => {
                Some(Aabb::new_positive((*start).into(), (*current).into()).loosened(bounds_margin))
            }
        }
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();

        let mut style = style.clone();

        for line in self.state_as_lines() {
            line.draw_composed(cx, &style);

            style.advance_seed();
        }

        match &self.state {
            GridBuilderState::FirstCell { start, current }
            | GridBuilderState::FirstCellFinished { start, current } => {
                indicators::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                indicators::draw_pos_indicator(cx, PenState::Down, *current, zoom);
            }
            GridBuilderState::Grids {
                start,
                cell_size,
                current,
            } => {
                indicators::draw_pos_indicator(cx, PenState::Up, *start, zoom);

                let cols = ((current - start)[0] / cell_size[0])
                    .floor()
                    .min(Self::CELL_GRID_DIMENSIONS_MAX as f64);
                let rows = ((current - start)[1] / cell_size[1])
                    .floor()
                    .min(Self::CELL_GRID_DIMENSIONS_MAX as f64);

                if cols > 0.0 && rows > 0.0 {
                    indicators::draw_pos_indicator(cx, PenState::Up, *start + cell_size, zoom);
                    indicators::draw_pos_indicator(
                        cx,
                        PenState::Up,
                        *start + cell_size.component_mul(&na::vector![cols, rows]),
                        zoom,
                    );
                }
            }
        }

        cx.restore().unwrap();
    }
}

impl GridBuilder {
    const FIRST_CELL_DIMENSIONS_MIN: f64 = 2.0;
    const CELL_GRID_DIMENSIONS_MAX: u32 = 100;

    fn state_as_lines(&self) -> Vec<Line> {
        match &self.state {
            GridBuilderState::FirstCell { start, current }
            | GridBuilderState::FirstCellFinished { start, current } => {
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

                    (
                        (cols.floor() as u32).min(Self::CELL_GRID_DIMENSIONS_MAX),
                        (rows.floor() as u32).min(Self::CELL_GRID_DIMENSIONS_MAX),
                    )
                };

                // lines of the upper side
                let mut lines = (0..cols)
                    .map(|col| Line {
                        start: na::vector![start[0] + cell_size[0] * col as f64, start[1]],
                        end: na::vector![start[0] + cell_size[0] * (col + 1) as f64, start[1]],
                    })
                    .collect::<Vec<Line>>();

                // lines of the left side
                lines.extend((0..rows).map(|row| Line {
                    start: na::vector![start[0], start[1] + cell_size[1] * row as f64],
                    end: na::vector![start[0], start[1] + cell_size[1] * (row + 1) as f64],
                }));

                // cell outlines
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
