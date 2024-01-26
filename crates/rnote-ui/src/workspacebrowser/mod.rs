// Modules
mod filerow;
mod widgethelper;
mod workspaceactions;
pub(crate) mod workspacesbar;

// Re-exports
pub(crate) use filerow::RnFileRow;
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
use std::cell::RefCell;
use std::path::PathBuf;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacebrowser.ui")]
    pub(crate) struct RnWorkspaceBrowser {
        pub(crate) action_group: gio::SimpleActionGroup,
        pub(crate) files_dirlist: DirectoryList,
        pub(crate) files_selection_model: RefCell<SingleSelection>,

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
            let files_dirlist = DirectoryList::new(Some("standard::*"), None as Option<&gio::File>);
            files_dirlist.set_monitored(true);

            Self {
                action_group: gio::SimpleActionGroup::new(),
                files_dirlist,
                files_selection_model: RefCell::new(SingleSelection::default()),

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
        self.setup_file_rows(appwindow);
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

    fn setup_actions(&self, _appwindow: &RnAppWindow) {
        self.imp()
            .action_group
            .add_action(&workspaceactions::create_folder(self));
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

    fn setup_file_rows(&self, appwindow: &RnAppWindow) {
        let file_filter = create_file_filter();
        let filter_list_model = FilterListModel::new(
            Some(FilterListModel::new(
                Some(self.imp().files_dirlist.clone()),
                Some(file_filter.clone()),
            )),
            Some(create_hidden_filter()),
        );
        let multi_sorter = MultiSorter::new();
        multi_sorter.append(create_folder_sorter());
        multi_sorter.append(create_alphanumeric_sorter());
        let multi_sort_model =
            SortListModel::new(Some(filter_list_model), Some(multi_sorter.clone()));

        *self.imp().files_selection_model.borrow_mut() =
            SingleSelection::new(Some(multi_sort_model));
        self.imp()
            .files_listview
            .get()
            .set_model(Some(&*self.imp().files_selection_model.borrow()));
        self.imp()
            .files_listview
            .get()
            .set_factory(Some(&create_files_list_factory(appwindow)));

        self.imp().files_listview.get().connect_activate(clone!(@weak file_filter, @weak multi_sorter, @weak appwindow => move |files_listview, position| {
            let model = files_listview.model().unwrap();
            let fileinfo = model.item(position).unwrap().downcast::<gio::FileInfo>().unwrap();

            if let Some(input_file) = fileinfo.attribute_object("standard::file") {
                glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                    appwindow.open_file_w_dialogs(input_file.downcast::<gio::File>().unwrap(), None, true).await;
                }));
            };

            multi_sorter.changed(SorterChange::Different);
            file_filter.changed(FilterChange::Different);
        }));

        self.imp().files_dirlist.connect_file_notify(
                clone!(@weak self as workspacebrowser, @weak appwindow, @weak file_filter, @weak multi_sorter => move |files_dirlist| {
                    // Disable the dir up row when no file is set or has no parent
                    workspacebrowser.imp().dir_controls_dir_up_button.set_sensitive(files_dirlist.file().and_then(|f| f.parent()).is_some());

                    multi_sorter.changed(SorterChange::Different);
                    file_filter.changed(FilterChange::Different);
                }),
            );

        self.imp().files_dirlist.connect_items_changed(
            clone!(@weak file_filter, @weak multi_sorter => move |_, _, _, _| {
                multi_sorter.changed(SorterChange::Different);
                file_filter.changed(FilterChange::Different);
            }),
        );
    }

    /// Set the selected file in the list with its position/index.
    pub(crate) fn dirlist_set_selected(&self, position: u32) {
        self.imp()
            .files_selection_model
            .borrow()
            .set_selected(position);
    }
}

fn create_files_list_factory(appwindow: &RnAppWindow) -> SignalListItemFactory {
    let files_list_factory = SignalListItemFactory::new();

    files_list_factory.connect_setup(clone!(@weak appwindow => move |_, list_item| {
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

    files_list_factory
}

fn create_file_filter() -> FileFilter {
    let filefilter = FileFilter::new();
    filefilter.add_mime_type("application/rnote");
    filefilter.add_mime_type("application/pdf");
    filefilter.add_mime_type("application/x-xopp");
    filefilter.add_mime_type("image/svg+xml");
    filefilter.add_mime_type("image/png");
    filefilter.add_mime_type("image/jpeg");
    filefilter.add_mime_type("text/plain");
    filefilter.add_mime_type("inode/directory");
    filefilter.add_suffix("rnote");
    filefilter.add_suffix("pdf");
    filefilter.add_suffix("xopp");
    filefilter.add_suffix("svg");
    filefilter.add_suffix("png");
    filefilter.add_suffix("jpg");
    filefilter.add_suffix("jpeg");
    filefilter.add_suffix("txt");
    filefilter
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

fn create_folder_sorter() -> CustomSorter {
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

fn create_alphanumeric_sorter() -> CustomSorter {
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
