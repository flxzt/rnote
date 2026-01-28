// Imports
use adw::prelude::AnimationExt;
use gtk4::{
    Align, Button, CssProvider, ToggleButton, Widget, gdk, glib, glib::clone, graphene, prelude::*,
    subclass::prelude::*,
};
use once_cell::sync::Lazy;
use rnote_compose::{Color, color};
use rnote_engine::ext::GdkRGBAExt;
use std::cell::{Cell, OnceCell};

mod imp {
    const BOTTOM_BAR_PROPORTION: f32 = 0.15;
    const BRIGHTNESS_HOVER: f32 = 0.93;
    const BRIGHTNESS_ACTIVE: f32 = 0.86;
    const REPEAT_RATIO: f32 = 1.8;
    const OFFSET_RATIO: f32 = 0.0;
    const ANIMATION_TIME_MS: u32 = 150;
    // to keep synchronized with style.css.colorpad.border-radius
    const BORDER_RADIUS: f32 = 4.0;

    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnColorPad {
        pub(crate) color: Cell<gdk::RGBA>,
        pub(crate) previous_color: Cell<gdk::RGBA>,
        pub(super) animation_toggle_active: OnceCell<adw::TimedAnimation>,
        pub(super) display_progress: Cell<f64>,
        pub(super) animation_color_change: OnceCell<adw::TimedAnimation>,
        pub(super) color_change_progress: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnColorPad {
        const NAME: &'static str = "RnColorPad";
        type Type = super::RnColorPad;
        type ParentType = ToggleButton;
    }

    impl Default for RnColorPad {
        fn default() -> Self {
            Self {
                color: Cell::new(gdk::RGBA::from_compose_color(
                    super::RnColorPad::COLOR_DEFAULT,
                )),
                previous_color: Cell::new(gdk::RGBA::from_compose_color(
                    super::RnColorPad::COLOR_DEFAULT,
                )),
                animation_toggle_active: OnceCell::new(),
                display_progress: Cell::new(0.0),
                animation_color_change: OnceCell::new(),
                color_change_progress: Cell::new(0.0),
            }
        }
    }

    impl ObjectImpl for RnColorPad {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.set_hexpand(false);
            obj.set_vexpand(false);
            obj.set_halign(Align::Fill);
            obj.set_valign(Align::Center);
            obj.set_width_request(34);
            obj.set_height_request(34);
            obj.set_css_classes(&["colorpad"]);

            self.update_appearance(super::RnColorPad::COLOR_DEFAULT);

            let animation_target = adw::CallbackAnimationTarget::new(clone!(
                #[weak]
                obj,
                move |value| {
                    let imp = obj.imp();
                    imp.display_progress.set(value);
                    obj.queue_draw();
                }
            ));
            let anim = adw::TimedAnimation::builder()
                .widget(&*obj)
                .duration(ANIMATION_TIME_MS)
                .target(&animation_target)
                .build();
            anim.set_easing(adw::Easing::EaseInOutSine);
            let _ = self.animation_toggle_active.set(anim);

            obj.connect_toggled(clone!(
                #[weak(rename_to=colorsetter)]
                self,
                move |button| {
                    use adw::prelude::AnimationExt;
                    if let Some(animation) = colorsetter.animation_toggle_active.get() {
                        if button.is_active() {
                            animation.set_value_from(0.0);
                            animation.set_value_to(1.0);
                        } else {
                            animation.set_value_from(1.0);
                            animation.set_value_to(0.0);
                        }
                        animation.play();
                    }
                }
            ));

            let animation_color_target = adw::CallbackAnimationTarget::new(clone!(
                #[weak]
                obj,
                move |value| {
                    let imp = obj.imp();
                    imp.color_change_progress.set(value);
                    obj.queue_draw();
                }
            ));
            let anim_color_change = adw::TimedAnimation::builder()
                .widget(&*obj)
                .duration(ANIMATION_TIME_MS)
                .target(&animation_color_target)
                .build();
            anim_color_change.set_easing(adw::Easing::EaseInOutSine);
            anim_color_change.set_value_from(0.0);
            anim_color_change.set_value_to(1.0);
            let _ = self.animation_color_change.set(anim_color_change);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecBoxed::builder::<gdk::RGBA>("color").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "color" => {
                    let color = value
                        .get::<gdk::RGBA>()
                        .expect("value not of type `gdk::RGBA`");
                    self.previous_color.set(self.color.get());
                    self.color.set(color);

                    // trigger the color change animation
                    if let Some(animation_color) = self.animation_color_change.get() {
                        animation_color.play();
                    }
                    self.update_appearance(color.into_compose_color());
                    self.obj().queue_draw();
                }
                _ => panic!("invalid property name"),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "color" => self.color.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for RnColorPad {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            snapshot.save();

            let width = self.obj().width();
            let height = self.obj().height();

            let bounds = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
            let bounds_clipped = gtk4::gsk::RoundedRect::new(
                bounds,
                graphene::Size::new(BORDER_RADIUS, BORDER_RADIUS),
                graphene::Size::new(BORDER_RADIUS, BORDER_RADIUS),
                graphene::Size::new(BORDER_RADIUS, BORDER_RADIUS),
                graphene::Size::new(BORDER_RADIUS, BORDER_RADIUS),
            );
            snapshot.push_rounded_clip(&bounds_clipped);

            let previous_color = self.previous_color.get();
            let next_color = self.color.get();
            let progress = self.color_change_progress.get() as f32;

            let colorpad_color = self.color.get();

            let mut color = gdk::RGBA::new(
                progress * next_color.red() + (1.0 - progress) * previous_color.red(),
                progress * next_color.green() + (1.0 - progress) * previous_color.green(),
                progress * next_color.blue() + (1.0 - progress) * previous_color.blue(),
                progress * next_color.alpha() + (1.0 - progress) * previous_color.alpha(),
            );

            if colorpad_color.alpha() != 0.0 {
                if self
                    .obj()
                    .state_flags()
                    .contains(gtk4::StateFlags::PRELIGHT)
                {
                    // colorsetter:hover
                    color.set_red(color.red() * BRIGHTNESS_HOVER);
                    color.set_green(color.green() * BRIGHTNESS_HOVER);
                    color.set_blue(color.blue() * BRIGHTNESS_HOVER);
                }
                if self.obj().state_flags().contains(gtk4::StateFlags::ACTIVE)
                    && !self.obj().is_active()
                {
                    // colorsetter:active
                    color.set_red(color.red() * BRIGHTNESS_ACTIVE);
                    color.set_green(color.green() * BRIGHTNESS_ACTIVE);
                    color.set_blue(color.blue() * BRIGHTNESS_ACTIVE);
                }
            }
            // background image (checkboard pattern)
            if colorpad_color.alpha() != 1.0 {
                let checkboard_bounds = graphene::Rect::new(
                    OFFSET_RATIO * (width as f32),
                    OFFSET_RATIO * (height as f32),
                    width as f32 / (2.0 * REPEAT_RATIO),
                    height as f32 / (2.0 * REPEAT_RATIO),
                );
                let checkboard_repeat = graphene::Rect::new(
                    OFFSET_RATIO * (width as f32),
                    OFFSET_RATIO * (height as f32),
                    width as f32 / (REPEAT_RATIO),
                    height as f32 / (REPEAT_RATIO),
                );

                snapshot.push_repeat(&bounds, Some(&checkboard_repeat));
                snapshot.append_color(&gdk::RGBA::BLACK.with_alpha(0.75), &checkboard_bounds);
                snapshot.append_color(
                    &gdk::RGBA::BLACK.with_alpha(0.75),
                    &checkboard_bounds.offset_r(
                        width as f32 / (2.0 * REPEAT_RATIO),
                        height as f32 / (2.0 * REPEAT_RATIO),
                    ),
                );

                snapshot.pop();
            }

            snapshot.append_color(&color, &bounds);

            // bottom bar
            let current_foreground_color = if colorpad_color.alpha() == 0.0 {
                // accessing colors through the style context is deprecated,
                // but this needs new color API to fetch theme colors.
                // TODO: where is this set ? any way around this ?
                #[allow(deprecated)]
                self.obj()
                    .style_context()
                    .lookup_color("window_fg_color")
                    .unwrap_or(gdk::RGBA::BLACK)
            } else if colorpad_color.into_compose_color().luma() < color::FG_LUMINANCE_THRESHOLD {
                gdk::RGBA::WHITE
            } else {
                gdk::RGBA::BLACK
            };
            let foreground_color = if progress > 0.0 || progress < 1.0 {
                let previous_foreground_color = if previous_color.alpha() == 0.0 {
                    // accessing colors through the style context is deprecated,
                    // but this needs new color API to fetch theme colors.
                    // TODO: where is this set ? any way around this ?
                    #[allow(deprecated)]
                    self.obj()
                        .style_context()
                        .lookup_color("window_fg_color")
                        .unwrap_or(gdk::RGBA::BLACK)
                } else if colorpad_color.into_compose_color().luma() < color::FG_LUMINANCE_THRESHOLD
                {
                    gdk::RGBA::WHITE
                } else {
                    gdk::RGBA::BLACK
                };
                gdk::RGBA::new(
                    progress * current_foreground_color.red()
                        + (1.0 - progress) * previous_foreground_color.red(),
                    progress * current_foreground_color.green()
                        + (1.0 - progress) * previous_foreground_color.green(),
                    progress * current_foreground_color.blue()
                        + (1.0 - progress) * previous_foreground_color.blue(),
                    progress * current_foreground_color.alpha()
                        + (1.0 - progress) * previous_foreground_color.alpha(),
                )
            } else {
                current_foreground_color
            };

            let bounds_active = graphene::Rect::new(
                0.0,
                (1.0 - BOTTOM_BAR_PROPORTION * (self.display_progress.get() as f32))
                    * (height as f32),
                width as f32,
                BOTTOM_BAR_PROPORTION * (self.display_progress.get() as f32) * (height as f32),
            );
            snapshot.append_color(&foreground_color, &bounds_active);
            snapshot.pop();

            snapshot.restore();

            // 2. Default button rendering (icon/text) on top
            self.parent_snapshot(snapshot);

            self.obj().queue_draw();
        }
    }
    impl ButtonImpl for RnColorPad {}
    impl ToggleButtonImpl for RnColorPad {}

    impl RnColorPad {
        fn update_appearance(&self, color: Color) {
            // we still rely on the CSS to switch the icon light/dark mode
            let css = CssProvider::new();

            let colorpad_fg_color = if color.a == 0.0 {
                String::from("@window_fg_color")
            } else if color.luma() < color::FG_LUMINANCE_THRESHOLD {
                String::from("@light_1")
            } else {
                String::from("@dark_5")
            };

            let custom_css = format!("@define-color colorpad_fg_color {colorpad_fg_color};",);
            css.load_from_string(&custom_css);

            // adding custom css is deprecated.
            // TODO: We should refactor to drawing through snapshot().
            // Doing this will also get rid of the css checkerboard glitches that appear on some devices and scaling levels.
            #[allow(deprecated)]
            self.obj()
                .style_context()
                .add_provider(&css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            self.obj().queue_draw();
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnColorPad(ObjectSubclass<imp::RnColorPad>)
        @extends ToggleButton, Button, Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnColorPad {
    fn default() -> Self {
        Self::new()
    }
}

impl RnColorPad {
    pub(crate) const COLOR_DEFAULT: Color = Color::BLACK;

    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn color(&self) -> gdk::RGBA {
        self.property::<gdk::RGBA>("color")
    }

    #[allow(unused)]
    pub(crate) fn set_color(&self, color: gdk::RGBA) {
        self.set_property("color", color.to_value());
    }
}
