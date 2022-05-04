use glib::StaticType;

pub mod format;

mod bib;
mod entry;
mod entry_type;
mod parser;
mod utils;

pub use bib::Bib;
pub use entry::Entry;
pub use entry::EntryExt;
pub use entry::EntryImpl;
pub use entry_type::EntryType;
pub use format::Format;

pub fn init(gettext_package: &str, localedir: &str) {
    // Prepare i18n
    gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(gettext_package, localedir).expect("Unable to bind the text domain");
    gettextrs::textdomain(gettext_package).expect("Unable to switch to the text domain");

    Entry::static_type();
    EntryType::static_type();
    Bib::static_type();
}

#[cfg(test)]
mod tests {
    use super::*;

    use gio::prelude::*;

    const BIBFILE_DATA: &str = "@preamble{
        \"A bibtex preamble\" # \" another test\"
    }

    @Comment{
        Here is a comment.
    }

    Another comment!

    @string ( name= \"Charles Vandevoorde\")
    @string (github = \"https://github.com/charlesvdv\")

    @misc {my_citation_key,
        AUTHOR= name,
        title = \"nom-bibtex\",
        note = \"Github: \" # github
    }
";

    #[test]
    fn test() {
        let bib = Bib::default();
        bib.parse(BIBFILE_DATA).unwrap();

        let entry = bib.item(0).unwrap().downcast::<Entry>().unwrap();
        assert_eq!(entry.find_tag("author").unwrap(), "Charles Vandevoorde");
    }
}
