use crate::strokes::InputData;

use gtk4::{gdk, graphene, gsk, Snapshot};

#[derive(Clone, Debug)]
pub struct Eraser {
    width: f64,
    pub current_input: InputData,
    shown: bool,
}

impl Default for Eraser {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            current_input: InputData::default(),
            shown: false,
        }
    }
}

impl Eraser {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 30.0;

    pub fn new(width: f64, current_input: InputData) -> Self {
        Self {
            width,
            current_input,
            shown: false,
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn draw(&self, scalefactor: f64, snapshot: &Snapshot) {
        if self.shown {
            let bounds = graphene::Rect::new(
                (scalefactor * (self.current_input.pos()[0])) as f32 - self.width as f32 / 2.0,
                (scalefactor * (self.current_input.pos()[1])) as f32 - self.width as f32 / 2.0,
                self.width as f32,
                self.width as f32,
            );
            let border_color = gdk::RGBA {
                red: 0.8,
                green: 0.1,
                blue: 0.0,
                alpha: 0.5,
            };
            let border_width = 2.0;

            snapshot.append_color(
                &gdk::RGBA {
                    red: 0.7,
                    green: 0.2,
                    blue: 0.1,
                    alpha: 0.5,
                },
                &bounds,
            );

            snapshot.append_border(
                &gsk::RoundedRect::new(
                    graphene::Rect::new(bounds.x(), bounds.y(), bounds.width(), bounds.height()),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                ),
                &[border_width, border_width, border_width, border_width],
                &[border_color, border_color, border_color, border_color],
            );
        }
    }
}
