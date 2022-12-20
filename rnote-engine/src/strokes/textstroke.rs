use std::ops::Range;

use gtk4::pango;
use kurbo::Shape;
use once_cell::sync::Lazy;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::{RenderContext, TextLayout, TextLayoutBuilder};
use rnote_compose::helpers::{AabbHelpers, Affine2Helpers, Vector2Helpers};
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::{color, Color, Transform};
use serde::{Deserialize, Serialize};

use crate::{render, Camera, DrawBehaviour};

use super::strokebehaviour::GeneratedStrokeImages;
use super::StrokeBehaviour;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "font_style")]
pub enum FontStyle {
    #[serde(rename = "regular")]
    Regular,
    #[serde(rename = "italic")]
    Italic,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Regular
    }
}

impl From<piet::FontStyle> for FontStyle {
    fn from(piet_font_style: piet::FontStyle) -> Self {
        match piet_font_style {
            piet::FontStyle::Regular => Self::Regular,
            piet::FontStyle::Italic => Self::Italic,
        }
    }
}

impl From<FontStyle> for piet::FontStyle {
    fn from(font_style: FontStyle) -> Self {
        match font_style {
            FontStyle::Regular => piet::FontStyle::Regular,
            FontStyle::Italic => piet::FontStyle::Italic,
        }
    }
}

impl From<pango::Style> for FontStyle {
    fn from(pango_style: pango::Style) -> Self {
        match pango_style {
            pango::Style::Normal => Self::Regular,
            pango::Style::Oblique => Self::Italic,
            pango::Style::Italic => Self::Italic,
            _ => Self::Regular,
        }
    }
}

impl From<FontStyle> for pango::Style {
    fn from(font_style: FontStyle) -> Self {
        match font_style {
            FontStyle::Regular => pango::Style::Normal,
            FontStyle::Italic => pango::Style::Italic,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "text_style")]
pub enum TextAlignment {
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "end")]
    End,
    #[serde(rename = "fill")]
    Fill,
}

impl From<piet::TextAlignment> for TextAlignment {
    fn from(value: piet::TextAlignment) -> Self {
        match value {
            piet::TextAlignment::Start => Self::Start,
            piet::TextAlignment::End => Self::End,
            piet::TextAlignment::Center => Self::Center,
            piet::TextAlignment::Justified => Self::Fill,
        }
    }
}

impl From<TextAlignment> for piet::TextAlignment {
    fn from(value: TextAlignment) -> Self {
        match value {
            TextAlignment::Start => piet::TextAlignment::Start,
            TextAlignment::Center => piet::TextAlignment::Center,
            TextAlignment::End => piet::TextAlignment::End,
            TextAlignment::Fill => piet::TextAlignment::Justified,
        }
    }
}

