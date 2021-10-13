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
    pub fn line(&mut self, start: na::Vector2<f64>, end: na::Vector2<f64>) -> element::Path {
        let commands = if !self.config.disable_multistroke {
            renderer::doubleline(start, end, &mut self.config)
        } else {
            renderer::line(start, end, &mut self.config, true, false)
        };

        self.config
            .apply_to_line(element::Path::new().set("d", path::Data::from(commands)))
    }

    /// Generating a cubic bezier curve
    pub fn cubic_bezier(
        &mut self,
        start: na::Vector2<f64>,
        cp1: na::Vector2<f64>,
        cp2: na::Vector2<f64>,
        end: na::Vector2<f64>,
    ) -> element::Path {
        let commands = renderer::cubic_bezier(start, cp1, cp2, end, &mut self.config);

        self.config
            .apply_to_line(element::Path::new().set("d", path::Data::from(commands)))
    }

    /// Generating a rectangle
    pub fn rectangle(
        &mut self,
        top_left: na::Vector2<f64>,
        bottom_right: na::Vector2<f64>,
    ) -> element::Group {
        let mut commands = Vec::new();

        if !self.config.disable_multistroke {
            commands.append(&mut renderer::doubleline(
                top_left,
                na::vector![bottom_right[0], top_left[1]],
                &mut self.config,
            ));
            commands.append(&mut renderer::doubleline(
                na::vector![bottom_right[0], top_left[1]],
                bottom_right,
                &mut self.config,
            ));
            commands.append(&mut renderer::doubleline(
                bottom_right,
                na::vector![top_left[0], bottom_right[1]],
                &mut self.config,
            ));
            commands.append(&mut renderer::doubleline(
                na::vector![top_left[0], bottom_right[1]],
                top_left,
                &mut self.config,
            ));
        } else {
            commands.append(&mut renderer::line(
                top_left,
                na::vector![bottom_right[0], top_left[1]],
                &mut self.config,
                true,
                false,
            ));
            commands.append(&mut renderer::line(
                na::vector![bottom_right[0], top_left[1]],
                bottom_right,
                &mut self.config,
                true,
                false,
            ));
            commands.append(&mut renderer::line(
                bottom_right,
                na::vector![top_left[0], bottom_right[1]],
                &mut self.config,
                true,
                false,
            ));
            commands.append(&mut renderer::line(
                na::vector![top_left[0], bottom_right[1]],
                top_left,
                &mut self.config,
                true,
                false,
            ));
        }

        let rect = self
            .config
            .apply_to_rect(element::Path::new().set("d", path::Data::from(commands)));

        let fill_points = vec![
            na::vector![top_left[0], top_left[1]],
            na::vector![bottom_right[0], top_left[1]],
            na::vector![bottom_right[0], bottom_right[1]],
            na::vector![top_left[0], bottom_right[1]],
        ];
        let fill_polygon = self.fill_polygon(fill_points);

        element::Group::new().add(fill_polygon).add(rect)
    }

    /// Generating a fill polygon
    pub fn fill_polygon(&mut self, points: Vec<na::Vector2<f64>>) -> element::Path {
        let mut commands = Vec::new();

        commands.append(&mut renderer::fill_polygon(points, &mut self.config));

        self.config
            .apply_to_fill_polygon_solid(element::Path::new().set("d", path::Data::from(commands)))
    }

    /// Generating a ellipse
    pub fn ellipse(
        &mut self,
        center: na::Vector2<f64>,
        radius_x: f64,
        radius_y: f64,
    ) -> element::Group {
        let ellipse_result = renderer::ellipse(center, radius_x, radius_y, &mut self.config);

        let ellipse = self.config.apply_to_ellipse(
            element::Path::new().set("d", path::Data::from(ellipse_result.commands)),
        );

        let fill_polygon = self.fill_polygon(ellipse_result.estimated_points);

        element::Group::new().add(fill_polygon).add(ellipse)
    }
}
