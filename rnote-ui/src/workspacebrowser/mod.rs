pub mod filerow;

mod imp {
    use gtk4::{
        gio, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, DirectoryList,
        ListView, Widget,
    };
    use gtk4::{Button, Entry, Separator};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacebrowser.ui")]
    pub struct WorkspaceBrowser {
        #[template_child]
        pub open_workspace_button: TemplateChild<Button>,
        #[template_child]
        pub flap_close_buttonbox: TemplateChild<gtk4::Box>,
        #[template_child]
        pub flap_close_buttonseparator: TemplateChild<Separator>,
        #[template_child]
        pub flap_close_button: TemplateChild<Button>,
        #[template_child]
        pub workspace_pathup_button: TemplateChild<Button>,
        #[template_child]
        pub workspace_pathentry: TemplateChild<Entry>,
        #[template_child]
        pub workspace_controlbox: TemplateChild<gtk4::Box>,
        #[template_child]
        pub primary_listview: TemplateChild<ListView>,
        pub primary_dirlist: DirectoryList,
    }

    impl Default for WorkspaceBrowser {
        fn default() -> Self {
            let primary_dirlist =
                DirectoryList::new(Some("standard::*"), None as Option<&gio::File>);
            primary_dirlist.set_monitored(true);

            Self {
                flap_close_buttonbox: TemplateChild::<gtk4::Box>::default(),
                flap_close_buttonseparator: TemplateChild::<Separator>::default(),
                flap_close_button: TemplateChild::<Button>::default(),
                open_workspace_button: TemplateChild::<Button>::default(),
                workspace_pathup_button: TemplateChild::<Button>::default(),
                workspace_pathentry: TemplateChild::<Entry>::default(),
                workspace_controlbox: TemplateChild::<gtk4::Box>::default(),
                primary_listview: TemplateChild::<ListView>::default(),
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.workspace_pathup_button.get().connect_clicked(
                clone!(@weak obj => move |_workspace_pathup_button| {
                        if let Some(current_path) = obj.primary_path() {
                            if let Some(parent_path) = current_path.parent() {
                                obj.set_primary_path(Some(parent_path));
                            }
                        }
                }),
            );
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for WorkspaceBrowser {}
}
use std::path::{Path, PathBuf};

use crate::appwindow::RnoteAppWindow;
use gtk4::{
    gdk, gio, glib, glib::clone, glib::closure, prelude::*, subclass::prelude::*,
    ConstantExpression, CustomSorter, FileFilter, FilterChange, FilterListModel, ListItem,
    PropertyExpression, SignalListItemFactory, SingleSelection, SortListModel, SorterChange,
};
use gtk4::{Button, DirectoryList, Entry, ListView, MultiSorter, Separator, Widget};

use self::filerow::FileRow;

glib::wrapper! {
    pub struct WorkspaceBrowser(ObjectSubclass<imp::WorkspaceBrowser>)
        @extends gtk4::Widget;
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

    pub fn primary_dirlist(&self) -> DirectoryList {
        imp::WorkspaceBrowser::from_instance(self)
            .primary_dirlist
            .clone()
    }

    pub fn primary_listview(&self) -> ListView {
        imp::WorkspaceBrowser::from_instance(self)
            .primary_listview
            .clone()
    }

    pub fn workspace_controlbox(&self) -> gtk4::Box {
        imp::WorkspaceBrowser::from_instance(self)
            .workspace_controlbox
            .get()
    }

    pub fn flap_close_buttonbox(&self) -> gtk4::Box {
        imp::WorkspaceBrowser::from_instance(self)
            .flap_close_buttonbox
            .get()
    }

    pub fn flap_close_buttonseparator(&self) -> Separator {
        imp::WorkspaceBrowser::from_instance(self)
            .flap_close_buttonseparator
            .get()
    }

    pub fn flap_close_button(&self) -> Button {
        imp::WorkspaceBrowser::from_instance(self)
            .flap_close_button
            .get()
    }

    pub fn workspace_pathentry(&self) -> Entry {
        imp::WorkspaceBrowser::from_instance(self)
            .workspace_pathentry
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.imp().flap_close_button.get().connect_clicked(
            clone!(@weak appwindow => move |_flap_close_button| {
                if appwindow.flap().reveals_flap() && appwindow.flap().is_folded() {
                    appwindow.flap().set_reveal_flap(false);
                }
            }),
        );

        self.imp().open_workspace_button.get().connect_clicked(
            clone!(@weak appwindow => move |_open_workspace_button| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "open-workspace", None);
            }),
        );

        self.imp()
            .primary_dirlist
            .bind_property("file", &self.workspace_pathentry(), "text")
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
            let filerow = FileRow::new();
            list_item.set_child(Some(&filerow));

            let list_item_expr = ConstantExpression::new(list_item);
            let fileinfo_expr =
                PropertyExpression::new(ListItem::static_type(), Some(&list_item_expr), "item");

            let file_expr = fileinfo_expr.chain_closure::<Option<gio::File>>(closure!(
                |_: Option<glib::Object>, fileinfo_obj: Option<glib::Object>| {
                    fileinfo_obj
                        .map(|fileinfo_obj| {
                            fileinfo_obj
                                .downcast::<gio::FileInfo>()
                                .unwrap()
                                .attribute_object("standard::file")
                                .unwrap()
                                .downcast::<gio::File>()
                                .unwrap()
                        })
                        .to_value()
                }
            ));

            let content_provider_expr =
                fileinfo_expr.chain_closure::<gdk::ContentProvider>(closure!(
                    |_: Option<glib::Object>, fileinfo_obj: Option<glib::Object>| {
                        if let Some(fileinfo_obj) = fileinfo_obj {
                            if let Some(file) = fileinfo_obj
                                .downcast::<gio::FileInfo>()
                                .unwrap()
                                .attribute_object("standard::file")
                            {
                                let file = file
                                    .downcast::<gio::File>()
                                    .expect("failed to downcast::<gio::File>() from file GObject");

                                return gdk::ContentProvider::for_value(&file.to_value());
                            }
                        }

                        gdk::ContentProvider::for_value(&None::<gio::File>.to_value())
                    }
                ));

            let icon_name_expr =
                fileinfo_expr.chain_closure::<gio::ThemedIcon>(closure!(|_: Option<
                    glib::Object,
                >,
                                                                         fileinfo_obj: Option<
                    glib::Object,
                >| {
                    if let Some(fileinfo_obj) = fileinfo_obj {
                        if let Some(themed_icon) = fileinfo_obj
                            .downcast::<gio::FileInfo>()
                            .unwrap()
                            .attribute_object("standard::symbolic-icon")
                        {
                            return themed_icon.downcast::<gio::ThemedIcon>().unwrap();
                        }
                    }

                    gio::ThemedIcon::from_names(&[
                        "workspace-folder-symbolic",
                        "folder-documents-symbolic",
                    ])
                }));

            let basename_expr =
                fileinfo_expr.chain_closure::<String>(closure!(|_: Option<glib::Object>,
                                                                fileinfo_obj: Option<
                    glib::Object,
                >| {
                    if let Some(fileinfo_obj) = fileinfo_obj {
                        if let Some(file) = fileinfo_obj
                            .downcast::<gio::FileInfo>()
                            .unwrap()
                            .attribute_object("standard::file")
                        {
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

                    String::from("")
                }));

            file_expr.bind(&filerow, "current-file", Widget::NONE);
            basename_expr.bind(&filerow.file_label(), "label", Widget::NONE);
            icon_name_expr.bind(&filerow.file_image(), "gicon", Widget::NONE);
            content_provider_expr.bind(
                &filerow.drag_source(),
                "content",
                Widget::NONE,
            );
        });
        let filefilter = FileFilter::new();
        filefilter.add_pattern("*.rnote");
        filefilter.add_pattern("*.xopp");
        filefilter.add_pattern("*.svg");
        filefilter.add_mime_type("image/svg+xml");
        filefilter.add_mime_type("image/png");
        filefilter.add_mime_type("image/jpeg");
        filefilter.add_mime_type("application/x-xopp");
        filefilter.add_mime_type("application/pdf");
        filefilter.add_mime_type("inode/directory");
        let filefilter_model =
            FilterListModel::new(Some(&self.imp().primary_dirlist), Some(&filefilter));

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

            if first_filetype == gio::FileType::Directory
                && second_filetype != gio::FileType::Directory
            {
                gtk4::Ordering::Smaller
            } else if first_filetype != gio::FileType::Directory
                && second_filetype == gio::FileType::Directory
            {
                gtk4::Ordering::Larger
            } else {
                gtk4::Ordering::Equal
            }
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

            first_display_name.cmp(second_display_name).into()
        });

