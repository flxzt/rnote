// Modules
mod filerow;
mod widgethelper;
mod workspaceactions;
pub(crate) mod workspacesbar;

// Re-exports
pub(crate) use filerow::RnFileRow;
pub(crate) use gtk4::{EveryFilter, FlattenListModel, ListHeader};
pub(crate) use workspacesbar::RnWorkspacesBar;

// Imports
use crate::appwindow::RnAppWindow;
use gtk4::{
    gdk, gio, glib, glib::clone, glib::closure, prelude::*, subclass::prelude::*, Button,
    CompositeTemplate, ConstantExpression, CustomFilter, CustomSorter, DirectoryList, FileFilter,
    FilterChange, FilterListModel, Grid, Label, ListItem, ListView, MultiSorter,
    PropertyExpression, ScrolledWindow, Separator, SignalListItemFactory, SingleSelection,
    SortListModel, SorterChange, Widget,
};
use std::path::PathBuf;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacebrowser.ui")]
    pub(crate) struct RnWorkspaceBrowser {
        pub(crate) action_group: gio::SimpleActionGroup,
        pub(crate) dir_list: DirectoryList,
        pub(crate) list_selection_model: SingleSelection,

        #[template_child]
        pub(crate) grid: TemplateChild<Grid>,
        #[template_child]
        pub(crate) dir_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) corner_filler: TemplateChild<Separator>,
        #[template_child]
        pub(crate) files_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) files_listview: TemplateChild<ListView>,
        #[template_child]
        pub(crate) active_workspace_name_label: TemplateChild<Label>,
        #[template_child]
        pub(crate) active_workspace_dir_label: TemplateChild<Label>,
        #[template_child]
        pub(crate) dir_controls_dir_up_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) dir_controls_actions_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) workspacesbar: TemplateChild<RnWorkspacesBar>,
    }

    impl Default for RnWorkspaceBrowser {
        fn default() -> Self {
            let dir_list = DirectoryList::new(Some("standard::*"), None as Option<&gio::File>);
            dir_list.set_monitored(true);

            Self {
                action_group: gio::SimpleActionGroup::new(),
                dir_list,
                list_selection_model: SingleSelection::default(),

                grid: TemplateChild::<Grid>::default(),
                dir_box: TemplateChild::<gtk4::Box>::default(),
                corner_filler: TemplateChild::<Separator>::default(),
                files_scroller: TemplateChild::<ScrolledWindow>::default(),
                files_listview: TemplateChild::<ListView>::default(),
                active_workspace_name_label: TemplateChild::<Label>::default(),
                active_workspace_dir_label: TemplateChild::<Label>::default(),
                dir_controls_dir_up_button: TemplateChild::<Button>::default(),
                dir_controls_actions_box: TemplateChild::<gtk4::Box>::default(),
                workspacesbar: TemplateChild::<RnWorkspacesBar>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnWorkspaceBrowser {
        const NAME: &'static str = "RnWorkspaceBrowser";
        type Type = super::RnWorkspaceBrowser;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnWorkspaceBrowser {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj()
                .insert_action_group("workspacebrowser", Some(&self.action_group));
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnWorkspaceBrowser {}
}

glib::wrapper! {
    pub(crate) struct RnWorkspaceBrowser(ObjectSubclass<imp::RnWorkspaceBrowser>)
        @extends gtk4::Widget;
}

impl Default for RnWorkspaceBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl RnWorkspaceBrowser {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn grid(&self) -> Grid {
        self.imp().grid.clone()
    }

    pub(crate) fn dir_box(&self) -> gtk4::Box {
        self.imp().dir_box.clone()
    }

    pub(crate) fn corner_filler(&self) -> Separator {
        self.imp().corner_filler.clone()
    }

    pub(crate) fn files_scroller(&self) -> ScrolledWindow {
        self.imp().files_scroller.clone()
    }

    pub(crate) fn workspacesbar(&self) -> RnWorkspacesBar {
        self.imp().workspacesbar.clone()
    }

    pub(crate) fn active_workspace_name_label(&self) -> Label {
        self.imp().active_workspace_name_label.clone()
    }

    pub(crate) fn active_workspace_dir_label(&self) -> Label {
        self.imp().active_workspace_dir_label.clone()
    }

    pub(crate) fn dir_controls_actions_box(&self) -> gtk4::Box {
        self.imp().dir_controls_actions_box.clone()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.imp().workspacesbar.get().init(appwindow);

        self.setup_dir_controls(appwindow);
        self.setup_files_list(appwindow);
        self.setup_actions(appwindow);
    }

    pub(crate) fn dir_list_file(&self) -> Option<gio::File> {
        self.imp().dir_list.file()
    }

    pub(crate) fn set_dir_list_file(&self, file: Option<&gio::File>) {
        self.imp().dir_list.set_file(file);
    }

    pub(crate) fn dir_list_dir(&self) -> Option<PathBuf> {
        self.imp().dir_list.file().and_then(|f| f.path())
    }

    pub(crate) fn refresh_dir_list_selected_workspace(&self) {
        if let Some(current_workspace_dir) = self
            .workspacesbar()
            .selected_workspacelistentry()
            .map(|e| PathBuf::from(e.dir()))
        {
            self.imp()
                .dir_list
                .set_file(Some(&gio::File::for_path(current_workspace_dir)));
        }
    }

    fn setup_actions(&self, appwindow: &RnAppWindow) {
        self.imp()
            .action_group
            .add_action(&workspaceactions::create_folder(self, appwindow));
    }

    fn setup_dir_controls(&self, appwindow: &RnAppWindow) {
        self.imp().dir_controls_dir_up_button.connect_clicked(clone!(@weak self as workspacebrowser, @weak appwindow => move |_| {
            if let Some(mut dir) = workspacebrowser.workspacesbar().selected_workspacelistentry().map(|e| PathBuf::from(e.dir())) {
                // don't canonicalize on windows, because that would convert the path to one with extended length syntax
                if !cfg!(target_os = "windows") {
                    dir = match dir.canonicalize() {
                        Ok(dir) => dir,
                        Err(e) => {
                            tracing::warn!("Could not canonicalize dir {dir:?} from workspacelistentry, Err: {e:?}");
                            return;
                        }
                    };
                }
                if let Some(parent) = dir.parent().map(|p| p.to_path_buf()) {
                    workspacebrowser.workspacesbar().set_selected_workspace_dir(parent);
                } else {
                    tracing::warn!("Can't move directory up from dir {dir:?} from workspacelistentry, has no parent.");
                }
            }
        }));
    }

    fn setup_files_list(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        let folders_filter = create_folders_filter();
        let notes_filter = create_notes_filter();
        let files_filter = create_files_filter();
        let folders_sorter = create_folders_sorter();
        let notes_sorter = create_notes_sorter();
        let files_sorter = create_files_sorter();

        let folders_list_model = SortListModel::new(
            Some(FilterListModel::new(
                Some(imp.dir_list.clone()),
                Some(folders_filter.clone()),
            )),
            Some(folders_sorter.clone()),
        );
        let notes_list_model = SortListModel::new(
            Some(FilterListModel::new(
                Some(imp.dir_list.clone()),
                Some(notes_filter.clone()),
            )),
            Some(notes_sorter.clone()),
        );
        let files_list_model = SortListModel::new(
            Some(FilterListModel::new(
                Some(imp.dir_list.clone()),
                Some(files_filter.clone()),
            )),
            Some(files_sorter.clone()),
        );
        let combined_list = gio::ListStore::new::<SortListModel>();
        combined_list.append(&folders_list_model);
        combined_list.append(&notes_list_model);
        combined_list.append(&files_list_model);

        imp.list_selection_model
            .set_model(Some(&SingleSelection::new(Some(FlattenListModel::new(
                Some(combined_list),
            )))));

        imp.files_listview
            .get()
            .set_model(Some(&imp.list_selection_model));
        imp.files_listview
            .get()
            .set_factory(Some(&create_files_list_row_factory(appwindow)));
        imp.files_listview
            .get()
            .set_header_factory(Some(&create_files_list_header_factory(appwindow)));

        self.imp().dir_list.connect_items_changed(clone!(
            @weak self as workspacebrowser,
            @weak folders_filter,
            @weak folders_sorter,
            @weak notes_filter,
            @weak notes_sorter,
            @weak files_filter,
            @weak files_sorter
            => move |_, _, _, _| {
                folders_filter.changed(FilterChange::Different);
                folders_sorter.changed(SorterChange::Different);
                notes_filter.changed(FilterChange::Different);
                notes_sorter.changed(SorterChange::Different);
                files_filter.changed(FilterChange::Different);
                files_sorter.changed(SorterChange::Different);
        }));

        imp.files_listview.get().connect_activate(clone!(@weak self as workspacebrowser,
            @weak appwindow,
            @weak folders_filter,
            @weak folders_sorter,
            @weak notes_filter,
            @weak notes_sorter,
            @weak files_filter,
            @weak files_sorter
            => move |listview, position| {
                let file_info = listview.model().unwrap().item(position).unwrap().downcast::<gio::FileInfo>().unwrap();
                if let Some(input_file) = file_info.attribute_object("standard::file") {
                    glib::spawn_future_local(clone!(@weak appwindow => async move {
                        appwindow.open_file_w_dialogs(input_file.downcast::<gio::File>().unwrap(), None, true).await;
                    }));
                };
                folders_filter.changed(FilterChange::Different);
                folders_sorter.changed(SorterChange::Different);
                notes_filter.changed(FilterChange::Different);
                notes_sorter.changed(SorterChange::Different);
                files_filter.changed(FilterChange::Different);
                files_sorter.changed(SorterChange::Different);
        }));

        self.imp().dir_list.connect_file_notify(
            clone!(@weak self as workspacebrowser => move |dir_list| {
                // Disable the dir up row when no file is set or has no parent.
                workspacebrowser
                    .imp()
                    .dir_controls_dir_up_button
                    .set_sensitive(dir_list.file().and_then(|f| f.parent()).is_some());
            }),
        );
    }

    /// Set the selected file in the files list with its position.
    pub(crate) fn files_list_set_selected(&self, position: Option<u32>) {
        self.imp()
            .list_selection_model
            .set_selected(position.unwrap_or(gtk4::INVALID_LIST_POSITION));
    }
}

fn create_files_list_row_factory(appwindow: &RnAppWindow) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();

    factory.connect_setup(clone!(@weak appwindow => move |_, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let filerow = RnFileRow::new();
            filerow.init(&appwindow);
            list_item.set_child(Some(&filerow));

            let list_item_expr = ConstantExpression::new(list_item);
            let fileinfo_expr =
                PropertyExpression::new(ListItem::static_type(), Some(&list_item_expr), "item");
            let position_expr =
                PropertyExpression::new(ListItem::static_type(), Some(&list_item_expr), "position");

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
                                let file = file.downcast::<gio::File>().unwrap();
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
                            let file = file.downcast::<gio::File>().unwrap();
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
            position_expr.bind(&filerow, "position", Widget::NONE);
            basename_expr.bind(&filerow.file_label(), "label", Widget::NONE);
            icon_name_expr.bind(&filerow.file_image(), "gicon", Widget::NONE);
            content_provider_expr.bind(&filerow.drag_source(), "content", Widget::NONE);
        }));

    factory
}

fn create_files_list_header_factory(appwindow: &RnAppWindow) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();

    factory.connect_setup(clone!(@weak appwindow => move |_, list_header| {
        let list_header = list_header.downcast_ref::<ListHeader>().unwrap();
        let separator = Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_start(12)
            .margin_end(12)
            .build();
        list_header.set_child(Some(&separator));
    }));
    factory
}

fn create_folders_filter() -> EveryFilter {
    let file_filter = FileFilter::new();
    file_filter.add_mime_type("inode/directory");
    let hidden_filter = create_hidden_filter();

    let every_filter = EveryFilter::new();
    every_filter.append(file_filter);
    every_filter.append(hidden_filter);
    every_filter
}

fn create_folders_sorter() -> MultiSorter {
    let sorter = MultiSorter::default();
    sorter.append(create_human_numeric_sorter());
    sorter
}

fn create_notes_filter() -> EveryFilter {
    let file_filter = FileFilter::new();
    file_filter.add_mime_type("application/rnote");
    file_filter.add_suffix("rnote");
    let hidden_filter = create_hidden_filter();

    let every_filter = EveryFilter::new();
    every_filter.append(file_filter);
    every_filter.append(hidden_filter);
    every_filter
}

fn create_notes_sorter() -> MultiSorter {
    let sorter = MultiSorter::default();
    sorter.append(create_human_numeric_sorter());
    sorter
}

fn create_files_filter() -> EveryFilter {
    let file_filter = FileFilter::new();
    file_filter.add_mime_type("application/pdf");
    file_filter.add_mime_type("application/x-xopp");
    file_filter.add_mime_type("image/svg+xml");
    file_filter.add_mime_type("image/png");
    file_filter.add_mime_type("image/jpeg");
    file_filter.add_mime_type("text/plain");
    file_filter.add_suffix("pdf");
    file_filter.add_suffix("xopp");
    file_filter.add_suffix("svg");
    file_filter.add_suffix("png");
    file_filter.add_suffix("jpg");
    file_filter.add_suffix("jpeg");
    file_filter.add_suffix("txt");
    let hidden_filter = create_hidden_filter();

    let every_filter = EveryFilter::new();
    every_filter.append(file_filter);
    every_filter.append(hidden_filter);
    every_filter
}

fn create_files_sorter() -> MultiSorter {
    let sorter = MultiSorter::default();
    sorter.append(create_human_numeric_sorter());
    sorter
}

fn create_hidden_filter() -> CustomFilter {
    CustomFilter::new(|file| {
        let fileinfo = file.downcast_ref::<gio::FileInfo>().unwrap();
        let name = fileinfo.name();

        !name
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    })
}

/// Sorts by if file is a folder
#[allow(unused)]
fn create_sorter_order_folder() -> CustomSorter {
    CustomSorter::new(move |obj1, obj2| {
        let first_fileinfo = obj1.clone().downcast::<gio::FileInfo>().unwrap();
        let first_filetype = first_fileinfo.file_type();
        let second_fileinfo = obj2.clone().downcast::<gio::FileInfo>().unwrap();
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
    })
}

fn create_human_numeric_sorter() -> CustomSorter {
    CustomSorter::new(move |obj1, obj2| {
        let first_fileinfo = obj1.clone().downcast::<gio::FileInfo>().unwrap();
        let first_file = first_fileinfo.attribute_object("standard::file").unwrap();
        let first_file = first_file.downcast::<gio::File>().unwrap();
        let first_display_name = first_file.basename().unwrap();
        let first_display_name = first_display_name.to_str().unwrap();

        let second_fileinfo = obj2.clone().downcast::<gio::FileInfo>().unwrap();
        let second_file = second_fileinfo.attribute_object("standard::file").unwrap();
        let second_file = second_file.downcast::<gio::File>().unwrap();
        let second_display_name = second_file.basename().unwrap();
        let second_display_name = second_display_name.to_str().unwrap();

        numeric_sort::cmp(first_display_name, second_display_name).into()
    })
}
