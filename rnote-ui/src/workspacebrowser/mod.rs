mod filerow;
mod widget_helper;
mod workspaceactions;
pub(crate) mod workspacesbar;

use std::path::PathBuf;

// Re-exports
pub(crate) use filerow::FileRow;
pub(crate) use workspacesbar::WorkspacesBar;

// Imports
use crate::appwindow::RnoteAppWindow;
use gtk4::{
    gdk, gio, glib, glib::clone, glib::closure, prelude::*, subclass::prelude::*, Button,
    CompositeTemplate, ConstantExpression, CustomSorter, DirectoryList, FileFilter, FilterChange,
    FilterListModel, Grid, ListItem, ListView, MultiSorter, PropertyExpression, ScrolledWindow,
    SignalListItemFactory, SingleSelection, SortListModel, SorterChange, Widget,
};
use gtk4::{CustomFilter, GestureClick, PropagationPhase};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacebrowser.ui")]
    pub(crate) struct WorkspaceBrowser {
        pub(crate) action_group: gio::SimpleActionGroup,
        pub(crate) files_dirlist: DirectoryList,

        #[template_child]
        pub(crate) grid: TemplateChild<Grid>,
        #[template_child]
        pub(crate) files_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) files_listview: TemplateChild<ListView>,
        #[template_child]
        pub(crate) dir_controls_dir_up_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) dir_controls_actions_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) workspacesbar: TemplateChild<WorkspacesBar>,
    }

    impl Default for WorkspaceBrowser {
        fn default() -> Self {
            let files_dirlist = DirectoryList::new(Some("standard::*"), None as Option<&gio::File>);
            files_dirlist.set_monitored(true);

            Self {
                action_group: gio::SimpleActionGroup::new(),
                files_dirlist,

                grid: TemplateChild::<Grid>::default(),
                files_scroller: TemplateChild::<ScrolledWindow>::default(),
                files_listview: TemplateChild::<ListView>::default(),
                dir_controls_dir_up_button: TemplateChild::<Button>::default(),
                dir_controls_actions_box: TemplateChild::<gtk4::Box>::default(),
                workspacesbar: TemplateChild::<WorkspacesBar>::default(),
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
        fn constructed(&self) {
            self.parent_constructed();

            self.instance()
                .insert_action_group("workspacebrowser", Some(&self.action_group));
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for WorkspaceBrowser {}
}

glib::wrapper! {
    pub(crate) struct WorkspaceBrowser(ObjectSubclass<imp::WorkspaceBrowser>)
        @extends gtk4::Widget;
}

impl Default for WorkspaceBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceBrowser {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn grid(&self) -> Grid {
        self.imp().grid.clone()
    }

    pub(crate) fn files_scroller(&self) -> ScrolledWindow {
        self.imp().files_scroller.clone()
    }

    pub(crate) fn workspacesbar(&self) -> WorkspacesBar {
        self.imp().workspacesbar.clone()
    }

    pub(crate) fn dir_controls_actions_box(&self) -> gtk4::Box {
        self.imp().dir_controls_actions_box.clone()
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        self.imp().workspacesbar.get().init(appwindow);

        setup_dir_controls(self, appwindow);
        setup_file_rows(self, appwindow);

        self.setup_actions(appwindow);
    }

    pub(crate) fn dirlist_file(&self) -> Option<gio::File> {
        self.imp().files_dirlist.file()
    }

    pub(crate) fn set_dirlist_file(&self, file: Option<&gio::File>) {
        self.imp().files_dirlist.set_file(file);
    }

    pub(crate) fn dirlist_dir(&self) -> Option<PathBuf> {
        self.imp().files_dirlist.file().and_then(|f| f.path())
    }

    pub(crate) fn refresh_dirlist_selected_workspace(&self) {
        if let Some(current_workspace_dir) = self
            .workspacesbar()
            .selected_workspacelistentry()
            .map(|e| PathBuf::from(e.dir()))
        {
            self.imp()
                .files_dirlist
                .set_file(Some(&gio::File::for_path(current_workspace_dir)));
        }
    }

    fn setup_actions(&self, _appwindow: &RnoteAppWindow) {
        self.imp()
            .action_group
            .add_action(&workspaceactions::create_folder(self));
    }
}

fn setup_dir_controls(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    let dir_up_click_gesture = GestureClick::builder()
        .propagation_phase(PropagationPhase::Capture)
        .button(gdk::BUTTON_PRIMARY)
        .build();
    wb.imp()
        .dir_controls_dir_up_button
        .get()
        .add_controller(&dir_up_click_gesture);

    dir_up_click_gesture.connect_released(clone!(@weak wb, @weak appwindow => move |_, n_press, _, _| {
        // Only activate on multi click
        if n_press > 1 {
            if let Some(parent_dir) = wb.workspacesbar().selected_workspacelistentry().and_then(|e| PathBuf::from(e.dir()).parent().map(|p| p.to_path_buf())) {
                wb.workspacesbar().set_selected_workspace_dir(parent_dir);
            }
        }
    }));
}

fn setup_file_rows(wb: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    let primary_list_factory = SignalListItemFactory::new();

    primary_list_factory.connect_setup(clone!(@weak appwindow => move |_, list_item| {
        let list_item = list_item.downcast_ref::<ListItem>().unwrap();

        let filerow = FileRow::new();
        filerow.init(&appwindow);

        list_item.set_child(Some(&filerow));

        let list_item_expr = ConstantExpression::new(&list_item);
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
    filefilter.add_mime_type("application/rnote");
    filefilter.add_mime_type("application/pdf");
    filefilter.add_mime_type("application/x-xopp");
    filefilter.add_mime_type("image/svg+xml");
    filefilter.add_mime_type("image/png");
    filefilter.add_mime_type("image/jpeg");
    filefilter.add_mime_type("application/x-xopp");
    filefilter.add_mime_type("inode/directory");
    filefilter.add_suffix("rnote");
    filefilter.add_suffix("pdf");
    filefilter.add_suffix("xopp");
    filefilter.add_suffix("svg");
    filefilter.add_suffix("png");
    filefilter.add_suffix("jpg");
    filefilter.add_suffix("jpeg");

    let hidden_filter = CustomFilter::new(|file| {
        let fileinfo = file.downcast_ref::<gio::FileInfo>().unwrap();
        let name = fileinfo.name();

        !name
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    });

    let filter_listmodel = FilterListModel::new(
        Some(&FilterListModel::new(
            Some(&wb.imp().files_dirlist),
            Some(&filefilter),
        )),
        Some(&hidden_filter),
    );

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
    let multi_sort_model = SortListModel::new(Some(&filter_listmodel), Some(&multisorter));

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

                if let Some(input_file) = fileinfo.attribute_object("standard::file") {
                    appwindow.open_file_w_dialogs(input_file.downcast::<gio::File>().unwrap(), None);
                };

                multisorter.changed(SorterChange::Different);
                filefilter.changed(FilterChange::Different);
            }));

    wb.imp().files_dirlist.connect_file_notify(
                clone!(@weak wb as workspacebrowser, @weak appwindow, @weak filefilter, @weak multisorter => move |files_dirlist| {
                    // Disable the dir up row when no file is set or has no parent
                    workspacebrowser.imp().dir_controls_dir_up_button.set_sensitive(files_dirlist.file().and_then(|f| f.parent()).is_some());

                    multisorter.changed(SorterChange::Different);
                    filefilter.changed(FilterChange::Different);
                }),
            );

    wb.imp().files_dirlist.connect_items_changed(clone!(@weak filefilter, @weak multisorter => move |_primary_dirlist, _position, _removed, _added| {
                multisorter.changed(SorterChange::Different);
                filefilter.changed(FilterChange::Different);
            }));
}
