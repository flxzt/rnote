use std::fmt::Display;
use std::ops::MulAssign;

use roughr::PathSegment;
use roughr::core::{Drawable, OpSet, OpSetType, OpType, Options};
use roughr::generator::Generator;
use roughr::{Float, FromPrimitive};
use roughr::{Point2D, Trig};
use vello_cpu::RenderContext;
use vello_cpu::color::{AlphaColor, Srgb};
use vello_cpu::kurbo::{self, BezPath, Cap, Join, PathEl, Point, Stroke};
use vello_cpu::peniko::Fill;

const DEFAULT_MITER_LIMIT: f64 = 10.0;

#[derive(Default)]
pub struct KurboGenerator {
    _gen: Generator,
    options: Option<Options>,
}

#[derive(Clone)]
pub struct KurboOpset<F: Float + Trig> {
    pub op_set_type: OpSetType,
    pub ops: BezPath,
    pub size: Option<Point2D<F>>,
    pub path: Option<String>,
}

pub trait ToKurboOpset<F: Float + Trig> {
    fn to_kurbo_opset(self) -> KurboOpset<F>;
}

impl<F: Float + Trig + FromPrimitive> ToKurboOpset<F> for OpSet<F> {
    fn to_kurbo_opset(self) -> KurboOpset<F> {
        KurboOpset {
            op_set_type: self.op_set_type.clone(),
            size: self.size,
            path: self.path.clone(),
            ops: opset_to_shape(&self),
        }
    }
}

pub struct KurboDrawable<F: Float + Trig> {
    pub shape: String,
    pub options: Options,
    pub sets: Vec<KurboOpset<F>>,
}

pub trait ToKurboDrawable<F: Float + Trig> {
    fn to_kurbo_drawable(self) -> KurboDrawable<F>;
}

impl<F: Float + Trig + FromPrimitive> ToKurboDrawable<F> for Drawable<F> {
    fn to_kurbo_drawable(self) -> KurboDrawable<F> {
        KurboDrawable {
            shape: self.shape,
            options: self.options,
            sets: self.sets.into_iter().map(|s| s.to_kurbo_opset()).collect(),
        }
    }
}

impl KurboGenerator {
    pub fn new(options: Options) -> Self {
        KurboGenerator {
            _gen: Generator::default(),
            options: Some(options),
        }
    }
}

