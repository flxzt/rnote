// Imports
use gtk4::graphene;
use gtk4::{gdk, glib, glib::clone, gsk, prelude::*, subclass::prelude::*};
use once_cell::sync::Lazy;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_engine::engine::StrokeContent;
use rnote_engine::ext::GdkRGBAExt;
use rnote_engine::render::Image;
use rnote_engine::tasks::{OneOffTaskError, OneOffTaskHandle};
use std::cell::{Cell, OnceCell, RefCell};
use std::time::Duration;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct StrokeContentPaintable {
        pub(super) paint_max_width: Cell<f64>,
        pub(super) paint_max_height: Cell<f64>,
        pub(super) paint_cache: RefCell<Option<Image>>,
        pub(super) paint_cache_texture: RefCell<Option<gdk::MemoryTexture>>,
        pub(super) draw_background: Cell<bool>,
        pub(super) draw_pattern: Cell<bool>,
        pub(super) optimize_printing: Cell<bool>,
        pub(super) margin: Cell<f64>,

        pub(super) stroke_content: RefCell<StrokeContent>,
        // The handle executing the paint task when regenerating the paint cache after a timeout
        pub(super) paint_task_handle: RefCell<Option<OneOffTaskHandle>>,
        // The handler that is spawn on the glib main context and integrates the received paint cache image
        pub(super) paint_task_handler: RefCell<Option<glib::SourceId>>,
        pub(super) paint_task_tx: OnceCell<glib::Sender<anyhow::Result<Image>>>,
        pub(super) paint_tasks_in_progress: Cell<usize>,
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
                    glib::ParamSpecDouble::builder("paint-max-width")
                        .default_value(1000.)
                        .build(),
                    glib::ParamSpecDouble::builder("paint-max-height")
                        .default_value(1000.)
                        .build(),
                    glib::ParamSpecBoolean::builder("draw-background")
                        .default_value(true)
                        .build(),
                    glib::ParamSpecBoolean::builder("draw-pattern")
                        .default_value(true)
                        .build(),
                    glib::ParamSpecBoolean::builder("optimize-printing")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecDouble::builder("margin")
                        .default_value(0.0)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "paint-max-width" => self.paint_max_width.get().to_value(),
                "paint-max-height" => self.paint_max_height.get().to_value(),
                "draw-background" => self.draw_background.get().to_value(),
                "draw-pattern" => self.draw_pattern.get().to_value(),
                "optimize-printing" => self.optimize_printing.get().to_value(),
                "margin" => self.margin.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "paint-max-width" => {
                    let paint_max_width = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`");
                    self.paint_max_width.replace(paint_max_width.max(0.0));
                    self.obj().repaint_cache_async();
                }
                "paint-max-height" => {
                    let paint_max_height = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`");
                    self.paint_max_height.replace(paint_max_height.max(0.0));
                    self.obj().repaint_cache_async();
                }
                "draw-background" => {
                    let draw_background = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.draw_background.replace(draw_background);
                    self.obj().repaint_cache_async();
                }
                "draw-pattern" => {
                    let draw_pattern = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.draw_pattern.replace(draw_pattern);
                    self.obj().repaint_cache_async();
                }
                "optimize-printing" => {
                    let optimize_printing = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.optimize_printing.replace(optimize_printing);
                    self.obj().repaint_cache_async();
                }
                "margin" => {
                    let margin = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`");
                    self.margin.replace(margin.max(0.0));
                    self.obj().repaint_cache_async();
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            // For some reason these default values are not set, but are instead 0.
            //
            // TODO: fix it
            obj.set_paint_max_width(1000.);
            obj.set_paint_max_height(1000.);
            let (tx, rx) = glib::MainContext::channel::<anyhow::Result<Image>>(
                glib::source::Priority::DEFAULT,
            );
            self.paint_task_tx.set(tx).unwrap();

            let handler = rx.attach(Some(&glib::MainContext::default()), clone!(@weak obj as paintable => @default-return glib::ControlFlow::Break, move |res| {
                paintable.imp().paint_task_handle.take();
                match res {
                    Ok(image) => {
                        paintable.imp().replace_paint_cache(image);
                        if paintable.imp().paint_tasks_in_progress.get() <= 1 {
                            paintable.imp().emit_repaint_in_progress(false);
                        }
                        paintable.imp().paint_tasks_in_progress.set(paintable.imp().paint_tasks_in_progress.get().saturating_sub(1));
                    }
                    Err(e) => {
                        tracing::error!("StrokeContentPaintable repainting cache image in task failed, Err: {e:?}");
                    }
                }
                glib::ControlFlow::Continue
            }));
            self.paint_task_handler.replace(Some(handler));
        }

        fn dispose(&self) {
            self.paint_task_handle.take();
            if let Some(s) = self.paint_task_handler.take() {
                s.remove();
            }
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![glib::subclass::Signal::builder("repaint-in-progress")
                    .param_types([bool::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl PaintableImpl for StrokeContentPaintable {
        fn flags(&self) -> gdk::PaintableFlags {
            gdk::PaintableFlags::empty()
        }

        fn intrinsic_width(&self) -> i32 {
            self.stroke_content
                .borrow()
                .size()
                .map(|s| (s[0] + 2. * self.margin.get()).ceil() as i32)
                .unwrap_or(0)
        }

        fn intrinsic_height(&self) -> i32 {
            self.stroke_content
                .borrow()
                .size()
                .map(|s| (s[1] + 2. * self.margin.get()).ceil() as i32)
                .unwrap_or(0)
        }

        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            if let Some(texture) = &*self.paint_cache_texture.borrow() {
                snapshot.append_scaled_texture(
                    texture,
                    gsk::ScalingFilter::Linear,
                    &graphene::Rect::new(0., 0., width as f32, height as f32),
                );
                // Draw a border
                snapshot.append_border(
                    &gsk::RoundedRect::from_rect(
                        graphene::Rect::new(0.0, 0.0, width as f32, height as f32),
                        0.,
                    ),
                    &[1.5; 4],
                    &[gdk::RGBA::from_piet_color(Self::CONTENT_BORDER_COLOR); 4],
                );
            }
        }
    }

    impl StrokeContentPaintable {
        #[allow(unused)]
        pub(super) fn emit_repaint_in_progress(&self, in_progress: bool) {
            self.obj()
                .emit_by_name::<()>("repaint-in-progress", &[&in_progress]);
        }

        pub(super) fn replace_paint_cache(&self, image: Image) {
            match image.to_memtexture() {
                Ok(texture) => {
                    self.paint_cache.replace(Some(image));
                    self.paint_cache_texture.replace(Some(texture));
                    self.obj().invalidate_contents();
                    self.obj().invalidate_size();
                }
                Err(e) => {
                    tracing::error!("StrokeContentPaintable creating memory texture from new cache image failed, Err: {e:?}");
                }
            }
        }
    }

    pub(super) fn paint_content(
        stroke_content: &StrokeContent,
        width: f64,
        height: f64,
        draw_background: bool,
        draw_pattern: bool,
        optimize_printing: bool,
        margin: f64,
    ) -> anyhow::Result<Image> {
        let Some(bounds) = stroke_content.bounds().map(|b| b.loosened(margin)) else {
            return Ok(Image::default());
        };
        if width <= 0. || height <= 0. {
            return Ok(Image::default());
        }
        let (scale_x, scale_y) = (width / bounds.extents()[0], height / bounds.extents()[1]);
        let image_scale = scale_x.max(scale_y);
        let surface_width = width.ceil() as i32;
        let surface_height = height.ceil() as i32;
        let target_surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, surface_width, surface_height)?;
        {
            let cairo_cx = cairo::Context::new(&target_surface)?;

            cairo_cx.scale(scale_x, scale_y);
            cairo_cx.translate(-bounds.mins[0], -bounds.mins[1]);

            // Draw the content
            stroke_content.draw_to_cairo(
                &cairo_cx,
                draw_background,
                draw_pattern,
                optimize_printing,
                margin,
                image_scale,
            )?;
        }

        Image::try_from_cairo_surface(
            target_surface,
            Aabb::new(na::point![0., 0.], na::point![width, height]),
        )
    }

    impl StrokeContentPaintable {
        const CONTENT_BORDER_COLOR: piet::Color = rnote_compose::color::GNOME_BRIGHTS[4];
    }
}

glib::wrapper! {
    pub(crate) struct StrokeContentPaintable(ObjectSubclass<imp::StrokeContentPaintable>)
        @implements gdk::Paintable;
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
        p.set_stroke_content(stroke_content);
        p
    }

    #[allow(unused)]
    pub(crate) fn paint_max_width(&self) -> f64 {
        self.property::<f64>("paint-max-width")
    }

    #[allow(unused)]
    pub(crate) fn set_paint_max_width(&self, paint_max_width: f64) {
        if self.imp().paint_max_width.get() != paint_max_width {
            self.set_property("paint-max-width", paint_max_width.to_value());
        }
    }

    #[allow(unused)]
    pub(crate) fn paint_max_height(&self) -> f64 {
        self.property::<f64>("paint-max-height")
    }

    #[allow(unused)]
    pub(crate) fn set_paint_max_height(&self, paint_max_height: f64) {
        if self.imp().paint_max_height.get() != paint_max_height {
            self.set_property("paint-max-height", paint_max_height.to_value());
        }
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

    #[allow(unused)]
    pub(crate) fn optimize_printing(&self) -> bool {
        self.property::<bool>("optimize-printing")
    }

    #[allow(unused)]
    pub(crate) fn set_optimize_printing(&self, optimize_printing: bool) {
        if self.imp().optimize_printing.get() != optimize_printing {
            self.set_property("optimize-printing", optimize_printing.to_value());
        }
    }

    #[allow(unused)]
    pub(crate) fn margin(&self) -> f64 {
        self.property::<f64>("margin")
    }

    #[allow(unused)]
    pub(crate) fn set_margin(&self, margin: f64) {
        if self.imp().margin.get() != margin {
            self.set_property("margin", margin.to_value());
        }
    }

    pub(crate) fn set_stroke_content(&self, stroke_content: StrokeContent) {
        self.imp().stroke_content.replace(stroke_content);
        self.repaint_cache_async();
        self.invalidate_size();
        self.invalidate_contents();
    }

    /// Regenerates the paint cache.
    #[allow(unused)]
    pub(crate) fn repaint_cache(&self) {
        let (width, height) = (
            (self.intrinsic_width() as f64).min(self.imp().paint_max_width.get()),
            (self.intrinsic_height() as f64).min(self.imp().paint_max_height.get()),
        );
        if width <= 0. && height <= 0. {
            return;
        }

        // emit `repaint-in-progress` signal even for the synchronous repaint for consistency.
        self.imp().emit_repaint_in_progress(true);

        match imp::paint_content(
            &self.imp().stroke_content.borrow(),
            width,
            height,
            self.imp().draw_background.get(),
            self.imp().draw_pattern.get(),
            self.imp().optimize_printing.get(),
            self.imp().margin.get(),
        ) {
            Ok(image) => match image.to_memtexture() {
                Ok(texture) => {
                    self.imp().paint_cache.replace(Some(image));
                    self.imp().paint_cache_texture.replace(Some(texture));
                    self.invalidate_contents();
                    self.invalidate_size();
                }
                Err(e) => {
                    tracing::error!("StrokeContentPaintable creating memory texture from repainted cache image failed, Err: {e:?}");
                }
            },
            Err(e) => {
                tracing::error!("Repainting StrokeContentPaintable cache image failed, Err: {e:?}");
            }
        }

        self.imp().emit_repaint_in_progress(false);
    }

    /// Regenerates the paint cache asynchronously.
    #[allow(unused)]
    pub(crate) fn repaint_cache_async(&self) {
        let (width, height) = (
            (self.intrinsic_width() as f64).min(self.imp().paint_max_width.get()),
            (self.intrinsic_height() as f64).min(self.imp().paint_max_height.get()),
        );
        if width <= 0. && height <= 0. {
            return;
        }
        let stroke_content = self.imp().stroke_content.borrow().clone();
        let draw_background = self.imp().draw_background.get();
        let draw_pattern = self.imp().draw_pattern.get();
        let optimize_printing = self.imp().optimize_printing.get();
        let margin = self.imp().margin.get();
        let tx = self.imp().paint_task_tx.get().unwrap().clone();

        self.imp().emit_repaint_in_progress(true);
        self.imp()
            .paint_tasks_in_progress
            .set(self.imp().paint_tasks_in_progress.get() + 1);

        rayon::spawn(move || {
            if let Err(e) = tx.send(imp::paint_content(
                &stroke_content,
                width,
                height,
                draw_background,
                draw_pattern,
                optimize_printing,
                margin,
            )) {
                tracing::error!("StrokeContentPaintable failed to send painted cache image through channel, Err: {e:?}");
            };
        });
    }

    /// Regenerates the paint cache after a timeout.
    ///
    /// Subsequent calls to this function reset the timeout.
    #[allow(unused)]
    pub(crate) fn repaint_cache_w_timeout(&self) {
        const TIMEOUT: Duration = Duration::from_millis(500);
        let (width, height) = (
            (self.intrinsic_width() as f64).min(self.imp().paint_max_width.get()),
            (self.intrinsic_height() as f64).min(self.imp().paint_max_height.get()),
        );
        if width <= 0. && height <= 0. {
            return;
        }
        let stroke_content = self.imp().stroke_content.borrow().clone();
        let draw_background = self.imp().draw_background.get();
        let draw_pattern = self.imp().draw_pattern.get();
        let optimize_printing = self.imp().optimize_printing.get();
        let margin = self.imp().margin.get();
        let mut reinstall_task = false;
        let tx = self.imp().paint_task_tx.get().unwrap().clone();

        let paint_task = move || {
            if let Err(e) = tx.send(imp::paint_content(
                &stroke_content,
                width,
                height,
                draw_background,
                draw_pattern,
                optimize_printing,
                margin,
            )) {
                tracing::error!("StrokeContentPaintable failed to send painted cache image through channel, Err: {e:?}");
            };
        };

        if let Some(handle) = self.imp().paint_task_handle.borrow_mut().as_mut() {
            match handle.replace_task(paint_task.clone()) {
                Ok(()) => {}
                Err(OneOffTaskError::TimeoutReached) => {
                    reinstall_task = true;
                }
                Err(e) => {
                    tracing::error!("Could not replace task for one off paint task, Err: {e:?}");
                    reinstall_task = true;
                }
            }
        } else {
            reinstall_task = true;
        }

        if reinstall_task {
            *self.imp().paint_task_handle.borrow_mut() =
                Some(OneOffTaskHandle::new(paint_task, TIMEOUT));
            self.imp().emit_repaint_in_progress(true);
            self.imp()
                .paint_tasks_in_progress
                .set(self.imp().paint_tasks_in_progress.get() + 1);
        }
    }
}
