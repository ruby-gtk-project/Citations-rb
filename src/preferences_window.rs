use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{prelude::*, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        #[template_child]
        pub bibliography_button_content: TemplateChild<adw::ButtonContent>,

        pub settings: gio::Settings,
    }

    impl Default for PreferencesWindow {
        fn default() -> Self {
            Self {
                bibliography_button_content: TemplateChild::default(),
                settings: gio::Settings::new(crate::config::APP_ID),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "PreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl PreferencesWindow {
        #[template_callback]
        fn on_bibliography_button_clicked(window: &super::PreferencesWindow) {
            let dialog = gtk::FileChooserNative::new(
                None,
                Some(window),
                gtk::FileChooserAction::SelectFolder,
                None,
                None,
            );

            dialog.connect_response(
                glib::clone!(@weak window, @strong dialog => move |_dialog, response| {
                    if response == gtk::ResponseType::Accept {
                        let folder = dialog.file().unwrap();
                        window.set_folder(&folder);
                    }
                    dialog.destroy();
                }),
            );

            dialog.show();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if let Some(path) = crate::utils::bibliography_path() {
                self.bibliography_button_content
                    .set_label(path.to_str().unwrap());
            }
        }
    }

    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
}

glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

impl PreferencesWindow {
    pub fn new(parent_window: Option<&impl IsA<gtk::Window>>) -> Self {
        glib::Object::new(&[("transient-for", &parent_window), ("modal", &true)]).unwrap()
    }

    fn set_folder(&self, folder: &gio::File) {
        let path = folder.path().unwrap().to_str().map(String::from).unwrap();
        self.imp().bibliography_button_content.set_label(&path);

        self.imp()
            .settings
            .set_string("bibliography-path", &path)
            .unwrap();
    }
}
