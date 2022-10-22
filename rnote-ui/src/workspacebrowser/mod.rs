mod widget_helper;

mod filerow;
mod workspace_action;
mod workspacelist;
mod workspacelistentry;
mod workspacerow;

// Re-exports
pub use filerow::FileRow;
pub use workspacelist::WorkspaceList;
pub use workspacelistentry::WorkspaceListEntry;
pub use workspacerow::WorkspaceRow;

use crate::appwindow::RnoteAppWindow;
use gtk4::{
    gdk, gio, glib, glib::clone, glib::closure, prelude::*, subclass::prelude::*, Button,
    CompositeTemplate, ConstantExpression, CustomSorter, DirectoryList, FileFilter, FilterChange,
    FilterListModel, Grid, ListBox, ListBoxRow, ListItem, ListView, MultiSorter,
    PropertyExpression, ScrolledWindow, SignalListItemFactory, SingleSelection, SortListModel,
    SorterChange, Widget,
};
use std::path::PathBuf;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacebrowser.ui")]
    pub struct WorkspaceBrowser {
        pub workspace_actions: gio::SimpleActionGroup,

        #[template_child]
        pub grid: TemplateChild<Grid>,
        #[template_child]
        pub add_workspace_button: TemplateChild<Button>,
        #[template_child]
        pub remove_workspace_button: TemplateChild<Button>,
        #[template_child]
        pub edit_workspace_button: TemplateChild<Button>,
        #[template_child]
        pub files_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub files_prefix_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub dir_up_row: TemplateChild<ListBoxRow>,
        #[template_child]
        pub files_listview: TemplateChild<ListView>,
        pub files_dirlist: DirectoryList,

        #[template_child]
        pub workspace_dir_actions_box: TemplateChild<gtk4::Box>,

        #[template_child]
        pub workspace_bar: TemplateChild<gtk4::Box>,
        #[template_child]
        pub workspace_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub workspace_listbox: TemplateChild<ListBox>,
        pub workspace_list: WorkspaceList,
    }

    impl Default for WorkspaceBrowser {
        fn default() -> Self {
            let primary_dirlist =
                DirectoryList::new(Some("standard::*"), None as Option<&gio::File>);
            primary_dirlist.set_monitored(true);

            Self {
                workspace_actions: gio::SimpleActionGroup::new(),
                grid: TemplateChild::<Grid>::default(),
                add_workspace_button: TemplateChild::<Button>::default(),
                remove_workspace_button: TemplateChild::<Button>::default(),
                edit_workspace_button: TemplateChild::<Button>::default(),
                files_scroller: TemplateChild::<ScrolledWindow>::default(),
                files_prefix_listbox: TemplateChild::<ListBox>::default(),
                dir_up_row: TemplateChild::<ListBoxRow>::default(),
                files_listview: TemplateChild::<ListView>::default(),
                workspace_dir_actions_box: TemplateChild::<gtk4::Box>::default(),
                files_dirlist: primary_dirlist,
                workspace_bar: TemplateChild::<gtk4::Box>::default(),
                workspace_scroller: TemplateChild::<ScrolledWindow>::default(),
                workspace_listbox: TemplateChild::<ListBox>::default(),
                workspace_list: WorkspaceList::default(),
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
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for WorkspaceBrowser {}
}

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

    pub fn grid(&self) -> Grid {
        self.imp().grid.clone()
    }

    pub fn files_scroller(&self) -> ScrolledWindow {
        self.imp().files_scroller.clone()
    }

    pub fn files_dirlist(&self) -> DirectoryList {
        self.imp().files_dirlist.clone()
    }

    pub fn files_listview(&self) -> ListView {
        self.imp().files_listview.clone()
    }

    pub fn workspace_bar(&self) -> gtk4::Box {
        self.imp().workspace_bar.clone()
    }

    pub fn workspace_scroller(&self) -> ScrolledWindow {
        self.imp().workspace_scroller.clone()
    }

    pub fn workspace_dir_actions_box(&self) -> gtk4::Box {
        self.imp().workspace_dir_actions_box.clone()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        setup_remove_workspace_button(self, appwindow);
        setup_add_workspace_button(self, appwindow);
        setup_edit_workspace_button(self, appwindow);

        setup_workspacelist(self, appwindow);
        setup_workspace_listbox(self, appwindow);

        setup_prefix_listbox(self, appwindow);
        setup_file_rows(self, appwindow);

        self.setup_dir_actions(appwindow);
    }

    pub fn add_workspace(&self, dir: PathBuf) {
        let entry = WorkspaceListEntry::from_path(dir);
        self.imp().workspace_list.push(entry);

        let n_items = self.imp().workspace_list.n_items();
        self.select_workspace_by_index(n_items.saturating_sub(1));
    }

    pub fn remove_current_workspace(&self) {
        let n_items = self.imp().workspace_list.n_items();

        // never remove the last row
        if n_items > 0 {
            let i = self
                .selected_workspace_index()
                .unwrap_or_else(|| n_items.saturating_sub(1));

            self.imp().workspace_list.remove(i as usize);

            self.select_workspace_by_index(i);
        }
    }

    pub fn select_workspace_by_index(&self, index: u32) {
        let n_items = self.imp().workspace_list.n_items();

        self.imp().workspace_listbox.select_row(
            self.imp()
                .workspace_listbox
                .row_at_index(index.min(n_items.saturating_sub(1)) as i32)
                .as_ref(),
        );
    }

    pub fn selected_workspace_index(&self) -> Option<u32> {
        self.imp()
            .workspace_listbox
            .selected_row()
            .map(|r| r.index() as u32)
    }

    pub fn selected_workspace_dir(&self) -> Option<PathBuf> {
        self.selected_workspace_index().and_then(|i| {
            self.imp()
                .workspace_list
                .item(i)
                .map(|o| PathBuf::from(o.downcast::<WorkspaceListEntry>().unwrap().dir()))
        })
    }

    pub fn set_current_workspace_dir(&self, dir: PathBuf) {
        let i = self.selected_workspace_index().unwrap_or(0);

        let row = self.imp().workspace_list.remove(i as usize);
        row.set_dir(dir.to_string_lossy().to_string());
        self.imp().workspace_list.insert(i as usize, row);

        self.select_workspace_by_index(i);
    }

    pub fn set_current_workspace_color(&self, color: gdk::RGBA) {
        let i = self.selected_workspace_index().unwrap_or(0);

        let row = self.imp().workspace_list.remove(i as usize);
        row.set_color(color);
        self.imp().workspace_list.insert(i as usize, row);

        self.select_workspace_by_index(i);
    }

    pub fn set_current_workspace_name(&self, name: String) {
        let i = self.selected_workspace_index().unwrap_or(0);

        let row = self.imp().workspace_list.remove(i as usize);
        row.set_name(name);
        self.imp().workspace_list.insert(i as usize, row);

        self.select_workspace_by_index(i);
    }

    pub fn current_selected_workspace_row(&self) -> Option<WorkspaceRow> {
        self.imp()
            .workspace_listbox
            .selected_row()
            .and_then(|row| row.child().map(|w| w.downcast::<WorkspaceRow>().unwrap()))
    }

    pub fn save_to_settings(&self, settings: &gio::Settings) {
        if let Err(e) = settings.set("workspace-list", &self.imp().workspace_list) {
            log::error!("saving `workspace-list` to settings failed with Err {}", e);
        }

        if let Err(e) = settings.set(
            "current-workspace-index",
            &self.selected_workspace_index().unwrap_or(0),
        ) {
            log::error!(
                "saving `current-workspace-index` to settings failed with Err {}",
                e
            );
        }
    }

    pub fn load_from_settings(&self, settings: &gio::Settings) {
        let workspace_list = settings.get::<WorkspaceList>("workspace-list");
        // Be sure to get the index before loading the workspaces, else the setting gets overriden
        let current_workspace_index = settings.uint("current-workspace-index");

        self.imp().workspace_list.replace_self(workspace_list);

        // current workspace index
        self.select_workspace_by_index(current_workspace_index);
    }

    fn setup_dir_actions(&self, _appwindow: &RnoteAppWindow) {
        self.insert_action_group("workspace_action", Some(&self.imp().workspace_actions));

        self.imp()
            .workspace_actions
            .add_action(&workspace_action::create_dir(self));
    }
}

fn setup_remove_workspace_button(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    wb.imp().remove_workspace_button.get().connect_clicked(
        clone!(@weak wb, @weak appwindow => move |_| {
            wb.remove_current_workspace();
        }),
    );
}

fn setup_add_workspace_button(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    wb.imp().add_workspace_button.get().connect_clicked(
        clone!(@weak wb, @weak appwindow => move |_add_workspace_button| {
            let dir = wb.selected_workspace_dir().unwrap_or(PathBuf::from("./"));
            wb.add_workspace(dir);

            // Popup the edit dialog after creation
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "edit-workspace", None);
        }),
    );
}

