// Imports
use crate::{RnAppWindow, RnCanvasWrapper};
use gtk4::{
    glib, glib::clone, pango, prelude::*, subclass::prelude::*, Button, CompositeTemplate,
    EmojiChooser, FontDialog, MenuButton, SpinButton, ToggleButton,
};
use rnote_engine::strokes::textstroke::{FontStyle, TextAlignment, TextAttribute, TextStyle};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/typewriterpage.ui")]
    pub(crate) struct RnTypewriterPage {
        pub(super) prev_picked_font_family: RefCell<Option<pango::FontFamily>>,

        #[template_child]
        pub(crate) fontdialog_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) font_size_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) emojichooser_menubutton: TemplateChild<MenuButton>,
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
        pub(crate) text_align_start_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_center_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_end_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) text_align_fill_togglebutton: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnTypewriterPage {
        const NAME: &'static str = "RnTypewriterPage";
        type Type = super::RnTypewriterPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnTypewriterPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnTypewriterPage {}
}

glib::wrapper! {
    pub(crate) struct RnTypewriterPage(ObjectSubclass<imp::RnTypewriterPage>)
        @extends gtk4::Widget;
}

impl Default for RnTypewriterPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnTypewriterPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn emojichooser_menubutton(&self) -> MenuButton {
        self.imp().emojichooser_menubutton.get()
    }

    #[allow(unused)]
    pub(crate) fn alignment(&self) -> Option<TextAlignment> {
        if self.imp().text_align_start_togglebutton.is_active() {
            Some(TextAlignment::Start)
        } else if self.imp().text_align_center_togglebutton.is_active() {
            Some(TextAlignment::Center)
        } else if self.imp().text_align_end_togglebutton.is_active() {
            Some(TextAlignment::End)
        } else if self.imp().text_align_fill_togglebutton.is_active() {
            Some(TextAlignment::Fill)
        } else {
            None
        }
    }

    pub(crate) fn set_alignment(&self, alignment: TextAlignment) {
        match alignment {
            TextAlignment::Start => self.imp().text_align_start_togglebutton.set_active(true),
            TextAlignment::Center => self.imp().text_align_center_togglebutton.set_active(true),
            TextAlignment::End => self.imp().text_align_end_togglebutton.set_active(true),
            TextAlignment::Fill => self.imp().text_align_fill_togglebutton.set_active(true),
        }
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.fontdialog_button.connect_clicked(clone!(@weak self as typewriterpage, @weak appwindow => move |_| {
            glib::spawn_future_local(clone!(@weak typewriterpage, @weak appwindow => async move {
                let dialog = FontDialog::builder().modal(false).build();
                let prev_picked_font_family = typewriterpage.imp().prev_picked_font_family.borrow().clone();

                match dialog.choose_family_future(Some(&appwindow), prev_picked_font_family.as_ref()).await {
                    Ok(new_font_family) => {
                        let canvas = appwindow.active_tab_wrapper().canvas();
                        let font_family_name = new_font_family.name().to_string();

                        typewriterpage.imp().prev_picked_font_family.borrow_mut().replace(new_font_family);
                        canvas.engine_mut().pens_config.typewriter_config.text_style.font_family.clone_from(&font_family_name);
                        let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {style.font_family = font_family_name});
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                    Err(e) => tracing::debug!("Did not choose new font family (Error or dialog dismissed by user), Err: {e:?}"),
                }
            }));
        }));

        // Font size
        imp.font_size_spinbutton.set_increments(1.0, 5.0);
        imp.font_size_spinbutton
            .set_range(TextStyle::FONT_SIZE_MIN, TextStyle::FONT_SIZE_MAX);
        imp.font_size_spinbutton
            .set_value(TextStyle::FONT_SIZE_DEFAULT);

        imp.font_size_spinbutton.connect_value_changed(
            clone!(@weak appwindow => move |spinbutton| {
                let font_size = spinbutton.value();
                let canvas = appwindow.active_tab_wrapper().canvas();

                canvas.engine_mut().pens_config.typewriter_config.text_style.font_size = font_size;
                let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {style.font_size = font_size});
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        // Emojis
        imp.emojichooser
            .connect_emoji_picked(clone!(@weak appwindow => move |_, emoji_str| {
                let canvas = appwindow.active_tab_wrapper().canvas();
                let widget_flags = canvas.engine_mut().insert_text(emoji_str.to_string(), None);
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }));

        // reset
        imp.text_reset_button
            .connect_clicked(clone!(@weak appwindow => move |_| {
                let canvas = appwindow.active_tab_wrapper().canvas();
                let widget_flags = canvas.engine_mut().text_selection_remove_attributes();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }));

        // Bold
        imp.text_bold_button
            .connect_clicked(clone!(@weak appwindow => move |_| {
                let canvas = appwindow.active_tab_wrapper().canvas();
                let widget_flags = canvas.engine_mut().text_selection_toggle_attribute(
                    TextAttribute::FontWeight(piet::FontWeight::BOLD.to_raw())
                );
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }));

        // Italic
        imp.text_italic_button
            .connect_clicked(clone!(@weak appwindow => move |_| {
                let canvas = appwindow.active_tab_wrapper().canvas();
                let widget_flags = canvas.engine_mut().text_selection_toggle_attribute(
                    TextAttribute::Style(FontStyle::Italic)
                );
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }));

        // Underline
        imp.text_underline_button
            .connect_clicked(clone!(@weak appwindow => move |_| {
                let canvas = appwindow.active_tab_wrapper().canvas();
                let widget_flags = canvas.engine_mut().text_selection_toggle_attribute(
                    TextAttribute::Underline(true)
                );
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }));

        // Strikethrough
        imp.text_strikethrough_button
            .connect_clicked(clone!(@weak appwindow => move |_| {
                let canvas = appwindow.active_tab_wrapper().canvas();
                let widget_flags = canvas.engine_mut().text_selection_toggle_attribute(
                    TextAttribute::Strikethrough(true)
                );
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }));

        // Alignment
        imp.text_align_start_togglebutton.connect_active_notify(
            clone!(@weak appwindow => move |togglebutton| {
                if !togglebutton.is_active() {
                    return
                }
                let canvas = appwindow.active_tab_wrapper().canvas();
                canvas.engine_mut().pens_config.typewriter_config.text_style.alignment = TextAlignment::Start;
                let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {style.alignment = TextAlignment::Start});
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        imp.text_align_center_togglebutton.connect_active_notify(
            clone!(@weak appwindow => move |togglebutton| {
                if !togglebutton.is_active() {
                    return
                }
                let canvas = appwindow.active_tab_wrapper().canvas();
                canvas.engine_mut().pens_config.typewriter_config.text_style.alignment = TextAlignment::Center;
                let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {style.alignment = TextAlignment::Center});
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        imp.text_align_end_togglebutton.connect_active_notify(
            clone!(@weak appwindow => move |togglebutton| {
                if !togglebutton.is_active() {
                    return
                }
                let canvas = appwindow.active_tab_wrapper().canvas();
                canvas.engine_mut().pens_config.typewriter_config.text_style.alignment = TextAlignment::End;
                let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {style.alignment = TextAlignment::End});
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        imp.text_align_fill_togglebutton.connect_active_notify(
            clone!(@weak appwindow => move |togglebutton| {
                if !togglebutton.is_active() {
                    return
                }
                let canvas = appwindow.active_tab_wrapper().canvas();
                canvas.engine_mut().pens_config.typewriter_config.text_style.alignment = TextAlignment::Fill;
                let widget_flags = canvas.engine_mut().text_selection_change_style(|style| {style.alignment = TextAlignment::Fill});
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );
    }

    pub(crate) fn refresh_ui(&self, active_tab: &RnCanvasWrapper) {
        let imp = self.imp();

        let typewriter_config = active_tab
            .canvas()
            .engine_ref()
            .pens_config
            .typewriter_config
            .clone();

        imp.font_size_spinbutton
            .set_value(typewriter_config.text_style.font_size);

        self.set_alignment(typewriter_config.text_style.alignment);
    }
}
