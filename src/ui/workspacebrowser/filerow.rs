mod imp {
    use gtk4::{gdk, DragSource, Image, Label};
    use gtk4::{glib, prelude::*, subclass::prelude::*, CompositeTemplate, Widget};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/filerow.ui")]
    pub struct FileRow {
        pub drag_source: DragSource,
        #[template_child]
        pub file_image: TemplateChild<Image>,
        #[template_child]
        pub file_label: TemplateChild<Label>,
    }

    impl Default for FileRow {
        fn default() -> Self {
            let drag_source = DragSource::builder()
                .name("workspacebrowser-file-drag-source")
                .actions(gdk::DragAction::COPY)
                .build();

            Self {
                drag_source,
                file_image: TemplateChild::<Image>::default(),
                file_label: TemplateChild::<Label>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileRow {
        const NAME: &'static str = "FileRow";
        type Type = super::FileRow;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FileRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.add_controller(&self.drag_source);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for FileRow {}
}

use gtk4::{DragSource, Image, Label};
use gtk4::{glib, subclass::prelude::*, Widget};

glib::wrapper! {
    pub struct FileRow(ObjectSubclass<imp::FileRow>)
        @extends Widget;
}

impl Default for FileRow {
    fn default() -> Self {
        Self::new()
    }
}

impl FileRow {
    pub fn new() -> Self {
        let filerow: Self = glib::Object::new(&[]).expect("Failed to create `FileRow`");
        filerow
    }

    pub fn file_image(&self) -> Image {
        imp::FileRow::from_instance(self).file_image.clone()
    }

    pub fn file_label(&self) -> Label {
        imp::FileRow::from_instance(self).file_label.clone()
    }

    pub fn drag_source(&self) -> DragSource {
        imp::FileRow::from_instance(self).drag_source.clone()
    }
}
