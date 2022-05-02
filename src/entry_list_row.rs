// SPDX-License-Identifier: GPL-3.0-or-later
use cratebibtex::EntryExt;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};
use once_cell::sync::Lazy;

mod imp {
    use super::*;

    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Papers/ui/entry_list_row.ui")]
    pub struct EntryListRow {
        #[template_child]
        pub child: TemplateChild<gtk::Widget>,
        #[template_child]
        pub author: TemplateChild<gtk::Label>,

        pub entry: RefCell<Option<cratebibtex::Entry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EntryListRow {
        const NAME: &'static str = "EntryListRow";
        type Type = super::EntryListRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EntryListRow {
        fn dispose(&self, _obj: &Self::Type) {
            self.child.unparent();
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
                "entry" => obj.set_entry(value.get().unwrap()),
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
        glib::Object::new(&[]).unwrap()
    }
}

impl EntryListRow {
    pub fn set_entry(&self, entry: &cratebibtex::Entry) {
        if let Some(old_entry) = self.imp().entry.replace(Some(entry.clone())) {
            if old_entry.citation_key() != entry.citation_key() {
                self.notify("entry");
            }
        } else {
            self.notify("entry");
        }
    }

    pub fn entry(&self) -> cratebibtex::Entry {
        self.imp().entry.borrow().clone().unwrap()
    }
}
