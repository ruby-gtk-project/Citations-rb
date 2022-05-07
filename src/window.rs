// SPDX-License-Identifier: GPL-3.0-or-later
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

use crate::application::Application;
use crate::config::{APP_ID, PROFILE, VERSION};
use crate::utils;

mod imp {
    use super::*;

    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/window.ui")]
    pub struct Window {
        pub bib: OnceCell<cratebibtex::Bib>,
        pub settings: gio::Settings,
        pub file: RefCell<Option<gio::File>>,
        pub undo_entry: RefCell<Option<(cratebibtex::Entry, u32, adw::Toast)>>,

        #[template_child]
        pub empty_state_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub entry_list: TemplateChild<crate::EntryList>,
        #[template_child]
        pub entry_page: TemplateChild<crate::EntryPage>,
        #[template_child]
        pub search_bar: TemplateChild<gtk::SearchBar>,
        #[template_child]
        pub search_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub split_button: TemplateChild<adw::SplitButton>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub secondary_menu_button: TemplateChild<gtk::Widget>,
        #[template_child]
        pub main_leaflet: TemplateChild<adw::Leaflet>,
    }

    impl Default for Window {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                bib: Default::default(),
                file: Default::default(),
                undo_entry: Default::default(),
                entry_list: TemplateChild::default(),
                empty_state_stack: TemplateChild::default(),
                entry_page: TemplateChild::default(),
                search_bar: TemplateChild::default(),
                search_button: TemplateChild::default(),
                search_entry: TemplateChild::default(),
                split_button: TemplateChild::default(),
                toast_overlay: TemplateChild::default(),
                secondary_menu_button: TemplateChild::default(),
                main_leaflet: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);

