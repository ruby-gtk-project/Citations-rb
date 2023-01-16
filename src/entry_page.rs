// SPDX-License-Identifier: GPL-3.0-or-later
use adw::prelude::*;
use gettextrs::gettext;
use gtk::subclass::prelude::*;
use gtk::{cairo, gdk, gio, glib, CompositeTemplate};

use crate::utils;
use crate::AddPdfDialog;

mod imp {
    use super::*;

    use adw::subclass::prelude::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/entry_page.ui")]
    pub struct EntryPage {
        #[template_child]
        pub doi_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub preview_error_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub preview_error: TemplateChild<gtk::Widget>,
        #[template_child]
        pub pdf_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub pdf_add_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub pdf_preview: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub empty_bin: TemplateChild<gtk::Widget>,
        #[template_child]
        pub pdf_picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub other_entry_type_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub entry_type_combo_row: TemplateChild<adw::ComboRow>,
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
        pub extras_box: TemplateChild<gtk::Widget>,
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
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action_async("entry-page.open-pdf", None, |obj, _, _| async move {
                if let Err(err) = obj.open_pdf_action().await {
                    log::error!("Could not set PDF: {err}");
                }
            });

            klass.install_action_async("entry-page.set-pdf", None, |obj, _, _| async move {
                if let Err(err) = obj.set_pdf_action().await {
                    log::error!("Could not set PDF: {err}");
                } else {
                    obj.setup_pdf();
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl EntryPage {
        #[template_callback]
        async fn on_trash_pdf_button_clicked(&self) {
            let citation_key = {
                let entry = self.entry.borrow();

                entry.as_ref().unwrap().citation_key()
            };
            if let Some(path) = crate::utils::bibliography_path() {
                let folder = gio::File::for_path(&path);
                let pdf = folder.child(&utils::pdf_filename(&citation_key));
                if let Err(err) = pdf.trash_future(glib::Priority::default()).await {
                    log::error!("Could not trash pdf: {err}");
                } else {
                    self.obj().setup_pdf();
                }
            }
        }

        #[template_callback]
        fn on_entry_type_selected_notify(
            &self,
            _pspec: &glib::ParamSpec,
            combo_row: &adw::ComboRow,
        ) {
            let type_ = cratebibtex::EntryType::from(combo_row.selected() as i32);
            if let Some(entry) = self.entry.borrow().as_ref() {
                entry.set_entry_type(type_);
                self.obj().bind_extras_list_model(entry);
                self.obj().setup_other_entry_row();
            }
        }

        #[template_callback]
        fn on_entry_type_entry_row_changed(&self, entry_row: &adw::EntryRow) {
            let text = entry_row.text();
            if text.is_empty() {
                return;
            }
            // TODO If the user types "article" the type should change to it.
            self.entry
                .borrow()
                .as_ref()
                .unwrap()
                .set_other_entry_type(&text);
        }

        #[template_callback]
        fn drop_down_entry_type_name(item: &adw::EnumListItem) -> String {
            let type_ = cratebibtex::EntryType::from(item.value());
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
            let value = self.doi_entry.text();
            if let Ok(uri) = crate::utils::doi_url(&value) {
                let window = self
                    .obj()
                    .root()
                    .unwrap()
                    .downcast::<crate::Window>()
                    .unwrap();
                gtk::show_uri(Some(&window), uri.as_str(), gdk::CURRENT_TIME);
            }
        }

        #[template_callback]
        fn on_copy_key_clicked(&self) {
            let key = self.entry.borrow().as_ref().unwrap().citation_key();
            self.obj().clipboard().set_text(&key);

            let window = self
                .obj()
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
                // TRANSLATORS Every citation has an identifier called key
                window.send_toast(&gettext("Citation Key already exists"));
            }
        }
    }

    impl ObjectImpl for EntryPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            self.doi_entry
                .connect_changed(glib::clone!(@weak obj => move |row| {
                    let sensitive =crate::utils::doi_url(&row.text()).is_ok();
                    obj.imp().doi_button.set_sensitive(sensitive);
                }));

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
                .bind("entry-type", &*self.entry_type_combo_row, "selected")
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

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "entry" => self.entry.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "entry" => self.obj().set_entry(value.get().ok()),
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
        glib::Object::new(&[])
    }
}

impl EntryPage {
    fn setup_other_entry_row(&self) {
        let entry = self.imp().entry.borrow().clone().unwrap();
        let type_ = entry.entry_type();
        let is_visible = type_ == cratebibtex::EntryType::Other;
        self.imp().other_entry_type_row.set_visible(is_visible);
        let editable_text = if !is_visible {
            "".to_string()
        } else {
            entry
                .other_entry_type()
                .unwrap_or_else(|| "other".to_string())
        };
        self.imp().other_entry_type_row.set_text(&editable_text);
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

                if key.to_lowercase() == "eprint" {
                    let button = gtk::Button::from_icon_name("external-link-symbolic");
                    button.set_valign(gtk::Align::Center);

                    button.set_sensitive(false);
                    row.connect_changed(glib::clone!(@weak button => move |row| {
                        let text = row.text();
                        let sensitive = crate::utils::eprint_url(&text).is_ok();
                        button.set_sensitive(sensitive);
                    }));

                    button.set_tooltip_text(Some(&gettext("Open in Browser")));
                    button.connect_clicked(glib::clone!(@weak row => move |_| {
                        let window = row
                            .root()
                            .unwrap()
                            .downcast::<crate::Window>()
                            .unwrap();

                        if let Ok(uri) = crate::utils::eprint_url(&row.text()) {
                            gtk::show_uri(Some(&window), uri.as_str(), gdk::CURRENT_TIME);
                        }
                    }));
                    button.add_css_class("flat");
                    row.add_suffix(&button);
                }

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
        let entry = self.imp().entry.borrow();
        if entry.is_none() {
            return;
        }
        let filename = utils::pdf_filename(&entry.as_ref().unwrap().citation_key());

        let pdf = if let Some(path) = crate::utils::bibliography_path() {
            let folder = gio::File::for_path(&path);
            let pdf = folder.child(&filename);
            if pdf.query_exists(gio::Cancellable::NONE) {
                self.imp()
                    .pdf_stack
                    .set_visible_child(&*self.imp().pdf_preview);
                self.imp().pdf_preview.set_description(Some(&filename));

                Some(pdf)
            } else {
                self.imp()
                    .pdf_stack
                    .set_visible_child(&*self.imp().pdf_add_list_box);
                self.imp().pdf_preview.set_description(None);

                None
            }
        } else {
            None
        };

        crate::spawn!(glib::clone!(@weak self as obj => async move {
            if let Err(err) = obj.set_pdf_preview(pdf.as_ref()).await {
                log::error!("Could not set pdf preview: {err}");
                obj.imp().preview_error_stack.set_visible_child(&*obj.imp().preview_error);
            } else {
                obj.imp().preview_error_stack.set_visible_child(&*obj.imp().pdf_picture);
            }
        }));
    }

