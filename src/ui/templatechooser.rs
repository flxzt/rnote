mod imp {
    use gtk4::{
        gio, glib, prelude::*, subclass::prelude::*, Align, Box, CompositeTemplate, DirectoryList,
        FileFilter, FilterListModel, Image, Label, ListBox, ListBoxRow, MenuButton, Orientation,
        Popover, TextView, Widget,
    };

    use crate::pens::brush;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/templatechooser.ui")]
    pub struct TemplateChooser {
        #[template_child]
        pub chooser_button: TemplateChild<MenuButton>,
        #[template_child]
        pub chooser_popover: TemplateChild<Popover>,
        #[template_child]
        pub help_button: TemplateChild<MenuButton>,
        #[template_child]
        pub help_popover: TemplateChild<Popover>,
        #[template_child]
        pub help_text: TemplateChild<TextView>,
        #[template_child]
        pub predefined_templates_list: TemplateChild<ListBox>,
        #[template_child]
        pub custom_templates_list: TemplateChild<ListBox>,
        #[template_child]
        pub predefined_template_experimental_listboxrow: TemplateChild<ListBoxRow>,
        pub templates_dirlist: DirectoryList,
    }

    impl Default for TemplateChooser {
        fn default() -> Self {
            let templates_dirlist = DirectoryList::new::<gio::File>(Some("standard::*"), None);
            templates_dirlist.set_monitored(true);

            Self {
                chooser_button: TemplateChild::<MenuButton>::default(),
                chooser_popover: TemplateChild::<Popover>::default(),
                help_button: TemplateChild::<MenuButton>::default(),
                help_popover: TemplateChild::<Popover>::default(),
                help_text: TemplateChild::<TextView>::default(),
                predefined_templates_list: TemplateChild::<ListBox>::default(),
                predefined_template_experimental_listboxrow: TemplateChild::<ListBoxRow>::default(),
                custom_templates_list: TemplateChild::<ListBox>::default(),
                templates_dirlist,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TemplateChooser {
        const NAME: &'static str = "TemplateChooser";
        type Type = super::TemplateChooser;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TemplateChooser {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let filefilter = FileFilter::new();
            filefilter.add_pattern("*.svg.templ");
            let filefilter_model =
                FilterListModel::new(Some(&self.templates_dirlist), Some(&filefilter));

            self.custom_templates_list
                .get()
                .bind_model(Some(&filefilter_model), move |fileinfo| {
                    let fileinfo = fileinfo.clone().downcast::<gio::FileInfo>().unwrap();
                    // Unwrap because DirectoryList always has the standard::file attribute set on its FileInfo's
                    let file = fileinfo
                        .attribute_object("standard::file")
                        .unwrap()
                        .downcast::<gio::File>()
                        .unwrap();

                    //let file = file.downcast::<gio::File>().unwrap();
                    let item_listboxrow = ListBoxRow::builder().build();
                    let item_box = Box::builder()
                        .orientation(Orientation::Horizontal)
                        .hexpand(true)
                        .halign(Align::Fill)
                        .build();
                    let item_label = Label::builder().build();
                    let item_icon = Image::builder().hexpand(true).halign(Align::End).build();

                    if let Some(basename) = file.basename() {
                        item_label.set_label(&basename.to_string_lossy())
                    } else {
                        item_label.set_label("invalid name")
                    }

                    match brush::validate_brush_template_for_file(&file) {
                        Ok(()) => {
                            item_icon.set_from_icon_name(Some("test-valid-symbolic"));
                            item_listboxrow.set_selectable(true);
                            item_listboxrow.set_sensitive(true);
                        }
                        Err(e) => {
                            log::warn!(
                                "validate_template_for_file() for file `{}` failed, {}",
                                file.basename().unwrap().to_str().unwrap(),
                                e
                            );
                            item_icon.set_from_icon_name(Some("test-invalid-symbolic"));
                            item_listboxrow.set_selectable(false);
                            item_listboxrow.set_sensitive(false);
                        }
                    }

                    item_box.append(&item_label);
                    item_box.append(&item_icon);
                    item_listboxrow.set_child(Some(&item_box));
                    item_listboxrow.upcast::<Widget>()
                });
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for TemplateChooser {}

    impl TemplateChooser {}
}

use std::path;

use crate::{config, pens::brush, ui::appwindow::RnoteAppWindow, utils};
use gtk4::{
    gio, glib, glib::clone, prelude::*, subclass::prelude::*, ListBoxRow, MenuButton, Popover,
    TextBuffer, Widget,
};

glib::wrapper! {
    pub struct TemplateChooser(ObjectSubclass<imp::TemplateChooser>)
        @extends Widget;
}

impl Default for TemplateChooser {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateChooser {
    pub fn new() -> Self {
        let template_chooser: Self =
            glib::Object::new(&[]).expect("Failed to create `TemplateChooser`");
        template_chooser
    }

    pub fn help_button(&self) -> MenuButton {
        imp::TemplateChooser::from_instance(self).help_button.get()
    }

    pub fn help_popover(&self) -> Popover {
        imp::TemplateChooser::from_instance(self).help_popover.get()
    }

    pub fn chooser_button(&self) -> MenuButton {
        imp::TemplateChooser::from_instance(self)
            .chooser_button
            .get()
    }

    pub fn chooser_popover(&self) -> Popover {
        imp::TemplateChooser::from_instance(self)
            .chooser_popover
            .get()
    }

    pub fn predefined_template_experimental_listboxrow(&self) -> ListBoxRow {
        imp::TemplateChooser::from_instance(self)
            .predefined_template_experimental_listboxrow
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::TemplateChooser::from_instance(self);
        let custom_templates_list = priv_.custom_templates_list.get();
        let predefined_templates_list = priv_.predefined_templates_list.get();
        let templates_dirlist = priv_.templates_dirlist.clone();

        priv_.predefined_templates_list.connect_row_selected(
            clone!(@weak appwindow, @weak templates_dirlist, @weak custom_templates_list => move |_predefined_templates_list, selection| {
                if let Some(selection) = selection {
                    custom_templates_list.select_row::<ListBoxRow>(None);

                    match selection.widget_name().as_str() {
                        "predefined_template_linear_row" => { 
                            (*appwindow.canvas().pens().borrow_mut()).brush.current_style = brush::BrushStyle::Linear;
                            },
                        "predefined_template_cubicbezier_row" => { 
                            (*appwindow.canvas().pens().borrow_mut()).brush.current_style = brush::BrushStyle::CubicBezier;
                            },
                        "predefined_template_experimental_row" => {
                            (*appwindow.canvas().pens().borrow_mut()).brush.current_style = brush::BrushStyle::Experimental;
                        }
                            _ => {
                                log::error!("selected unknown predefined template in templatechooser")
                            }
                    }
                }
            } ),
        );

        priv_.custom_templates_list.connect_row_selected(
            clone!(@weak appwindow, @weak templates_dirlist, @weak predefined_templates_list => move |_custom_templates_list, selection| {
                if let Some(selection) = selection {
                    predefined_templates_list.select_row::<ListBoxRow>(None);

                    if let Some(object) = templates_dirlist.item(selection.index() as u32) {
                        let file = object.downcast::<gio::FileInfo>().unwrap().attribute_object("standard::file")
                        .unwrap()
                        .downcast::<gio::File>()
                        .unwrap();

                        match brush::validate_brush_template_for_file(&file) {
                            Ok(()) => {
                                let file_contents = utils::load_file_contents(&file).unwrap();
                                (*appwindow.canvas().pens().borrow_mut()).brush.current_style = brush::BrushStyle::CustomTemplate(file_contents);
                            },
                            Err(e) => {
                                log::warn!(
                                    "validate_brush_template_for_file() for file `{}` failed, {}",
                                    file.basename().unwrap().to_string_lossy(),
                                    e
                                );
                            }
                        }
                    } else {
                        log::error!("failed to find item in templates_dirlist for selected row in predefined_templates_list at index `{}`", selection.index());
                    }
                }
            } ),
        );
    }

    pub fn set_help_text(&self, text: &str) {
        let priv_ = imp::TemplateChooser::from_instance(self);

        let help_text_buffer = TextBuffer::builder().text(text).build();
        priv_.help_text.get().set_buffer(Some(&help_text_buffer));
        priv_.help_text.get().queue_resize();
    }

    pub fn set_templates_path(&self, path: &path::Path) -> Result<(), ()> {
        let priv_ = imp::TemplateChooser::from_instance(self);

        log::debug!("setting path for brush templates_dirlist: `{:?}`", path);
        if let Some(templates_dir) = self.create_populate_templates_dir(path) {
            priv_.templates_dirlist.set_file(Some(&templates_dir));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn create_populate_templates_dir(&self, path: &path::Path) -> Option<gio::File> {
        let templates_dir = gio::File::for_path(path.to_path_buf());

        let templates_dir_opt =
            match templates_dir.make_directory_with_parents::<gio::Cancellable>(None) {
                Ok(()) => Some(templates_dir),
                Err(e) => match e.kind::<gio::IOErrorEnum>() {
                    Some(gio::IOErrorEnum::Exists) => Some(templates_dir),
                    _ => {
                        log::error!("failed to create templates_dir, {}", e);
                        return None;
                    }
                },
            };

        let templates_to_copy = [
            "brushstroke-linear.svg.templ",
            "brushstroke-cubicbezier.svg.templ",
        ];

        for template_name in templates_to_copy {
            let mut template_path = path.to_path_buf();
            template_path.push(template_name);
            let template_file = gio::File::for_path(template_path.clone());

            match template_file.create::<gio::Cancellable>(gio::FileCreateFlags::NONE, None) {
                Ok(file_output_stream) => {
                    if let Ok(template_string) = utils::load_string_from_resource(
                        (String::from(config::APP_IDPATH) + "templates/" + template_name).as_str(),
                    ) {
                        match file_output_stream.write::<gio::Cancellable>(template_string.as_bytes(), None) {
                                    Ok(_) => {
                                        file_output_stream.close::<gio::Cancellable>(None).unwrap_or_else(|e| {
                                            log::error!("failed to close output stream of default template `{}` in template_dir, {}", template_name, e);
                                        })
                                    },
                                    Err(e) => {
                                        log::error!("failed to write default template `{}` to template_dir, {}", template_name, e);
                                    }
                                }
                    } else {
                        log::error!("failed to load String from resource {}", template_name);
                    }
                }
                Err(e) => match e.kind::<gio::IOErrorEnum>() {
                    Some(gio::IOErrorEnum::Exists) => {
                        log::info!(
                            "template with name `{}`already exists in templates_dir",
                            template_name
                        )
                    }
                    _ => {
                        log::error!(
                            "failed to create default template `{}` in templates_dir, {}",
                            template_name,
                            e
                        )
                    }
                },
            }
        }

        templates_dir_opt
    }
}
