// SPDX-License-Identifier: GPL-3.0-or-later
use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gsv::prelude::*;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

const SAMPLE_ENTRY: &str = r"@book{lamport1985i1,
  title={\LaTeX: A Document Preparation System},
  author={Lamport, Leslie},
  year={1991},
  publisher={Addison-Wesley}
}";

mod imp {
    use super::*;

    use std::cell::RefCell;
    use std::sync::Once;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/new_entry_dialog.ui")]
    pub struct NewEntryDialog {
        #[template_child]
        pub combo_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub source_view: TemplateChild<gsv::View>,
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,

        pub sender: RefCell<Option<futures_channel::oneshot::Sender<Option<String>>>>,
        pub remove_text: Once,
    }

    impl Default for NewEntryDialog {
        fn default() -> Self {
            Self {
                combo_row: TemplateChild::default(),
                entry_row: TemplateChild::default(),
                source_view: TemplateChild::default(),
                view_stack: TemplateChild::default(),
                sender: RefCell::default(),
                remove_text: Once::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewEntryDialog {
        const NAME: &'static str = "NewEntryDialog";
        type Type = super::NewEntryDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl NewEntryDialog {
        #[template_callback]
        fn entry_type_name(item: &adw::EnumListItem) -> String {
            let type_ = cratebibtex::EntryType::from(item.value());
            type_.to_translatable_string()
        }

        #[template_callback]
        fn on_cancel_clicked(&self) {
            let sender = self.sender.take().unwrap();
            sender.send(None).unwrap();
            self.instance().destroy();
        }

        #[template_callback]
        fn on_create_clicked(&self) {
            let sender = self.sender.take().unwrap();

            let data = match &self.view_stack.visible_child_name().as_deref() {
                Some("new") => {
                    let citation_key = self.entry_row.text().to_string();
                    let type_ =
                        cratebibtex::EntryType::from(self.combo_row.selected() as i32).to_string();
                    format!("@{type_}{{{citation_key},}}")
                }
                Some("from_bibtex") => {
                    let buffer = self.source_view.buffer();
                    let (start, end) = buffer.bounds();
                    buffer.text(&start, &end, true).into()
                }
                _ => unreachable!(),
            };
            sender.send(Some(data)).unwrap();

            self.instance().destroy();
        }
    }

    impl ObjectImpl for NewEntryDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let language_manager = gsv::LanguageManager::default();
            let lang = language_manager.language("bibtex").unwrap();
            let buffer = gsv::Buffer::with_language(&lang);

            buffer.set_highlight_syntax(true);
            buffer.set_highlight_matching_brackets(false);
            self.source_view.set_buffer(Some(&buffer));

            obj.update_style_scheme();
            let manager = adw::StyleManager::default();
            manager.connect_dark_notify(glib::clone!(@weak obj => move |_| {
                obj.update_style_scheme();
            }));

            // TODO Use placeholder-text see
            // https://gitlab.gnome.org/GNOME/gtk/-/merge_requests/4771
            let instructions = gettext("Insert a BibTeX citation here");
            buffer.set_text(&format!("{instructions}\n\n{SAMPLE_ENTRY}"));

            let controller = gtk::EventControllerFocus::new();
            controller.connect_enter(glib::clone!(@weak obj => move |_| {
                obj.imp().remove_text.call_once(glib::clone!(@weak obj => move || {
                    let buffer = obj.imp().source_view.buffer();
                    buffer.set_text("");
                }));
            }));
            self.source_view.get().add_controller(&controller);
        }
    }

    impl WidgetImpl for NewEntryDialog {}
    impl WindowImpl for NewEntryDialog {}
    impl AdwWindowImpl for NewEntryDialog {}
}

glib::wrapper! {
    pub struct NewEntryDialog(ObjectSubclass<imp::NewEntryDialog>)
        @extends gtk::Widget, adw::Window, gtk::Window;
}

impl Default for NewEntryDialog {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

impl NewEntryDialog {
    pub fn new<T: glib::IsA<gtk::Window>>(window: &T) -> Self {
        glib::Object::new(&[("modal", &true), ("transient-for", &window)]).unwrap()
    }

    pub async fn run(&self) -> Option<String> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        self.imp().sender.replace(Some(sender));

        self.present();

        if let Ok(response) = receiver.await {
            response
        } else {
            None
        }
    }

    fn update_style_scheme(&self) {
        let manager = adw::StyleManager::default();
        let scheme_name = if manager.is_dark() {
            "Adwaita-dark"
        } else {
            "Adwaita"
        };
        let scheme = gsv::StyleSchemeManager::default()
            .scheme(scheme_name)
            .unwrap();

        let buffer = self
            .imp()
            .source_view
            .buffer()
            .downcast::<gsv::Buffer>()
            .unwrap();
        buffer.set_style_scheme(Some(&scheme));
    }
}
