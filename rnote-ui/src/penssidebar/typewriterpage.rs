use crate::{appwindow::RnoteAppWindow, ColorPicker};
use gtk4::pango;
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, Button, CompositeTemplate,
    EmojiChooser, FontChooserLevel, FontChooserWidget, Image, MenuButton, Popover, SpinButton,
    ToggleButton,
};
use rnote_engine::engine::EngineViewMut;
use rnote_engine::strokes::textstroke::{FontStyle, TextAlignment, TextAttribute};
use rnote_engine::{strokes::textstroke::TextStyle, utils::GdkRGBAHelpers};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/typewriterpage.ui")]
    pub struct TypewriterPage {
        #[template_child]
        pub fontchooser_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub fontchooser_buttonimage: TemplateChild<Image>,
        #[template_child]
        pub fontchooser_popover: TemplateChild<Popover>,
        #[template_child]
        pub fontchooser: TemplateChild<FontChooserWidget>,
        #[template_child]
        pub fontchooser_cancelbutton: TemplateChild<Button>,
        #[template_child]
        pub fontchooser_selectbutton: TemplateChild<Button>,
        #[template_child]
        pub font_size_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub emojichooser: TemplateChild<EmojiChooser>,
        #[template_child]
        pub text_reset_button: TemplateChild<Button>,
        #[template_child]
        pub text_bold_button: TemplateChild<Button>,
        #[template_child]
        pub text_italic_button: TemplateChild<Button>,
        #[template_child]
        pub text_underline_button: TemplateChild<Button>,
        #[template_child]
        pub text_strikethrough_button: TemplateChild<Button>,
        #[template_child]
        pub colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub text_align_start_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub text_align_center_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub text_align_end_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub text_align_fill_togglebutton: TemplateChild<ToggleButton>,
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
    pub struct TypewriterPage(ObjectSubclass<imp::TypewriterPage>)
        @extends gtk4::Widget;
}

impl Default for TypewriterPage {
    fn default() -> Self {
        Self::new()
    }
}

impl TypewriterPage {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn fontchooser_menubutton(&self) -> MenuButton {
        self.imp().fontchooser_menubutton.get()
    }

    pub fn fontchooser(&self) -> FontChooserWidget {
        self.imp().fontchooser.get()
    }

    pub fn colorpicker(&self) -> ColorPicker {
        self.imp().colorpicker.get()
    }

    pub fn font_size_spinbutton(&self) -> SpinButton {
        self.imp().font_size_spinbutton.get()
    }

    pub fn text_align_start_togglebutton(&self) -> ToggleButton {
        self.imp().text_align_start_togglebutton.get()
    }

    pub fn text_align_center_togglebutton(&self) -> ToggleButton {
        self.imp().text_align_center_togglebutton.get()
    }

    pub fn text_align_end_togglebutton(&self) -> ToggleButton {
        self.imp().text_align_end_togglebutton.get()
    }

