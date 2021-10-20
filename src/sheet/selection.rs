mod imp {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use crate::strokes;

    use gtk4::{glib, glib::subclass::Signal, prelude::*, subclass::prelude::*};
    use once_cell::sync::Lazy;

    #[derive(Debug)]
    pub struct Selection {
        pub strokes: Rc<RefCell<Vec<strokes::StrokeStyle>>>,
        pub bounds: Cell<Option<p2d::bounding_volume::AABB>>,
        pub shown: Cell<bool>,
    }

    impl Default for Selection {
        fn default() -> Self {
            Self {
                strokes: Rc::new(RefCell::new(Vec::new())),
                bounds: Cell::new(None),
                shown: Cell::new(false),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Selection {
        const NAME: &'static str = "Selection";
        type Type = super::Selection;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Selection {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    // Signal name
                    "redraw",
                    // Types of the values which will be sent to the signal handler
                    &[],
                    // Type of the value the signal handler sends back
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_boolean(
                    // Name
                    "shown",
                    // Nickname
                    "shown",
                    // Short description
                    "shown",
                    // Default value
                    false,
                    // The property can be read and written to
                    glib::ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "shown" => self.shown.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "shown" => {
                    let shown: bool = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`.");
                    self.shown.replace(shown);
                }
                _ => unimplemented!(),
            }
        }
    }
}

use std::{cell::RefCell, error::Error, rc::Rc};

use crate::{
    pens::selector::Selector,
    strokes::{self, compose, StrokeBehaviour, StrokeStyle},
    ui::appwindow::RnoteAppWindow,
};
use gtk4::{
    gdk, gio, glib, glib::clone, graphene, gsk, prelude::*, subclass::prelude::*, Snapshot,
};

use p2d::bounding_volume::BoundingVolume;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

glib::wrapper! {
    pub struct Selection(ObjectSubclass<imp::Selection>);
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for Selection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Sheet", 2)?;
        state.serialize_field("strokes", &*self.strokes().borrow())?;
        state.serialize_field("bounds", &self.bounds())?;
        state.serialize_field("shown", &self.shown())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Selection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        #[allow(non_camel_case_types)]
        enum Field {
            strokes,
            bounds,
            shown,
            unknown,
        }

        struct SelectionVisitor;
        impl<'de> Visitor<'de> for SelectionVisitor {
            type Value = Selection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Selection")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let strokes = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let bounds = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let shown = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let selection = Selection::new();
                *selection.strokes().borrow_mut() = strokes;
                selection.set_bounds(bounds);
                selection.set_shown(shown);
                Ok(selection)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut strokes = None;
                let mut bounds = None;
                let mut shown = None;
                while let Some(key) = match map.next_key() {
                    Ok(key) => key,
                    Err(e) => {
                        log::warn!("{}", e);
                        Some(Field::unknown)
                    }
                } {
                    match key {
                        Field::strokes => {
                            if strokes.is_some() {
                                return Err(de::Error::duplicate_field("strokes"));
                            }
                            strokes = Some(map.next_value()?);
                        }
                        Field::bounds => {
                            if bounds.is_some() {
                                return Err(de::Error::duplicate_field("bounds"));
                            }
                            bounds = Some(map.next_value()?);
                        }
                        Field::shown => {
                            if shown.is_some() {
                                return Err(de::Error::duplicate_field("shown"));
                            }
                            shown = Some(map.next_value()?);
                        }
                        Field::unknown => {
                            // throw away the value
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let selection_default = Selection::default();

                let strokes = strokes.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("strokes");
                    log::error!("{}", err);
                    Vec::new()
                });
                let bounds = bounds.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("bounds");
                    log::error!("{}", err);
                    selection_default.bounds()
                });
                let shown = shown.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("shown");
                    log::error!("{}", err);
                    selection_default.shown()
                });

                let selection = Selection::new();
                *selection.strokes().borrow_mut() = strokes;
                selection.set_bounds(bounds);
                selection.set_shown(shown);
                Ok(selection)
            }
        }

        const FIELDS: &[&str] = &["strokes", "bounds", "shown"];
        deserializer.deserialize_struct("Selection", FIELDS, SelectionVisitor)
    }
}

impl Selection {
    pub fn new() -> Self {
        let selection: Selection = glib::Object::new(&[]).expect("Failed to create Selection");
        selection
    }

    pub fn strokes(&self) -> Rc<RefCell<Vec<strokes::StrokeStyle>>> {
        let priv_ = imp::Selection::from_instance(self);

        priv_.strokes.clone()
    }

    pub fn bounds(&self) -> Option<p2d::bounding_volume::AABB> {
        imp::Selection::from_instance(self).bounds.get()
    }

    pub fn set_bounds(&self, bounds: Option<p2d::bounding_volume::AABB>) {
        imp::Selection::from_instance(self).bounds.set(bounds);
    }

    pub fn shown(&self) -> bool {
        self.property("shown").unwrap().get::<bool>().unwrap()
    }

    pub fn set_shown(&self, shown: bool) {
        self.set_property("shown", shown.to_value()).unwrap()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.connect_local(
            "redraw",
            false,
            clone!(@weak self as selection, @weak appwindow => @default-return None, move |_args| {
                    StrokeStyle::update_all_rendernodes(
                        &mut *selection.strokes().borrow_mut(),
                        appwindow.canvas().scalefactor(),
                        &*appwindow.canvas().renderer().borrow(),
                    );

                    appwindow.selection_modifier().queue_resize();
                    appwindow.selection_modifier().queue_draw();

                    appwindow.canvas().queue_resize();
                    appwindow.canvas().queue_draw();
                    None
            }),
        )
        .unwrap();
    }

    pub fn update_selection(
        &self,
        selector: &Selector,
        other_strokes: &mut Vec<StrokeStyle>,
        viewport: Option<p2d::bounding_volume::AABB>,
    ) {
        let mut to_remove_from_strokes = Vec::<usize>::new();
        let mut to_remove_from_selection = Vec::<usize>::new();

        let selector_bounds = if let Some(selector_bounds) = selector.bounds {
            selector_bounds
        } else {
            return;
        };

        let mut path_bounds = p2d::bounding_volume::AABB::new_invalid();

        for inputdata in selector.path.iter() {
            path_bounds.take_point(na::Point2::from(inputdata.pos()));
        }

        // remove from selection, add to other strokes
        for (i, stroke) in self.strokes().borrow().iter().enumerate() {
            // skip if stroke is not in viewport
            if let Some(viewport) = viewport {
                if !viewport.intersects(&stroke.bounds()) {
                    continue;
                }
            }
            match stroke {
                strokes::StrokeStyle::MarkerStroke(markerstroke) => {
                    if selector_bounds.contains(&markerstroke.bounds) {
                        other_strokes.push(self.strokes().borrow()[i].clone());
                        to_remove_from_selection.push(i);
                    } else if selector_bounds.intersects(&markerstroke.bounds) {
                        let mut contains_all = true;
                        'selection_markerstroke_check: for hitbox_elem in markerstroke.hitbox.iter()
                        {
                            if !path_bounds.contains(hitbox_elem) {
                                contains_all = false;
                                break 'selection_markerstroke_check;
                            }
                        }

                        if contains_all {
                            other_strokes.push(self.strokes().borrow()[i].clone());
                            to_remove_from_selection.push(i);
                        }
                    }
                }
                strokes::StrokeStyle::BrushStroke(brushstroke) => {
                    if selector_bounds.contains(&brushstroke.bounds) {
                        other_strokes.push(self.strokes().borrow()[i].clone());
                        to_remove_from_selection.push(i);
                    } else if selector_bounds.intersects(&brushstroke.bounds) {
                        let mut contains_all = true;
                        'selection_brushstroke_check: for hitbox_elem in brushstroke.hitbox.iter() {
                            if !path_bounds.contains(hitbox_elem) {
                                contains_all = false;
                                break 'selection_brushstroke_check;
                            }
                        }

                        if contains_all {
                            other_strokes.push(self.strokes().borrow()[i].clone());
                            to_remove_from_selection.push(i);
                        }
                    }
                }
                strokes::StrokeStyle::ShapeStroke(shapestroke) => {
                    if path_bounds.contains(&shapestroke.bounds) {
                        other_strokes.push(self.strokes().borrow()[i].clone());
                        to_remove_from_selection.push(i);
                    }
                }
                strokes::StrokeStyle::VectorImage(vector_image) => {
                    if path_bounds.contains(&vector_image.bounds) {
                        other_strokes.push(self.strokes().borrow()[i].clone());
                        to_remove_from_selection.push(i);
                    }
                }
                strokes::StrokeStyle::BitmapImage(vector_image) => {
                    if !path_bounds.contains(&vector_image.bounds) {
                        other_strokes.push(self.strokes().borrow()[i].clone());
                        to_remove_from_selection.push(i);
                    }
                }
            }
        }
        for (to_remove_index, i) in to_remove_from_selection.iter().enumerate() {
            self.remove_stroke(i - to_remove_index);
        }

        // remove from other strokes, add to selection
        for (i, stroke) in other_strokes.iter().enumerate() {
            // skip if stroke is not in viewport
            if let Some(viewport) = viewport {
                if !viewport.intersects(&stroke.bounds()) {
                    continue;
                }
            }
            match stroke {
                strokes::StrokeStyle::MarkerStroke(markerstroke) => {
                    if selector_bounds.contains(&markerstroke.bounds) {
                        self.push_to_selection(other_strokes[i].clone());
                        to_remove_from_strokes.push(i);
                    } else if selector_bounds.intersects(&markerstroke.bounds) {
                        let mut contains_all = true;
                        'strokes_markerstroke_check: for hitbox_elem in markerstroke.hitbox.iter() {
                            if !path_bounds.contains(hitbox_elem) {
                                contains_all = false;
                                break 'strokes_markerstroke_check;
                            }
                        }

                        if contains_all {
                            self.push_to_selection(other_strokes[i].clone());
                            to_remove_from_strokes.push(i);
                        }
                    }
                }
                strokes::StrokeStyle::BrushStroke(brushstroke) => {
                    if selector_bounds.contains(&brushstroke.bounds) {
                        self.push_to_selection(other_strokes[i].clone());
                        to_remove_from_strokes.push(i);
                    } else if selector_bounds.intersects(&brushstroke.bounds) {
                        let mut contains_all = true;
                        'strokes_brushstroke_check: for hitbox_elem in brushstroke.hitbox.iter() {
                            if !path_bounds.contains(hitbox_elem) {
                                contains_all = false;
                                break 'strokes_brushstroke_check;
                            }
                        }

                        if contains_all {
                            self.push_to_selection(other_strokes[i].clone());
                            to_remove_from_strokes.push(i);
                        }
                    }
                }
                strokes::StrokeStyle::ShapeStroke(shapestroke) => {
                    if path_bounds.contains(&shapestroke.bounds) {
                        self.push_to_selection(other_strokes[i].clone());
                        to_remove_from_strokes.push(i);
                    }
                }
                strokes::StrokeStyle::VectorImage(vectorimage) => {
                    if path_bounds.contains(&vectorimage.bounds) {
                        self.push_to_selection(other_strokes[i].clone());
                        to_remove_from_strokes.push(i);
                    }
                }
                strokes::StrokeStyle::BitmapImage(bitmapimage) => {
                    if path_bounds.contains(&bitmapimage.bounds) {
                        self.push_to_selection(other_strokes[i].clone());
                        to_remove_from_strokes.push(i);
                    }
                }
            }
        }
        for (to_remove_index, i) in to_remove_from_strokes.iter().enumerate() {
            other_strokes.remove(i - to_remove_index);
        }

        self.set_bounds(StrokeStyle::gen_bounds(&self.strokes().borrow()));
        self.emit_by_name("redraw", &[]).unwrap();
        self.set_shown(!self.strokes().borrow().is_empty())
    }

    pub fn push_to_selection(&self, stroke: strokes::StrokeStyle) {
        let priv_ = imp::Selection::from_instance(self);

        priv_.strokes.borrow_mut().push(stroke);

        self.set_bounds(StrokeStyle::gen_bounds(&self.strokes().borrow()));
        self.emit_by_name("redraw", &[]).unwrap();
        self.set_shown(true);
    }

    pub fn pop_from_selection(&self) -> Option<strokes::StrokeStyle> {
        let priv_ = imp::Selection::from_instance(self);

        let stroke = priv_.strokes.borrow_mut().pop();

        if self.strokes().borrow().is_empty() {
            self.set_shown(false);
        }

        self.set_bounds(StrokeStyle::gen_bounds(&self.strokes().borrow()));
        self.emit_by_name("redraw", &[]).unwrap();
        stroke
    }

    pub fn remove_stroke(&self, index: usize) -> Option<strokes::StrokeStyle> {
        let priv_ = imp::Selection::from_instance(self);

        let stroke = if index < priv_.strokes.borrow().len() {
            Some(priv_.strokes.borrow_mut().remove(index))
        } else {
            None
        };

        if self.strokes().borrow().is_empty() {
            self.set_shown(false);
        } else {
            self.set_shown(true);
        }

        self.set_bounds(StrokeStyle::gen_bounds(&self.strokes().borrow()));
        self.emit_by_name("redraw", &[]).unwrap();
        stroke
    }

    pub fn remove_strokes(&self) -> Vec<strokes::StrokeStyle> {
        let priv_ = imp::Selection::from_instance(self);

        let selected = priv_.strokes.borrow().clone();
        *priv_.strokes.borrow_mut() = Vec::new();

        self.set_shown(false);

        self.set_bounds(StrokeStyle::gen_bounds(&self.strokes().borrow()));
        self.emit_by_name("redraw", &[]).unwrap();
        selected
    }

    pub fn resize_selection(&self, new_bounds: p2d::bounding_volume::AABB) {
        let priv_ = imp::Selection::from_instance(self);

        if let Some(selection_bounds) = self.bounds() {
            let new_selected: Vec<strokes::StrokeStyle> = self
                .strokes()
                .borrow_mut()
                .iter_mut()
                .map(|stroke| {
                    stroke.resize(Self::calc_new_stroke_bounds(
                        stroke,
                        selection_bounds,
                        new_bounds,
                    ));
                    stroke.clone()
                })
                .collect();

            *priv_.strokes.borrow_mut() = new_selected;

            self.set_bounds(Some(new_bounds));
            self.emit_by_name("redraw", &[]).unwrap();
        }
    }

    pub fn calc_new_stroke_bounds(
        stroke: &StrokeStyle,
        selection_bounds: p2d::bounding_volume::AABB,
        new_bounds: p2d::bounding_volume::AABB,
    ) -> p2d::bounding_volume::AABB {
        let offset = na::vector![
            new_bounds.mins[0] - selection_bounds.mins[0],
            new_bounds.mins[1] - selection_bounds.mins[1]
        ];

        let scalevector = na::vector![
            (new_bounds.maxs[0] - new_bounds.mins[0])
                / (selection_bounds.maxs[0] - selection_bounds.mins[0]),
            (new_bounds.maxs[1] - new_bounds.mins[1])
                / (selection_bounds.maxs[1] - selection_bounds.mins[1])
        ];

        p2d::bounding_volume::AABB::new(
            na::point![
                (stroke.bounds().mins[0] - selection_bounds.mins[0]) * scalevector[0]
                    + selection_bounds.mins[0]
                    + offset[0],
                (stroke.bounds().mins[1] - selection_bounds.mins[1]) * scalevector[1]
                    + selection_bounds.mins[1]
                    + offset[1]
            ],
            na::point![
                (stroke.bounds().mins[0] - selection_bounds.mins[0]) * scalevector[0]
                    + selection_bounds.mins[0]
                    + offset[0]
                    + (stroke.bounds().maxs[0] - stroke.bounds().mins[0]) * scalevector[0],
                (stroke.bounds().mins[1] - selection_bounds.mins[1]) * scalevector[1]
                    + selection_bounds.mins[1]
                    + offset[1]
                    + (stroke.bounds().maxs[1] - stroke.bounds().mins[1]) * scalevector[1]
            ],
        )
    }

    pub fn translate_selection(&self, offset: na::Vector2<f64>) {
        let priv_ = imp::Selection::from_instance(self);

        let new_selected: Vec<strokes::StrokeStyle> = self
            .strokes()
            .borrow_mut()
            .iter_mut()
            .map(|stroke| {
                stroke.translate(offset);
                stroke.clone()
            })
            .collect();

        *priv_.strokes.borrow_mut() = new_selected;

        self.set_bounds(StrokeStyle::gen_bounds(&self.strokes().borrow()));
        self.emit_by_name("redraw", &[]).unwrap();
    }

    pub fn draw(&self, scalefactor: f64, snapshot: &Snapshot) {
        let priv_ = imp::Selection::from_instance(self);

        StrokeStyle::draw_strokes(&priv_.strokes.borrow(), snapshot);

        for stroke in priv_.strokes.borrow().iter() {
            match stroke {
                strokes::StrokeStyle::MarkerStroke(markerstroke) => {
                    self.draw_selected_bounds(markerstroke.bounds, scalefactor, snapshot);
                }
                strokes::StrokeStyle::BrushStroke(brushstroke) => {
                    self.draw_selected_bounds(brushstroke.bounds, scalefactor, snapshot);
                }
                strokes::StrokeStyle::ShapeStroke(shapestroke) => {
                    self.draw_selected_bounds(shapestroke.bounds, scalefactor, snapshot);
                }
                strokes::StrokeStyle::VectorImage(vector_image) => {
                    self.draw_selected_bounds(vector_image.bounds, scalefactor, snapshot);
                }
                strokes::StrokeStyle::BitmapImage(bitmapimage) => {
                    self.draw_selected_bounds(bitmapimage.bounds, scalefactor, snapshot);
                }
            }
        }

        self.draw_selection_bounds(scalefactor, snapshot);
    }

    pub fn draw_selection_bounds(&self, scalefactor: f64, snapshot: &Snapshot) {
        if let Some(bounds) = self.bounds() {
            let selection_bounds = graphene::Rect::new(
                bounds.mins[0] as f32,
                bounds.mins[1] as f32,
                (bounds.maxs[0] - bounds.mins[0]) as f32,
                (bounds.maxs[1] - bounds.mins[1]) as f32,
            )
            .scale(scalefactor as f32, scalefactor as f32);

            let selection_border_color = gdk::RGBA {
                red: 0.49,
                green: 0.56,
                blue: 0.63,
                alpha: 0.3,
            };
            let selection_border_width = 4.0;

            snapshot.append_color(
                &gdk::RGBA {
                    red: 0.49,
                    green: 0.56,
                    blue: 0.63,
                    alpha: 0.1,
                },
                &selection_bounds,
            );
            snapshot.append_border(
                &gsk::RoundedRect::new(
                    graphene::Rect::new(
                        selection_bounds.x(),
                        selection_bounds.y(),
                        selection_bounds.width(),
                        selection_bounds.height(),
                    ),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                ),
                &[
                    selection_border_width,
                    selection_border_width,
                    selection_border_width,
                    selection_border_width,
                ],
                &[
                    selection_border_color,
                    selection_border_color,
                    selection_border_color,
                    selection_border_color,
                ],
            );
        }
    }

    pub fn draw_selected_bounds(
        &self,
        bounds: p2d::bounding_volume::AABB,
        scalefactor: f64,
        snapshot: &Snapshot,
    ) {
        let bounds = graphene::Rect::new(
            bounds.mins[0] as f32,
            bounds.mins[1] as f32,
            (bounds.maxs[0] - bounds.mins[0]) as f32,
            (bounds.maxs[1] - bounds.mins[1]) as f32,
        )
        .scale(scalefactor as f32, scalefactor as f32);
        let border_color = gdk::RGBA {
            red: 0.0,
            green: 0.2,
            blue: 0.8,
            alpha: 0.4,
        };
        let border_width = 2.0;

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

    pub fn export_selection_as_svg(&self, file: gio::File) -> Result<(), Box<dyn Error>> {
        let priv_ = imp::Selection::from_instance(self);

        if let Some(bounds) = self.bounds() {
            let mut data = String::new();
            for stroke in &*priv_.strokes.borrow() {
                let data_entry =
                    stroke.gen_svg_data(na::vector![-bounds.mins[0], -bounds.mins[1]])?;

                data.push_str(&data_entry);
            }

            let wrapper_bounds = p2d::bounding_volume::AABB::new(
                na::point![0.0, 0.0],
                na::point![
                    bounds.maxs[0] - bounds.mins[0],
                    bounds.maxs[1] - bounds.mins[1]
                ],
            );
            data = compose::wrap_svg(
                data.as_str(),
                Some(wrapper_bounds),
                Some(wrapper_bounds),
                true,
                false,
            );

            let output_stream = file.replace::<gio::Cancellable>(
                None,
                false,
                gio::FileCreateFlags::REPLACE_DESTINATION,
                None,
            )?;
            output_stream.write::<gio::Cancellable>(data.as_bytes(), None)?;
            output_stream.close::<gio::Cancellable>(None)?;
        }

        Ok(())
    }
}
