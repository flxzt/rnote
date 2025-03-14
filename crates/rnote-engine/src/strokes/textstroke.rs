// Imports
use super::Content;
use crate::engine::Spellcheck;
use crate::{Camera, Drawable};
use itertools::Itertools;
use kurbo::{BezPath, Shape};
use p2d::bounding_volume::Aabb;
use piet::{RenderContext, TextLayout, TextLayoutBuilder};
use rnote_compose::ext::{AabbExt, Affine2Ext, Vector2Ext};
use rnote_compose::shapes::Shapeable;
use rnote_compose::transform::Transformable;
use rnote_compose::{Color, Transform, color};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Range;
use tracing::error;
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "text_attribute")]
pub enum TextAttribute {
    /// The font family.
    #[serde(rename = "font_family")]
    FontFamily(String),
    /// The font size, in points.
    #[serde(rename = "font_size")]
    FontSize(f64),
    /// The font weight.
    #[serde(rename = "font_weight")]
    FontWeight(u16),
    /// The foreground color of the text.
    #[serde(rename = "text_color")]
    TextColor(Color),
    /// The font style.
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
            TextAttribute::FontFamily(font_family) => piet_text
                .font_family(font_family.as_str())
                .map(piet::TextAttribute::FontFamily)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "query piet font family returned None for font family '{font_family}"
                    )
                }),
            TextAttribute::FontSize(font_size) => Ok(piet::TextAttribute::FontSize(font_size)),
            TextAttribute::FontWeight(font_weight) => Ok(piet::TextAttribute::Weight(
                piet::FontWeight::new(font_weight),
            )),
            TextAttribute::TextColor(color) => {
                Ok(piet::TextAttribute::TextColor(piet::Color::from(color)))
            }
            TextAttribute::Style(style) => {
                Ok(piet::TextAttribute::Style(piet::FontStyle::from(style)))
            }
            TextAttribute::Underline(underline) => Ok(piet::TextAttribute::Underline(underline)),
            TextAttribute::Strikethrough(strikethrough) => {
                Ok(piet::TextAttribute::Strikethrough(strikethrough))
            }
        }
    }

    fn same_variant(&self, other: &TextAttribute) -> bool {
        match (self, other) {
            (TextAttribute::FontFamily(_), TextAttribute::FontFamily(_))
            | (TextAttribute::FontSize(_), TextAttribute::FontSize(_))
            | (TextAttribute::FontWeight(_), TextAttribute::FontWeight(_))
            | (TextAttribute::TextColor(_), TextAttribute::TextColor(_))
            | (TextAttribute::Style(_), TextAttribute::Style(_))
            | (TextAttribute::Underline(_), TextAttribute::Underline(_))
            | (TextAttribute::Strikethrough(_), TextAttribute::Strikethrough(_)) => true,
            (_, _) => false,
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
    max_width: Option<f64>,
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

    pub fn max_width(&self) -> Option<f64> {
        self.max_width
    }

    pub fn set_max_width(&mut self, max_width: Option<f64>) {
        self.max_width = max_width.map(|w| w.max(0.));
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

        // We need to sort the ranges before adding them to the text layout, else attributes might be skipped.
        // The cairo backend asserts for it in debug builds.
        //
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
            .map_err(|e| anyhow::anyhow!("Building piet text layout failed, Err: {e:?}"))
    }

    pub fn untransformed_size<T>(&self, piet_text: &mut T, text: String) -> Option<na::Vector2<f64>>
    where
        T: piet::Text,
    {
        let text_layout = self.build_text_layout(piet_text, text).ok()?;

        let size = text_layout.size();
        Some(na::vector![size.width, size.height])
    }

    /// The cursors line metric relative to the textstroke bounds.
    pub fn lines<T>(&self, piet_text: &mut T, text: String) -> anyhow::Result<Vec<piet::LineMetric>>
    where
        T: piet::Text,
    {
        let text_layout = self.build_text_layout(piet_text, text)?;

        Ok((0..text_layout.line_count())
            .map(|line| text_layout.line_metric(line).unwrap())
            .collect::<Vec<piet::LineMetric>>())
    }

    /// The cursors line metric relative to the textstroke bounds.
    ///
    /// Index must be at a grapheme boundary.
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
        cursor: &GraphemeCursor,
    ) -> anyhow::Result<piet::HitTestPosition>
    where
        T: piet::Text,
    {
        let text_layout = self.build_text_layout(piet_text, text)?;

        Ok(text_layout.hit_test_text_position(cursor.cur_cursor()))
    }

    pub fn get_rects_for_indices(
        &self,
        text: String,
        start_index: usize,
        end_index: usize,
    ) -> anyhow::Result<Vec<kurbo::Rect>> {
        let text_layout = self
            .build_text_layout(&mut piet_cairo::CairoText::new(), text)
            .map_err(|e| anyhow::anyhow!("Building text layout failed, Err: {e:?}"))?;

        let range = if end_index >= start_index {
            start_index..end_index
        } else {
            end_index..start_index
        };

        Ok(text_layout.rects_for_range(range))
    }

    /// Draw the cursor.
    pub fn draw_cursor(
        &self,
        cx: &mut impl piet::RenderContext,
        text: String,
        cursor: &GraphemeCursor,
        transform: &Transform,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        const CURSOR_COLOR: piet::Color = color::GNOME_DARKS[2];
        const CURSOR_OUTLINE_COLOR: piet::Color = color::GNOME_BRIGHTS[0];
        let text_cursor_width = 2.0 / camera.total_zoom();

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

            cx.stroke_styled(
                text_cursor,
                &CURSOR_OUTLINE_COLOR,
                text_cursor_width,
                &piet::StrokeStyle::default().line_cap(piet::LineCap::Butt),
            );
            cx.stroke_styled(
                text_cursor,
                &CURSOR_COLOR,
                text_cursor_width * 0.8,
                &piet::StrokeStyle::default().line_cap(piet::LineCap::Butt),
            );
        }

        Ok(())
    }

    pub fn draw_text_error(
        &self,
        cx: &mut impl piet::RenderContext,
        text: String,
        start_index: usize,
        end_index: usize,
        transform: &Transform,
        camera: &Camera,
    ) {
        const ERROR_COLOR: piet::Color = color::GNOME_REDS[2];
        const STYLE: piet::StrokeStyle = piet::StrokeStyle::new().line_cap(piet::LineCap::Round);

        let scale = 1.0 / camera.total_zoom();

        if let Ok(selection_rects) =
            self.get_rects_for_indices(text.clone(), start_index, end_index)
        {
            // Get baseline for the current line. Really unnecessary to do this for every error since the font size is uniform,
            // but piet does not provide any other way to get the baseline.

            if let Ok(line_metric) = self.cursor_line_metric(cx.text(), text, start_index) {
                for selection_rect in selection_rects {
                    let width = selection_rect.width();
                    let origin = transform.to_kurbo()
                        * kurbo::Point::new(
                            selection_rect.x0,
                            selection_rect.y0 + line_metric.baseline + 2.0,
                        );

                    let path = create_wavy_line(origin, width, scale);
                    cx.stroke_styled(path, &ERROR_COLOR, 1.5 * scale, &STYLE);
                }
            }
        }
    }

    pub fn draw_text_selection(
        &self,
        cx: &mut impl piet::RenderContext,
        text: String,
        cursor: &GraphemeCursor,
        selection_cursor: &GraphemeCursor,
        transform: &Transform,
        camera: &Camera,
    ) {
        const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[2];
        const FILL_COLOR: piet::Color = color::GNOME_BLUES[1].with_a8(25);
        let outline_width = 1.5 / camera.total_zoom();

        if let Ok(selection_rects) =
            self.get_rects_for_indices(text, cursor.cur_cursor(), selection_cursor.cur_cursor())
        {
            for selection_rect in selection_rects {
                let outline = transform.to_kurbo() * selection_rect.to_path(0.5);

                cx.fill(&outline, &FILL_COLOR);
                cx.stroke(&outline, &OUTLINE_COLOR, outline_width);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SpellcheckResult {
    pub language: Option<String>,
    pub errors: BTreeMap<usize, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "textstroke")]
pub struct TextStroke {
    #[serde(rename = "text")]
    pub text: String,
    /// The transformation.
    ///
    /// The translation part Is the position of the upper left corner
    #[serde(rename = "transform")]
    pub transform: Transform,
    #[serde(rename = "text_style")]
    pub text_style: TextStyle,
    #[serde(skip)]
    pub spellcheck_result: SpellcheckResult,
}

impl Default for TextStroke {
    fn default() -> Self {
        Self {
            text: String::default(),
            transform: Transform::default(),
            text_style: TextStyle::default(),
            spellcheck_result: SpellcheckResult::default(),
        }
    }
}

impl Transformable for TextStroke {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.transform.append_translation_mut(offset);
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.transform.append_rotation_wrt_point_mut(angle, center);
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.transform.append_scale_mut(scale);
    }
}

impl Shapeable for TextStroke {
    fn bounds(&self) -> Aabb {
        let untransformed_size = self
            .text_style
            .untransformed_size(&mut piet_cairo::CairoText::new(), self.text.clone())
            .unwrap_or_else(|| na::Vector2::repeat(self.text_style.font_size))
            .maxs(&na::vector![1.0, 1.0]);

        self.transform
            .transform_aabb(Aabb::new(na::point![0.0, 0.0], untransformed_size.into()))
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let text_layout = match self
            .text_style
            .build_text_layout(&mut piet_cairo::CairoText::new(), self.text.clone())
        {
            Ok(text_layout) => text_layout,
            Err(e) => {
                error!("Building text layout failed while calculating the hitboxes, Err: {e:?}");
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
            hitboxes.push(
                self.transform.transform_aabb(Aabb::new_positive(
                    na::point![0.0, 0.0],
                    na::vector![text_size.width, text_size.height]
                        .maxs(&na::vector![1.0, 1.0])
                        .into(),
                )),
            )
        }

        hitboxes
    }

    fn outline_path(&self) -> kurbo::BezPath {
        self.bounds().to_kurbo_rect().to_path(0.25)
    }
}

impl Content for TextStroke {
    fn update_geometry(&mut self) {}
}

impl Drawable for TextStroke {
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
            spellcheck_result: SpellcheckResult::default(),
        }
    }

    pub fn get_text_slice_for_range(&self, range: Range<usize>) -> &str {
        &self.text[range]
    }

    /// Get a cursor matching best for the given coordinate.
    ///
    /// `coord` must be in global coordinate space.
    pub fn get_cursor_for_global_coord(
        &self,
        coord: na::Vector2<f64>,
    ) -> anyhow::Result<GraphemeCursor> {
        let text_layout = self
            .text_style
            .build_text_layout(&mut piet_cairo::CairoText::new(), self.text.clone())
            .map_err(|e| anyhow::anyhow!("Building text layout failed, Err: {e:?}"))?;
        let hit_test_point = text_layout.hit_test_point(
            self.transform
                .affine
                .inverse()
                .transform_point(&coord.into())
                .coords
                .to_kurbo_point(),
        );

        Ok(GraphemeCursor::new(
            hit_test_point.idx,
            self.text.len(),
            true,
        ))
    }

    fn check_spelling_words(&mut self, words: Vec<(usize, String)>, dict: &enchant::Dict) {
        for (word_start_index, word) in words {
            if let Ok(valid_word) = dict.check(word.as_str()) {
                let word_end_index = word_start_index + word.len();
                let word_range = word_start_index..word_end_index;

                self.spellcheck_result
                    .errors
                    .retain(|key, _| !word_range.contains(key));

                // TODO: maybe faster for large texts
                // let keys_to_remove = self
                //     .error_words
                //     .range(word_range)
                //     .map(|(&key, _)| key)
                //     .collect_vec();

                // for existing_word in keys_to_remove {
                //     self.error_words.remove(&existing_word);
                // }

                if !valid_word {
                    self.spellcheck_result
                        .errors
                        .insert(word_start_index, word.len());
                }
            } else {
                error!("Failed to check spelling for word '{word}'");
            }
        }
    }

    pub fn check_spelling_refresh_cache(&mut self, spellcheck: &Spellcheck) {
        if let Some(dict) = &spellcheck.dict {
            let language = dict.get_lang();

            let language_changed = self
                .spellcheck_result
                .language
                .clone()
                .is_none_or(|cached_language| cached_language != language);

            if language_changed {
                self.spellcheck_result.errors.clear();
                self.spellcheck_result.language = Some(language.to_owned());

                let words = self
                    .text
                    .unicode_word_indices()
                    .map(|(index, word)| (index, word.to_owned()))
                    .collect_vec();

                self.check_spelling_words(words, dict);
            }
        } else {
            self.spellcheck_result.errors.clear();
            self.spellcheck_result.language = None;
        }
    }

    pub fn get_spellcheck_corrections_at_index(
        &self,
        spellcheck: &Spellcheck,
        index: usize,
    ) -> Option<Vec<String>> {
        let Some(dict) = &spellcheck.dict else {
            return None;
        };

        let start_index = self.get_prev_word_start_index(index);

        if let Some(length) = self.spellcheck_result.errors.get(&start_index) {
            let word = self.get_text_slice_for_range(start_index..start_index + length);
            return Some(dict.suggest(word));
        }

        None
    }

    pub fn apply_spellcheck_correction_at_cursor(
        &mut self,
        cursor: &mut GraphemeCursor,
        correction: &str,
    ) {
        let cur_pos = cursor.cur_cursor();
        let start_index = self.get_prev_word_start_index(cur_pos);

        if let Some(length) = self.spellcheck_result.errors.get(&start_index) {
            let old_length = *length;
            let new_length = correction.len();

            self.text
                .replace_range(start_index..start_index + old_length, correction);

            self.spellcheck_result.errors.remove(&start_index);

            // translate the text attributes
            self.translate_attrs_after_cursor(
                start_index + old_length,
                (new_length as i32) - (old_length as i32),
            );

            *cursor = GraphemeCursor::new(start_index + new_length, self.text.len(), true);
        }
    }

    pub fn check_spelling_range(
        &mut self,
        start_index: usize,
        end_index: usize,
        spellcheck: &Spellcheck,
    ) {
        if let Some(dict) = &spellcheck.dict {
            let words = self.get_surrounding_words(start_index, end_index);
            self.check_spelling_words(words, dict);
        }
    }

    pub fn insert_text_after_cursor(
        &mut self,
        text: &str,
        cursor: &mut GraphemeCursor,
        spellcheck: &Spellcheck,
    ) {
        let cur_pos = cursor.cur_cursor();
        let next_pos = cur_pos + text.len();

        self.text.insert_str(cur_pos, text);

        // translate the text attributes
        self.translate_attrs_after_cursor(cur_pos, text.len() as i32);

        self.check_spelling_range(cur_pos, next_pos, spellcheck);

        *cursor = GraphemeCursor::new(next_pos, self.text.len(), true);
    }

    pub fn remove_grapheme_before_cursor(
        &mut self,
        cursor: &mut GraphemeCursor,
        spellcheck: &Spellcheck,
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

                self.check_spelling_range(prev_pos, cur_pos, spellcheck);
            }

            // New text length, new cursor
            *cursor = GraphemeCursor::new(cursor.cur_cursor(), self.text.len(), true);
        }
    }

    pub fn remove_grapheme_after_cursor(
        &mut self,
        cursor: &mut GraphemeCursor,
        spellcheck: &Spellcheck,
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

                self.check_spelling_range(cur_pos, next_pos, spellcheck);
            }

            // New text length, new cursor
            *cursor = GraphemeCursor::new(cur_pos, self.text.len(), true);
        }
    }

    pub fn remove_word_before_cursor(
        &mut self,
        cursor: &mut GraphemeCursor,
        spellcheck: &Spellcheck,
    ) {
        let cur_pos = cursor.cur_cursor();
        let prev_pos = self.get_prev_word_start_index(cur_pos);

        if cur_pos != prev_pos {
            self.text.replace_range(prev_pos..cur_pos, "");

            // translate the text attributes
            self.translate_attrs_after_cursor(
                prev_pos,
                prev_pos as i32 - cur_pos as i32 + "".len() as i32,
            );

            self.check_spelling_range(prev_pos, cur_pos, spellcheck);

            // New text length, new cursor
            *cursor = GraphemeCursor::new(prev_pos, self.text.len(), true);
        }
    }

    pub fn remove_word_after_cursor(
        &mut self,
        cursor: &mut GraphemeCursor,
        spellcheck: &Spellcheck,
    ) {
        let cur_pos = cursor.cur_cursor();
        let next_pos = self.get_next_word_end_index(cur_pos);

        if cur_pos != next_pos {
            self.text.replace_range(cur_pos..next_pos, "");

            // translate the text attributes
            self.translate_attrs_after_cursor(
                cur_pos,
                -(next_pos as i32 - cur_pos as i32) + "".len() as i32,
            );

            self.check_spelling_range(cur_pos, next_pos, spellcheck);

            // New text length, new cursor
            *cursor = GraphemeCursor::new(cur_pos, self.text.len(), true);
        }
    }

    pub fn replace_text_between_selection_cursors(
        &mut self,
        cursor: &mut GraphemeCursor,
        selection_cursor: &mut GraphemeCursor,
        replace_text: &str,
        spellcheck: &Spellcheck,
    ) {
        let cursor_pos = cursor.cur_cursor();
        let selection_cursor_pos = selection_cursor.cur_cursor();

        let cursor_range = if cursor_pos < selection_cursor_pos {
            cursor_pos..selection_cursor_pos
        } else {
            selection_cursor_pos..cursor_pos
        };

        self.text.replace_range(cursor_range.clone(), replace_text);

        *cursor = GraphemeCursor::new(
            cursor_range.start + replace_text.len(),
            self.text.len(),
            true,
        );
        *selection_cursor = GraphemeCursor::new(
            cursor_range.start + replace_text.len(),
            self.text.len(),
            true,
        );

        self.translate_attrs_after_cursor(
            cursor.cur_cursor(),
            -(cursor_range.end as i32 - cursor_range.start as i32) + replace_text.len() as i32,
        );

        self.check_spelling_range(
            cursor_range.start,
            cursor_range.start + replace_text.len(),
            spellcheck,
        );
    }

    /// Translate the ranged text attributes after the given cursor.
    ///
    /// Overlapping ranges are extended / shrunk
    ///
    /// * `from_pos` is always the start of the range to translate.
    /// * `offset` is the translation. The end of the range is calculated by adding the **absolute** value of the offset.
    fn translate_attrs_after_cursor(&mut self, from_pos: usize, offset: i32) {
        let translated_words = if offset < 0 {
            let to_pos = from_pos.saturating_add(offset.unsigned_abs() as usize);
            self.spellcheck_result
                .errors
                .split_off(&from_pos)
                .split_off(&to_pos)
        } else {
            self.spellcheck_result.errors.split_off(&from_pos)
        };

        for (word_start, word_length) in translated_words {
            let Some(new_word_start) = word_start.checked_add_signed(offset as isize) else {
                continue;
            };

            if new_word_start >= from_pos {
                self.spellcheck_result
                    .errors
                    .insert(new_word_start, word_length);
            }
        }

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

    /// Remove all attributes in the given range.
    pub fn remove_attrs_for_range(&mut self, range: Range<usize>) {
        // partition into attrs that intersect the range, and those who don't and will be retained
        let (intersecting_attrs, mut retained_attrs) = get_intersecting_attrs_for_range(
            &range,
            self.text_style.ranged_text_attributes.clone(),
        );

        // Truncate and filter the ranges of intersecting attrs
        let truncated_attrs = remove_intersecting_attrs_in_range(&range, intersecting_attrs);

        // Set the updated attributes
        self.text_style.ranged_text_attributes = {
            retained_attrs.extend(truncated_attrs);
            retained_attrs
        };
    }

    /// Replace the attribute of the same type in the given range.
    pub fn replace_attr_for_range(&mut self, range: Range<usize>, text_attribute: TextAttribute) {
        let (intersecting_attrs, mut retained_attrs) = get_intersecting_attrs_for_range(
            &range,
            self.text_style.ranged_text_attributes.clone(),
        );
        let truncated_attrs = remove_intersecting_attrs_in_range(
            &range,
            intersecting_attrs
                .into_iter()
                .filter(|attr| attr.attribute.same_variant(&text_attribute))
                .collect(),
        );
        self.text_style.ranged_text_attributes = {
            retained_attrs.extend(truncated_attrs);
            retained_attrs.push(RangedTextAttribute {
                range,
                attribute: text_attribute,
            });
            retained_attrs
        };
    }

    pub fn toggle_attrs_for_range(&mut self, range: Range<usize>, text_attribute: TextAttribute) {
        let (matching_attributes, mut non_matching_attrs) = self
            .text_style
            .ranged_text_attributes
            .clone()
            .into_iter()
            .partition(|attr| attr.attribute.same_variant(&text_attribute));

        let (intersecting_attrs, retained_attrs) =
            get_intersecting_attrs_for_range(&range, matching_attributes);

        let toggled_attribute = intersecting_attrs
            .clone()
            .into_iter()
            .sorted_by(|a, b| (a.range.end - a.range.start).cmp(&(b.range.end - b.range.start)))
            // Filter out any that became empty or are contained in the given range
            .collect::<Vec<RangedTextAttribute>>()
            .first()
            .map(|attr| match &attr.attribute {
                TextAttribute::Strikethrough(strike) => Some(TextAttribute::Strikethrough(!strike)),
                TextAttribute::Underline(underline) => Some(TextAttribute::Underline(!underline)),
                TextAttribute::Style(FontStyle::Regular) => {
                    Some(TextAttribute::Style(FontStyle::Italic))
                }
                TextAttribute::Style(FontStyle::Italic) => {
                    Some(TextAttribute::Style(FontStyle::Regular))
                }
                TextAttribute::FontWeight(_bold_weight) => None,
                _ => Some(text_attribute.clone()),
            })
            .unwrap_or_else(|| Some(text_attribute.clone()));

        let truncated_attrs = remove_intersecting_attrs_in_range(&range, intersecting_attrs);

        non_matching_attrs.extend(retained_attrs);
        non_matching_attrs.extend(truncated_attrs);
        if let Some(attribute) = toggled_attribute {
            non_matching_attrs.push(RangedTextAttribute { attribute, range });
        }

        self.text_style.ranged_text_attributes = non_matching_attrs;
    }

    pub fn update_selection_entire_text(
        &self,
        cursor: &mut GraphemeCursor,
        selection_cursor: &mut GraphemeCursor,
    ) {
        cursor.set_cursor(self.text.len());
        selection_cursor.set_cursor(0);
    }

    fn get_surrounding_words(&self, start_index: usize, end_index: usize) -> Vec<(usize, String)> {
        let mut words = Vec::new();

        for (word_start, word) in self.text.unicode_word_indices() {
            let word_end = word_start + word.len();

            if word_end >= start_index && word_start <= end_index {
                words.push((word_start, word.to_owned()));
            }
        }

        // debug!("surrounding words: {words:?}");

        words
    }

    fn get_prev_word_start_index(&self, current_char_index: usize) -> usize {
        for (start_index, _) in self.text.unicode_word_indices().rev() {
            if start_index < current_char_index {
                return start_index;
            }
        }

        current_char_index
    }

    fn get_prev_word_boundary_index(&self, current_char_index: usize) -> usize {
        for (start_index, word) in self.text.unicode_word_indices().rev() {
            let end_index = start_index + word.len();

            if end_index < current_char_index {
                return end_index;
            }

            if start_index < current_char_index {
                return start_index;
            }
        }

        current_char_index
    }

    fn get_next_word_end_index(&self, current_char_index: usize) -> usize {
        for (start_index, word) in self.text.unicode_word_indices() {
            let end_index = start_index + word.len();

            if end_index > current_char_index {
                return end_index;
            }
        }

        current_char_index
    }

    fn get_next_word_boundary_index(&self, current_char_index: usize) -> usize {
        for (start_index, word) in self.text.unicode_word_indices() {
            if start_index >= current_char_index {
                return start_index;
            }

            let end_index = start_index + word.len();

            if end_index >= current_char_index {
                return end_index;
            }
        }

        current_char_index
    }

    pub fn move_cursor_back(&self, cursor: &mut GraphemeCursor) {
        // Cant fail, we are providing the entire text
        cursor.prev_boundary(&self.text, 0).unwrap();
    }

    pub fn move_cursor_forward(&self, cursor: &mut GraphemeCursor) {
        // Cant fail, we are providing the entire text
        cursor.next_boundary(&self.text, 0).unwrap();
    }

    pub fn move_cursor_word_back(&self, cursor: &mut GraphemeCursor) {
        cursor.set_cursor(self.get_prev_word_start_index(cursor.cur_cursor()));
    }

    pub fn move_cursor_word_boundary_back(&self, cursor: &mut GraphemeCursor) {
        cursor.set_cursor(self.get_prev_word_boundary_index(cursor.cur_cursor()));
    }

    pub fn move_cursor_word_forward(&self, cursor: &mut GraphemeCursor) {
        cursor.set_cursor(self.get_next_word_end_index(cursor.cur_cursor()));
    }

    pub fn move_cursor_word_boundary_forward(&self, cursor: &mut GraphemeCursor) {
        cursor.set_cursor(self.get_next_word_boundary_index(cursor.cur_cursor()));
    }

    pub fn move_cursor_text_start(&self, cursor: &mut GraphemeCursor) {
        cursor.set_cursor(0);
    }

    pub fn move_cursor_text_end(&self, cursor: &mut GraphemeCursor) {
        cursor.set_cursor(self.text.len());
    }

    pub fn move_cursor_line_start(&self, cursor: &mut GraphemeCursor) {
        if let (Ok(lines), Ok(hittest_position)) = (
            self.text_style
                .lines(&mut piet_cairo::CairoText::new(), self.text.clone()),
            self.text_style.cursor_hittest_position(
                &mut piet_cairo::CairoText::new(),
                self.text.clone(),
                cursor,
            ),
        ) {
            cursor.set_cursor(lines[hittest_position.line].start_offset);
        }
    }

    pub fn move_cursor_line_end(&self, cursor: &mut GraphemeCursor) {
        if let (Ok(lines), Ok(hittest_position)) = (
            self.text_style
                .lines(&mut piet_cairo::CairoText::new(), self.text.clone()),
            self.text_style.cursor_hittest_position(
                &mut piet_cairo::CairoText::new(),
                self.text.clone(),
                cursor,
            ),
        ) {
            let line_metric = &lines[hittest_position.line];
            let mut offset = line_metric.end_offset;

            // Move cursor in front of new line characters if they exist.
            if offset > line_metric.start_offset
                && (self.text.chars().nth(offset - 1) == Some('\n'))
            {
                offset -= 1;
            }

            if offset > line_metric.start_offset
                && (self.text.chars().nth(offset - 1) == Some('\r'))
            {
                offset -= 1;
            }

            cursor.set_cursor(offset);
        }
    }

    pub fn move_cursor_line_down(&self, cursor: &mut GraphemeCursor) {
        if let (Ok(text_layout), Ok(lines), Ok(hittest_position)) = (
            self.text_style
                .build_text_layout(&mut piet_cairo::CairoText::new(), self.text.clone()),
            self.text_style
                .lines(&mut piet_cairo::CairoText::new(), self.text.clone()),
            self.text_style.cursor_hittest_position(
                &mut piet_cairo::CairoText::new(),
                self.text.clone(),
                cursor,
            ),
        ) {
            let next_line = (hittest_position.line + 1).min(lines.len().saturating_sub(1));

            if next_line != hittest_position.line {
                // offset the cursor in the next line based on the hit of the x offset of the current cursor,
                // it matches intuition best when fonts are not monospace.
                let hit_test_point = text_layout.hit_test_point(kurbo::Point::new(
                    hittest_position.point.x,
                    lines[next_line].y_offset + lines[next_line].height * 0.5,
                ));

                cursor.set_cursor(hit_test_point.idx);
            }
        }
    }

    pub fn move_cursor_line_up(&self, cursor: &mut GraphemeCursor) {
        if let (Ok(text_layout), Ok(lines), Ok(hittest_position)) = (
            self.text_style
                .build_text_layout(&mut piet_cairo::CairoText::new(), self.text.clone()),
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
                let hit_test_point = text_layout.hit_test_point(kurbo::Point::new(
                    hittest_position.point.x,
                    lines[prev_line].y_offset + lines[prev_line].height * 0.5,
                ));

                cursor.set_cursor(hit_test_point.idx);
            }
        }
    }
}