    pub fn text_align_fill_togglebutton(&self) -> ToggleButton {
        self.imp().text_align_fill_togglebutton.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let fontchooser = self.imp().fontchooser.get();
        let fontchooser_popover = self.imp().fontchooser_popover.get();

        // Font chooser
        self.imp().fontchooser_cancelbutton.connect_clicked(
            clone!(@weak fontchooser, @weak fontchooser_popover => move |_fontchooser_cancelbutton| {
                fontchooser_popover.popdown();
            }),
        );

        self.imp().fontchooser_selectbutton.connect_clicked(
            clone!(@weak fontchooser, @weak fontchooser_popover => move |_fontchooser_selectbutton| {
                if let Some(font) = fontchooser.font() {
                    fontchooser.emit_by_name::<()>("font-activated", &[&font.to_value()]);
                }

                fontchooser_popover.popdown();
            }),
        );

        // Listening to connect_font_notify would always activate at app startup. font_activated only emits when the user interactively selects a font (with double click or Enter)
        // or we activate the signal manually elsewhere in the code
        self.fontchooser().connect_font_activated(clone!(@weak fontchooser_popover, @weak appwindow => move |fontchooser, _font| {
            if let Some(font_family) = fontchooser.font_family().map(|font_family| font_family.name().to_string()) {
                {
                    let engine = appwindow.canvas().engine();
                    let engine = &mut *engine.borrow_mut();

                    engine.penholder.typewriter.text_style.font_family = font_family.clone();

                    let widget_flags = engine.penholder.typewriter.change_text_style_in_modifying_stroke(
                        |text_style| {
                            text_style.font_family = font_family;
                        },
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx(),
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing typewriter font, Err `{}`", e);
                }

                fontchooser_popover.popdown();
            }
        }));

        // Font size
        self.font_size_spinbutton().set_increments(1.0, 5.0);
        self.font_size_spinbutton()
            .set_range(TextStyle::FONT_SIZE_MIN, TextStyle::FONT_SIZE_MAX);
        self.font_size_spinbutton()
            .set_value(TextStyle::FONT_SIZE_DEFAULT);

        self.font_size_spinbutton().connect_value_changed(
            clone!(@weak appwindow => move |font_size_spinbutton| {
                let font_size = font_size_spinbutton.value();

                {
                    let engine = appwindow.canvas().engine();
                    let engine = &mut *engine.borrow_mut();

                    engine.penholder.typewriter.text_style.font_size = font_size;

                    let widget_flags = engine.penholder.typewriter.change_text_style_in_modifying_stroke(
                        |text_style| {
                            text_style.font_size = font_size;
                        },
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx(),
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing typewriter font size, Err `{}`", e);
                }
            }),
        );

        // Update the font chooser font size, to display the preview text in the correct size
        self.font_size_spinbutton()
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
        self.colorpicker().connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |colorpicker, _paramspec| {
                let color = colorpicker.property::<gdk::RGBA>("current-color").into_compose_color();

                {
                    let engine = appwindow.canvas().engine();
                    let engine = &mut *engine.borrow_mut();

                    engine.penholder.typewriter.text_style.color = color;

                    let widget_flags = engine.penholder.typewriter.change_text_style_in_modifying_stroke(
                        |text_style| {
                            text_style.color = color;
                        },
                        &mut EngineViewMut {
                            tasks_tx: engine.tasks_tx(),
                            doc: &mut engine.document,
                            store: &mut engine.store,
                            camera: &mut engine.camera,
                            audioplayer: &mut engine.audioplayer
                    });
                    appwindow.handle_widget_flags(widget_flags);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing typewriter color, Err `{}`", e);
                }
            }),
        );

        // Emojis
        self.imp().emojichooser.connect_emoji_picked(
            clone!(@weak appwindow => move |_emojichooser, emoji_str| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                let widget_flags = engine.penholder.typewriter.insert_text(
                    emoji_str.to_string(),
                    None,
                    &mut EngineViewMut {
                        tasks_tx: engine.tasks_tx(),
                        doc: &mut engine.document,
                        store: &mut engine.store,
                        camera: &mut engine.camera,
                        audioplayer: &mut engine.audioplayer
                });
                appwindow.handle_widget_flags(widget_flags);
            }),
        );

        // reset
        self.imp().text_reset_button.connect_clicked(clone!(@weak appwindow => move |_text_reset_button| {
            let engine = appwindow.canvas().engine();
            let engine = &mut *engine.borrow_mut();

            let widget_flags = engine.penholder.typewriter.remove_text_attributes_current_selection(
                &mut EngineViewMut {
                    tasks_tx: engine.tasks_tx(),
                    doc: &mut engine.document,
                    store: &mut engine.store,
                    camera: &mut engine.camera,
                    audioplayer: &mut engine.audioplayer
            });
            appwindow.handle_widget_flags(widget_flags);
        }));

