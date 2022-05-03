// SPDX-License-Identifier: GPL-3.0-or-later
use cratebibtex::EntryExt;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use gtk::{prelude::*, CompositeTemplate};

use crate::utils;

mod imp {
    use super::*;

    use adw::subclass::prelude::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/entry_page.ui")]
    pub struct EntryPage {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub entry_type_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub empty_page: TemplateChild<gtk::Widget>,
        #[template_child]
        pub non_empty_page: TemplateChild<gtk::Widget>,
        #[template_child]
        pub author_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub year_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub title_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub doi_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub extras_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub extras_list_box: TemplateChild<gtk::ListBox>,

        pub entry: RefCell<Option<cratebibtex::Entry>>,
        pub bindings: glib::BindingGroup,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EntryPage {
        const NAME: &'static str = "EntryPage";
        type Type = super::EntryPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl EntryPage {
        #[template_callback]
        fn on_doi_clicked(&self) {
            let uri = format!("https://doi.org/{}", self.doi_entry.text());
            let window = self
                .instance()
                .root()
                .unwrap()
                .downcast::<crate::Window>()
                .unwrap();
            gtk::show_uri(Some(&window), &uri, gdk::CURRENT_TIME);
        }
    }

    impl ObjectImpl for EntryPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // FIXME See https://github.com/gtk-rs/gtk-rs-core/pull/665
            //
            // self.bindings
            //     .bind("author", &*self.author_entry, "text")
            //     .build();
            // self.bindings
            //     .bind("title", &*self.title_entry, "text")
            //     .build();
            // self.bindings
            //     .bind("year", &*self.year_entry, "text")
            //     .build();
            // self.bindings
            //     .bind("doi", &*self.doi_entry, "text")
            //     .build();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "entry",
                    "entry",
                    "entry",
                    cratebibtex::Entry::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "entry" => self.entry.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "entry" => obj.set_entry(value.get().ok()),
                _ => unimplemented!(),
            };
        }
    }
    impl WidgetImpl for EntryPage {}
    impl BinImpl for EntryPage {}
}

glib::wrapper! {
    pub struct EntryPage(ObjectSubclass<imp::EntryPage>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for EntryPage {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

use adw::prelude::PreferencesRowExt;

impl EntryPage {
    pub fn set_entry(&self, entry: Option<&cratebibtex::Entry>) {
        if entry != self.imp().entry.replace(entry.cloned()).as_ref() {
            self.notify("entry");

            if let Some(entry) = entry {
                self.bind_entry(entry);
                self.imp()
                    .entry_type_label
                    .set_label(&entry.entry_type().to_translatable_string());
                self.imp()
                    .stack
                    .set_visible_child(&self.imp().non_empty_page.get());
                let fields: Vec<String> = entry.fields();
                let model =
                    gtk::StringList::new(&fields.iter().map(|x| x.as_str()).collect::<Vec<&str>>());
                self.imp().extras_box.set_visible(model.item(0).is_some());
                self.imp().extras_list_box.bind_model(Some(&model), glib::clone!(@weak entry => @default-panic, move |obj| {
                    let key = obj.downcast_ref::<gtk::StringObject>().unwrap().string();

                    let row = adw::EntryRow::new();
                    row.set_title(&utils::translated_key(&key));
                    entry.bind_property(&key, &row, "text").flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL).build();

                    row.upcast()
            }));
            } else {
                self.imp()
                    .stack
                    .set_visible_child(&self.imp().empty_page.get());

                self.imp()
                    .extras_list_box
                    .bind_model(gio::ListModel::NONE, move |_| adw::Bin::new().upcast());
            }
        }
    }

    pub fn entry(&self) -> Option<cratebibtex::Entry> {
        self.imp().entry.borrow().clone()
    }

    fn bind_entry(&self, entry: &cratebibtex::Entry) {
        self.imp().bindings.set_source(Some(entry));

        self.imp()
            .bindings
            .bind("author", &*self.imp().author_entry, "text")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();
        self.imp()
            .bindings
            .bind("title", &*self.imp().title_entry, "text")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();
        self.imp()
            .bindings
            .bind("year", &*self.imp().year_entry, "text")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();
        self.imp()
            .bindings
            .bind("doi", &*self.imp().doi_entry, "text")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();
    }

    pub fn set_empty_page(&self) {
        self.imp()
            .stack
            .set_visible_child(&self.imp().empty_page.get());
    }
}