fn get_intersecting_attrs_for_range(
    range: &Range<usize>,
    ranged_text_attributes: Vec<RangedTextAttribute>,
) -> (Vec<RangedTextAttribute>, Vec<RangedTextAttribute>) {
    ranged_text_attributes
        .into_iter()
        .partition(|attr| attr.range.end > range.start && attr.range.start < range.end)
}

fn remove_intersecting_attrs_in_range(
    range: &Range<usize>,
    intersecting_attrs: Vec<RangedTextAttribute>,
) -> Vec<RangedTextAttribute> {
    intersecting_attrs
        .into_iter()
        .flat_map(|mut attr| {
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
        .collect::<Vec<RangedTextAttribute>>()
}

fn create_wavy_line(origin: kurbo::Point, max_width: f64, scale: f64) -> BezPath {
    const WIDTH: f64 = 3.5;
    const HEIGHT: f64 = 4.0;

    if !max_width.is_finite() {
        return BezPath::new();
    }

    let width = WIDTH * scale;
    let half_height = (HEIGHT / 2.0) * scale;

    let mut path = BezPath::new();
    path.move_to(origin + (0.0, half_height));

    let mut x = 0.0;
    let mut direction = 1.0;

    while x < max_width {
        let center_point = origin + (x, half_height);

        let stationary_point = center_point + (width / 2.0, half_height * direction);
        let next_center_point = center_point + (width, 0.0);

        path.quad_to(stationary_point, next_center_point);

        x += width;
        direction = -direction;
    }

    path
}
