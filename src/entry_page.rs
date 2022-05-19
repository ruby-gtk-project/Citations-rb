// SPDX-License-Identifier: GPL-3.0-or-later
use adw::prelude::*;
use gettextrs::gettext;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};

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
        pub pdf_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub entry_type_drop_down: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub entry_type_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub citation_key_entry: TemplateChild<adw::EntryRow>,
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
        fn on_entry_type_selected_notify(
            &self,
            _pspec: &glib::ParamSpec,
            drop_down: &gtk::DropDown,
        ) {
            let type_ = cratebibtex::EntryType::from(drop_down.selected());
            let entry = self.entry.borrow().clone().unwrap();
            entry.set_entry_type(type_);
            self.instance().bind_extras_list_model(&entry);
            self.instance().setup_other_entry_row();
        }

        #[template_callback]
        fn on_entry_type_entry_row_changed(&self, entry_row: &adw::EntryRow) {
            let text = entry_row.text();
            if text.is_empty() {
                return;
            }
            let entry = self.entry.borrow().clone().unwrap();
            // TODO If the user types "article" the type should change to it.
            entry.set_other_entry_type(&text);
        }

        #[template_callback]
        fn drop_down_entry_type_name(item: &adw::EnumListItem) -> String {
            let type_ = cratebibtex::EntryType::from(item.value() as u32);
            if type_ == cratebibtex::EntryType::Other {
                gettext("Other…")
            } else {
                type_.to_translatable_string()
            }
        }

        #[template_callback]
        fn entry_type_name_str(&self, type_: cratebibtex::EntryType) -> String {
            type_.to_translatable_string()
        }

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

        #[template_callback]
        fn on_copy_key_clicked(&self) {
            let key = self.entry.borrow().as_ref().unwrap().citation_key();
            self.instance().clipboard().set_text(&key);

            let window = self
                .instance()
                .root()
                .unwrap()
                .downcast::<crate::Window>()
                .unwrap();
            window.send_toast(&gettext("Copied"));
        }

        #[template_callback]
        async fn on_citation_key_apply(entry_row: &adw::EntryRow, entry_page: &super::EntryPage) {
            let new_key = entry_row.text();
            let entry = entry_page.imp().entry.borrow().clone().unwrap();
            let old_key = entry.citation_key();
            let window = entry_page
                .root()
                .unwrap()
                .downcast::<crate::Window>()
                .unwrap();

            if !window.bib().key_exists(&new_key) || old_key == new_key {
                entry_row.remove_css_class("error");
                entry.set_citation_key(&new_key);
                if let Err(err) = entry_page.move_pdf_for_entry(&old_key).await {
                    log::error!("Could not move pdf: {err}");
                }
            } else {
                entry_row.add_css_class("error");
                window.send_toast(&gettext("Citation Key already exists"));
            }
        }

        #[template_callback]
        fn on_pdf_row_activated(&self) {
            let entry = self.entry.borrow().clone().unwrap();
            let window = self
                .instance()
                .root()
                .unwrap()
                .downcast::<crate::Window>()
                .unwrap();

            if let Some(path) = crate::utils::bibliography_path() {
                let folder = gio::File::for_path(&path);
                // TODO Add djvu support, is it needed?
                let pdf = folder.child(&utils::pdf_filename(&entry.citation_key()));
                if pdf.query_exists(gio::Cancellable::NONE) {
                    gtk::show_uri(Some(&window), &pdf.uri(), gdk::CURRENT_TIME);
                } else {
                    let obj = self.instance();
                    crate::spawn!(glib::clone!(@weak obj => async move {
                        if let Err(err) = obj.set_pdf_for_entry().await {
                            log::error!("Could not set pdf: {err}");
                        } else {
                            obj.setup_pdf();
                        }
                    }));
                }
            }
        }
    }

    impl ObjectImpl for EntryPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // FIXME See https://github.com/gtk-rs/gtk-rs-core/pull/665
            let entry = cratebibtex::Entry::default();
            self.bindings.set_source(Some(&entry));

            self.bindings
                .bind("author", &*self.author_entry, "text")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .transform_to(|_binding, value| {
                    value
                        .get::<String>()
                        .map(|x| {
                            x.split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .to_value()
                        })
                        .ok()
                })
                .build();
            self.bindings
                .bind("title", &*self.title_entry, "text")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .transform_to(|_binding, value| {
                    value
                        .get::<String>()
                        .map(|x| {
                            x.split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .to_value()
                        })
                        .ok()
                })
                .build();
            self.bindings
                .bind("year", &*self.year_entry, "text")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();
            self.bindings
                .bind("doi", &*self.doi_entry, "text")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();
            self.bindings
                .bind("entry-type", &*self.entry_type_drop_down, "selected")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
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

impl EntryPage {
    fn setup_other_entry_row(&self) {
        let entry = self.imp().entry.borrow().clone().unwrap();
        let type_ = entry.entry_type();
        let editable = type_ == cratebibtex::EntryType::Other;
        self.imp().entry_type_entry_row.set_editable(editable);
        let editable_text = if !editable {
            "".to_string()
        } else {
            entry
                .other_entry_type()
                .unwrap_or_else(|| "other".to_string())
        };
        self.imp().entry_type_entry_row.set_text(&editable_text);
    }