        let multisorter = MultiSorter::new();
        multisorter.append(&folder_sorter);
        multisorter.append(&alphanumeric_sorter);
        let multi_sort_model = SortListModel::new(Some(&filefilter_model), Some(&multisorter));

        let primary_selection_model = SingleSelection::new(Some(&multi_sort_model));

        self.imp()
            .primary_listview
            .get()
            .set_factory(Some(&primary_list_factory));
        self.imp()
            .primary_listview
            .get()
            .set_model(Some(&primary_selection_model));

        self.imp().primary_listview.get().connect_activate(clone!(@weak filefilter, @weak multisorter, @weak appwindow => move |primary_listview, position| {
            let model = primary_listview.model().expect("model for primary_listview does not exist.");
            let fileinfo = model.item(position).expect("selected item in primary_listview does not exist.").downcast::<gio::FileInfo>().expect("selected item in primary_list is not of Type `gio::FileInfo`");

            if let Some(file) = fileinfo.attribute_object("standard::file") {
                let file = file.downcast::<gio::File>().unwrap();

                appwindow.open_file_w_dialogs(&file, None);
            };

            multisorter.changed(SorterChange::Different);
            filefilter.changed(FilterChange::Different);
        }));

        self.imp().primary_dirlist.connect_file_notify(
            clone!(@weak appwindow, @weak filefilter, @weak multisorter => move |_primary_dirlist| {
                multisorter.changed(SorterChange::Different);
                filefilter.changed(FilterChange::Different);
            }),
        );

        self.imp().primary_dirlist.connect_items_changed(clone!(@weak filefilter, @weak multisorter => move |_primary_dirlist, _position, _removed, _added| {
            multisorter.changed(SorterChange::Different);
            filefilter.changed(FilterChange::Different);
        }));
    }

    pub fn primary_path(&self) -> Option<PathBuf> {
        if let Some(file) = self.imp().primary_dirlist.file() {
            file.path()
        } else {
            None
        }
    }

    pub fn set_primary_path(&self, path: Option<&Path>) {
        let path = path.map(|path| gio::File::for_path(path));

        self.imp().primary_dirlist.set_file(path.as_ref());
    }
}