impl TextAlignment {
    #[allow(unused)]
    pub fn from_pango_layout(pango_layout: pango::Layout) -> Self {
        if pango_layout.is_justify() {
            Self::Fill
        } else {
            match pango_layout.alignment() {
                pango::Alignment::Left => Self::Start,
                pango::Alignment::Center => Self::Center,
                pango::Alignment::Right => Self::End,
                _ => Self::Start,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "text_attribute")]
pub enum TextAttribute {
    /// The font family.
    #[serde(rename = "font_family")]
    FontFamily(String),
    /// The font size, in points.
    #[serde(rename = "font_size")]
    FontSize(f64),
    /// The font weight
    #[serde(rename = "font_weight")]
    FontWeight(u16),
    /// The foreground color of the text.
    #[serde(rename = "text_color")]
    TextColor(Color),
    /// The font style
    #[serde(rename = "font_style")]
    Style(FontStyle),
    /// Underline.
    #[serde(rename = "underline")]
    Underline(bool),
    /// Strikethrough.
    #[serde(rename = "strikethrough")]
    Strikethrough(bool),
}

impl From<piet::TextAttribute> for TextAttribute {
    fn from(value: piet::TextAttribute) -> Self {
        match value {
            piet::TextAttribute::FontFamily(font_family) => {
                Self::FontFamily(font_family.name().to_string())
            }
            piet::TextAttribute::FontSize(font_size) => Self::FontSize(font_size),
            piet::TextAttribute::Weight(font_weight) => Self::FontWeight(font_weight.to_raw()),
            piet::TextAttribute::TextColor(color) => Self::TextColor(Color::from(color)),
            piet::TextAttribute::Style(font_style) => Self::Style(font_style.into()),
            piet::TextAttribute::Underline(underline) => Self::Underline(underline),
            piet::TextAttribute::Strikethrough(strikethrough) => Self::Strikethrough(strikethrough),
        }
    }
}

impl TextAttribute {
    pub fn try_into_piet<T>(self, piet_text: &mut T) -> anyhow::Result<piet::TextAttribute>
    where
        T: piet::Text,
    {
        match self {
            TextAttribute::FontFamily(font_family) => piet_text.font_family(font_family.as_str()).map(
                piet::TextAttribute::FontFamily)
                    .ok_or_else(|| anyhow::anyhow!("piet font_family() failed in textattribute try_into_piet() with font family name: {}", font_family)),
            TextAttribute::FontSize(font_size) => Ok(piet::TextAttribute::FontSize(font_size)),
            TextAttribute::FontWeight(font_weight) => Ok(piet::TextAttribute::Weight(piet::FontWeight::new(font_weight))),
            TextAttribute::TextColor(color) => Ok(piet::TextAttribute::TextColor(piet::Color::from(color))),
            TextAttribute::Style(style) => Ok(piet::TextAttribute::Style(piet::FontStyle::from(style))),
            TextAttribute::Underline(underline) => Ok(piet::TextAttribute::Underline(underline)),
            TextAttribute::Strikethrough(strikethrough) => Ok(piet::TextAttribute::Strikethrough(strikethrough)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "ranged_text_attribute")]
pub struct RangedTextAttribute {
    #[serde(rename = "range")]
    pub range: Range<usize>,
    #[serde(rename = "attribute")]
    pub attribute: TextAttribute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "text_style")]
pub struct TextStyle {
    #[serde(rename = "font_family")]
    pub font_family: String,
    #[serde(rename = "font_size")]
    pub font_size: f64,
    #[serde(rename = "font_weight")]
    pub font_weight: u16,
    #[serde(rename = "font_style")]
    pub font_style: FontStyle,
    #[serde(rename = "color")]
    pub color: Color,
    #[serde(rename = "max_width")]
    pub max_width: Option<f64>,
    #[serde(rename = "alignment")]
    pub alignment: TextAlignment,

    #[serde(rename = "ranged_text_attributes")]
    pub ranged_text_attributes: Vec<RangedTextAttribute>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: String::from(Self::FONT_FAMILY_DEFAULT),
            font_size: Self::FONT_SIZE_DEFAULT,
            font_weight: Self::FONT_WEIGHT_DEFAULT,
            font_style: FontStyle::default(),
            color: Self::FONT_COLOR_DEFAULT,
            max_width: None,
            alignment: TextAlignment::Start,
            ranged_text_attributes: vec![],
        }
    }
}

impl TextStyle {
    pub const FONT_FAMILY_DEFAULT: &'static str = "serif";
    pub const FONT_SIZE_DEFAULT: f64 = 32.0;
    pub const FONT_SIZE_MIN: f64 = 1.0;
    pub const FONT_SIZE_MAX: f64 = 512.0;
    pub const FONT_WEIGHT_DEFAULT: u16 = 500;
    pub const FONT_COLOR_DEFAULT: Color = Color::BLACK;

    pub fn load_pango_font_desc(&mut self, pango_font_desc: pango::FontDescription) {
        if let Some(font_family) = pango_font_desc.family() {
            self.font_family = font_family.to_string();
        }

        let font_size = f64::from(pango_font_desc.size()) / f64::from(pango::SCALE);
        // Is <= 0.0 when no font size is selected
        if font_size > 0.0 {
            self.font_size = font_size;
        }

        self.font_weight = crate::utils::pango_font_weight_to_raw(pango_font_desc.weight());
        self.font_style = pango_font_desc.style().into();
    }