    pub fn set_entry(&self, entry: Option<&cratebibtex::Entry>) {
        if entry != self.imp().entry.replace(entry.cloned()).as_ref() {
            self.notify("entry");
            self.imp().bindings.set_source(entry);

            if let Some(entry) = entry {
                self.setup_other_entry_row();
                self.imp()
                    .stack
                    .set_visible_child(&self.imp().non_empty_page.get());
                self.imp()
                    .citation_key_entry
                    .set_text(&entry.citation_key());
                self.bind_extras_list_model(entry);
                self.setup_pdf();
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

    pub fn bind_extras_list_model(&self, entry: &cratebibtex::Entry) {
        let fields: Vec<String> = entry.fields();
        let model = gtk::StringList::new(&fields.iter().map(|x| x.as_str()).collect::<Vec<&str>>());
        self.imp().extras_box.set_visible(model.item(0).is_some());
        self.imp().extras_list_box.bind_model(
            Some(&model),
            glib::clone!(@weak entry => @default-panic, move |obj| {
                let key = obj.downcast_ref::<gtk::StringObject>().unwrap().string();

                let row = adw::EntryRow::new();
                row.set_title(&utils::translated_key(&key));
                entry.bind_property(&key, &row, "text")
                     .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                     .build();

                row.upcast()
            }),
        );
    }

    pub fn entry(&self) -> Option<cratebibtex::Entry> {
        self.imp().entry.borrow().clone()
    }

    fn setup_pdf(&self) {
        let entry = self.imp().entry.borrow().clone();
        if entry.is_none() {
            return;
        }
        let entry = entry.unwrap();

        if let Some(path) = crate::utils::bibliography_path() {
            let folder = gio::File::for_path(&path);
            let pdf = folder.child(&utils::pdf_filename(&entry.citation_key()));
            if pdf.query_exists(gio::Cancellable::NONE) {
                self.imp().pdf_action_row.set_title(&gettext("Open PDF"));
                self.imp().pdf_action_row.set_subtitle("");
            } else {
                self.imp().pdf_action_row.set_title(&gettext("Set PDF"));
                self.imp()
                    .pdf_action_row
                    .set_subtitle("Copy file to bibiliography folder");
            }
        } else {
            self.imp()
                .pdf_action_row
                .set_title(&gettext("Bibliography folder not set"));
            // NOTE Do not translate the path.
            self.imp()
                .pdf_action_row
                .set_subtitle(&gettext("Make sure ~/Documents/Bibliography exists"));
            // TODO
            // self.imp().pdf_action_row.set_subtitle("Open settings");
            // window.show_preferences_window();
        }
    }

    async fn set_pdf_for_entry(&self) -> anyhow::Result<()> {
        let window = self.root().unwrap().downcast::<crate::Window>().unwrap();
        if let Some(path) = crate::utils::bibliography_path() {
            let folder = gio::File::for_path(&path);
            let entry = self.imp().entry.borrow().clone().unwrap();

            let dialog = gtk::FileChooserNative::new(
                None,
                Some(&window),
                gtk::FileChooserAction::Open,
                None,
                None,
            );

            let pdf_filter = gtk::FileFilter::new();
            pdf_filter.set_name(Some(&gettext("PDF Files")));
            pdf_filter.add_mime_type("application/pdf");
            dialog.add_filter(&pdf_filter);

            if dialog.run_future().await == gtk::ResponseType::Accept {
                let file = dialog.file().unwrap();
                let filename = utils::pdf_filename(&entry.citation_key());
                let dest = folder.child(&filename);

                if let Err(err) = async_std::fs::create_dir_all(&path).await {
                    log::error!("Could not create bibliography directory: {err}");
                }

                file.copy_future(&dest, gio::FileCopyFlags::NONE, glib::Priority::default())
                    .0
                    .await?;
                window.bib().set_modified(true);
                log::debug!("Copied {filename} to {}", path.to_str().unwrap());
            }
        }

        Ok(())
    }

    async fn move_pdf_for_entry(&self, old_citation_key: &str) -> anyhow::Result<()> {
        let entry = self.imp().entry.borrow().clone().unwrap();
        let new_citation_key = entry.citation_key();

        if let Some(path) = crate::utils::bibliography_path() {
            let folder = gio::File::for_path(&path);
            let pdf = folder.child(&utils::pdf_filename(&old_citation_key));
            if pdf.query_exists(gio::Cancellable::NONE) {
                let new_pdf = folder.child(&utils::pdf_filename(&new_citation_key));
                pdf.move_future(
                    &new_pdf,
                    gio::FileCopyFlags::NONE,
                    glib::Priority::default(),
                )
                .0
                .await?;
                log::debug!(
                    "Moved {} to {}",
                    pdf.path().unwrap().to_str().unwrap(),
                    new_pdf.path().unwrap().to_str().unwrap()
                );
            }
        }

        Ok(())
    }
}
