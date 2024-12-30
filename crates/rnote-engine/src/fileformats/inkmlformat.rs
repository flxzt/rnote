use crate::strokes::BrushStroke;
use crate::strokes::Stroke;
use rnote_compose::penpath::Element;
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::style::PressureCurve;
use rnote_compose::Color;
use rnote_compose::PenPath;
use std::sync::Arc;
use writer_inkml::{Brush, FormattedStroke};

pub fn inkml_to_stroke(
    formatted_stroke: FormattedStroke,
    brush: Brush,
    dpi: &f64,
) -> Option<Arc<Stroke>> {
    let mut smooth_options = SmoothOptions::default();
    smooth_options.stroke_color = Some(Color::new(
        brush.color.0 as f64 / 255.0,
        brush.color.1 as f64 / 255.0,
        brush.color.2 as f64 / 255.0,
        1.0 - brush.transparency as f64 / 255.0,
    ));

    // converting from mm to px
    smooth_options.stroke_width = dpi * brush.stroke_width / (10.0 * 2.54);

    // pressure curve
    if brush.ignorepressure {
        smooth_options.pressure_curve = PressureCurve::Const;
    } else {
        smooth_options.pressure_curve = PressureCurve::Linear;
    }

    let penpath = PenPath::try_from_elements(
        formatted_stroke
            .x
            .into_iter()
            .zip(formatted_stroke.y)
            .zip(formatted_stroke.f)
            .map(|((x, y), f)| Element::new(*dpi * na::vector![x, y] / 2.54, f)),
    );
    if penpath.is_some() {
        let new_stroke = BrushStroke::from_penpath(
            penpath.unwrap(),
            rnote_compose::Style::Smooth(smooth_options),
        );
        Some(Arc::new(Stroke::BrushStroke(new_stroke)))
    } else {
        None
    }
}
