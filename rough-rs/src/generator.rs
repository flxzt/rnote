use svg::node::element;

use crate::{options::{FillStyle, Options}, renderer, utils};

pub struct RoughGenerator {
    config: Options,
}

impl Default for RoughGenerator {
    fn default() -> Self {
        Self {
            config: Options {
                max_randomness_offset: Some(2.0),
                roughness: Some(1.0),
                bowing: Some(1.0),
                stroke: Some(utils::Color::new(0.0, 0.0, 0.0, 1.0)),
                stroke_width: Some(1.0),
                curve_fitting: Some(0.95),
                curve_tightness: Some(0.0),
                curve_stepcount: Some(9.0),
                fill_style: Some(FillStyle::default()),
                fill_weight: Some(-1.0),
                hachure_angle: Some(-41.0),
                hachure_gap: Some(-1.0),
                dash_offset: Some(-1.0),
                dash_gap: Some(-1.0),
                zigzag_offset: Some(-1.0),
                combine_nested_svg_paths: Some(false),
                disable_multistroke: Some(false),
                disable_multistroke_fill: Some(false),
                preserve_vertices: Some(false),
                ..Options::default()
            },
        }
    }
}

impl RoughGenerator {
    #[allow(dead_code)]
    fn merge_defaults(self) -> Self {
        Self {
            config: self.config.merge(Self::default().config),
            .. self
        }
    }

    pub fn new(config: Option<Options>) -> Self {
        Self {
            config: config.unwrap_or_default().merge(Self::default().config),
            .. Self::default()
        }
    }

    pub fn line(
        &mut self,
        start: na::Vector2<f64>,
        end: na::Vector2<f64>,
    ) -> element::Path {
        self.config.fill = Some(String::from("None"));
        let data = renderer::double_line(start, end, &self.config, false);


        let path = element::Path::new().set("d", data);

        self.config.apply_to_path(path)
    }
}