    pub fn extract_pango_font_desc(&self) -> pango::FontDescription {
        let mut pango_font_desc = pango::FontDescription::new();
        pango_font_desc.set_family(self.font_family.as_str());
        pango_font_desc.set_size((self.font_size * f64::from(pango::SCALE)).round() as i32);
        pango_font_desc.set_weight(crate::utils::raw_font_weight_to_pango(self.font_weight));
        pango_font_desc.set_style(self.font_style.into());

        /*
               log::debug!(
                   "extract_pango_font_descr\nfamily: {:?}; size: {:?}, weight: {:?}, style: {:?}",
                   pango_font_desc.family(),
                   pango_font_desc.size(),
                   pango_font_desc.weight(),
                   pango_font_desc.style()
               );
        */

        pango_font_desc
    }

    pub fn build_text_layout<T>(
        &self,
        piet_text: &mut T,
        text: String,
    ) -> anyhow::Result<T::TextLayout>
    where
        T: piet::Text,
    {
        let font_family = piet_text
            .font_family(&self.font_family)
            .unwrap_or(piet::FontFamily::SERIF);

        let mut text_layout_builder = piet_text
            .new_text_layout(text)
            .font(font_family, self.font_size)
            .alignment(self.alignment.into())
            .default_attribute(piet::TextAttribute::Weight(piet::FontWeight::new(
                self.font_weight,
            )))
            .default_attribute(piet::TextAttribute::Style(self.font_style.into()))
            .text_color(self.color.into());

        if let Some(max_width) = self.max_width {
            text_layout_builder = text_layout_builder.max_width(max_width);
        }

        // We need to sort the ranges before adding them to the text layout, else attributes might be skipped. (the cairo backend asserts for it in debug builds)
        // see https://docs.rs/piet/latest/piet/trait.TextLayoutBuilder.html#tymethod.range_attribute
        let mut ranged_text_attributes = self.ranged_text_attributes.clone();
        ranged_text_attributes
            .sort_unstable_by(|first, second| first.range.start.cmp(&second.range.start));

        // Apply ranged attributes
        for (range, piet_attr) in ranged_text_attributes
            .into_iter()
            .filter_map(|ranged_attr| {
                Some((
                    ranged_attr.range,
                    ranged_attr.attribute.try_into_piet(piet_text).ok()?,
                ))
            })
        {
            text_layout_builder = text_layout_builder.range_attribute(range, piet_attr);
        }

        text_layout_builder
            .build()
            .map_err(|e| anyhow::anyhow!("{e:?}"))
    }

    pub fn untransformed_size<T>(&self, piet_text: &mut T, text: String) -> Option<na::Vector2<f64>>
    where
        T: piet::Text,
    {
        let text_layout = self.build_text_layout(piet_text, text).ok()?;

        let size = text_layout.size();
        Some(na::vector![size.width, size.height])
    }

    /// the cursors line metric relative to the textstroke bounds.
    /// Index must be at a grapheme boundary
    pub fn lines<T>(&self, piet_text: &mut T, text: String) -> anyhow::Result<Vec<piet::LineMetric>>
    where
        T: piet::Text,
    {
        let text_layout = self.build_text_layout(piet_text, text)?;

        Ok((0..text_layout.line_count())
            .map(|line| text_layout.line_metric(line).unwrap())
            .collect::<Vec<piet::LineMetric>>())
    }

    /// the cursors line metric relative to the textstroke bounds.
    /// Index must be at a grapheme boundary
    pub fn cursor_line_metric<T>(
        &self,
        piet_text: &mut T,
        text: String,
        index: usize,
    ) -> anyhow::Result<piet::LineMetric>
    where
        T: piet::Text,
    {
        let lines = self.lines(piet_text, text)?;
        let cur_line = piet::util::line_number_for_position(&lines, index);

        Ok(lines[cur_line].to_owned())
    }

    pub fn cursor_hittest_position<T>(
        &self,
        piet_text: &mut T,
        text: String,
        cursor: &unicode_segmentation::GraphemeCursor,
    ) -> anyhow::Result<piet::HitTestPosition>
    where
        T: piet::Text,
    {
        let text_layout = self.build_text_layout(piet_text, text)?;

        Ok(text_layout.hit_test_text_position(cursor.cur_cursor()))
    }

