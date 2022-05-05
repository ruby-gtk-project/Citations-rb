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

pub fn build_google_schoolar_url(
    author: &str,
    title: &str,
    year: Option<String>,
) -> anyhow::Result<String> {
    let mut url = "https://scholar.google.com/scholar?".to_string();

    url.push_str(&format!("q={author} {title}"));

    if let Some(i) = year {
        // from
        url.push_str(&format!("&as_ylo={i}"));
        // to
        url.push_str(&format!("&as_yhi={i}"));
    }
    url.push_str("&scisbd=0");

    // Lang
    // if let Some(i) = lang {
    //     url.push_str("&hl=");
    //     url.push_str(i);
    // }
    url.push_str("&num=10");
    url.push_str("&start=0");
    url.push_str("&safe=active");
    url.push_str("&filter=1");
    url.push_str("&as_vis=1");

    let sanitized = url::Url::parse(&url)?;
    Ok(sanitized.to_string())
}