impl<F: Float + Trig> KurboDrawable<F> {
    pub fn draw(&self, ctx: &mut RenderContext) {
        for set in self.sets.iter() {
            match set.op_set_type {
                OpSetType::Path => {
                    let ctx_state = ctx.save_current_state();
                    if self.options.stroke_line_dash.is_some() {
                        let line_cap = convert_line_cap_from_roughr_to_kurbo(self.options.line_cap);
                        let (line_join, miter_limit) =
                            convert_line_join_from_roughr_to_kurbo(self.options.line_join);

                        ctx.set_stroke(Stroke {
                            width: self.options.stroke_width.unwrap_or(1.0) as f64,
                            join: line_join,
                            miter_limit,
                            start_cap: line_cap,
                            end_cap: line_cap,
                            dash_pattern: self
                                .options
                                .stroke_line_dash
                                .clone()
                                .unwrap_or(Vec::new())
                                .into(),
                            dash_offset: self.options.stroke_line_dash_offset.unwrap_or(1.0f64),
                            ..Default::default()
                        });

                        ctx.set_paint(
                            self.options
                                .stroke
                                .map(|c| AlphaColor::<Srgb>::new([c.red, c.green, c.blue, c.alpha]))
                                .unwrap_or_else(|| AlphaColor::<Srgb>::new([1.0, 1.0, 1.0, 1.0])),
                        );
                        ctx.stroke();
                    } else {
                        ctx.set_stroke(Stroke {
                            width: self.options.stroke_width.unwrap_or(1.0) as f64,
                            ..Default::default()
                        });
                        ctx.set_paint(
                            self.options
                                .stroke
                                .map(|c| AlphaColor::<Srgb>::new([c.red, c.green, c.blue, c.alpha]))
                                .unwrap_or_else(|| AlphaColor::<Srgb>::new([1.0, 1.0, 1.0, 1.0])),
                        );
                        ctx.stroke();
                    }
                    ctx.restore_state(ctx_state);
                }
                OpSetType::FillPath => {
                    let ctx_state = ctx.save_current_state();
                    match self.shape.as_str() {
                        "curve" | "polygon" | "path" => {
                            ctx.set_paint(
                                self.options
                                    .fill
                                    .map(|c| {
                                        AlphaColor::<Srgb>::new([c.red, c.green, c.blue, c.alpha])
                                    })
                                    .unwrap_or_else(|| {
                                        AlphaColor::<Srgb>::new([1.0, 1.0, 1.0, 1.0])
                                    }),
                            );
                            ctx.set_fill_rule(Fill::EvenOdd);
                            ctx.fill_path(&set.ops);
                        }
                        _ => {
                            ctx.set_paint(
                                self.options
                                    .fill
                                    .map(|c| {
                                        AlphaColor::<Srgb>::new([c.red, c.green, c.blue, c.alpha])
                                    })
                                    .unwrap_or_else(|| {
                                        AlphaColor::<Srgb>::new([1.0, 1.0, 1.0, 1.0])
                                    }),
                            );
                            ctx.fill_path(&set.ops);
                        }
                    }
                    ctx.restore_state(ctx_state);
                }
                OpSetType::FillSketch => {
                    let ctx_state = ctx.save_current_state();
                    let mut fweight = self.options.fill_weight.unwrap_or_default();
                    if fweight < 0.0 {
                        fweight = self.options.stroke_width.unwrap_or(1.0) / 2.0;
                    }

                    if self.options.fill_line_dash.is_some() {
                        let line_cap = convert_line_cap_from_roughr_to_kurbo(self.options.line_cap);
                        let (line_join, miter_limit) =
                            convert_line_join_from_roughr_to_kurbo(self.options.line_join);

                        ctx.set_stroke(Stroke {
                            width: fweight as f64,
                            join: line_join,
                            miter_limit,
                            start_cap: line_cap,
                            end_cap: line_cap,
                            dash_pattern: self
                                .options
                                .fill_line_dash
                                .clone()
                                .unwrap_or(Vec::new())
                                .into(),
                            dash_offset: self.options.fill_line_dash_offset.unwrap_or(0.0f64),
                            ..Default::default()
                        });
                        ctx.set_paint(
                            self.options
                                .fill
                                .map(|c| AlphaColor::<Srgb>::new([c.red, c.green, c.blue, c.alpha]))
                                .unwrap_or_else(|| AlphaColor::<Srgb>::new([1.0, 1.0, 1.0, 1.0])),
                        );
                        ctx.stroke_path(&set.ops);
                    } else {
                        ctx.set_stroke(Stroke {
                            width: fweight as f64,
                            ..Default::default()
                        });
                        ctx.set_paint(
                            self.options
                                .fill
                                .map(|c| AlphaColor::<Srgb>::new([c.red, c.green, c.blue, c.alpha]))
                                .unwrap_or_else(|| AlphaColor::<Srgb>::new([1.0, 1.0, 1.0, 1.0])),
                        );
                        ctx.stroke_path(&set.ops);
                    }
                    ctx.restore_state(ctx_state);
                }
            }
        }
    }
}

fn opset_to_shape<F: Trig + Float + FromPrimitive>(op_set: &OpSet<F>) -> BezPath {
    let mut path: BezPath = BezPath::new();
    for item in op_set.ops.iter() {
        match item.op {
            OpType::Move => path.extend([PathEl::MoveTo(Point::new(
                item.data[0].to_f64().unwrap(),
                item.data[1].to_f64().unwrap(),
            ))]),
            OpType::BCurveTo => path.extend([PathEl::CurveTo(
                Point::new(
                    item.data[0].to_f64().unwrap(),
                    item.data[1].to_f64().unwrap(),
                ),
                Point::new(
                    item.data[2].to_f64().unwrap(),
                    item.data[3].to_f64().unwrap(),
                ),
                Point::new(
                    item.data[4].to_f64().unwrap(),
                    item.data[5].to_f64().unwrap(),
                ),
            )]),
            OpType::LineTo => {
                path.extend([PathEl::LineTo(Point::new(
                    item.data[0].to_f64().unwrap(),
                    item.data[1].to_f64().unwrap(),
                ))]);
            }
        }
    }
    path
}

