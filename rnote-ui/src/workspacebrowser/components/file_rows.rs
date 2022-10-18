use cairo::glib::{StaticType, Cast, ToValue};
use gtk4::{
    SignalListItemFactory,
    glib, glib::{clone, closure}, ConstantExpression, PropertyExpression, ListItem, gio, gdk, prelude::{FileExt, ListModelExt}, Widget, FileFilter, FilterListModel, CustomSorter, MultiSorter, subclass::prelude::ObjectSubclassIsExt, SorterChange, FilterChange, traits::{SorterExt, FilterExt, WidgetExt}, SortListModel, SingleSelection
};

use crate::{WorkspaceBrowser, RnoteAppWindow, workspacebrowser::{FileRow, WorkspaceRow, WorkspaceListEntry}};

pub fn setup_file_rows(workspacebrowser: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
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
            let filefilter_model =
                FilterListModel::new(Some(&workspacebrowser.imp().files_dirlist), Some(&filefilter));

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

            workspacebrowser.imp()
                .files_listview
                .get()
                .set_factory(Some(&primary_list_factory));
            workspacebrowser.imp()
                .files_listview
                .get()
                .set_model(Some(&primary_selection_model));

            workspacebrowser.imp().files_listview.get().connect_activate(clone!(@weak filefilter, @weak multisorter, @weak appwindow => move |files_listview, position| {
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

            workspacebrowser.imp().files_dirlist.connect_file_notify(
                clone!(@weak workspacebrowser, @weak appwindow, @weak filefilter, @weak multisorter => move |files_dirlist| {
                    // Disable the dir up row when no file is set or has no parent
                    workspacebrowser.imp().dir_up_row.set_sensitive(files_dirlist.file().and_then(|f| f.parent()).is_some());

                    multisorter.changed(SorterChange::Different);
                    filefilter.changed(FilterChange::Different);
                }),
            );

            workspacebrowser.imp().files_dirlist.connect_items_changed(clone!(@weak filefilter, @weak multisorter => move |_primary_dirlist, _position, _removed, _added| {
                multisorter.changed(SorterChange::Different);
                filefilter.changed(FilterChange::Different);
            }));

            // setup workspace rows
            let appwindow_c = appwindow.clone();
            workspacebrowser.imp()
                .workspace_listbox
                .bind_model(Some(&workspacebrowser.imp().workspace_list), move |obj| {
                    let entry = obj.to_owned().downcast::<WorkspaceListEntry>().unwrap();
                    let workspace_row = WorkspaceRow::new(entry);
                    workspace_row.init(&appwindow_c);

                    workspace_row.upcast::<Widget>()
                });
}
