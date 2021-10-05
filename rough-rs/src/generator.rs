use svg::node::element::{self, path};

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
        }
    }

    /// Generating a single line element
    pub fn line(
        &mut self,
        start: na::Vector2<f64>,
        end: na::Vector2<f64>,
        options: Option<&Options>,
    ) -> element::Path {
        let commands = renderer::line(start, end, options.unwrap_or(&self.config), true, false);

        self.config
            .apply_to_path(element::Path::new().set("d", path::Data::from(commands)))
    }

    /// Generating a cubic bezier curve
    pub fn cubic_bezier(
        &mut self,
        start: na::Vector2<f64>,
        cp1: na::Vector2<f64>,
        cp2: na::Vector2<f64>,
        end: na::Vector2<f64>,
        options: Option<&Options>,
    ) -> element::Path {
        let commands =
            renderer::cubic_bezier(start, cp1, cp2, end, options.unwrap_or(&self.config));

        self.config
            .apply_to_path(element::Path::new().set("d", path::Data::from(commands)))
    }
}
