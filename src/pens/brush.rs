use std::{cell::RefCell, error::Error, rc::Rc};

use gtk4::{gdk, gio};
use rand::{distributions::Uniform, prelude::Distribution};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{
    config,
    strokes::{self, brushstroke::BrushStroke, compose, render, InputData, StrokeBehaviour},
    utils,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TemplateType {
    Linear,
    CubicBezier,
    Experimental,
    Custom(String),
}

impl Default for TemplateType {
    fn default() -> Self {
        Self::CubicBezier
    }
}

impl TemplateType {
    pub fn template_name(&self) -> String {
        // Must match file names in resources as they will be installed pkgdatadir and templates_config_dir
        match self {
            Self::Linear => String::from("brushstroke-linear"),
            Self::CubicBezier => String::from("brushstroke-cubicbezier"),
            Self::Experimental => String::from("brushstroke-experimental"),
            Self::Custom(_) => String::from("brushstroke-custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brush {
    width: f64,
    sensitivity: f64,
    color: strokes::Color,
    // Templates get Rc::clone()'d into the individual strokes so overwriting templates does not affect existing strokes.
    #[serde(skip, default = "Brush::default_brush_templates")]
    pub templates: Rc<RefCell<Tera>>,
    pub current_template: TemplateType,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            sensitivity: Self::SENSITIVITY_DEFAULT,
            color: strokes::Color::from_gdk(Self::COLOR_DEFAULT),
            templates: Self::default_brush_templates(),
            current_template: TemplateType::default(),
        }
    }
}

impl Brush {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 12.0;
    pub const SENSITIVITY_MIN: f64 = 0.0;
    pub const SENSITIVITY_MAX: f64 = 1.0;
    pub const SENSITIVITY_DEFAULT: f64 = 0.5;

    pub const COLOR_DEFAULT: gdk::RGBA = gdk::RGBA {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }

    pub fn sensitivity(&self) -> f64 {
        self.sensitivity
    }

    pub fn set_sensitivity(&mut self, sensitivity: f64) {
        self.sensitivity = sensitivity.clamp(Self::SENSITIVITY_MIN, Self::SENSITIVITY_MAX);
    }

    pub fn color(&self) -> strokes::Color {
        self.color
    }

    pub fn set_color(&mut self, color: strokes::Color) {
        self.color = color;
    }

    pub fn replace_custom_template(&self, file: &gio::File) -> Result<(), Box<dyn Error>> {
        let template_string = utils::load_file_contents(file)?;

        self.templates.borrow_mut().add_raw_template(
            TemplateType::Custom(String::from(""))
                .template_name()
                .as_str(),
            template_string.as_str(),
        )?;

        Ok(())
    }

    pub fn register_custom_template(&self) -> Result<(), Box<dyn Error>> {
        if let TemplateType::Custom(template_string) = self.current_template.to_owned() {
            self.templates.borrow_mut().add_raw_template(
                TemplateType::Custom(String::from(""))
                    .template_name()
                    .as_str(),
                template_string.as_str(),
            )?;
        }
        Ok(())
    }

    pub fn default_brush_templates() -> Rc<RefCell<Tera>> {
        let mut templates = Tera::default();

        let brushstroke_linear_template_path = String::from(config::APP_IDPATH)
            + "templates/"
            + TemplateType::Linear.template_name().as_str()
            + ".svg.templ";
        templates
            .add_raw_template(
                TemplateType::Linear.template_name().as_str(),
                utils::load_string_from_resource(brushstroke_linear_template_path.as_str())
                    .expect("failed to load string from resource")
                    .as_str(),
            )
            .expect("Failed to add linear template for brushstroke to `templates`");

        let brushstroke_cubicbezier_template_path = String::from(config::APP_IDPATH)
            + "templates/"
            + TemplateType::CubicBezier.template_name().as_str()
            + ".svg.templ";
        templates
            .add_raw_template(
                TemplateType::CubicBezier.template_name().as_str(),
                utils::load_string_from_resource(brushstroke_cubicbezier_template_path.as_str())
                    .expect("failed to load string from resource")
                    .as_str(),
            )
            .expect("Failed to add cubicbezier template for brushstroke to `templates`");

        Rc::new(RefCell::new(templates))
    }
}

pub fn validate_brush_template_for_file(file: &gio::File) -> Result<(), Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let strokes_uniform = Uniform::from(0..=3);

    let bounds = p2d::bounding_volume::AABB::new(na::point![0.0, 0.0], na::point![2000.0, 2000.0]);
    let mut brush = Brush::default();
    let renderer = render::Renderer::default();

    brush.replace_custom_template(file)?;
    brush.current_template = TemplateType::Custom(utils::load_file_contents(file)?);

    for _i in 0..=strokes_uniform.sample(&mut rng) {
        let validation_stroke =
            BrushStroke::validation_stroke(&InputData::validation_data(bounds), &brush).unwrap();
        let svg = compose::wrap_svg(
            validation_stroke
                .gen_svg_data(na::vector![0.0, 0.0])?
                .as_str(),
            Some(bounds),
            Some(bounds),
            true,
            false,
        );
        //log::warn!("\n### validating file `{:?}`###, contents:\n {}", file.path(), svg);
        let _rendernode = renderer.gen_rendernode(bounds, 1.0, svg.as_str())?;
    }

    Ok(())
}
