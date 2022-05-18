// SPDX-License-Identifier: GPL-3.0-or-later
mod application;
#[rustfmt::skip]
mod citation_preview;
mod config;
mod entry_list;
mod entry_list_row;
mod entry_page;
mod new_entry_dialog;
mod preferences_window;
mod utils;
mod window;

use gettextrs::{gettext, LocaleCategory};
use gtk::{gio, glib, prelude::*};

pub use citation_preview::CitationPreview;
pub use entry_list::EntryList;
pub use entry_list_row::EntryListRow;
pub use entry_page::EntryPage;
pub use new_entry_dialog::NewEntryDialog;
pub use preferences_window::PreferencesWindow;
pub use window::Window;

use self::application::Application;
use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};

fn main() {
    // Initialize logger
    tracing_subscriber::fmt::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");
    cratebibtex::init(GETTEXT_PACKAGE, LOCALEDIR);

    // NOTE: This is the app name
    glib::set_application_name(&gettext("Citations"));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = Application::new();

    // Define gobject types
    EntryList::static_type();
    EntryListRow::static_type();
    EntryPage::static_type();
    CitationPreview::static_type();
    Window::static_type();
    NewEntryDialog::static_type();
    PreferencesWindow::static_type();

    app.run();
}
