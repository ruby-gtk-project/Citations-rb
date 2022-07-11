// SPDX-License-Identifier: GPL-3.0-or-later
use adw::prelude::*;
use gettextrs::gettext;
use gtk::subclass::prelude::*;
use gtk::{cairo, gdk, gio, glib, CompositeTemplate};

use crate::i18n::gettext_f;
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
        pub pdf_preview: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub set_pdf_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub empty_bin: TemplateChild<gtk::Widget>,
        #[template_child]
        pub pdf_picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub pdf_url_entry_row: TemplateChild<adw::EntryRow>,
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
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);

            klass.install_action("entry-page.open-pdf", None, move |obj, _, _| {
                obj.open_pdf_action();
            });

            klass.install_action("entry-page.set-pdf", None, move |obj, _, _| {
                crate::spawn!(glib::clone!(@weak obj => async move {
                    if let Err(err) = obj.set_pdf_action().await {
                        log::error!("Could not set PDF: {err}");
                    } else {
                        obj.setup_pdf();
                    }
                }));
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
                    self.instance().setup_pdf();
                }
            }
        }

        #[template_callback]
        async fn on_download_pdf_button_clicked(&self) {
            if let Err(err) = self.instance().download_pdf_for_entry().await {
                log::error!("Could not download pdf: {err}");
                self.pdf_url_entry_row.add_css_class("error");
            } else {
                self.pdf_url_entry_row.remove_css_class("error");
                let window = self
                    .instance()
                    .root()
                    .unwrap()
                    .downcast::<crate::Window>()
                    .unwrap();
                window.send_toast(&gettext("PDF Downloaded"));
                self.instance().setup_pdf();
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
                self.instance().bind_extras_list_model(entry);
                self.instance().setup_other_entry_row();
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
            let uri = if value.contains("https://") || value.contains("http://") {
                value.to_string()
            } else {
                format!("https://doi.org/{}", value.replace("doi.org/", ""))
            };
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
                // TRANSLATORS Every citation has an identifier called key
                window.send_toast(&gettext("Citation Key already exists"));
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
                entry.bind_property(&key, &row, "text")
                     .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                     .build();

                if key.to_lowercase() == "eprint" {
                    let button = gtk::Button::from_icon_name("external-link-symbolic");
                    button.set_valign(gtk::Align::Center);

                    button.set_tooltip_text(Some(&gettext("Open in Browser")));
                    button.connect_clicked(glib::clone!(@weak row => move |_| {
                        let window = row
                            .root()
                            .unwrap()
                            .downcast::<crate::Window>()
                            .unwrap();

                        let value = row.text();
                        let uri = if value.contains("https://") || value.contains("http://") {
                            value.to_string()
                        } else {
                            // TODO Read archive prefix to deduce if it is not an arXiv
                            format!("https://arxiv.org/abs/{}", value
                                .replace("arxiv.org/abs/", "")
                                .replace("arxiv.org/", ""))
                        };

                        gtk::show_uri(Some(&window), &uri, gdk::CURRENT_TIME);
                    }));
                    button.add_css_class("flat");
                    row.add_suffix(&button);
                }

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
                self.imp().pdf_preview.show();
                self.imp().pdf_preview.set_description(Some(&filename));

                Some(pdf)
            } else {
                self.imp().pdf_preview.hide();
                self.imp().pdf_preview.set_description(None);

                None
            }
        } else {
            self.imp()
                .set_pdf_expander_row
                .set_title(&gettext("Bibliography folder not set"));
            let bibliography_path = crate::utils::bibliography_path_string();
            self.imp().set_pdf_expander_row.set_subtitle(&gettext_f(
                // TRANSLATORS Do NOT translate {path}, it is a variable name
                "Make sure that {path} exists",
                &[("path", &bibliography_path)],
            ));
            self.imp().set_pdf_expander_row.set_enable_expansion(false);
            // TODO
            // self.imp().pdf_action_row.set_subtitle("Open settings");
            // window.show_preferences_window();

            None
        };

        crate::spawn!(glib::clone!(@weak self as obj => async move {
            if let Err(err) = obj.set_pdf_preview(pdf.as_ref()).await {
                log::error!("Could not set pdf preview: {err}");
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

    async fn download_pdf_for_entry(&self) -> anyhow::Result<()> {
        let url_text = self.imp().pdf_url_entry_row.text();
        let url = url::Url::parse(&url_text)?;

        if let Some(path) = crate::utils::bibliography_path() {
            let mut res = match surf::get(&url).await {
                Ok(res) => res,
                Err(err) => anyhow::bail!("{err}"),
            };
            let bytes = match res.body_bytes().await {
                Ok(bytes) => bytes,
                Err(err) => anyhow::bail!("{err}"),
            };
            let bytes = glib::Bytes::from_owned(bytes);

            // Validate the document
            poppler::Document::from_bytes(&bytes, None)?;

            let citation_key = {
                let entry = self.imp().entry.borrow();

                entry.as_ref().unwrap().citation_key()
            };

            let folder = gio::File::for_path(&path);
            let pdf = folder.child(&utils::pdf_filename(&citation_key));

            pdf.replace_contents_future(bytes, None, false, gio::FileCreateFlags::NONE)
                .await
                .map_err(|err| err.1)?;
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

    fn open_pdf_action(&self) {
        let entry = self.imp().entry.borrow();
        let citation_key = entry.as_ref().unwrap().citation_key();
        let window = self.root().unwrap().downcast::<crate::Window>().unwrap();

        if let Some(path) = crate::utils::bibliography_path() {
            let folder = gio::File::for_path(&path);
            // TODO Add djvu support, is it needed?
            let pdf = folder.child(&utils::pdf_filename(&citation_key));
            if pdf.query_exists(gio::Cancellable::NONE) {
                gtk::show_uri(Some(&window), &pdf.uri(), gdk::CURRENT_TIME);
            }
        } else {
            log::error!("Could not open PDF");
        }
    }

    async fn set_pdf_action(&self) -> anyhow::Result<()> {
        let window = self.root().unwrap().downcast::<crate::Window>().unwrap();
        let citation_key = self.imp().entry.borrow().as_ref().unwrap().citation_key();

        if let Some(path) = crate::utils::bibliography_path() {
            // TODO Add djvu support, is it needed?
            let folder = gio::File::for_path(&path);

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
                let filename = utils::pdf_filename(&citation_key);
                let dest = folder.child(&filename);

                if let Err(err) = async_std::fs::create_dir_all(&path).await {
                    log::error!("Could not create bibliography directory: {err}");
                }

                file.copy_future(&dest, gio::FileCopyFlags::NONE, glib::Priority::default())
                    .0
                    .await?;
                log::debug!("Copied {filename} to {}", path.to_str().unwrap());
            }
        }

        Ok(())
    }
}