impl KurboGenerator {
    pub fn line<F: Trig + Float + FromPrimitive>(
        &self,
        x1: F,
        y1: F,
        x2: F,
        y2: F,
    ) -> KurboDrawable<F> {
        let drawable = self._gen.line(x1, y1, x2, y2, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn rectangle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> KurboDrawable<F> {
        let drawable = self._gen.rectangle(x, y, width, height, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn ellipse<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> KurboDrawable<F> {
        let drawable = self._gen.ellipse(x, y, width, height, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn circle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        diameter: F,
    ) -> KurboDrawable<F> {
        let drawable = self._gen.circle(x, y, diameter, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn linear_path<F: Trig + Float + FromPrimitive>(
        &self,
        points: &[Point2D<F>],
        close: bool,
    ) -> KurboDrawable<F> {
        let drawable = self._gen.linear_path(points, close, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn polygon<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        points: &[Point2D<F>],
    ) -> KurboDrawable<F> {
        let drawable = self._gen.polygon(points, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn arc<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
        start: F,
        stop: F,
        closed: bool,
    ) -> KurboDrawable<F> {
        let drawable = self
            ._gen
            .arc(x, y, width, height, start, stop, closed, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn bezier_quadratic<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        start: Point2D<F>,
        cp: Point2D<F>,
        end: Point2D<F>,
    ) -> KurboDrawable<F> {
        let drawable = self._gen.bezier_quadratic(start, cp, end, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn bezier_cubic<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        start: Point2D<F>,
        cp1: Point2D<F>,
        cp2: Point2D<F>,
        end: Point2D<F>,
    ) -> KurboDrawable<F> {
        let drawable = self._gen.bezier_cubic(start, cp1, cp2, end, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn curve<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        points: &[Point2D<F>],
    ) -> KurboDrawable<F> {
        let drawable = self._gen.curve(points, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn path<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        svg_path: String,
    ) -> KurboDrawable<F> {
        let drawable = self._gen.path(svg_path, &self.options);
        drawable.to_kurbo_drawable()
    }

    pub fn bez_path<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        bezier_path: BezPath,
    ) -> KurboDrawable<F> {
        let segments = bezpath_to_svg_segments(&bezier_path);
        self._gen
            .path_from_segments(segments, &self.options)
            .to_kurbo_drawable()
    }
}

fn convert_line_cap_from_roughr_to_kurbo(roughr_line_cap: Option<roughr::core::LineCap>) -> Cap {
    match roughr_line_cap {
        Some(roughr::core::LineCap::Butt) => Cap::Butt,
        Some(roughr::core::LineCap::Round) => Cap::Round,
        Some(roughr::core::LineCap::Square) => Cap::Square,
        None => Cap::Butt,
    }
}

fn convert_line_join_from_roughr_to_kurbo(
    roughr_line_join: Option<roughr::core::LineJoin>,
) -> (Join, f64) {
    match roughr_line_join {
        Some(roughr::core::LineJoin::Miter { limit }) => (Join::Miter, limit),
        Some(roughr::core::LineJoin::Round) => (Join::Round, DEFAULT_MITER_LIMIT),
        Some(roughr::core::LineJoin::Bevel) => (Join::Bevel, DEFAULT_MITER_LIMIT),
        None => (Join::Miter, DEFAULT_MITER_LIMIT),
    }
}

pub fn bezpath_to_svg_segments(path: &BezPath) -> Vec<PathSegment> {
    let mut segments = Vec::new();

    for elem in path.elements() {
        match elem {
            kurbo::PathEl::MoveTo(p) => {
                segments.push(PathSegment::MoveTo {
                    abs: true,
                    x: p.x,
                    y: p.y,
                });
            }
            kurbo::PathEl::LineTo(p) => {
                segments.push(PathSegment::LineTo {
                    abs: true,
                    x: p.x,
                    y: p.y,
                });
            }
            kurbo::PathEl::QuadTo(p1, p2) => {
                segments.push(PathSegment::Quadratic {
                    abs: true,
                    x1: p1.x,
                    y1: p1.y,
                    x: p2.x,
                    y: p2.y,
                });
            }
            kurbo::PathEl::CurveTo(p1, p2, p3) => {
                segments.push(PathSegment::CurveTo {
                    abs: true,
                    x1: p1.x,
                    y1: p1.y,
                    x2: p2.x,
                    y2: p2.y,
                    x: p3.x,
                    y: p3.y,
                });
            }
            kurbo::PathEl::ClosePath => {
                segments.push(PathSegment::ClosePath { abs: true });
            }
        }
    }

    segments
}