        // Bold
        self.imp().text_bold_button.connect_clicked(
            clone!(@weak appwindow => move |_text_bold_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                let widget_flags = engine.penholder.typewriter.add_text_attribute_current_selection(
                    TextAttribute::FontWeight(piet::FontWeight::BOLD.to_raw()),
                    &mut EngineViewMut {
                        tasks_tx: engine.tasks_tx(),
                        doc: &mut engine.document,
                        store: &mut engine.store,
                        camera: &mut engine.camera,
                        audioplayer: &mut engine.audioplayer
                });
                appwindow.handle_widget_flags(widget_flags);
            }),
        );

        // Italic
        self.imp().text_italic_button.connect_clicked(
            clone!(@weak appwindow => move |_text_italic_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                let widget_flags = engine.penholder.typewriter.add_text_attribute_current_selection(
                    TextAttribute::Style(FontStyle::Italic),
                    &mut EngineViewMut {
                        tasks_tx: engine.tasks_tx(),
                        doc: &mut engine.document,
                        store: &mut engine.store,
                        camera: &mut engine.camera,
                        audioplayer: &mut engine.audioplayer
                });
                appwindow.handle_widget_flags(widget_flags);
            }),
        );

        // Underline
        self.imp().text_underline_button.connect_clicked(
            clone!(@weak appwindow => move |_text_underline_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                let widget_flags = engine.penholder.typewriter.add_text_attribute_current_selection(
                    TextAttribute::Underline(true),
                    &mut EngineViewMut {
                        tasks_tx: engine.tasks_tx(),
                        doc: &mut engine.document,
                        store: &mut engine.store,
                        camera: &mut engine.camera,
                        audioplayer: &mut engine.audioplayer
                });
                appwindow.handle_widget_flags(widget_flags);
            }),
        );

        // Strikethrough
        self.imp().text_strikethrough_button.connect_clicked(
            clone!(@weak appwindow => move |_text_strikethrough_button| {
                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                let widget_flags = engine.penholder.typewriter.add_text_attribute_current_selection(
                    TextAttribute::Strikethrough(true),
                    &mut EngineViewMut {
                        tasks_tx: engine.tasks_tx(),
                        doc: &mut engine.document,
                        store: &mut engine.store,
                        camera: &mut engine.camera,
                        audioplayer: &mut engine.audioplayer
                });
                appwindow.handle_widget_flags(widget_flags);
            }),
        );

        // Alignment
        self.text_align_start_togglebutton().connect_active_notify(
            clone!(@weak appwindow => move |text_align_start_togglebutton| {
                if text_align_start_togglebutton.is_active() {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();
                        engine.penholder.typewriter.text_style.alignment = TextAlignment::Start;

                        let widget_flags = engine.penholder.typewriter.change_text_style_in_modifying_stroke(
                            |text_style| {
                                text_style.alignment = TextAlignment::Start;
                            },
                            &mut EngineViewMut {
                                tasks_tx: engine.tasks_tx(),
                                doc: &mut engine.document,
                                store: &mut engine.store,
                                camera: &mut engine.camera,
                                audioplayer: &mut engine.audioplayer
                        });
                        appwindow.handle_widget_flags(widget_flags);
                    }

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing typewriter alignment, Err `{}`", e);
                    }
                }

            }),
        );
        self.text_align_center_togglebutton().connect_active_notify(
            clone!(@weak appwindow => move |text_align_center_togglebutton| {
                if text_align_center_togglebutton.is_active() {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();
                        engine.penholder.typewriter.text_style.alignment = TextAlignment::Center;

                        let widget_flags = engine.penholder.typewriter.change_text_style_in_modifying_stroke(
                            |text_style| {
                                text_style.alignment = TextAlignment::Center;
                            },
                            &mut EngineViewMut {
                                tasks_tx: engine.tasks_tx(),
                                doc: &mut engine.document,
                                store: &mut engine.store,
                                camera: &mut engine.camera,
                                audioplayer: &mut engine.audioplayer
                        });
                        appwindow.handle_widget_flags(widget_flags);
                    }

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing typewriter alignment, Err `{}`", e);
                    }
                }
            }),
        );
        self.text_align_end_togglebutton().connect_active_notify(
            clone!(@weak appwindow => move |text_align_end_togglebutton| {
                if text_align_end_togglebutton.is_active() {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();
                        engine.penholder.typewriter.text_style.alignment = TextAlignment::End;

                        let widget_flags = engine.penholder.typewriter.change_text_style_in_modifying_stroke(
                            |text_style| {
                                text_style.alignment = TextAlignment::End;
                            },
                            &mut EngineViewMut {
                                tasks_tx: engine.tasks_tx(),
                                doc: &mut engine.document,
                                store: &mut engine.store,
                                camera: &mut engine.camera,
                                audioplayer: &mut engine.audioplayer
                        });
                        appwindow.handle_widget_flags(widget_flags);
                    }

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing typewriter alignment, Err `{}`", e);
                    }
                }
            }),
        );
        self.text_align_fill_togglebutton().connect_active_notify(
            clone!(@weak appwindow => move |text_align_fill_togglebutton| {
                if text_align_fill_togglebutton.is_active() {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();
                        engine.penholder.typewriter.text_style.alignment = TextAlignment::Fill;

                        let widget_flags = engine.penholder.typewriter.change_text_style_in_modifying_stroke(
                            |text_style| {
                                text_style.alignment = TextAlignment::Fill;
                            },
                            &mut EngineViewMut {
                                tasks_tx: engine.tasks_tx(),
                                doc: &mut engine.document,
                                store: &mut engine.store,
                                camera: &mut engine.camera,
                                audioplayer: &mut engine.audioplayer
                        });
                        appwindow.handle_widget_flags(widget_flags);
                    }

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing typewriter alignment, Err `{}`", e);
                    }
                }
            }),
        );
    }

    pub fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let typewriter = appwindow
            .canvas()
            .engine()
            .borrow()
            .penholder
            .typewriter
            .clone();

        self.fontchooser()
            .set_font_desc(&typewriter.text_style.extract_pango_font_desc());
        self.font_size_spinbutton()
            .set_value(typewriter.text_style.font_size);
        self.colorpicker()
            .set_current_color(Some(typewriter.text_style.color));

        match typewriter.text_style.alignment {
            TextAlignment::Start => self.text_align_start_togglebutton().set_active(true),
            TextAlignment::Center => self.text_align_center_togglebutton().set_active(true),
            TextAlignment::End => self.text_align_end_togglebutton().set_active(true),
            TextAlignment::Fill => self.text_align_fill_togglebutton().set_active(true),
        }
    }
}
