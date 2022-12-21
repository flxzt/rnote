use crate::{appwindow::RnoteAppWindow, ColorPicker};
use gtk4::pango;
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, Button, CompositeTemplate,
    EmojiChooser, FontChooserLevel, FontChooserWidget, Image, MenuButton, Popover, SpinButton,
    ToggleButton,
};
use rnote_engine::engine::EngineViewMut;
use rnote_engine::pens::Pen;
use rnote_engine::strokes::textstroke::{FontStyle, TextAlignment, TextAttribute};
use rnote_engine::{strokes::textstroke::TextStyle, utils::GdkRGBAHelpers};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/typewriterpage.ui")]
    pub(crate) struct TypewriterPage {
        #[template_child]
        pub(crate) fontchooser_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) fontchooser_buttonimage: TemplateChild<Image>,
        #[template_child]
        pub(crate) fontchooser_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) fontchooser: TemplateChild<FontChooserWidget>,
        #[template_child]
        pub(crate) fontchooser_cancelbutton: TemplateChild<Button>,
        #[template_child]
        pub(crate) fontchooser_selectbutton: TemplateChild<Button>,
        #[template_child]
        pub(crate) font_size_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) emojichooser: TemplateChild<EmojiChooser>,
        #[template_child]
        pub(crate) text_reset_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_bold_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_italic_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_underline_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) text_strikethrough_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub(crate) text_align_start_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_center_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_end_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_fill_togglebutton: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TypewriterPage {
        const NAME: &'static str = "TypewriterPage";
        type Type = super::TypewriterPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TypewriterPage {
        fn constructed(&self) {
            self.parent_constructed();

            // Sets the level of the font chooser (we want FAMILY, as we have separate widgets for weight, style, etc.)
            self.fontchooser.set_level(FontChooserLevel::FAMILY);
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for TypewriterPage {}
}

glib::wrapper! {
    pub(crate) struct TypewriterPage(ObjectSubclass<imp::TypewriterPage>)
        @extends gtk4::Widget;
}

impl Default for TypewriterPage {
    fn default() -> Self {
        Self::new()
    }
}

impl TypewriterPage {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn colorpicker(&self) -> ColorPicker {
        self.imp().colorpicker.get()
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        let fontchooser = imp.fontchooser.get();
        let fontchooser_popover = imp.fontchooser_popover.get();

        // Font chooser
        imp.fontchooser_cancelbutton.connect_clicked(
            clone!(@weak fontchooser, @weak fontchooser_popover => move |_fontchooser_cancelbutton| {
                fontchooser_popover.popdown();
            }),
        );

        imp.fontchooser_selectbutton.connect_clicked(
            clone!(@weak fontchooser, @weak fontchooser_popover => move |_fontchooser_selectbutton| {
                if let Some(font) = fontchooser.font() {
                    fontchooser.emit_by_name::<()>("font-activated", &[&font.to_value()]);
                }

                fontchooser_popover.popdown();
            }),
        );

        // Listening to connect_font_notify would always activate at app startup. font_activated only emits when the user interactively selects a font (with double click or Enter)
        // or we activate the signal manually elsewhere in the code
        imp.fontchooser.connect_font_activated(clone!(@weak fontchooser_popover, @weak appwindow => move |fontchooser, _font| {
            if let Some(font_family) = fontchooser.font_family().map(|font_family| font_family.name().to_string()) {
                {
                    let engine = appwindow.canvas().engine();
                    let engine = &mut *engine.borrow_mut();

                    engine.pens_config.typewriter_config.text_style.font_family = font_family.clone();

                    if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                        let widget_flags = typewriter.change_text_style_in_modifying_stroke(
                            |text_style| {
                                text_style.font_family = font_family;
                            },
                            &mut EngineViewMut {
                                tasks_tx: engine.tasks_tx.clone(),
                                pens_config: &mut engine.pens_config,
                                doc: &mut engine.document,
                                store: &mut engine.store,
                                camera: &mut engine.camera,
                                audioplayer: &mut engine.audioplayer
                        });
                        appwindow.handle_widget_flags(widget_flags);
                    }
                }

                fontchooser_popover.popdown();
            }
        }));

        // Font size
        imp.font_size_spinbutton.set_increments(1.0, 5.0);
        imp.font_size_spinbutton
            .set_range(TextStyle::FONT_SIZE_MIN, TextStyle::FONT_SIZE_MAX);
        imp.font_size_spinbutton
            .set_value(TextStyle::FONT_SIZE_DEFAULT);

        imp.font_size_spinbutton.connect_value_changed(
            clone!(@weak appwindow => move |font_size_spinbutton| {
                let font_size = font_size_spinbutton.value();

                {
                    let engine = appwindow.canvas().engine();
                    let engine = &mut *engine.borrow_mut();

                    engine.pens_config.typewriter_config.text_style.font_size = font_size;

                    if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                        let widget_flags = typewriter.change_text_style_in_modifying_stroke(
                            |text_style| {
                                text_style.font_size = font_size;
                            },
                            &mut EngineViewMut {
                                tasks_tx: engine.tasks_tx.clone(),
                                pens_config: &mut engine.pens_config,
                                doc: &mut engine.document,
                                store: &mut engine.store,
                                camera: &mut engine.camera,
                                audioplayer: &mut engine.audioplayer
                        });
                        appwindow.handle_widget_flags(widget_flags);
                    }
                }
            }),
        );

        // Update the font chooser font size, to display the preview text in the correct size
        imp.font_size_spinbutton
            .bind_property("value", &fontchooser, "font-desc")
            .transform_to(|binding, val: f64| {
                let fontchooser = binding
                    .target()
                    .unwrap()
                    .downcast::<FontChooserWidget>()
                    .unwrap();
                let mut font_desc = fontchooser.font_desc()?;

                font_desc.set_size((val * f64::from(pango::SCALE)).round() as i32);

                Some(font_desc.to_value())
            })
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        // Color
        imp.colorpicker.connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |colorpicker, _paramspec| {
                let color = colorpicker.property::<gdk::RGBA>("current-color").into_compose_color();

                {
                    let engine = appwindow.canvas().engine();
                    let engine = &mut *engine.borrow_mut();

                    engine.pens_config.typewriter_config.text_style.color = color;

                    if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                        let widget_flags = typewriter.change_text_style_in_modifying_stroke(
                            |text_style| {
                                text_style.color = color;
                            },
                            &mut EngineViewMut {
                                tasks_tx: engine.tasks_tx.clone(),
                                pens_config: &mut engine.pens_config,
                                doc: &mut engine.document,
                                store: &mut engine.store,
                                camera: &mut engine.camera,
                                audioplayer: &mut engine.audioplayer
                        });
                        appwindow.handle_widget_flags(widget_flags);
                    }
                }
            }),
        );

        // Emojis
        imp.emojichooser.connect_emoji_picked(
            clone!(@weak appwindow => move |_emojichooser, emoji_str| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                    let widget_flags = typewriter.insert_text(
                        emoji_str.to_string(),
                        None,
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx.clone(),
                            pens_config: &mut engine.pens_config,
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }

            }),
        );

        // reset
        imp.text_reset_button.connect_clicked(
            clone!(@weak appwindow => move |_text_reset_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                    let widget_flags = typewriter.remove_text_attributes_current_selection(
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx.clone(),
                            pens_config: &mut engine.pens_config,
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }
            }),
        );

        // Bold
        imp.text_bold_button
            .connect_clicked(clone!(@weak appwindow => move |_text_bold_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                    let widget_flags = typewriter.add_text_attribute_current_selection(
                        TextAttribute::FontWeight(piet::FontWeight::BOLD.to_raw()),
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx.clone(),
                            pens_config: &mut engine.pens_config,
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }
            }));

        // Italic
        imp.text_italic_button.connect_clicked(
            clone!(@weak appwindow => move |_text_italic_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                    let widget_flags = typewriter.add_text_attribute_current_selection(
                        TextAttribute::Style(FontStyle::Italic),
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx.clone(),
                            pens_config: &mut engine.pens_config,
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }
            }),
        );

        // Underline
        imp.text_underline_button.connect_clicked(
            clone!(@weak appwindow => move |_text_underline_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                    let widget_flags = typewriter.add_text_attribute_current_selection(
                        TextAttribute::Underline(true),
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx.clone(),
                            pens_config: &mut engine.pens_config,
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }
            }),
        );

        // Strikethrough
        imp.text_strikethrough_button.connect_clicked(
            clone!(@weak appwindow => move |_text_strikethrough_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                    let widget_flags = typewriter.add_text_attribute_current_selection(
                        TextAttribute::Strikethrough(true),
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx.clone(),
                            pens_config: &mut engine.pens_config,
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }
            }),
        );

        // Alignment
        imp.text_align_start_togglebutton.connect_active_notify(
            clone!(@weak appwindow => move |text_align_start_togglebutton| {
                if text_align_start_togglebutton.is_active() {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();
                        engine.pens_config.typewriter_config.text_style.alignment = TextAlignment::Start;

                        if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                            let widget_flags = typewriter.change_text_style_in_modifying_stroke(
                                |text_style| {
                                    text_style.alignment = TextAlignment::Start;
                                },
                                &mut EngineViewMut {
                                    tasks_tx: engine.tasks_tx.clone(),
                                    pens_config: &mut engine.pens_config,
                                    doc: &mut engine.document,
                                    store: &mut engine.store,
                                    camera: &mut engine.camera,
                                    audioplayer: &mut engine.audioplayer
                            });
                            appwindow.handle_widget_flags(widget_flags);
                        }
                    }
                }

            }),
        );

        imp.text_align_center_togglebutton.connect_active_notify(
            clone!(@weak appwindow => move |text_align_center_togglebutton| {
                if text_align_center_togglebutton.is_active() {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();
                        engine.pens_config.typewriter_config.text_style.alignment = TextAlignment::Center;

                        if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                            let widget_flags = typewriter.change_text_style_in_modifying_stroke(
                                |text_style| {
                                    text_style.alignment = TextAlignment::Center;
                                },
                                &mut EngineViewMut {
                                    tasks_tx: engine.tasks_tx.clone(),
                                    pens_config: &mut engine.pens_config,
                                    doc: &mut engine.document,
                                    store: &mut engine.store,
                                    camera: &mut engine.camera,
                                    audioplayer: &mut engine.audioplayer
                            });
                            appwindow.handle_widget_flags(widget_flags);
                        }
                    }
                }
            }),
        );

        imp.text_align_end_togglebutton.connect_active_notify(
            clone!(@weak appwindow => move |text_align_end_togglebutton| {
                if text_align_end_togglebutton.is_active() {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();
                        engine.pens_config.typewriter_config.text_style.alignment = TextAlignment::End;

                        if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                            let widget_flags = typewriter.change_text_style_in_modifying_stroke(
                                |text_style| {
                                    text_style.alignment = TextAlignment::End;
                                },
                                &mut EngineViewMut {
                                    tasks_tx: engine.tasks_tx.clone(),
                                    pens_config: &mut engine.pens_config,
                                    doc: &mut engine.document,
                                    store: &mut engine.store,
                                    camera: &mut engine.camera,
                                    audioplayer: &mut engine.audioplayer
                            });
                            appwindow.handle_widget_flags(widget_flags);
                        }
                    }
                }
            }),
        );

        imp.text_align_fill_togglebutton.connect_active_notify(
            clone!(@weak appwindow => move |text_align_fill_togglebutton| {
                if text_align_fill_togglebutton.is_active() {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();
                        engine.pens_config.typewriter_config.text_style.alignment = TextAlignment::Fill;

                        if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                            let widget_flags = typewriter.change_text_style_in_modifying_stroke(
                                |text_style| {
                                    text_style.alignment = TextAlignment::Fill;
                                },
                                &mut EngineViewMut {
                                    tasks_tx: engine.tasks_tx.clone(),
                                    pens_config: &mut engine.pens_config,
                                    doc: &mut engine.document,
                                    store: &mut engine.store,
                                    camera: &mut engine.camera,
                                    audioplayer: &mut engine.audioplayer
                            });
                            appwindow.handle_widget_flags(widget_flags);
                        }
                    }
                }
            }),
        );
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        let typewriter_config = appwindow
            .canvas()
            .engine()
            .borrow()
            .pens_config
            .typewriter_config
            .clone();

        imp.fontchooser
            .set_font_desc(&typewriter_config.text_style.extract_pango_font_desc());
        imp.font_size_spinbutton
            .set_value(typewriter_config.text_style.font_size);
        imp.colorpicker
            .set_current_color(gdk::RGBA::from_compose_color(
                typewriter_config.text_style.color,
            ));

        match typewriter_config.text_style.alignment {
            TextAlignment::Start => imp.text_align_start_togglebutton.set_active(true),
            TextAlignment::Center => imp.text_align_center_togglebutton.set_active(true),
            TextAlignment::End => imp.text_align_end_togglebutton.set_active(true),
            TextAlignment::Fill => imp.text_align_fill_togglebutton.set_active(true),
        }
    }
}
