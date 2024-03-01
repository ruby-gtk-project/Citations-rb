// SPDX-License-Identifier: GPL-3.0-or-later
use gettextrs::gettext;
use gio::prelude::*;
use gtk::{gio, glib};
use std::path::PathBuf;
use tokio;

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
        // TRANSLATORS Method of publication, see https://www.bibtex.com/f/howpublished-field/
        "howpublished" => gettext("How Published"),
        "note" => gettext("Note"),
        "booktitle" => gettext("Book Title"),
        "series" => gettext("Series"),
        "archiveprefix" => gettext("Archive Prefix"),
        // TRANSLATORS As in digital print
        "eprint" => gettext("ePrint"),
        // TRANSLATORS Identifier used by arXiv, see https://arxiv.org/help/hypertex/bibstyles
        "primaryclass" => gettext("Primary Class"),
        "month" => gettext("Month"),
        // TRANSLATORS As in chief editor
        "editor" => gettext("Editor"),
        // TRANSLATORS As in association
        "organization" => gettext("Organization"),
        "school" => gettext("School"),
        "institution" => gettext("Institution"),
        "issn" => "ISSN".to_string(),
        "isbn" => "ISBN".to_string(),
        _ => unreachable!(),
    }
}

pub fn build_google_schoolar_url(author: &str, title: &str) -> anyhow::Result<url::Url> {
    let authors = cratebibtex::format::format_authors(author);

    let url = format!("https://scholar.google.com/scholar?q={authors} {title}&scisbd=0&num=10&start=0&safe=active&filter=1&as_vis=1");

    Ok(url::Url::parse(&url)?)
}

pub fn build_arxiv_url(author: &str, title: &str) -> anyhow::Result<url::Url> {
    let authors = cratebibtex::format::format_authors(author);
    let url = format!(
        "https://arxiv.org/search/advanced?advanced=&terms-0-operator=AND&terms-0-term={title}&terms-0-field=title&terms-1-operator=OR&terms-1-term={authors}&terms-1-field=author&date-filter_by=all_dates&date-year=&date-from_date=&date-to_date=&date-date_type=submitted_date&classification-physics_archives=all&classification-include_cross_list=include&abstracts=show&size=25&order=-announced_date_first",
    );

    Ok(url::Url::parse(&url)?)
}

pub fn bibliography_path() -> Option<PathBuf> {
    // TODO Use a user defined directory, see
    // https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome/-/issues/41
    glib::user_special_dir(glib::UserDirectory::Documents).map(|path| path.join("Bibliography"))
}

pub fn bibliography_path_string() -> String {
    bibliography_path()
        .map(|path| {
            path.to_str()
                .unwrap()
                .replace(glib::home_dir().to_str().unwrap(), "~")
        })
        .unwrap_or_else(|| "~/Documents/Bibliography".to_string())
}

pub fn pdf_filename(citation_key: &str) -> String {
    sanitize_filename::sanitize(format!("{citation_key}.pdf"))
}

pub(crate) fn remove_recent_file(settings: &gio::Settings, path: &str) {
    settings
        .set_strv(
            "recent-files",
            settings
                .strv("recent-files")
                .into_iter()
                .filter(|x| x != path)
                .collect::<glib::StrV>(),
        )
        .unwrap();
}

pub(crate) fn eprint_url(url: &str) -> anyhow::Result<url::Url> {
    if url.is_empty() {
        anyhow::bail!("String is empty");
    }

    let uri = if url.contains("https://") || url.contains("http://") {
        url::Url::parse(url)?
    } else {
        // TODO Read archive prefix to deduce if it is not an arXiv
        let pre_validated = format!(
            "https://arxiv.org/abs/{}",
            url.replace("arxiv.org/abs/", "").replace("arxiv.org/", "")
        );

        url::Url::parse(&pre_validated)?
    };

    Ok(uri)
}

pub(crate) fn doi_url(url: &str) -> anyhow::Result<url::Url> {
    if url.is_empty() {
        anyhow::bail!("String is empty");
    }

    let uri = if url.contains("https://") || url.contains("http://") {
        url::Url::parse(url)?
    } else {
        let pre_validated = format!("https://doi.org/{}", url.replace("doi.org/", ""));

        url::Url::parse(&pre_validated)?
    };

    Ok(uri)
}

pub(crate) fn has_pdf(citation_key: &str) -> bool {
    if let Some(path) = crate::utils::bibliography_path() {
        let folder = gio::File::for_path(path);
        let pdf = folder.child(crate::utils::pdf_filename(citation_key));

        pdf.query_exists(gio::Cancellable::NONE)
    } else {
        false
    }
}

pub(crate) async fn open_pdf<W: glib::object::IsA<gtk::Window>>(
    citation_key: &str,
    window: &W,
) -> anyhow::Result<()> {
    if let Some(path) = crate::utils::bibliography_path() {
        let folder = gio::File::for_path(&path);
        let pdf = folder.child(crate::utils::pdf_filename(citation_key));
        let launcher = gtk::FileLauncher::new(Some(&pdf));

        launcher.launch_future(Some(window)).await?;

        Ok(())
    } else {
        anyhow::bail!("There is not bibliography path");
    }
}

fn set_default_bibkey(bib: &str) -> String {
    let entry = cratebibtex::Entry::from_bibtex(&bib).unwrap();
    let authors = entry.author();
    let lastname = authors.split_once(&[' ', ',']).map(|(v, _)| v).unwrap();
    let bib_key = format!("{}{}", lastname.to_lowercase(), entry.year());

    entry.set_citation_key(bib_key);
    entry.serialize()
}

pub(crate) fn doi2bib(doi: &str) -> anyhow::Result<String> {
    let doi2bib = doi2bib::Doi2Bib::new().unwrap();
    let doi_str = doi
        .replace("https://", "")
        .replace("http://", "")
        .replace("doi.org/", "");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let bib = rt.block_on(doi2bib.resolve_doi(&doi_str))?;
    Ok(set_default_bibkey(&bib))
}
