// Imports
use gtk4::{gdk, glib, graphene, prelude::*, subclass::prelude::*};
use once_cell::sync::Lazy;
use rnote_engine::engine::StrokeContent;
use std::cell::{Cell, RefCell};

mod imp {
    use p2d::bounding_volume::Aabb;
    use rnote_engine::utils::GrapheneRectHelpers;

    use super::*;

    #[derive(Debug, Default)]
    pub struct StrokeContentPaintable {
        pub(super) draw_background: Cell<bool>,
        pub(super) draw_pattern: Cell<bool>,
        pub(super) stroke_content: RefCell<Option<StrokeContent>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StrokeContentPaintable {
        const NAME: &'static str = "StrokeContentPaintable";
        type Type = super::StrokeContentPaintable;
        type Interfaces = (gdk::Paintable,);
    }

    impl ObjectImpl for StrokeContentPaintable {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::builder("draw-background")
                        .default_value(true)
                        .build(),
                    glib::ParamSpecBoolean::builder("draw-pattern")
                        .default_value(true)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "draw-background" => self.draw_background.get().to_value(),
                "draw-pattern" => self.draw_pattern.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "draw-background" => {
                    let draw_background = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.draw_background.replace(draw_background);
                    self.obj().invalidate_contents();
                }
                "draw-pattern" => {
                    let draw_pattern = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.draw_pattern.replace(draw_pattern);
                    self.obj().invalidate_contents();
                }
                _ => unimplemented!(),
            }
        }
    }

    impl PaintableImpl for StrokeContentPaintable {
        fn flags(&self) -> gdk::PaintableFlags {
            gdk::PaintableFlags::empty()
        }

        fn intrinsic_width(&self) -> i32 {
            self.stroke_content
                .borrow()
                .as_ref()
                .and_then(|c| c.size().map(|s| s[0].ceil() as i32))
                .unwrap_or(0)
        }

        fn intrinsic_height(&self) -> i32 {
            self.stroke_content
                .borrow()
                .as_ref()
                .and_then(|c| c.size().map(|s| s[1].ceil() as i32))
                .unwrap_or(0)
        }

        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            let Some(stroke_content) = &*self.stroke_content.borrow() else {
                return;
            };
            let Some(bounds) = stroke_content.bounds() else {
                return;
            };
            let intrinsic_size = na::vector![
                self.intrinsic_width() as f64,
                self.intrinsic_height() as f64
            ];
            if intrinsic_size[0] <= 0.0 || intrinsic_size[1] <= 0.0 {
                return;
            }
            let scale_factor = (width / intrinsic_size[0]).min(height / intrinsic_size[1]);
            let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(Aabb::new(
                na::point![0.0, 0.0],
                intrinsic_size.into(),
            )));

            cairo_cx.scale(scale_factor, scale_factor);
            cairo_cx.translate(-bounds.mins[0], -bounds.mins[1]);
            if let Err(e) = stroke_content.draw_to_cairo(
                &cairo_cx,
                self.draw_background.get(),
                self.draw_pattern.get(),
                scale_factor,
            ) {
                log::error!("drawing content of StrokeContentPaintable failed, Err: {e:?}");
            }
        }
    }
}

glib::wrapper! {
    pub struct StrokeContentPaintable(ObjectSubclass<imp::StrokeContentPaintable>) @implements gdk::Paintable;
}

impl Default for StrokeContentPaintable {
    fn default() -> Self {
        Self::new()
    }
}

impl StrokeContentPaintable {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn from_stroke_content(stroke_content: StrokeContent) -> Self {
        let p = Self::new();
        p.set_stroke_content(Some(stroke_content));
        p
    }

    #[allow(unused)]
    pub(crate) fn draw_background(&self) -> bool {
        self.property::<bool>("draw-background")
    }

    #[allow(unused)]
    pub(crate) fn set_draw_background(&self, draw_background: bool) {
        if self.imp().draw_background.get() != draw_background {
            self.set_property("draw-background", draw_background.to_value());
        }
    }

    #[allow(unused)]
    pub(crate) fn draw_pattern(&self) -> bool {
        self.property::<bool>("draw-pattern")
    }

    #[allow(unused)]
    pub(crate) fn set_draw_pattern(&self, draw_pattern: bool) {
        if self.imp().draw_pattern.get() != draw_pattern {
            self.set_property("draw-pattern", draw_pattern.to_value());
        }
    }

    pub(crate) fn set_stroke_content(&self, stroke_content: Option<StrokeContent>) {
        self.imp().stroke_content.replace(stroke_content);
        self.invalidate_size();
        self.invalidate_contents();
    }
}
