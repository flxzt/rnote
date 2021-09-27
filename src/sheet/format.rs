use gtk4::{gdk, graphene, gsk, Snapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Format {
    pub width: i32,
    pub height: i32,
    pub dpi: i32,
}

impl Default for Format {
    fn default() -> Self {
        Self::A4_150DPI_PORTRAIT
    }
}

impl Format {
    pub const WIDTH_MIN: i32 = 1;
    pub const WIDTH_MAX: i32 = 5000;
    pub const HEIGHT_MIN: i32 = 1;
    pub const HEIGHT_MAX: i32 = 5000;
    pub const DPI_MIN: i32 = 1;
    pub const DPI_MAX: i32 = 2000;
    pub const FORMAT_COLOR: gdk::RGBA = gdk::RGBA {
        red: 0.6,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
    // Preconfigured Formats
    pub const A4_150DPI_PORTRAIT: Format = Format {
        width: 1240,
        height: 1754,
        dpi: 150,
    };
    pub const A4_150DPI_LANDSCAPE: Format = Format {
        width: 1754,
        height: 1240,
        dpi: 150,
    };
    pub const A4_300DPI_PORTRAIT: Format = Format {
        width: 2480,
        height: 3508,
        dpi: 300,
    };
    pub const A4_300DPI_LANDSCAPE: Format = Format {
        width: 3508,
        height: 2480,
        dpi: 300,
    };
    pub const A3_150DPI_PORTRAIT: Format = Format {
        width: 1754,
        height: 2480,
        dpi: 150,
    };
    pub const A3_150DPI_LANDSCAPE: Format = Format {
        width: 2480,
        height: 1754,
        dpi: 150,
    };
    pub const A3_300DPI_PORTRAIT: Format = Format {
        width: 3508,
        height: 4961,
        dpi: 300,
    };
    pub const A3_300DPI_LANDSCAPE: Format = Format {
        width: 4961,
        height: 3508,
        dpi: 300,
    };

    pub fn try_parse_width(text: &str) -> Option<i32> {
        let width_range = Format::WIDTH_MIN..=Format::WIDTH_MAX;
        if let Ok(parsed_no) = text.parse::<i32>() {
            if width_range.contains(&parsed_no) {
                Some(parsed_no)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn try_parse_height(text: &str) -> Option<i32> {
        let width_range = Format::HEIGHT_MIN..=Format::HEIGHT_MAX;
        if let Ok(parsed_no) = text.parse::<i32>() {
            if width_range.contains(&parsed_no) {
                Some(parsed_no)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn try_parse_dpi(text: &str) -> Option<i32> {
        let width_range = Format::DPI_MIN..=Format::DPI_MAX;
        if let Ok(parsed_no) = text.parse::<i32>() {
            if width_range.contains(&parsed_no) {
                Some(parsed_no)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn draw(&self, total_height: i32, snapshot: &Snapshot, scalefactor: f64) {
        for i in 0..=total_height / self.height {
            let border_radius = graphene::Size::new(0.0, 0.0);
            let border_width = 2.0;
            let border_bounds = graphene::Rect::new(
                0.0,
                (i * self.height) as f32 - border_width / 2.0,
                self.width as f32,
                ((i + 1) * self.height) as f32 + border_width,
            );

            let rounded_rect = gsk::RoundedRect::new(
                border_bounds
                    .clone()
                    .scale(scalefactor as f32, scalefactor as f32),
                border_radius.clone(),
                border_radius.clone(),
                border_radius.clone(),
                border_radius,
            );
            snapshot.append_border(
                &rounded_rect,
                &[border_width, border_width, border_width, border_width],
                &[
                    Self::FORMAT_COLOR,
                    Self::FORMAT_COLOR,
                    Self::FORMAT_COLOR,
                    Self::FORMAT_COLOR,
                ],
            );
        }
    }
}
