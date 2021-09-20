mod imp {
    use gtk4::{
        gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate, DirectoryList, ListView,
        Widget,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/felixzwettler/rnote/ui/workspacebrowser.ui")]
    pub struct WorkspaceBrowser {
        #[template_child]
        pub primary_list: TemplateChild<ListView>,
        pub primary_dirlist: DirectoryList,
    }

    impl Default for WorkspaceBrowser {
        fn default() -> Self {
            let primary_dirlist = DirectoryList::new::<gio::File>(Some("standard::*"), None);
            primary_dirlist.set_monitored(true);

            Self {
                primary_list: TemplateChild::<ListView>::default(),
                primary_dirlist,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WorkspaceBrowser {
        const NAME: &'static str = "WorkspaceBrowser";
        type Type = super::WorkspaceBrowser;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WorkspaceBrowser {
        fn constructed(&self, _obj: &Self::Type) {}

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for WorkspaceBrowser {}
}
use std::path::{Path, PathBuf};

use crate::ui::appwindow::RnoteAppWindow;
use crate::ui::dialogs;
use crate::{app::RnoteApp, utils};
use gtk4::{
    gio, glib, glib::clone, prelude::*, subclass::prelude::*, Align, ClosureExpression,
    ConstantExpression, CustomSorter, FileFilter, FilterChange, FilterListModel, Label, ListItem,
    PropertyExpression, SignalListItemFactory, SingleSelection, SortListModel, SorterChange,
    Widget,
};
use gtk4::{pango, Image, MultiSorter, Orientation};

glib::wrapper! {
    pub struct WorkspaceBrowser(ObjectSubclass<imp::WorkspaceBrowser>)
        @extends Widget;
}

impl Default for WorkspaceBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceBrowser {
    pub fn new() -> Self {
        let workspacebrowser: Self =
            glib::Object::new(&[]).expect("Failed to create `WorkspaceBrowser`");
        workspacebrowser
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::WorkspaceBrowser::from_instance(self);

        priv_.primary_dirlist.connect_file_notify(clone!(@weak appwindow => move |primary_dirlist| {
            if let Some(file) = primary_dirlist.file() {
                if let Some(path) = file.path() {
                    appwindow.app_settings().set_string("workspace-dir", &path.to_string_lossy()).unwrap();
                }
            }
        }));

        priv_
            .primary_dirlist
            .bind_property("file", &appwindow.workspace_pathentry(), "text")
            .transform_to(|_, value| {
                let file = value.get::<Option<gio::File>>().unwrap();
                if let Some(file) = file {
                    if let Some(path) = file.path() {
                        return Some(path.to_string_lossy().to_value());
                    }
                }

                Some(String::from("").to_value())
            })
            .transform_from(|_, value| {
                let file = gio::File::for_path(&PathBuf::from(value.get::<String>().unwrap()));

                Some(file.to_value())
            })
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        let primary_list_factory = SignalListItemFactory::new();

        primary_list_factory.connect_setup(move |_, list_item| {
            let label = Label::builder().halign(Align::Start).ellipsize(pango::EllipsizeMode::End).build();
            let image = Image::new();
            let item_box = gtk4::Box::builder().
                orientation(Orientation::Horizontal)
                .halign(Align::Fill)
                .valign(Align::Fill)
                .hexpand(true)
                .vexpand(true)
                .build();

            item_box.style_context().add_class("workspace_listitem");
            item_box.prepend(&image);
            item_box.append(&label);
            list_item.set_child(Some(&item_box));

            let list_item_expression = ConstantExpression::new(list_item);
            let fileinfo_expression = PropertyExpression::new(
                ListItem::static_type(),
                Some(&list_item_expression),
                "item",
            );

            let icon_name_expression = ClosureExpression::new(
                clone!(@strong fileinfo_expression => move |expressions| {
                    if let Some(fileinfo) = expressions[1].get::<Option<glib::Object>>().expect(
                        "failed to get::<glib::Object>() from fileinfo_expression[1]. Wrong Type",
                    ) {
                        if let Ok(fileinfo) = fileinfo.downcast::<gio::FileInfo>() {
                            if let Some(themed_icon) = fileinfo.attribute_object("standard::symbolic-icon") {
                                return themed_icon.downcast::<gio::ThemedIcon>().unwrap();
                            }
                        }
                    }

                    gio::ThemedIcon::from_names(&["workspace-folder-symbolic", "folder-documents-symbolic"])
                }),
                &[fileinfo_expression.clone().upcast()]
            );

            let basename_expression = ClosureExpression::new(
                clone!(@strong fileinfo_expression => move |expressions| {
                    if let Some(fileinfo) = expressions[1].get::<Option<glib::Object>>().expect(
                        "failed to get::<glib::Object>() from fileinfo_expression[1]. Wrong Type",
                    ) {
                        if let Ok(fileinfo) = fileinfo.downcast::<gio::FileInfo>() {
                            if let Some(file) = fileinfo.attribute_object("standard::file") {
                                let file = file
                                    .downcast::<gio::File>()
                                    .expect("failed to downcast::<gio::File>() from file GObject");

                                return String::from(
                                    file.basename()
                                        .expect("failed to get file.basename()")
                                        .to_string_lossy(),
                                );
                            }
                        }
                    }

                    String::from(".")
                }),
                &[fileinfo_expression.clone().upcast()]
            );

            basename_expression.bind(&label, "label", Some(&label));
            //icon_name_expression.bind(&image, "icon-name", Some(&image));
            icon_name_expression.bind(&image, "gicon", Some(&image));
        });
        let filefilter = FileFilter::new();
        filefilter.add_pattern("*.rnote");
        filefilter.add_pattern("*.svg");
        filefilter.add_mime_type("image/svg+xml");
        filefilter.add_mime_type("image/png");
        filefilter.add_mime_type("inode/directory");
        let filefilter_model =
            FilterListModel::new(Some(&priv_.primary_dirlist), Some(&filefilter));

        let folder_sorter = CustomSorter::new(move |obj1, obj2| {
            let first_fileinfo = obj1
                .clone()
                .downcast::<gio::FileInfo>()
                .expect("failed to downcast obj1");
            let first_filetype = first_fileinfo.file_type();

            let second_fileinfo = obj2
                .clone()
                .downcast::<gio::FileInfo>()
                .expect("failed to downcast obj2");
            let second_filetype = second_fileinfo.file_type();

            let ordering = if first_filetype == gio::FileType::Directory
                && second_filetype != gio::FileType::Directory
            {
                gtk4::Ordering::Smaller
            } else if first_filetype != gio::FileType::Directory
                && second_filetype == gio::FileType::Directory
            {
                gtk4::Ordering::Larger
            } else {
                gtk4::Ordering::Equal
            };

            ordering
        });

        let alphanumeric_sorter = CustomSorter::new(move |obj1, obj2| {
            let first_fileinfo = obj1
                .clone()
                .downcast::<gio::FileInfo>()
                .expect("failed to downcast obj1");
            let first_file = first_fileinfo.attribute_object("standard::file").unwrap();
            let first_file = first_file.downcast::<gio::File>().unwrap();
            let first_display_name = first_file.basename().unwrap();
            let first_display_name = first_display_name.to_str().unwrap();

            let second_fileinfo = obj2
                .clone()
                .downcast::<gio::FileInfo>()
                .expect("failed to downcast obj2");
            let second_file = second_fileinfo.attribute_object("standard::file").unwrap();
            let second_file = second_file.downcast::<gio::File>().unwrap();
            let second_display_name = second_file.basename().unwrap();
            let second_display_name = second_display_name.to_str().unwrap();

            first_display_name.cmp(&second_display_name).into()
        });

        let multisorter = MultiSorter::new();
        multisorter.append(&folder_sorter);
        multisorter.append(&alphanumeric_sorter);
        let multi_sort_model = SortListModel::new(Some(&filefilter_model), Some(&multisorter));

        let primary_selection_model = SingleSelection::new(Some(&multi_sort_model));

        priv_
            .primary_list
            .get()
            .set_factory(Some(&primary_list_factory));
        priv_
            .primary_list
            .get()
            .set_model(Some(&primary_selection_model));

        priv_.primary_list.get().connect_activate(clone!(@weak filefilter, @weak alphanumeric_sorter, @weak appwindow => move |primary_list, position| {
            let model = primary_list.model().expect("model for primary_list does not exist.");
            let fileinfo = model.item(position).expect("selected item in primary_list does not exist.").downcast::<gio::FileInfo>().expect("selected item in primary_list is not of Type `gio::FileInfo`");

            if let Some(file) = fileinfo.attribute_object("standard::file") {
                let file = file.downcast::<gio::File>().unwrap();

                *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().borrow_mut() = Some(file.clone());

                if let Some(file) = &*appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().borrow() {
                    match utils::FileType::lookup_file_type(&file) {
                        utils::FileType::Rnote => {
                            *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().output_file().borrow_mut() = Some(file.clone());
                            dialogs::dialog_open_overwrite(&appwindow);
                        },
                        utils::FileType::Svg | utils::FileType::BitmapImage => {
                            if let Some(input_file) = appwindow
                                .application()
                                .unwrap()
                                .downcast::<RnoteApp>()
                                .unwrap()
                                .input_file()
                                .borrow()
                                .to_owned()
                            {
                                if let Err(e) = appwindow.load_in_file(&input_file) {
                                    log::error!("failed to load in input file, {}", e);
                                }
                            }
                        },
                        utils::FileType::Folder => {
                            if let Some(path) = file.path() {
                                appwindow.workspacebrowser().set_primary_path(&path);
                            }
                        },
                        utils::FileType::Unknown => {
                                log::warn!("tried to open unsupported file type.");
                        }
                    }
                } else {
                    log::warn!("No input file to open.");
                }
            };

            alphanumeric_sorter.changed(SorterChange::Different);
            filefilter.changed(FilterChange::Different);
        }));

        priv_.primary_dirlist.connect_file_notify(
            clone!(@weak filefilter, @weak alphanumeric_sorter => move |_| {

                alphanumeric_sorter.changed(SorterChange::Different);
                filefilter.changed(FilterChange::Different);
            }),
        );
    }

    pub fn primary_path(&self) -> Option<PathBuf> {
        let priv_ = imp::WorkspaceBrowser::from_instance(self);

        if let Some(file) = priv_.primary_dirlist.file() {
            return file.path();
        } else {
            None
        }
    }

    pub fn set_primary_path(&self, path: &Path) {
        let priv_ = imp::WorkspaceBrowser::from_instance(self);

        priv_
            .primary_dirlist
            .set_file(Some(&gio::File::for_path(path)));
    }

    pub fn remove_primary_path(&self) {
        let priv_ = imp::WorkspaceBrowser::from_instance(self);

        priv_.primary_dirlist.set_file::<gio::File>(None);
    }
}
