use crate::RnAppWindow;
use adw::prelude::*;
use gtk4::{CompositeTemplate, Widget, glib, subclass::prelude::*};
use rnote_engine::WidgetFlags;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/searchpanel.ui")]
    pub(crate) struct RnSearchPanel {
        #[template_child]
        pub(crate) search_bar: TemplateChild<gtk4::SearchBar>,
        #[template_child]
        pub(crate) search_entry: TemplateChild<gtk4::SearchEntry>,
        #[template_child]
        pub(crate) content_stack: TemplateChild<gtk4::Stack>,
        #[template_child]
        pub(crate) status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub(crate) results_list: TemplateChild<gtk4::ListBox>,
        pub(crate) debounce_id: RefCell<Option<glib::SourceId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnSearchPanel {
        const NAME: &'static str = "RnSearchPanel";
        type Type = super::RnSearchPanel;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnSearchPanel {
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
    impl WidgetImpl for RnSearchPanel {}
}

glib::wrapper! {
    pub(crate) struct RnSearchPanel(ObjectSubclass<imp::RnSearchPanel>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnSearchPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl RnSearchPanel {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn init(&self, window: &RnAppWindow) {
        let imp = self.imp();

        // live search
        imp.search_entry.connect_search_changed(glib::clone!(
            #[weak(rename_to = panel)]
            self,
            #[weak]
            window,
            move |entry| {
                let query = entry.text().to_string();

                if let Some(id) = panel.imp().debounce_id.borrow_mut().take() {
                    id.remove();
                }

                let id = glib::source::timeout_add_local(
                    std::time::Duration::from_millis(300),
                    glib::clone!(
                        #[weak(rename_to = panel)]
                        panel,
                        #[weak]
                        window,
                        #[upgrade_or]
                        glib::ControlFlow::Break,
                        move || {
                            panel.perform_search(&query, &window);
                            panel.imp().debounce_id.replace(None);
                            glib::ControlFlow::Break
                        }
                    ),
                );

                panel.imp().debounce_id.replace(Some(id));
            }
        ));

        // enter to cylce through results
        imp.search_entry.connect_activate(glib::clone!(
            #[weak]
            window,
            #[weak(rename_to = panel)]
            self,
            move |_| {
                if let Some(canvas) = window.active_tab_canvas() {
                    let mut engine_mut = canvas.engine_mut();
                    let flags = engine_mut.focus_next_search_result();
                    window.handle_widget_flags(flags, &canvas);

                    let index = engine_mut.current_search_index as i32;
                    if let Some(row) = panel.imp().results_list.row_at_index(index) {
                        panel.imp().results_list.select_row(Some(&row));
                        row.grab_focus();
                    }
                }
            }
        ));

        // Handle clicking a specific result in the ListBox
        imp.results_list.connect_row_activated(glib::clone!(
            #[weak]
            window,
            #[weak(rename_to = panel)]
            self,
            move |_, row| {
                let index = row.index() as usize;

                if let Some(canvas) = window.active_tab_canvas() {
                    let mut engine_mut = canvas.engine_mut();
                    let flags = engine_mut.focus_search_result_at_index(index);
                    window.handle_widget_flags(flags, &canvas);

                    if let Some(active_row) = panel.imp().results_list.row_at_index(index as i32) {
                        panel.imp().results_list.select_row(Some(&active_row));
                        active_row.grab_focus();
                    }
                }
            }
        ));

        imp.search_entry.connect_stop_search(glib::clone!(
            #[weak]
            window,
            #[weak(rename_to = panel)]
            self,
            move |_| {
                panel.perform_search("", &window);
                window.split_view().set_show_sidebar(false);
            }
        ));

        self.perform_search("", window);
    }

    fn perform_search(&self, query: &str, window: &RnAppWindow) {
        let imp = self.imp();

        // clear existing
        while let Some(child) = imp.results_list.first_child() {
            imp.results_list.remove(&child);
        }
        imp.results_list.unselect_all();

        let query_trimmed = query.trim();

        // empty status page
        if query_trimmed.is_empty() {
            imp.status_page
                .set_icon_name(Some("system-search-symbolic"));
            imp.status_page.set_title("Search");
            imp.status_page
                .set_description(Some("Start typing to search the document."));
            imp.content_stack.set_visible_child_name("status");
        }

        if let Some(canvas) = window.active_tab_canvas() {
            let mut engine_mut = canvas.engine_mut();

            let results = engine_mut.search_document(query);

            if !query_trimmed.is_empty() {
                if results.is_empty() {
                    imp.status_page.set_icon_name(Some("edit-find-symbolic"));
                    imp.status_page.set_title("No Results");
                    imp.status_page
                        .set_description(Some("No matching text found."));
                    imp.content_stack.set_visible_child_name("status");
                } else {
                    imp.content_stack.set_visible_child_name("results");
                    const MAX_RESULTS: usize = 100;

                    for _result in results.iter().take(MAX_RESULTS) {
                        let snippet = _result.text.to_string();

                        let row = adw::ActionRow::builder()
                            .title(&snippet)
                            .selectable(true)
                            .activatable(true)
                            .build();

                        imp.results_list.append(&row);
                    }

                    if results.len() > MAX_RESULTS {
                        let row = adw::ActionRow::builder()
                            .title(format!("...and {} more", results.len() - MAX_RESULTS))
                            .sensitive(false)
                            .build();
                        imp.results_list.append(&row);
                    }

                    if let Some(row) = imp.results_list.row_at_index(0) {
                        imp.results_list.select_row(Some(&row));
                    }
                }
            }

            engine_mut.set_search_results(results);

            let mut flags = WidgetFlags::default();
            flags.redraw = true;
            window.handle_widget_flags(flags, &canvas);
        }
    }

    pub(crate) fn open_search(&self) {
        let imp = self.imp();
        imp.search_bar.set_search_mode(true);
        imp.search_entry.grab_focus();
    }
}
