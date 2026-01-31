// Imports
use gtk4::{
    Align, Button, PositionType, ToggleButton, Widget, gdk, glib, glib::clone, graphene,
    prelude::*, subclass::prelude::*,
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
    const BORDER_SIZE: f32 = 0.5;

    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnColorSetter {
        pub(crate) color: Cell<gdk::RGBA>,
        pub(crate) position: Cell<PositionType>,
        pub(super) animation: OnceCell<adw::TimedAnimation>,
        pub(super) display_progress: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnColorSetter {
        const NAME: &'static str = "RnColorSetter";
        type Type = super::RnColorSetter;
        type ParentType = ToggleButton;
    }

    impl Default for RnColorSetter {
        fn default() -> Self {
            Self {
                color: Cell::new(gdk::RGBA::from_compose_color(
                    super::RnColorSetter::COLOR_DEFAULT,
                )),
                position: Cell::new(PositionType::Right),
                animation: OnceCell::new(),
                display_progress: Cell::new(0.0),
            }
        }
    }

    impl ObjectImpl for RnColorSetter {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.set_hexpand(false);
            obj.set_vexpand(false);
            obj.set_halign(Align::Fill);
            obj.set_valign(Align::Fill);
            obj.set_width_request(34);
            obj.set_height_request(34);
            obj.set_css_classes(&["colorsetter"]);

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
            let _ = self.animation.set(anim);

            obj.connect_toggled(clone!(
                #[weak(rename_to=colorsetter)]
                self,
                move |button| {
                    use adw::prelude::AnimationExt;
                    if let Some(animation) = colorsetter.animation.get() {
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

            obj.queue_draw();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::builder::<gdk::RGBA>("color").build(),
                    glib::ParamSpecEnum::builder_with_default::<PositionType>(
                        "position",
                        PositionType::Right,
                    )
                    .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "color" => {
                    let color = value
                        .get::<gdk::RGBA>()
                        .expect("value not of type `gdk::RGBA`");
                    self.color.set(color);

                    self.obj().queue_draw();
                }
                "position" => {
                    let position = value
                        .get::<PositionType>()
                        .expect("value not of type `PositionType`");

                    self.position.replace(position);
                }
                _ => panic!("invalid property name"),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "color" => self.color.get().to_value(),
                "position" => self.position.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for RnColorSetter {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let width = self.obj().width();
            let height = self.obj().height();

            let bounds = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);

            let color_stroke = self.color.get();
            let mut color = self.color.get().clone();

            if color_stroke.alpha() != 0.0 {
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
            if color_stroke.alpha() != 1.0 {
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
            let foreground_color = if color_stroke.alpha() == 0.0 {
                // accessing colors through the style context is deprecated,
                // but this needs new color API to fetch theme colors.
                // TODO: where is this set ? any way around this ?
                #[allow(deprecated)]
                self.obj()
                    .style_context()
                    .lookup_color("window_fg_color")
                    .unwrap_or(gdk::RGBA::BLACK)
            } else if color_stroke.into_compose_color().luma() < color::FG_LUMINANCE_THRESHOLD {
                gdk::RGBA::WHITE
            } else {
                gdk::RGBA::BLACK
            };

            let bounds_active = graphene::Rect::new(
                0.0,
                (1.0 - BOTTOM_BAR_PROPORTION * (self.display_progress.get() as f32))
                    * (height as f32),
                width as f32,
                BOTTOM_BAR_PROPORTION * (self.display_progress.get() as f32) * (height as f32),
            );
            snapshot.append_color(&foreground_color, &bounds_active);

            // gray borders for visibility
            let border_bounds = gtk4::gsk::RoundedRect::new(
                bounds,
                graphene::Size::zero(),
                graphene::Size::zero(),
                graphene::Size::zero(),
                graphene::Size::zero(),
            );
            snapshot.append_border(
                &border_bounds,
                &[BORDER_SIZE, BORDER_SIZE, BORDER_SIZE, BORDER_SIZE],
                &[
                    gdk::RGBA::BLACK,
                    gdk::RGBA::BLACK,
                    gdk::RGBA::BLACK,
                    gdk::RGBA::BLACK,
                ],
            );

            self.obj().queue_draw();
        }
    }

    impl ButtonImpl for RnColorSetter {}

    impl ToggleButtonImpl for RnColorSetter {}

    impl RnColorSetter {}
}

glib::wrapper! {
    pub(crate) struct RnColorSetter(ObjectSubclass<imp::RnColorSetter>)
        @extends ToggleButton, Button, Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Actionable;
}

impl Default for RnColorSetter {
    fn default() -> Self {
        Self::new()
    }
}

impl RnColorSetter {
    pub(crate) const COLOR_DEFAULT: Color = Color::BLACK;

    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn position(&self) -> PositionType {
        self.property::<PositionType>("position")
    }

    #[allow(unused)]
    pub(crate) fn set_position(&self, position: PositionType) {
        self.set_property("position", position.to_value());
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