fn setup_edit_workspace_button(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    wb.imp()
        .edit_workspace_button
        .get()
        .connect_clicked(clone!(@weak appwindow => move |_| {
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "edit-workspace", None);
        }));
}

fn setup_workspacelist(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    wb.imp().workspace_list.connect_items_changed(
        clone!(@weak wb, @weak appwindow => move |folders_model, _, _, _| {
            wb.imp().remove_workspace_button.get().set_sensitive(folders_model.n_items() > 1);
            wb.imp().edit_workspace_button.get().set_sensitive(folders_model.n_items() > 0);

            wb.save_to_settings(&appwindow.app_settings());
        }),
    );
}

fn setup_workspace_listbox(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    wb.imp().workspace_listbox.connect_selected_rows_changed(
        clone!(@weak appwindow, @weak wb => move |_| {
            if let Some(dir) = wb.current_selected_workspace_row().map(|row| row.entry().dir()) {
                wb.imp().files_dirlist.set_file(Some(&gio::File::for_path(dir)));

                wb.save_to_settings(&appwindow.app_settings());
            }

        }),
    );
}

fn setup_prefix_listbox(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    wb.imp().files_prefix_listbox.connect_row_activated(
        clone!(@weak wb, @weak appwindow => move |_, row| {
            if row == &wb.imp().dir_up_row.get() {
                if let Some(parent_dir) = wb.selected_workspace_dir().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
                    wb.set_current_workspace_dir(parent_dir.to_path_buf());
                }
            }
        }));
}

