// SPDX-License-Identifier: GPL-3.0-or-later
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/new_entry_dialog.ui")]
    pub struct NewEntryDialog {
        #[template_child]
        pub combo_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub entry_row: TemplateChild<adw::EntryRow>,

        pub sender: RefCell<
            Option<futures_channel::oneshot::Sender<Option<(String, cratebibtex::EntryType)>>>,
        >,
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
        fn entry_type_name(item: adw::EnumListItem) -> String {
            let type_ = cratebibtex::EntryType::from(item.value() as u32);
            type_.to_translatable_string()
        }

        #[template_callback]
        fn on_cancel_clicked(&self) {
            let sender = self.sender.replace(None).unwrap();
            sender.send(None).unwrap();
            self.instance().destroy();
        }

        #[template_callback]
        fn on_create_clicked(&self) {
            // TODO verify there isnt another entry with that key already.
            let citation_key = self.entry_row.text().to_string();
            let type_ = cratebibtex::EntryType::from(self.combo_row.selected() as u32);
            let sender = self.sender.replace(None).unwrap();
            sender.send(Some((citation_key, type_))).unwrap();
            self.instance().destroy();
        }
    }

    impl ObjectImpl for NewEntryDialog {}
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

    pub async fn run(&self) -> Option<(String, cratebibtex::EntryType)> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        self.imp().sender.replace(Some(sender));

        self.present();

        if let Ok(response) = receiver.await {
            response
        } else {
            None
        }
    }
}
