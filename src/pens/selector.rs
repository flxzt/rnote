use std::{error::Error, ops::Deref};

use crate::{
    config,
    strokes::{compose, render, InputData},
    utils,
};

use gtk4::{gdk, gio, gsk, Snapshot};
use p2d::bounding_volume::BoundingVolume;

#[derive(Clone, Debug)]
pub struct Selector {
    pub path: Vec<InputData>,
    pub bounds: Option<p2d::bounding_volume::AABB>,
    pub caironode: gsk::CairoNode,
    shown: bool,
}

impl Default for Selector {
    fn default() -> Self {
        Self::new()
    }
}

impl Selector {
    pub const PATH_WIDTH: f64 = 2.0;
    pub const PATH_COLOR: gdk::RGBA = gdk::RGBA {
        red: 0.7,
        green: 0.7,
        blue: 0.7,
        alpha: 0.7,
    };

    pub fn new() -> Self {
        Self {
            path: vec![],
            bounds: None,
            caironode: utils::default_caironode(),
            shown: false,
        }
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn new_path(&mut self, inputdata: InputData) {
        self.clear_path();
        self.push_elem(inputdata);
    }

    pub fn push_elem(&mut self, inputdata: InputData) {
        self.path.push(inputdata);

        self.update_bounds_to_last_elem();
    }

    pub fn clear_path(&mut self) {
        self.bounds = None;
        self.path.clear();
    }

    pub fn update_caironode(&mut self, scalefactor: f64) {
        self.caironode = self
            .gen_caironode(scalefactor)
            .expect("failed to gen_caironode() in update_caironode() of selector");
    }

    pub fn gen_caironode(&self, scalefactor: f64) -> Result<gsk::CairoNode, Box<dyn Error>> {
        if let Some(bounds) = self.bounds {
            let svg = compose::wrap_svg(
                self.gen_svg_path(na::vector![0.0, 0.0]).as_str(),
                None,
                Some(bounds),
                true,
                false,
            );
            render::gen_caironode_for_svg(bounds, scalefactor, svg.as_str())
        } else {
            Ok(utils::default_caironode())
        }
    }

    fn update_bounds_to_last_elem(&mut self) {
        // Making sure bounds are always outside of coord + width
        if let Some(last) = self.path.last() {
            let pos_bounds = p2d::bounding_volume::AABB::new(
                na::Point2::from(last.pos() - na::vector![Self::PATH_WIDTH, Self::PATH_WIDTH]),
                na::Point2::from(last.pos() + na::vector![Self::PATH_WIDTH, Self::PATH_WIDTH]),
            );

            if let Some(ref mut bounds) = self.bounds {
                bounds.merge(&pos_bounds);
            } else {
                self.bounds = Some(pos_bounds);
            }
        }
    }

    pub fn gen_svg_path(&self, offset: na::Vector2<f64>) -> String {
        let mut cx = tera::Context::new();

        let padding = 2;
        let color = format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            (Self::PATH_COLOR.red * 255.0) as i32,
            (Self::PATH_COLOR.green * 255.0) as i32,
            (Self::PATH_COLOR.blue * 255.0) as i32,
            (Self::PATH_COLOR.alpha * 255.0) as i32,
        );

        let path = self
            .path
            .iter()
            .peekable()
            .enumerate()
            .map(|(i, element)| {
                if i == 0 {
                    format!(
                        "M {0} {1}",
                        element.pos()[0] + offset[0],
                        element.pos()[1] + offset[1]
                    )
                } else {
                    format!(
                        "L {} {}",
                        element.pos()[0] + offset[0],
                        element.pos()[1] + offset[1]
                    )
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        cx.insert("padding", &padding);
        cx.insert("color", &color);
        cx.insert("strokewidth", &Self::PATH_WIDTH);
        cx.insert("path", &path);
        cx.insert(
            "attributes",
            "fill=\"rgba(230, 230, 230, 0.15)\" stroke-dasharray=\"4 6\"",
        );

        let templ = String::from_utf8(
            gio::resources_lookup_data(
                (String::from(config::APP_IDPATH) + "templates/selectorstroke.svg.templ").as_str(),
                gio::ResourceLookupFlags::NONE,
            )
            .unwrap()
            .deref()
            .to_vec(),
        )
        .unwrap();
        let svg =
            tera::Tera::one_off(templ.as_str(), &cx, false).expect("create svg for selectorstorke");

        svg
    }

    pub fn draw(&self, snapshot: &Snapshot) {
        if self.shown {
            snapshot.append_node(&self.caironode);
        }
    }
}