    pub async fn set_pdf_preview(&self, file: Option<&gio::File>) -> anyhow::Result<()> {
        if let Some(file) = file {
            let (bytes, _etag) = file.load_bytes_future().await?;
            let document = poppler::Document::from_bytes(&bytes, None)?;
            let page = match document.page(0) {
                Some(page) => page,
                None => anyhow::bail!("Document does not have any pages"),
            };
            let (width, height) = page.size();
            let surface =
                cairo::ImageSurface::create(cairo::Format::Rgb24, width as i32, height as i32)?;
            let ctxt = cairo::Context::new(&surface)?;
            ctxt.set_source_rgb(1.0, 1.0, 1.0);
            ctxt.paint()?;
            page.render(&ctxt);

            let stream = gio::MemoryOutputStream::new_resizable();
            let mut write_stream = stream.into_write();
            surface.write_to_png(&mut write_stream)?;
            let stream = write_stream.output_stream();
            stream.close(gio::Cancellable::NONE)?;
            let bytes = stream.steal_as_bytes();
            let texture = gdk::Texture::from_bytes(&bytes)?;

            self.imp().pdf_picture.set_paintable(Some(&texture));
        } else {
            self.imp().pdf_picture.set_paintable(gdk::Texture::NONE);
        }

        Ok(())
    }

    async fn move_pdf_for_entry(&self, old_citation_key: &str) -> anyhow::Result<()> {
        let entry = self.imp().entry.borrow().clone().unwrap();
        let new_citation_key = entry.citation_key();

        if let Some(path) = crate::utils::bibliography_path() {
            let folder = gio::File::for_path(&path);
            let pdf = folder.child(&utils::pdf_filename(old_citation_key));
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

    /// Used so that both views in the leaflet don't show a status page
    /// simultaneosly. Shows a status page with some text.
    pub fn set_empty_status_page(&self) {
        self.imp()
            .stack
            .set_visible_child(&self.imp().empty_page.get());
    }

    /// Shows an empty bin
    pub fn set_no_items_placeholder_view(&self) {
        self.imp()
            .stack
            .set_visible_child(&self.imp().empty_bin.get());
    }

    async fn open_pdf_action(&self) -> anyhow::Result<()> {
        let entry = self.imp().entry.borrow();
        let citation_key = entry.as_ref().unwrap().citation_key();
        let window = self.root().unwrap().downcast::<crate::Window>().unwrap();

        crate::utils::open_pdf(&citation_key, &window).await
    }

    async fn set_pdf_action(&self) -> anyhow::Result<()> {
        let window = self.root().unwrap().downcast::<crate::Window>().unwrap();
        let citation_key = self.imp().entry.borrow().as_ref().unwrap().citation_key();

        let dialog = AddPdfDialog::new(&window, citation_key);
        dialog.connect_local(
            "pdf-set",
            false,
            glib::clone!(@weak self as obj, @weak window => @default-return None, move |_| {
                obj.setup_pdf();
                window.send_toast(&gettext("PDF added"));

                None
            }),
        );
        dialog.present();

        Ok(())
    }
}
