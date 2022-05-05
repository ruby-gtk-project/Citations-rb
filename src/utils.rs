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

    if let Some(year) = year {
        if !year.is_empty() && year.parse::<u32>().is_ok() {
            // from
            url.push_str(&format!("&as_ylo={year}"));
            // to
            url.push_str(&format!("&as_yhi={year}"));
        }
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

pub fn build_arxiv_url(author: &str, title: &str, year: Option<String>) -> anyhow::Result<String> {
    let mut url = format!("https://arxiv.org/search/advanced?advanced=&terms-0-operator=AND&terms-0-term={title}&terms-0-field=title&terms-1-operator=OR&terms-1-term={author}&terms-1-field=author&classification-physics_archives=all&classification-include_cross_list=include&abstracts=show&size=25&order=-announced_date_first");
    if let Some(year) = year {
        if !year.is_empty() && year.parse::<u32>().is_ok() {
            url.push_str(&format!("&date-filter_by=specific_year&date-year={year}&date-from_date=&date-to_date=&date-date_type=submitted_date"));
        } else {
            url.push_str("&date-filter_by=all_dates&date-year=&date-from_date=&date-to_date=&date-date_type=submitted_date");
        }
    } else {
        url.push_str("&date-filter_by=all_dates&date-year=&date-from_date=&date-to_date=&date-date_type=submitted_date");
    }
    let sanitized = url::Url::parse(&url)?;
    Ok(sanitized.to_string())
}
