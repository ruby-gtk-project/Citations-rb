// SPDX-License-Identifier: GPL-3.0-or-later
use log::{debug, info};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use crate::window::Window;

mod imp {
    use super::*;

    use adw::subclass::prelude::*;

    #[derive(Debug, Default)]
    pub struct Application;

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            debug!("Application::activate");
            self.parent_activate();

            let window = Window::new(&self.obj());
            let group = gtk::WindowGroup::new();
            group.add_window(&window);

            window.present();
        }

        fn startup(&self) {
            debug!("Application::startup");
            self.parent_startup();
            let app = self.obj();

            gsv::init();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_gactions();
            app.setup_accels();
        }

        fn open(&self, files: &[gio::File], _hint: &str) {
            debug!("Application::open");
            files
                .iter()
                .filter(|f| {
                    let exists = f.query_exists(gio::Cancellable::NONE);
                    if !exists {
                        log::error!("File {} does not exists", f.uri());
                    }
                    exists
                })
                .for_each(|f| {
                    let window = Window::new(&self.obj());
                    let group = gtk::WindowGroup::new();
                    group.add_window(&window);

                    crate::spawn!(glib::clone!(@weak window, @strong f => async move {
                        match window.open(&f).await {
                            Ok(_) => window.present(),
                            Err(err) => log::error!("Could not open {}: {err}", f.uri()),
                        }
                    }));
                });
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/org/gnome/World/Citations/")
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .build()
    }
}

impl Application {
    fn setup_gactions(&self) {
        let actions = [gio::ActionEntryBuilder::new("quit")
            .activate(|app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                if let Some(window) = app.active_window() {
                    window.close();
                }
                app.quit();
            })
            .build()];

        self.add_action_entries(actions);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("win.open('')", &["<Control>o"]);
        self.set_accels_for_action("win.save", &["<Control>s"]);
        self.set_accels_for_action("win.search", &["<Control>f"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
    }

    pub fn run(&self) -> glib::ExitCode {
        info!("Citations ({})", APP_ID);
        info!("Version: {} ({})", VERSION, PROFILE);
        info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}
