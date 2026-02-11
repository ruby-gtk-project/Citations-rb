// SPDX-License-Identifier: GPL-3.0-or-later
use gettextrs::gettext;
use gtk::subclass::prelude::*;
use gtk::{CompositeTemplate, prelude::*};
use gtk::{gdk, glib};

mod imp {
    use super::*;

    use glib::Properties;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/org/gnome/World/Citations/ui/entry_list_row.ui")]
    #[properties(wrapper_type = super::EntryListRow)]
    pub struct EntryListRow {
        #[template_child]
        pub child: TemplateChild<gtk::Widget>,
        #[template_child]
        pub author: TemplateChild<gtk::Label>,
        #[template_child]
        pub popover: TemplateChild<gtk::PopoverMenu>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub year_label: TemplateChild<gtk::Label>,

        #[property(get, set = Self::set_entry, explicit_notify)]
        entry: RefCell<Option<cratebibtex::Entry>>,

        binding_group: glib::BindingGroup,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EntryListRow {
        const NAME: &'static str = "EntryListRow";
        type Type = super::EntryListRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            klass.set_layout_manager_type::<gtk::BinLayout>();

            // Action used only for the extra menu
            klass.install_action_async("entry-row.open-pdf", None, async |row, _, _| {
                let citation_key = row.entry().unwrap().citation_key();
                let window = row.root().and_downcast::<crate::Window>().unwrap();

                if let Err(err) = crate::utils::open_pdf(&citation_key, &window).await {
                    log::error!("Could not open pdf: {err}");
                }
            });

            klass.install_action("entry-row.delete", None, |row, _, _| {
                let window = row.root().and_downcast::<crate::Window>().unwrap();
                window.delete_entry(&row.entry().unwrap());
            });

            klass.install_action("entry-row.copy-key", None, |row, _, _| {
                let citation_key = row.entry().unwrap().citation_key();
                row.clipboard().set_text(&citation_key);

                let window = row.root().and_downcast::<crate::Window>().unwrap();
                window.send_toast(&gettext("Copied"));
            });

            klass.install_action("entry-row.popup", None, |row, _, _| {
                row.open_context_menu(-1.0, -1.0);
            });

            klass.add_binding_action(
                gdk::Key::Menu,
                gdk::ModifierType::NO_MODIFIER_MASK,
                "entry-row.popup",
            );

            klass.add_binding_action(
                gdk::Key::F10,
                gdk::ModifierType::SHIFT_MASK,
                "entry-row.popup",
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl EntryListRow {
        fn set_entry(&self, entry: Option<cratebibtex::Entry>) {
            if self.entry.borrow().as_ref() == entry.as_ref() {
                return;
            }

            self.binding_group.set_source(entry.as_ref());

            self.entry.replace(entry);

            self.obj().notify_entry();
        }

        #[template_callback]
        fn menu_gesture_pressed(
            gesture: &gtk::Gesture,
            _n_press: i32,
            x: f64,
            y: f64,
            obj: &super::EntryListRow,
        ) {
            if gesture
                .current_event()
                .is_none_or(|e| !e.triggers_context_menu())
            {
                gesture.set_state(gtk::EventSequenceState::Denied);
                return;
            }

            obj.open_context_menu(x, y);
            gesture.set_state(gtk::EventSequenceState::Claimed);
            gesture.reset();
        }

        #[template_callback]
        fn menu_long_gesture_pressed(
            gesture: &gtk::Gesture,
            x: f64,
            y: f64,
            obj: &super::EntryListRow,
        ) {
            obj.open_context_menu(x, y);
            gesture.set_state(gtk::EventSequenceState::Claimed);
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for EntryListRow {
        fn dispose(&self) {
            self.dispose_template();
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.obj()
                .update_property(&[gtk::accessible::Property::HasPopup(true)]);

            self.binding_group
                .bind("author", &self.author.get(), "label")
                .sync_create()
                .transform_to(|_binding, value| {
                    value
                        .get::<String>()
                        .ok()
                        .map(|s| pretty_author(&s).to_value())
                })
                .build();

            self.binding_group
                .bind("year", &self.year_label.get(), "label")
                .sync_create()
                .build();

            self.binding_group
                .bind("title", &self.title_label.get(), "label")
                .sync_create()
                .transform_to(|_binding, value| {
                    value
                        .get::<String>()
                        .ok()
                        .map(|s| pretty_title(&s).to_value())
                })
                .build();
        }
    }
    impl WidgetImpl for EntryListRow {}
}

glib::wrapper! {
    pub struct EntryListRow(ObjectSubclass<imp::EntryListRow>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Default for EntryListRow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EntryListRow {
    fn open_context_menu(&self, x: f64, y: f64) {
        let imp = self.imp();

        let has_pdf = crate::utils::has_pdf(&self.entry().unwrap().citation_key());
        self.action_set_enabled("entry-row.open-pdf", has_pdf);

        let rectangle = if x > -0.5 && y > -0.5 {
            gdk::Rectangle::new(x as i32, y as i32, 0, 0)
        } else if self.direction() == gtk::TextDirection::Rtl {
            gdk::Rectangle::new(self.width(), self.height(), 0, 0)
        } else {
            gdk::Rectangle::new(0, self.height(), 0, 0)
        };
        imp.popover.set_pointing_to(Some(&rectangle));
        imp.popover.popup();
    }
}

fn pretty_title(title: &str) -> String {
    let title = texer::texer(title);
    if title.is_empty() {
        gettext("Title not set")
    } else {
        title.into_owned()
    }
}

fn pretty_author(author: &str) -> String {
    let author = texer::texer(author);
    if author.is_empty() {
        gettext("Author not set")
    } else {
        author.into_owned()
    }
}