    pub fn get_selection_rects_for_cursors(
        &self,
        text: String,
        cursor: &unicode_segmentation::GraphemeCursor,
        selection_cursor: &unicode_segmentation::GraphemeCursor,
    ) -> anyhow::Result<Vec<kurbo::Rect>> {
        let text_layout = self
            .build_text_layout(&mut piet_cairo::CairoText::new(), text)
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let range = if selection_cursor.cur_cursor() >= cursor.cur_cursor() {
            cursor.cur_cursor()..selection_cursor.cur_cursor()
        } else {
            selection_cursor.cur_cursor()..cursor.cur_cursor()
        };

        Ok(text_layout.rects_for_range(range))
    }

    /// The line metric is relative to the transform
    pub fn draw_cursor(
        &self,
        cx: &mut impl piet::RenderContext,
        text: String,
        cursor: &unicode_segmentation::GraphemeCursor,
        transform: &Transform,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        const CURSOR_COLOR: piet::Color = color::GNOME_DARKS[2];
        const CURSOR_OUTLINE_COLOR: piet::Color = color::GNOME_BRIGHTS[0];
        let text_cursor_width = 3.0 / camera.total_zoom();

        if let Ok(cursor_line_metric) =
            self.cursor_line_metric(cx.text(), text.clone(), cursor.cur_cursor())
        {
            let x_pos = self
                .cursor_hittest_position(cx.text(), text, cursor)?
                .point
                .x;

            let text_cursor = transform.to_kurbo()
                * kurbo::Line::new(
                    kurbo::Point::new(x_pos, cursor_line_metric.y_offset),
                    kurbo::Point::new(
                        x_pos,
                        cursor_line_metric.y_offset + cursor_line_metric.height,
                    ),
                );

            cx.stroke(text_cursor, &CURSOR_OUTLINE_COLOR, text_cursor_width);
            cx.stroke(text_cursor, &CURSOR_COLOR, text_cursor_width * 0.7);
        }

        Ok(())
    }

    pub fn draw_text_selection(
        &self,
        cx: &mut impl piet::RenderContext,
        text: String,
        cursor: &unicode_segmentation::GraphemeCursor,
        selection_cursor: &unicode_segmentation::GraphemeCursor,
        transform: &Transform,
        camera: &Camera,
    ) {
        static OUTLINE_COLOR: Lazy<piet::Color> =
            Lazy::new(|| color::GNOME_BLUES[2].with_alpha(0.941));
        static FILL_COLOR: Lazy<piet::Color> =
            Lazy::new(|| color::GNOME_BLUES[0].with_alpha(0.090));
        let outline_width = 1.5 / camera.total_zoom();

        if let Ok(selection_rects) =
            self.get_selection_rects_for_cursors(text, cursor, selection_cursor)
        {
            for selection_rect in selection_rects {
                let selection_rectpath = transform.to_kurbo() * selection_rect.to_path(0.1);

                cx.fill(selection_rectpath.clone(), &*FILL_COLOR);
                cx.stroke(selection_rectpath, &*OUTLINE_COLOR, outline_width);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "textstroke")]
pub struct TextStroke {
    #[serde(rename = "text")]
    pub text: String,
    /// The translation part is the position of the upper left corner
    #[serde(rename = "transform")]
    pub transform: Transform,
    #[serde(rename = "text_style")]
    pub text_style: TextStyle,
}

impl Default for TextStroke {
    fn default() -> Self {
        Self {
            text: String::default(),
            transform: Transform::default(),
            text_style: TextStyle::default(),
        }
    }
}

impl TransformBehaviour for TextStroke {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.transform.append_translation_mut(offset);
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.transform.append_rotation_wrt_point_mut(angle, center);
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.transform.append_scale_mut(scale);
    }
}

impl ShapeBehaviour for TextStroke {
    fn bounds(&self) -> Aabb {
        let untransformed_size = self
            .text_style
            .untransformed_size(&mut piet_cairo::CairoText::new(), self.text.clone())
            .unwrap_or_else(|| na::Vector2::repeat(self.text_style.font_size))
            .maxs(&na::vector![1.0, 1.0]);

        self.transform.transform_aabb(Aabb::new(
            na::point![0.0, 0.0],
            na::Point2::from(untransformed_size),
        ))
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let text_layout = match self
            .text_style
            .build_text_layout(&mut piet_cairo::CairoText::new(), self.text.clone())
        {
            Ok(text_layout) => text_layout,
            Err(e) => {
                log::error!(
                    "build_text_layout() failed while calculating the hitboxes, Err: {e:?}"
                );

                return vec![self.bounds()];
            }
        };

        let mut hitboxes: Vec<Aabb> = text_layout
            .rects_for_range(0..self.text.len())
            .into_iter()
            .map(|rect| self.transform.transform_aabb(Aabb::from_kurbo_rect(rect)))
            .collect();

        let text_size = text_layout.size();

        if hitboxes.is_empty() {
            hitboxes.push(self.transform.transform_aabb(Aabb::new_positive(
                na::point![0.0, 0.0],
                na::Point2::from(
                    na::vector![text_size.width, text_size.height].maxs(&na::vector![1.0, 1.0]),
                ),
            )))
        }

        hitboxes
    }
}

impl StrokeBehaviour for TextStroke {
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error> {
        let bounds = self.bounds();

        // We need to generate the svg with the cairo backend, because text layout would differ with the svg backend
        render::Svg::gen_with_piet_cairo_backend(
            |cx| {
                cx.transform(kurbo::Affine::translate(-bounds.mins.coords.to_kurbo_vec()));
                self.draw(cx, 1.0)
            },
            bounds,
        )
    }

