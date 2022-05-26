// SPDX-License-Identifier: GPL-3.0-or-later
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};

mod imp {
    use super::*;

    use adw::subclass::prelude::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/World/Citations/ui/entry_list.ui")]
    pub struct EntryList {
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub empty_list: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        pub selection_model: RefCell<gtk::SingleSelection>,
        pub filter_model: gtk::FilterListModel,
        pub filter: gtk::StringFilter,
        pub id: RefCell<Option<glib::SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EntryList {
        const NAME: &'static str = "EntryList";
        type Type = super::EntryList;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EntryList {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "entry-selected",
                    &[Option::<cratebibtex::Entry>::static_type().into()],
                    <()>::static_type().into(),
                )
                .run_last()
                .build()]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let selection = gtk::SingleSelection::new(Some(&self.filter_model));
            selection.set_can_unselect(true);
            selection.set_autoselect(false);
            selection.connect_selected_notify(glib::clone!(@weak obj => move |model| {
                let pos = model.selected();
                if let Some(entry) = model.item(pos) {
                    let entry = entry
                        .downcast::<cratebibtex::Entry>()
                        .unwrap();
                    obj.entry_selected(Some(&entry));
                } else {
                    obj.entry_selected(None);
                }
            }));

            let factory = gtk::SignalListItemFactory::new();

            factory.connect_setup(|_, item| {
                let row = crate::EntryListRow::default();
                item.set_child(Some(&row));
            });

            factory.connect_bind(|_, item| {
                let child = item.child().unwrap();
                let row = child.downcast_ref::<crate::EntryListRow>().unwrap();

                let item = item.item().unwrap();
                row.set_entry(item.downcast_ref::<cratebibtex::Entry>().unwrap());
            });
            self.list_view.set_factory(Some(&factory));
            self.list_view.set_model(Some(&selection));

            self.selection_model.replace(selection);

            let search_expression = gtk::ClosureExpression::new::<String, &[gtk::Expression], _>(
                &[],
                glib::closure!(|entry: cratebibtex::Entry| {
                    let author = entry.find_tag("author").unwrap_or_else(|| "".into());
                    let title = entry.find_tag("title").unwrap_or_else(|| "".into());
                    let year = entry.find_tag("year").unwrap_or_else(|| "".into());

                    format!("{author} {title} {year}")
                }),
            );
            self.filter.set_expression(Some(&search_expression));
            self.filter
                .set_match_mode(gtk::StringFilterMatchMode::Substring);
            self.filter.set_ignore_case(true);

            self.filter_model.set_filter(Some(&self.filter));
        }
    }
    impl WidgetImpl for EntryList {}
    impl BinImpl for EntryList {}
}

glib::wrapper! {
    pub struct EntryList(ObjectSubclass<imp::EntryList>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for EntryList {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

impl EntryList {
    pub fn populate(&self, bib: &cratebibtex::Bib) {
        self.imp().filter_model.set_model(Some(bib));
        self.set_empty_view(bib.item(0).is_none());

        let id =
            bib.connect_items_changed(glib::clone!(@weak self as obj => move|model, _, _, _| {
                obj.set_empty_view(model.item(0).is_none());
            }));

        if let Some(old_id) = self.imp().id.replace(Some(id)) {
            bib.disconnect(old_id);
        }
    }

    pub fn filter(&self, query: &str) {
        self.imp().filter.set_search(Some(query));
    }

    fn set_empty_view(&self, is_empty: bool) {
        if is_empty {
            self.imp()
                .stack
                .set_visible_child(&self.imp().empty_list.get());
        } else {
            self.imp()
                .stack
                .set_visible_child(&self.imp().scrolled_window.get());
        }
    }

    fn entry_selected(&self, entry: Option<&cratebibtex::Entry>) {
        self.emit_by_name::<()>("entry-selected", &[&entry]);
    }

    pub fn set_selected(&self, pos: u32) {
        self.imp().selection_model.borrow().set_selected(pos);
        self.imp()
            .list_view
            .activate_action("list.scroll-to-item", Some(&pos.to_variant()))
            .unwrap();
    }

    pub fn unselect(&self) {
        let selection_model = self.imp().selection_model.borrow();
        selection_model.unselect_item(selection_model.selected());
    }
}