fn setup_file_rows(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    let primary_list_factory = SignalListItemFactory::new();

    primary_list_factory.connect_setup(clone!(@weak appwindow => move |_, list_item| {
                let filerow = FileRow::new();
                filerow.init(&appwindow);

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
                    fileinfo_expr.chain_closure::<gio::ThemedIcon>(closure!(|_: Option<glib::Object>, fileinfo_obj: Option<glib::Object>| {
                        if let Some(fileinfo_obj) = fileinfo_obj {
                            if let Some(themed_icon) = fileinfo_obj
                                .downcast::<gio::FileInfo>()
                                .unwrap()
                                .attribute_object("standard::icon")
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
                    fileinfo_expr.chain_closure::<String>(closure!(|_: Option<glib::Object>, fileinfo_obj: Option<glib::Object>| {
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
                content_provider_expr.bind(&filerow.drag_source(), "content", Widget::NONE);
            }));

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
    let filefilter_model = FilterListModel::new(Some(&wb.imp().files_dirlist), Some(&filefilter));

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

        if first_filetype == gio::FileType::Directory && second_filetype != gio::FileType::Directory
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

    wb.imp()
        .files_listview
        .get()
        .set_factory(Some(&primary_list_factory));
    wb.imp()
        .files_listview
        .get()
        .set_model(Some(&primary_selection_model));

    wb.imp().files_listview.get().connect_activate(clone!(@weak filefilter, @weak multisorter, @weak appwindow => move |files_listview, position| {
                let model = files_listview.model().expect("model for primary_listview does not exist.");
                let fileinfo = model.item(position)
                    .expect("selected item in primary_listview does not exist.")
                    .downcast::<gio::FileInfo>().expect("selected item in primary_list is not of Type `gio::FileInfo`");

                if let Some(file) = fileinfo.attribute_object("standard::file") {
                    let file = file.downcast::<gio::File>().unwrap();

                    appwindow.open_file_w_dialogs(&file, None);
                };

                multisorter.changed(SorterChange::Different);
                filefilter.changed(FilterChange::Different);
            }));

    wb.imp().files_dirlist.connect_file_notify(
                clone!(@weak wb as workspacebrowser, @weak appwindow, @weak filefilter, @weak multisorter => move |files_dirlist| {
                    // Disable the dir up row when no file is set or has no parent
                    workspacebrowser.imp().dir_up_row.set_sensitive(files_dirlist.file().and_then(|f| f.parent()).is_some());

                    multisorter.changed(SorterChange::Different);
                    filefilter.changed(FilterChange::Different);
                }),
            );

    wb.imp().files_dirlist.connect_items_changed(clone!(@weak filefilter, @weak multisorter => move |_primary_dirlist, _position, _removed, _added| {
                multisorter.changed(SorterChange::Different);
                filefilter.changed(FilterChange::Different);
            }));

    // setup workspace rows
    let appwindow_c = appwindow.clone();
    wb.imp()
        .workspace_listbox
        .bind_model(Some(&wb.imp().workspace_list), move |obj| {
            let entry = obj.to_owned().downcast::<WorkspaceListEntry>().unwrap();
            let workspace_row = WorkspaceRow::new(entry);
            workspace_row.init(&appwindow_c);

            workspace_row.upcast::<Widget>()
        });
}
