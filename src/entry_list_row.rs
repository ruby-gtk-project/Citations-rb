// SPDX-License-Identifier: GPL-3.0-or-later
use gettextrs::gettext;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};
use gtk::{prelude::*, CompositeTemplate};
use once_cell::sync::Lazy;

mod imp {
    use super::*;

    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/entry_list_row.ui")]
    pub struct EntryListRow {
        #[template_child]
        pub child: TemplateChild<gtk::Widget>,
        #[template_child]
        pub author: TemplateChild<gtk::Label>,
        #[template_child]
        pub popover: TemplateChild<gtk::PopoverMenu>,

        pub entry: RefCell<Option<cratebibtex::Entry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EntryListRow {
        const NAME: &'static str = "EntryListRow";
        type Type = super::EntryListRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();

            // Action used only for the extra menu
            klass.install_action("entry-row.open-pdf", None, move |row, _, _| {
                let citation_key = row.entry().citation_key();
                let window = row.root().unwrap().downcast::<crate::Window>().unwrap();

                crate::utils::open_pdf(&citation_key, &window);
            });

            klass.install_action("entry-row.delete", None, move |row, _, _| {
                let window = row.root().unwrap().downcast::<crate::Window>().unwrap();
                window.delete_entry(&row.entry());
            });

            klass.install_action("entry-row.copy-key", None, move |row, _, _| {
                let citation_key = row.entry().citation_key();
                row.clipboard().set_text(&citation_key);

                let window = row.root().unwrap().downcast::<crate::Window>().unwrap();
                window.send_toast(&gettext("Copied"));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl EntryListRow {
        #[template_callback]
        fn pretty_author(&self, author: &str) -> String {
            let author = cratebibtex::format::texer(author);
            if author.is_empty() {
                "<i>Author not set</i>".to_string()
            } else {
                author.into_owned()
            }
        }

        #[template_callback]
        fn pretty_title(&self, title: &str) -> String {
            let title = cratebibtex::format::texer(title);
            if title.is_empty() {
                "<i>Title not set</i>".to_string()
            } else {
                title.into_owned()
            }
        }

        #[template_callback]
        fn menu_gesture_clicked(obj: &super::EntryListRow, _n_press: i32, x: f64, y: f64) {
            let has_pdf = crate::utils::has_pdf(&obj.entry().citation_key());
            obj.action_set_enabled("entry-row.open-pdf", has_pdf);

            if x > -1.0 && y > -1.0 {
                let rectangle = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                obj.imp().popover.set_pointing_to(Some(&rectangle));
            } else {
                obj.imp().popover.set_pointing_to(None);
            }
            obj.imp().popover.popup();
        }
    }

    impl ObjectImpl for EntryListRow {
        fn dispose(&self) {
            self.child.unparent();
            self.popover.unparent();
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
                "entry" => self.instance().set_entry(value.get().unwrap()),
                _ => unimplemented!(),
            };
        }
    }
    impl WidgetImpl for EntryListRow {}
}

glib::wrapper! {
    pub struct EntryListRow(ObjectSubclass<imp::EntryListRow>)
        @extends gtk::Widget;
}

impl Default for EntryListRow {
    fn default() -> Self {
        glib::Object::new(&[])
    }
}

impl EntryListRow {
    pub fn set_entry(&self, entry: &cratebibtex::Entry) {
        if Some(entry) != self.imp().entry.replace(Some(entry.clone())).as_ref() {
            self.notify("entry");
        }
    }

    pub fn entry(&self) -> cratebibtex::Entry {
        self.imp().entry.borrow().clone().unwrap()
    }
}
