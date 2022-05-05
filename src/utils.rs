// SPDX-License-Identifier: GPL-3.0-or-later
use gettextrs::gettext;

#[macro_export]
macro_rules! spawn {
    ($future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local($future);
    };
    ($priority:expr, $future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local_with_priority($priority, $future);
    };
}

pub fn translated_key(key: &str) -> String {
    match key {
        "author" => gettext("Author"),
        "year" => gettext("Year"),
        "title" => gettext("Title"),
        "volume" => gettext("Volume"),
        "number" => gettext("Number"),
        "pages" => gettext("Pages"),
        "publisher" => gettext("Publisher"),
        "journal" => gettext("Journal"),
        "address" => gettext("Address"),
        "howpublished" => gettext("How Published"),
        "note" => gettext("Note"),
        "booktitle" => gettext("Book Title"),
        "series" => gettext("Series"),
        "archiveprefix" => gettext("Archive Prefix"),
        "eprint" => gettext("ePrint"),
        "primaryclass" => gettext("Primary Class"),
        "month" => gettext("Month"),
        "editor" => gettext("Editor"),
        "organization" => gettext("Organization"),
        "school" => gettext("School"),
        "institution" => gettext("Institution"),
        "issn" => "ISSN".to_string(),
        "isbn" => "ISBN".to_string(),
        _ => unreachable!(),
    }
}
