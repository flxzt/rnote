// Modules
mod workspacelist;
mod workspacelistentry;
mod workspacerow;

use gtk4::ConstantExpression;
// Re-exports
pub(crate) use workspacelist::RnWorkspaceList;
pub(crate) use workspacelistentry::RnWorkspaceListEntry;
pub(crate) use workspacerow::RnWorkspaceRow;

// Imports
use crate::appwindow::RnAppWindow;
use crate::dialogs;
use gtk4::{
    gdk, gio, glib, glib::clone, prelude::*, subclass::prelude::*, Button, CompositeTemplate,
    ListBox, ScrolledWindow, Widget,
};
use std::path::PathBuf;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/workspacesbar/workspacesbar.ui")]
    pub(crate) struct RnWorkspacesBar {
        pub(crate) action_group: gio::SimpleActionGroup,
        pub(crate) workspace_list: RnWorkspaceList,

        #[template_child]
        pub(crate) workspaces_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) workspaces_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) move_selected_workspace_up_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) move_selected_workspace_down_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) add_workspace_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) remove_selected_workspace_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) edit_selected_workspace_button: TemplateChild<Button>,
    }

    impl Default for RnWorkspacesBar {
        fn default() -> Self {
            Self {
                action_group: gio::SimpleActionGroup::new(),
                workspace_list: RnWorkspaceList::default(),

                workspaces_scroller: Default::default(),
                workspaces_listbox: Default::default(),
                move_selected_workspace_up_button: Default::default(),
                move_selected_workspace_down_button: Default::default(),
                add_workspace_button: Default::default(),
                remove_selected_workspace_button: Default::default(),
                edit_selected_workspace_button: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnWorkspacesBar {
        const NAME: &'static str = "RnWorkspacesBar";
        type Type = super::RnWorkspacesBar;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnWorkspacesBar {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj()
                .insert_action_group("workspacesbar", Some(&self.action_group));
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnWorkspacesBar {}
}

glib::wrapper! {
    pub(crate) struct RnWorkspacesBar(ObjectSubclass<imp::RnWorkspacesBar>)
        @extends gtk4::Widget;
}

impl Default for RnWorkspacesBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RnWorkspacesBar {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn action_group(&self) -> gio::SimpleActionGroup {
        self.imp().action_group.clone()
    }

    pub(crate) fn workspaces_scroller(&self) -> ScrolledWindow {
        self.imp().workspaces_scroller.clone()
    }

    pub(crate) fn push_workspace(&self, entry: RnWorkspaceListEntry) {
        self.imp().workspace_list.push(entry);

        let n_items = self.imp().workspace_list.n_items();
        self.select_workspace_by_index(n_items.saturating_sub(1));
    }

    pub(crate) fn insert_workspace(&self, i: u32, entry: RnWorkspaceListEntry) {
        self.imp().workspace_list.insert(i as usize, entry);

        self.select_workspace_by_index(i);
    }

    pub(crate) fn remove_selected_workspace(&self) {
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

    pub(crate) fn move_selected_workspace_up(&self) {
        let n_items = self.imp().workspace_list.n_items();

        if n_items > 1 {
            let i = self
                .selected_workspace_index()
                .unwrap_or_else(|| n_items.saturating_sub(1));

            let entry = self.imp().workspace_list.remove(i as usize);

            self.insert_workspace(i.saturating_sub(1), entry);
            self.select_workspace_by_index(i.saturating_sub(1));
        }
    }

    pub(crate) fn move_selected_workspace_down(&self) {
        let n_items = self.imp().workspace_list.n_items();
        let i_max = n_items.saturating_sub(1);

        if n_items > 1 {
            let i = self.selected_workspace_index().unwrap_or(i_max);

            let entry = self.imp().workspace_list.remove(i as usize);

            let insert_i = (i + 1).min(i_max);
            self.insert_workspace(insert_i, entry);
            self.select_workspace_by_index(insert_i);
        }
    }

    pub(crate) fn select_workspace_by_index(&self, index: u32) {
        let n_items = self.imp().workspace_list.n_items();

        self.imp().workspaces_listbox.select_row(
            self.imp()
                .workspaces_listbox
                .row_at_index(index.min(n_items.saturating_sub(1)) as i32)
                .as_ref(),
        );
    }

    pub(crate) fn selected_workspace_index(&self) -> Option<u32> {
        self.imp()
            .workspaces_listbox
            .selected_row()
            .map(|r| r.index() as u32)
    }

    pub(crate) fn selected_workspacelistentry(&self) -> Option<RnWorkspaceListEntry> {
        self.selected_workspace_index().and_then(|i| {
            self.imp()
                .workspace_list
                .item(i)
                .map(|o| o.downcast::<RnWorkspaceListEntry>().unwrap())
        })
    }

    pub(crate) fn replace_selected_workspacelistentry(&self, entry: RnWorkspaceListEntry) {
        if let Some(i) = self.selected_workspace_index() {
            self.imp().workspace_list.replace(i as usize, entry);

            self.select_workspace_by_index(i);
        }
    }

    #[allow(unused)]
    pub(crate) fn set_selected_workspace_dir(&self, dir: PathBuf) {
        if let Some(i) = self.selected_workspace_index() {
            let entry = self.imp().workspace_list.remove(i as usize);
            entry.set_dir(dir.to_string_lossy().into());
            self.imp().workspace_list.insert(i as usize, entry);

            self.select_workspace_by_index(i);
        }
    }

    #[allow(unused)]
    pub(crate) fn set_selected_workspace_icon(&self, icon: String) {
        if let Some(i) = self.selected_workspace_index() {
            let row = self.imp().workspace_list.remove(i as usize);
            row.set_icon(icon);
            self.imp().workspace_list.insert(i as usize, row);

            self.select_workspace_by_index(i);
        }
    }

    #[allow(unused)]
    pub(crate) fn set_selected_workspace_color(&self, color: gdk::RGBA) {
        if let Some(i) = self.selected_workspace_index() {
            let row = self.imp().workspace_list.remove(i as usize);
            row.set_color(color);
            self.imp().workspace_list.insert(i as usize, row);

            self.select_workspace_by_index(i);
        }
    }

    #[allow(unused)]
    pub(crate) fn set_selected_workspace_name(&self, name: String) {
        if let Some(i) = self.selected_workspace_index() {
            let row = self.imp().workspace_list.remove(i as usize);
            row.set_name(name);
            self.imp().workspace_list.insert(i as usize, row);

            self.select_workspace_by_index(i);
        }
    }

    pub(crate) fn save_to_settings(&self, settings: &gio::Settings) {
        if let Err(e) = settings.set("workspace-list", self.imp().workspace_list.to_variant()) {
            log::error!("saving `workspace-list` to settings failed , Err: {e:?}");
        }

        if let Err(e) = settings.set(
            "selected-workspace-index",
            self.selected_workspace_index().unwrap_or(0),
        ) {
            log::error!("saving `selected-workspace-index` to settings failed , Err: {e:?}");
        }
    }

    pub(crate) fn load_from_settings(&self, settings: &gio::Settings) {
        let workspace_list = settings.get::<RnWorkspaceList>("workspace-list");
        // Be sure to get the index before loading the workspaces, else the setting gets overridden
        let selected_workspace_index = settings.uint("selected-workspace-index");

        // don't canonicalize on windows, because that would convert the path to one with extended length syntax
        if !cfg!(target_os = "windows") {
            for entry in &workspace_list.iter() {
                if let Err(e) = entry.canonicalize_dir() {
                    log::warn!(
                    "failed to canonicalize dir {:?} for workspacelistentry with name: {}, Err: {e:?}",
                    entry.dir(),
                    entry.name()
                )
                }
            }
        }
        self.imp().workspace_list.replace_self(workspace_list);

        self.select_workspace_by_index(selected_workspace_index);
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.setup_actions(appwindow);

        self.imp().workspace_list.connect_items_changed(
            clone!(@weak self as workspacesbar, @weak appwindow => move |list, _, _, _| {
                workspacesbar.imp().remove_selected_workspace_button.get().set_sensitive(list.n_items() > 1);
                workspacesbar.imp().edit_selected_workspace_button.get().set_sensitive(list.n_items() > 0);

                workspacesbar.save_to_settings(&appwindow.app_settings());
            }),
        );

        let workspace_listbox = self.imp().workspaces_listbox.get();
        workspace_listbox.connect_selected_rows_changed(
            clone!(@weak appwindow, @weak self as workspacesbar => move |_| {
                if let Some(entry) = workspacesbar.selected_workspacelistentry() {
                    let dir = entry.dir();
                    let name = entry.name();
                    appwindow.sidebar().workspacebrowser().active_workspace_name_label().set_label(&name);
                    appwindow.sidebar().workspacebrowser().active_workspace_dir_label().set_label(&dir);
                    appwindow.sidebar().workspacebrowser().set_dirlist_file(Some(&gio::File::for_path(dir)));

                    workspacesbar.save_to_settings(&appwindow.app_settings());
                }

            }),
        );

        workspace_listbox.bind_model(
            Some(&self.imp().workspace_list),
            clone!(@weak appwindow => @default-panic, move |obj| {
                let entry = obj.to_owned().downcast::<RnWorkspaceListEntry>().unwrap();
                let workspacerow = RnWorkspaceRow::new(&entry);
                workspacerow.init(&appwindow);

                let entry_expr = ConstantExpression::new(&entry);
                entry_expr.bind(&workspacerow, "entry", None::<&glib::Object>);

                workspacerow.upcast::<Widget>()
            }),
        );

        self.imp().move_selected_workspace_up_button.get().connect_clicked(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&workspacesbar.action_group(), "move-selected-workspace-up", None);
            }));

        self.imp().move_selected_workspace_down_button.get().connect_clicked(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&workspacesbar.action_group(), "move-selected-workspace-down", None);
            }));

        self.imp().add_workspace_button.get().connect_clicked(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&workspacesbar.action_group(), "add-workspace", None);
            }));

        self.imp().remove_selected_workspace_button.get().connect_clicked(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&workspacesbar.action_group(), "remove-selected-workspace", None);
            }));

        self.imp().edit_selected_workspace_button.get().connect_clicked(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&workspacesbar.action_group(), "edit-selected-workspace", None);
            }));
    }

    fn setup_actions(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        let action_move_selected_workspace_up =
            gio::SimpleAction::new("move-selected-workspace-up", None);
        imp.action_group
            .add_action(&action_move_selected_workspace_up);
        let action_move_selected_workspace_down =
            gio::SimpleAction::new("move-selected-workspace-down", None);
        imp.action_group
            .add_action(&action_move_selected_workspace_down);
        let action_add_workspace = gio::SimpleAction::new("add-workspace", None);
        imp.action_group.add_action(&action_add_workspace);
        let action_remove_selected_workspace =
            gio::SimpleAction::new("remove-selected-workspace", None);
        imp.action_group
            .add_action(&action_remove_selected_workspace);
        let action_edit_selected_workspace =
            gio::SimpleAction::new("edit-selected-workspace", None);
        imp.action_group.add_action(&action_edit_selected_workspace);

        // Move selected workspace up
        action_move_selected_workspace_up.connect_activate(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_, _| {
                workspacesbar.move_selected_workspace_up();
            }),
        );

        // Move selected workspace down
        action_move_selected_workspace_down.connect_activate(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_, _| {
                workspacesbar.move_selected_workspace_down();
            }),
        );

        // Add workspace
        action_add_workspace.connect_activate(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_, _| {
                glib::MainContext::default().spawn_local(clone!(@weak workspacesbar, @weak appwindow => async move {
                    let entry = workspacesbar.selected_workspacelistentry().unwrap_or_default();
                    workspacesbar.push_workspace(entry);

                    // Popup the edit dialog after creation
                    dialogs::dialog_edit_selected_workspace(&appwindow).await;
                }));
            }),
        );

        // Remove selected workspace
        action_remove_selected_workspace.connect_activate(
            clone!(@weak self as workspacesbar, @weak appwindow => move |_, _| {
                    workspacesbar.remove_selected_workspace();
            }),
        );

        // Edit selected workspace
        action_edit_selected_workspace.connect_activate(clone!(@weak appwindow => move |_, _| {
            glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                dialogs::dialog_edit_selected_workspace(&appwindow).await;
            }));
        }));
    }
}
