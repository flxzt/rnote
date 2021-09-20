use std::{cell::RefCell, rc::Rc};

use gtk4::gdk;
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{config, pens, utils};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Marker {
    width: f64,
    color: pens::Color,
    #[serde(skip, default = "Marker::default_marker_templates")]
    pub template: Rc<RefCell<Tera>>,
}

impl Default for Marker {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            color: pens::Color::from_gdk(Self::COLOR_DEFAULT),
            template: Self::default_marker_templates(),
        }
    }
}

impl Marker {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 5.0;

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

    pub fn color(&self) -> gdk::RGBA {
        self.color.to_gdk()
    }

    pub fn set_color(&mut self, color: gdk::RGBA) {
        self.color = pens::Color::from_gdk(color);
    }

    pub fn default_marker_templates() -> Rc<RefCell<Tera>> {
        let markerstroke_template_path = String::from(config::APP_IDPATH)
            + "templates/"
            + Self::template_name().as_str()
            + ".svg.templ";
        let mut templates = Tera::default();
        templates
            .add_raw_template(
                Self::template_name().as_str(),
                utils::load_string_from_resource(markerstroke_template_path.as_str())
                    .expect("failed to load string from resource")
                    .as_str(),
            )
            .expect("Failed to add default template for markerstroke to `templates`");
        Rc::new(RefCell::new(templates))
    }

    pub fn template_name() -> String {
        String::from("markerstroke")
    }
}