    fn gen_images(
        &self,
        viewport: Aabb,
        image_scale: f64,
    ) -> Result<GeneratedStrokeImages, anyhow::Error> {
        let bounds = self.bounds();

        if viewport.contains(&bounds) {
            Ok(GeneratedStrokeImages::Full(vec![
                render::Image::gen_with_piet(
                    |piet_cx| self.draw(piet_cx, image_scale),
                    bounds,
                    image_scale,
                )?,
            ]))
        } else if let Some(intersection_bounds) = viewport.intersection(&bounds) {
            Ok(GeneratedStrokeImages::Partial {
                images: vec![render::Image::gen_with_piet(
                    |piet_cx| self.draw(piet_cx, image_scale),
                    intersection_bounds,
                    image_scale,
                )?],
                viewport,
            })
        } else {
            Ok(GeneratedStrokeImages::Partial {
                images: vec![],
                viewport,
            })
        }
    }
}

impl DrawBehaviour for TextStroke {
    fn draw(&self, cx: &mut impl RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        if let Ok(text_layout) = self
            .text_style
            .build_text_layout(cx.text(), self.text.clone())
        {
            cx.transform(self.transform.affine.to_kurbo());
            cx.draw_text(&text_layout, kurbo::Point::new(0.0, 0.0))
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl TextStroke {
    pub fn new(text: String, upper_left_pos: na::Vector2<f64>, text_style: TextStyle) -> Self {
        Self {
            text,
            transform: Transform::new_w_isometry(na::Isometry2::new(upper_left_pos, 0.0)),
            text_style,
        }
    }

    pub fn get_text_slice_for_range(&self, range: Range<usize>) -> &str {
        &self.text[range]
    }

    // Gets a cursor matching best for the given coord. The coord is in global coordinate space
    pub fn get_cursor_for_global_coord(
        &self,
        coord: na::Vector2<f64>,
    ) -> anyhow::Result<unicode_segmentation::GraphemeCursor> {
        let text_layout = self
            .text_style
            .build_text_layout(&mut piet_cairo::CairoText::new(), self.text.clone())
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let hit_test_point = text_layout.hit_test_point(
            (self.transform.affine.inverse() * na::Point2::from(coord))
                .coords
                .to_kurbo_point(),
        );

        Ok(unicode_segmentation::GraphemeCursor::new(
            hit_test_point.idx,
            self.text.len(),
            true,
        ))
    }

    pub fn insert_text_after_cursor(
        &mut self,
        text: &str,
        cursor: &mut unicode_segmentation::GraphemeCursor,
    ) {
        self.text.insert_str(cursor.cur_cursor(), text);

        // translate the text attributes
        self.translate_attrs_after_cursor(cursor.cur_cursor(), text.len() as i32);

        *cursor = unicode_segmentation::GraphemeCursor::new(
            cursor.cur_cursor() + text.len(),
            self.text.len(),
            true,
        );
    }

    pub fn remove_grapheme_before_cursor(
        &mut self,
        cursor: &mut unicode_segmentation::GraphemeCursor,
    ) {
        if !self.text.is_empty() && self.text.len() >= cursor.cur_cursor() {
            let cur_pos = cursor.cur_cursor();

            if let Some(prev_pos) = cursor.prev_boundary(&self.text, 0).unwrap() {
                self.text.replace_range(prev_pos..cur_pos, "");

                // translate the text attributes
                self.translate_attrs_after_cursor(
                    prev_pos,
                    prev_pos as i32 - cur_pos as i32 + "".len() as i32,
                );
            }

            // New text length, new cursor
            *cursor = unicode_segmentation::GraphemeCursor::new(
                cursor.cur_cursor(),
                self.text.len(),
                true,
            );
        }
    }

    pub fn remove_grapheme_after_cursor(
        &mut self,
        cursor: &mut unicode_segmentation::GraphemeCursor,
    ) {
        if !self.text.is_empty() && self.text.len() > cursor.cur_cursor() {
            let cur_pos = cursor.cur_cursor();

            if let Some(next_pos) = cursor.clone().next_boundary(&self.text, 0).unwrap() {
                self.text.replace_range(cur_pos..next_pos, "");

                // translate the text attributes
                self.translate_attrs_after_cursor(
                    cur_pos,
                    -(next_pos as i32 - cur_pos as i32) + "".len() as i32,
                );
            }

            // New text length, new cursor
            *cursor = unicode_segmentation::GraphemeCursor::new(cur_pos, self.text.len(), true);
        }
    }

    pub fn replace_text_between_selection_cursors(
        &mut self,
        cursor: &mut unicode_segmentation::GraphemeCursor,
        selection_cursor: &mut unicode_segmentation::GraphemeCursor,
        replace_text: &str,
    ) {
        let cursor_pos = cursor.cur_cursor();
        let selection_cursor_pos = selection_cursor.cur_cursor();

        let cursor_range = if cursor_pos < selection_cursor_pos {
            cursor_pos..selection_cursor_pos
        } else {
            selection_cursor_pos..cursor_pos
        };

        self.text.replace_range(cursor_range.clone(), replace_text);

        *cursor = unicode_segmentation::GraphemeCursor::new(
            cursor_range.start + replace_text.len(),
            self.text.len(),
            true,
        );
        *selection_cursor = unicode_segmentation::GraphemeCursor::new(
            cursor_range.start + replace_text.len(),
            self.text.len(),
            true,
        );

        self.translate_attrs_after_cursor(
            cursor.cur_cursor(),
            -(cursor_range.end as i32 - cursor_range.start as i32) + replace_text.len() as i32,
        );
    }

    // Translates the ranged text attributes after the given cursor. Overlapping ranges are extended / shrunk
    fn translate_attrs_after_cursor(&mut self, from_pos: usize, offset: i32) {
        for attr in self.text_style.ranged_text_attributes.iter_mut() {
            if attr.range.start > from_pos {
                if offset >= 0 {
                    attr.range.start = attr
                        .range
                        .start
                        .saturating_add(offset.unsigned_abs() as usize);
                    attr.range.end = attr
                        .range
                        .end
                        .saturating_add(offset.unsigned_abs() as usize);
                } else {
                    attr.range.start = attr
                        .range
                        .start
                        .saturating_sub(offset.unsigned_abs() as usize);
                    attr.range.end = attr
                        .range
                        .end
                        .saturating_sub(offset.unsigned_abs() as usize);
                }
            } else if attr.range.end > from_pos {
                if offset >= 0 {
                    attr.range.end = attr
                        .range
                        .end
                        .saturating_add(offset.unsigned_abs() as usize);
                } else {
                    attr.range.end = attr
                        .range
                        .end
                        .saturating_sub(offset.unsigned_abs() as usize);
                }
            }
        }
    }

    /// Removes all attr in the given range
    pub fn remove_attrs_for_range(&mut self, range: Range<usize>) {
        // partition into attrs that intersect the range, and those who don't and will be retained
        let (intersecting_attrs, mut retained_attrs): (
            Vec<RangedTextAttribute>,
            Vec<RangedTextAttribute>,
        ) = self
            .text_style
            .ranged_text_attributes
            .clone()
            .into_iter()
            .partition(|attr| attr.range.end > range.start && attr.range.start < range.end);

        // Truncate and filter the ranges of intersecting attrs
        let truncated_attrs = intersecting_attrs
            .into_iter()
            .flat_map(|mut attr| {
                //log::debug!("attr.range: {:?}, range: {:?}", attr.range, range);

                if attr.range.start <= range.start && attr.range.end >= range.end {
                    // if the attribute completely contains the given range, split it
                    let mut first_split_attr = attr.clone();
                    first_split_attr.range.end = range.start;
                    let mut second_split_attr = attr;
                    second_split_attr.range.start = range.end;

                    vec![first_split_attr, second_split_attr]
                } else if attr.range.start <= range.start && attr.range.end > range.start {
                    // overlapping from the left, so truncate the end
                    attr.range.end = range.start;
                    vec![attr]
                } else if attr.range.end >= range.end && attr.range.start < range.end {
                    // overlapping from the right, so truncate the start
                    attr.range.start = range.end;
                    vec![attr]
                } else {
                    // Else the attribute is in the range, so we discard it
                    vec![]
                }
            })
            // Filter out any that became empty or are contained in the given range
            .filter(|attr| !attr.range.is_empty())
            .collect::<Vec<RangedTextAttribute>>();

        // Set the updated attributes
        self.text_style.ranged_text_attributes = {
            retained_attrs.extend(truncated_attrs);
            retained_attrs
        };
    }

    pub fn update_selection_entire_text(
        &self,
        cursor: &mut unicode_segmentation::GraphemeCursor,
        selection_cursor: &mut unicode_segmentation::GraphemeCursor,
    ) {
        *cursor = unicode_segmentation::GraphemeCursor::new(self.text.len(), self.text.len(), true);
        *selection_cursor = unicode_segmentation::GraphemeCursor::new(0, self.text.len(), true);
    }

    pub fn move_cursor_back(&self, cursor: &mut unicode_segmentation::GraphemeCursor) {
        // Cant fail, we are providing the entire text
        cursor.prev_boundary(&self.text, 0).unwrap();
    }

    pub fn move_cursor_forward(&self, cursor: &mut unicode_segmentation::GraphemeCursor) {
        // Cant fail, we are providing the entire text
        cursor.next_boundary(&self.text, 0).unwrap();
    }

    pub fn move_cursor_line_down(&self, cursor: &mut unicode_segmentation::GraphemeCursor) {
        if let (Ok(lines), Ok(hittest_position)) = (
            self.text_style
                .lines(&mut piet_cairo::CairoText::new(), self.text.clone()),
            self.text_style.cursor_hittest_position(
                &mut piet_cairo::CairoText::new(),
                self.text.clone(),
                cursor,
            ),
        ) {
            let next_line = (hittest_position.line + 1).min(lines.len() - 1);

            if next_line != hittest_position.line {
                let current_line_rel_offset =
                    cursor.cur_cursor() - lines[hittest_position.line].start_offset;
                let next_line_max_offset =
                    lines[next_line].end_offset - 1 - lines[next_line].start_offset;

                let line_rel_offset = current_line_rel_offset.min(next_line_max_offset);

                *cursor = unicode_segmentation::GraphemeCursor::new(
                    lines[next_line].start_offset + line_rel_offset,
                    self.text.len(),
                    true,
                );
            }
        }
    }

    pub fn move_cursor_line_up(&self, cursor: &mut unicode_segmentation::GraphemeCursor) {
        if let (Ok(lines), Ok(hittest_position)) = (
            self.text_style
                .lines(&mut piet_cairo::CairoText::new(), self.text.clone()),
            self.text_style.cursor_hittest_position(
                &mut piet_cairo::CairoText::new(),
                self.text.clone(),
                cursor,
            ),
        ) {
            let prev_line = hittest_position.line.saturating_sub(1);

            if prev_line != hittest_position.line {
                let current_line_rel_offset =
                    cursor.cur_cursor() - lines[hittest_position.line].start_offset;
                let prev_line_max_offset =
                    lines[prev_line].end_offset - 1 - lines[prev_line].start_offset;

                let line_rel_offset = current_line_rel_offset.min(prev_line_max_offset);

                *cursor = unicode_segmentation::GraphemeCursor::new(
                    lines[prev_line].start_offset + line_rel_offset,
                    self.text.len(),
                    true,
                );
            }
        }
    }
}