            klass.install_action("win.open", Some("s"), move |window, _, param| {
                let path = param.unwrap().get::<String>().unwrap();
                crate::spawn!(glib::clone!(@weak window => async move {
                    if let Err(err) = window.open_action(path).await {
                        log::error!("Could not open file: {err}");
                    }
                }));
            });
            klass.install_action("win.new_biblio", None, move |window, _, _| {
                crate::spawn!(glib::clone!(@weak window => async move {
                    if let Err(err) = window.new_biblio_action().await {
                        log::error!("Could not create file: {err}");
                    }
                }));
            });
            klass.install_action("win.new_entry", None, move |window, _, _| {
                window.new_entry_dialog();
            });
            klass.install_action("win.save", None, move |window, _, _| {
                crate::spawn!(glib::clone!(@weak window = > async move {
                    if let Err(err) = window.save().await {
                        log::error!("Could not save bibliography: {err}");
                    }
                }));
            });
            klass.install_action("win.delete_entry", None, move |obj, _, _| {
                obj.delete_entry();
            });
            klass.install_action("win.search-google-scholar", None, move |obj, _, _| {
                if let Err(err) = obj.search_google_schoolar_entry() {
                    log::error!("Could not search in Google Schoolar: {err}");
                }
            });
            klass.install_action("win.search-arxiv", None, move |obj, _, _| {
                if let Err(err) = obj.search_arxiv_entry() {
                    log::error!("Could not search in arXiv: {err}");
                }
            });
            klass.install_action("win.undo_delete", None, move |obj, _, _| {
                if let Some((entry, pos, _toast)) = obj.imp().undo_entry.replace(None) {
                    obj.imp().bib.get().unwrap().add_entry_at_pos(entry, pos);
                }
            });
            klass.install_action("win.about", None, move |obj, _, _| {
                obj.show_about_dialog();
            });
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl Window {
        #[template_callback]
        fn on_toggled(&self, button: gtk::ToggleButton) {
            let search_started = button.is_active();
            if !search_started {
                self.search_entry.set_text("");
            }
        }

        #[template_callback]
        fn on_changed(&self, search_entry: gtk::SearchEntry) {
            let query = search_entry.text();
            self.entry_list.filter(&query);
        }

        #[template_callback]
        fn on_entry_selected(&self, entry: Option<&cratebibtex::Entry>) {
            self.entry_page.set_entry(entry);
            self.secondary_menu_button.set_sensitive(entry.is_some());
            self.main_leaflet
                .navigate(adw::NavigationDirection::Forward);
        }

        #[template_callback]
        fn on_back_button_clicked(&self) {
            self.main_leaflet.navigate(adw::NavigationDirection::Back);
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            // Load latest window state
            obj.load_window_size();

            obj.populate_recent_files();
            self.settings.connect_changed(
                Some("recent-files"),
                glib::clone!(@weak obj => move |_,_| {
                    obj.populate_recent_files();
                }),
            );

            obj.action_set_enabled("win.new_entry", false);
            obj.action_set_enabled("win.save", false);
            obj.action_set_enabled("win.delete_entry", false);

            // Setup drag-n-drop
            let target = gtk::DropTarget::new(
                gio::File::static_type(),
                gdk::DragAction::COPY | gdk::DragAction::MOVE,
            );

            target.connect_drop(
                glib::clone!(@weak obj => @default-return false, move |_, value, _, _| {
                    if let Ok(file) = value.get::<gio::File>() {
                        if file.query_exists(gio::Cancellable::NONE) {
                            crate::spawn!(glib::clone!(@weak obj => async move {
                                if let Err(err) = obj.open_wrapper(&file).await {
                                    log::error!("Could not open file: {err:?}")
                                };
                            }));
                        }
                        return true;
                    }
                    false
                }),
            );

            obj.add_controller(&target);
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        // Save window state on delete event
        fn close_request(&self, window: &Self::Type) -> gtk::Inhibit {
            if let Err(err) = window.save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            // Pass close request on to the parent
            self.parent_close_request(window)
        }
    }

    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create Window")
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    async fn open_action(&self, path: String) -> anyhow::Result<()> {
        if path.is_empty() {
            let dialog = gtk::FileChooserNative::new(
                None,
                Some(self),
                gtk::FileChooserAction::Open,
                None,
                None,
            );

            let bib_filter = gtk::FileFilter::new();
            bib_filter.set_name(Some(&gettext("Bibtex Files")));
            bib_filter.add_mime_type("text/plain");
            bib_filter.add_mime_type("text/x-bibtex");
            bib_filter.add_pattern("*.bib");
            dialog.add_filter(&bib_filter);

            let response = dialog.run_future().await;
            if response == gtk::ResponseType::Accept {
                let file = dialog.file().unwrap();
                self.open_wrapper(&file).await?;
            }
        } else {
            let file = gio::File::for_path(&path);
            self.open_wrapper(&file).await?;
        }

        Ok(())
    }

    async fn new_biblio_action(&self) -> anyhow::Result<()> {
        let dialog = gtk::FileChooserNative::new(
            Some(&gettext("Create New Bibliography")),
            Some(self),
            gtk::FileChooserAction::Save,
            Some(&gettext("Create")),
            None,
        );

        let bib_filter = gtk::FileFilter::new();
        bib_filter.set_name(Some(&gettext("Bibtex Files")));
        bib_filter.add_mime_type("text/plain");
        bib_filter.add_mime_type("text/x-bibtex");
        bib_filter.add_pattern("*.bib");
        dialog.add_filter(&bib_filter);

        if gtk::ResponseType::Accept == dialog.run_future().await {
            let file = dialog.file().unwrap();
            let stream = file
                .create_future(gio::FileCreateFlags::NONE, glib::Priority::default())
                .await?;
            stream.close_future(glib::Priority::default()).await?;
            self.open_wrapper(&file).await?;
        }

        Ok(())
    }

    async fn open_wrapper(&self, file: &gio::File) -> anyhow::Result<()> {
        if self.imp().bib.get().is_some() {
            let app = self
                .application()
                .unwrap()
                .downcast::<crate::Application>()
                .unwrap();
            let window = Window::new(&app);
            let group = gtk::WindowGroup::new();
            group.add_window(&window);
            window.open(file).await?;

            window.present();
        } else {
            self.open(file).await?;
        }

        Ok(())
    }

    async fn open(&self, file: &gio::File) -> anyhow::Result<()> {
        let data = String::from_utf8(file.load_contents_future().await?.0)?;
        let settings = &self.imp().settings;
        let bib = cratebibtex::Bib::default();
        bib.parse(&data)?;

        self.imp().entry_list.populate(&bib);
        self.imp().bib.set(bib).unwrap();
        self.imp().file.replace(Some(file.clone()));
        self.imp().search_bar.set_key_capture_widget(Some(self));
        self.imp()
            .empty_state_stack
            .set_visible_child(&*self.imp().main_leaflet);

        self.action_set_enabled("win.new_entry", true);
        self.action_set_enabled("win.save", true);
        self.action_set_enabled("win.delete_entry", true);

        let mut recent_files = settings.strv("recent-files");
        let path = file
            .path()
            .unwrap()
            .to_str()
            .map(glib::GString::from)
            .unwrap();
        if !recent_files.contains(&path) {
            recent_files.push(path);
            settings
                .set_strv(
                    "recent-files",
                    &recent_files
                        .iter()
                        .map(glib::GString::as_str)
                        .collect::<Vec<&str>>(),
                )
                .unwrap();
        }

        Ok(())
    }

    fn populate_recent_files(&self) {
        let recent_files = self.imp().settings.strv("recent-files");
        let menu = gio::Menu::new();
        let section = gio::Menu::new();
        let mut some_item = false;

        for file in recent_files
            .iter()
            .rev()
            .map(glib::GString::to_string)
            .map(gio::File::for_path)
            .filter(|file| file.query_exists(gio::Cancellable::NONE))
        {
            let basename = file.basename().unwrap();
            let path = file.path().unwrap().to_str().map(String::from).unwrap();
            let action = format!("win.open::{path}");
            section.append(basename.to_str(), Some(&action));
            some_item = true;
        }
        menu.append_section(Some(&gettext("Recent Files")), &section);
        menu.freeze();

        if some_item {
            self.imp().split_button.set_menu_model(Some(&menu));
        } else {
            self.imp().split_button.set_menu_model(gio::MenuModel::NONE);
        }
    }

    pub fn bib(&self) -> cratebibtex::Bib {
        self.imp().bib.get().cloned().unwrap()
    }

    pub fn send_toast(&self, text: &str) {
        let toast = adw::Toast::new(text);
        self.imp().toast_overlay.add_toast(&toast);
    }

    pub fn new_entry_dialog(&self) {
        let dialog = crate::NewEntryDialog::new(self);

        crate::spawn!(
            glib::clone!(@weak self as window, @weak dialog => async move {
                if let Some((citation_key, type_)) = dialog.run().await {
                    if window.imp().bib.get().unwrap().key_exists(&citation_key) {
                        window.send_toast(&gettext("Entry with key {citation_key} already exists"));
                    } else {
                        window.new_entry(&citation_key, type_);
                    }
                }
            })
        );
    }

    pub fn new_entry(&self, citation_key: &str, type_: cratebibtex::EntryType) {
        let entry = cratebibtex::Entry::new(citation_key, type_);
        let pos = self.imp().bib.get().unwrap().add_entry(entry);
        self.imp().entry_list.set_selected(pos);
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let content = self.bib().serialize();
        let file = self.imp().file.borrow().clone().unwrap();
        file.replace_contents_future(content, None, true, gio::FileCreateFlags::NONE)
            .await
            .map_err(|err| err.1)?;

        self.send_toast(&gettext("Bibliography saved"));

        Ok(())
    }

    pub fn search_google_schoolar_entry(&self) -> anyhow::Result<()> {
        use anyhow::Context;

        let entry = self.imp().entry_page.entry().context("No entry selected")?;

        let author = entry.find_tag_pretty("author").unwrap_or_else(|| "".into());
        let title = entry.find_tag_pretty("title").unwrap_or_else(|| "".into());

        let uri = utils::build_google_schoolar_url(&author, &title)?;
        log::debug!("Trying to open {uri}");

        gtk::show_uri(Some(self), &uri, gdk::CURRENT_TIME);

        Ok(())
    }

    pub fn search_arxiv_entry(&self) -> anyhow::Result<()> {
        use anyhow::Context;

        let entry = self.imp().entry_page.entry().context("No entry selected")?;

        let author = entry.find_tag_pretty("author").unwrap_or_else(|| "".into());
        let title = entry.find_tag_pretty("title").unwrap_or_else(|| "".into());

        let uri = utils::build_arxiv_url(&author, &title)?;
        log::debug!("Trying to open {uri}");

        gtk::show_uri(Some(self), &uri, gdk::CURRENT_TIME);

        Ok(())
    }

    fn delete_entry(&self) {
        if let Some(entry) = self.imp().entry_page.entry() {
            if let Some(pos) = self.bib().find_pos(&entry) {
                let toast = adw::Toast::new(&gettext("Entry Deleted"));
                toast.set_action_name(Some("win.undo_delete"));
                toast.set_button_label(Some(&gettext("Undo")));
                toast.connect_dismissed(glib::clone!(@weak self as obj => move |_| {
                    obj.imp().undo_entry.replace(None);
                }));

                if let Some((_, _, toast)) =
                    self.imp()
                        .undo_entry
                        .replace(Some((entry, pos, toast.clone())))
                {
                    toast.dismiss();
                };
                self.bib().remove_pos(pos);

                self.imp().toast_overlay.add_toast(&toast);
                self.imp()
                    .main_leaflet
                    .navigate(adw::NavigationDirection::Back);
            };
        };
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialog::builder()
            .logo_icon_name(APP_ID)
            .license_type(gtk::License::Gpl30)
            .comments(&gettext("Manage your bibliography"))
            .website("https://gitlab.gnome.org/World/citations/")
            .version(VERSION)
            .transient_for(self)
            .translator_credits(&gettext("translator-credits"))
            .modal(true)
            .authors(vec!["Maximiliano Sandoval <msandova@gnome.org>".into()])
            .artists(vec!["Maximiliano Sandoval <msandova@gnome.org>".into()])
            .build();

        dialog.present();
    }
}
