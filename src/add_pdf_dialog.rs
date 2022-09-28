use gettextrs::gettext;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;

use crate::i18n::gettext_f;
use crate::utils;

mod imp {
    use super::*;

    use glib::subclass::Signal;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/add_pdf_dialog.ui")]
    pub struct AddPdfDialog {
        pub citation_key: OnceCell<String>,

        #[template_child]
        pub download_button: TemplateChild<gtk::Widget>,
        #[template_child]
        pub pdf_url_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub error_box: TemplateChild<gtk::Widget>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddPdfDialog {
        const NAME: &'static str = "AddPdfDialog";
        type Type = super::AddPdfDialog;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl AddPdfDialog {
        #[template_callback]
        async fn set_from_file_button_clicked(&self) {
            if let Err(err) = self.instance().set_pdf_from_file().await {
                log::error!("could not set pdf: {err}");
                let toast = adw::Toast::new(&gettext("Could not set PDF"));
                self.toast_overlay.add_toast(&toast);
            }
        }

        #[template_callback]
        async fn on_download_pdf_button_clicked(&self) {
            self.download_button.set_sensitive(false);
            if let Err(err) = self.instance().download_pdf_for_entry().await {
                log::error!("Could not download pdf: {err}");
                self.pdf_url_entry_row.add_css_class("error");
                let toast = adw::Toast::new(&gettext("Could not download PDF"));
                self.toast_overlay.add_toast(&toast);
                self.download_button.set_sensitive(true);
            } else {
                self.pdf_url_entry_row.remove_css_class("error");
                self.instance().emit_by_name::<()>("pdf-set", &[]);
                self.instance().close();
            }
        }
    }

    impl ObjectImpl for AddPdfDialog {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("pdf-set", &[], <()>::static_type().into())
                    .run_last()
                    .build()]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if crate::utils::bibliography_path().is_none() {
                let bibliography_path = crate::utils::bibliography_path_string();
                self.error_box.show();
                self.error_label.set_label(&gettext_f(
                    // TRANSLATORS Do NOT translate {path}, it is a variable name
                    "Make sure that {path} exists",
                    &[("path", &bibliography_path)],
                ));
            }
        }
    }
    impl WidgetImpl for AddPdfDialog {}
    impl WindowImpl for AddPdfDialog {}
}

glib::wrapper! {
    pub struct AddPdfDialog(ObjectSubclass<imp::AddPdfDialog>)
        @extends gtk::Widget, gtk::Window;
}

impl Default for AddPdfDialog {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

impl AddPdfDialog {
    pub fn new<W: glib::IsA<gtk::Window>>(window: &W, citation_key: String) -> Self {
        let dialog: Self = glib::Object::new(&[("transient-for", window)]).unwrap();
        dialog.imp().citation_key.set(citation_key).unwrap();

        dialog
    }

    async fn download_pdf_for_entry(&self) -> anyhow::Result<()> {
        let url_text = self.imp().pdf_url_entry_row.text();
        let url = url::Url::parse(&url_text)?;

        if let Some(path) = crate::utils::bibliography_path() {
            self.imp().progress_bar.show();
            let duration = std::time::Duration::from_millis(100);
            let source = glib::timeout_add_local(
                duration,
                glib::clone!(@weak self as obj => @default-return glib::Continue(false), move || {
                    obj.imp().progress_bar.pulse();

                    glib::Continue(true)
                }),
            );

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

            let citation_key = self.imp().citation_key.get().unwrap();

            let folder = gio::File::for_path(&path);
            let pdf = folder.child(&utils::pdf_filename(citation_key));

            pdf.replace_contents_future(bytes, None, false, gio::FileCreateFlags::NONE)
                .await
                .map_err(|err| err.1)?;

            source.remove();
        }

        Ok(())
    }

    async fn set_pdf_from_file(&self) -> anyhow::Result<()> {
        let citation_key = self.imp().citation_key.get().unwrap();

        if let Some(path) = crate::utils::bibliography_path() {
            // TODO Add djvu support, is it needed?
            let folder = gio::File::for_path(&path);

            let dialog = gtk::FileChooserNative::new(
                None,
                Some(self),
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
                let filename = utils::pdf_filename(citation_key);
                let dest = folder.child(&filename);

                if let Err(err) = async_std::fs::create_dir_all(&path).await {
                    log::error!("Could not create bibliography directory: {err}");
                }

                file.copy_future(&dest, gio::FileCopyFlags::NONE, glib::Priority::default())
                    .0
                    .await?;
                log::debug!("Copied {filename} to {}", path.to_str().unwrap());

                self.emit_by_name::<()>("pdf-set", &[]);
                self.close();
            }
        }

        Ok(())
    }
}
