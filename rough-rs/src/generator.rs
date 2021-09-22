use svg::node::element;

use crate::{options::Options, renderer};

/// The Rough generator.
pub struct RoughGenerator {
    /// The config for the Rough generator
    pub config: Options,
}

impl Default for RoughGenerator {
    fn default() -> Self {
        Self {
            config: Options {
                ..Options::default()
            },
        }
    }
}

impl RoughGenerator {
    /// Creating a new instance of RoughGenerator
    pub fn new(config: Option<Options>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            ..Self::default()
        }
    }

    /// Generating a single line element
    pub fn line(
        &mut self,
        start: na::Vector2<f64>,
        end: na::Vector2<f64>,
        options: Option<&Options>,
    ) -> element::Path {
        let svg_path = renderer::line(start, end, options.unwrap_or(&self.config), true, false);

        self.config.apply_to_path(svg_path)
    }

    /// Generating a cubic bezier curve
    pub fn cubic_bezier(
        &mut self,
        start: na::Vector2<f64>,
        first: na::Vector2<f64>,
        second: na::Vector2<f64>,
        end: na::Vector2<f64>,
        options: Option<&Options>,
    ) -> element::Path {
        let svg_path =
            renderer::cubic_bezier(start, first, second, end, options.unwrap_or(&self.config));

        self.config.apply_to_path(svg_path)
    }
}
