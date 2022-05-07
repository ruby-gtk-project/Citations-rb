use glib::prelude::*;
use glib::subclass::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::rc::Rc;

use crate::Format;
use crate::{format, utils};

// TODO Make a better and more efficient parser
mod imp {
    use super::*;

    use once_cell::sync::Lazy;
    use std::cell::Cell;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct Entry {
        pub tags: RefCell<HashMap<String, String>>,
        pub entry_type: Cell<crate::EntryType>,
        pub citation_key: RefCell<Option<String>>,
        pub inner: RefCell<Option<Rc<nom_bibtex::Bibliography>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Entry {
        const NAME: &'static str = "Entry";
        type Type = super::Entry;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Entry {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                utils::new_props(&[
                    "author",
                    "year",
                    "title",
                    "volume",
                    "number",
                    "pages",
                    "publisher",
                    "journal",
                    "address",
                    "howpublished",
                    "note",
                    "booktitle",
                    "series",
                    "archiveprefix",
                    "eprint",
                    "primaryclass",
                    "month",
                    "editor",
                    "organization",
                    "school",
                    "institution",
                    "doi",
                    "url",
                    "issn",
                    "isbn",
                ])
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let pspec_name = pspec.name();
            match pspec_name {
                "citation-key" => obj.citation_key().to_value(),
                "entry-type" => obj.entry_type().to_value(),
                _ => obj
                    .find_tag(pspec_name)
                    .unwrap_or_else(|| "".into())
                    .to_value(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            let pspec_name = pspec.name();
            match pspec_name {
                "citation-key" => {
                    let value = value.get::<String>().unwrap_or_else(|_| "".into());
                    obj.set_citation_key(&value)
                }
                "entry-type" => obj.set_entry_type(value.get::<crate::EntryType>().unwrap()),
                _ => {
                    let value = value.get::<String>().unwrap_or_else(|_| "".into());
                    obj.set_prop(pspec_name, &value)
                }
            }
        }
    }
    impl crate::EntryImpl for Entry {}
}

glib::wrapper! {
    pub struct Entry(ObjectSubclass<imp::Entry>);
}

impl Default for Entry {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

impl From<&nom_bibtex::Bibliography> for Entry {
    fn from(bib: &nom_bibtex::Bibliography) -> Self {
        let tags = bib.tags();
        let entry = Self::default();
        entry.imp().tags.replace(tags);
        entry
            .imp()
            .citation_key
            .replace(Some(bib.citation_key().into()));
        entry.imp().entry_type.replace(bib.entry_type().into());

        entry
    }
}

impl Entry {
    pub fn set_entry_type(&self, entry_type: crate::EntryType) {
        if entry_type != self.imp().entry_type.replace(entry_type) {
            self.notify("entry-type");
        }
    }

    pub fn set_citation_key(&self, citation_key: &str) {
        let citation_key = citation_key.to_string();
        if Some(citation_key.clone()) != self.imp().citation_key.replace(Some(citation_key)) {
            self.notify("citation-key");
        }
    }

    pub fn tags(&self) -> HashMap<String, String> {
        self.imp().tags.borrow().clone()
    }

    pub fn new(citation_key: &str, type_: crate::EntryType) -> Self {
        let entry = Self::default();
        entry
            .imp()
            .citation_key
            .replace(Some(citation_key.to_string()));
        entry.imp().entry_type.replace(type_);

        entry
    }

    pub fn find_tag_pretty(&self, tag: &str) -> Option<String> {
        let value = self.find_tag(tag)?;

        Some(format::texer(&value))
    }

    pub fn fields(&self) -> Vec<String> {
        self.entry_type().fields()
    }

    pub fn format_citation(&self, citation: &str, format: Format) -> String {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"%(?P<tag>[A-Z]+)%").unwrap());

        let mut formated = String::from(citation);

        match format {
            Format::Plain => {
                formated = format::format_plain(citation);
            }
            Format::Markdown => (),
            Format::LaTeX => {
                formated = format::format_latex(citation, &self.citation_key());
            }
        }

        let name_prop = match format {
            Format::LaTeX => self.find_tag("author"),
            _ => self.find_tag_pretty("author"),
        };
        formated = formated.replace(
            "%AUTHOR%",
            &format::format_authors(name_prop.unwrap_or_else(|| "".into()).as_str()),
        );

        formated = RE
            .replace_all(&formated, |capture: &regex::Captures| {
                let tag = capture["tag"].to_lowercase();
                let opt_prop = match format {
                    Format::LaTeX => self.find_tag(&tag),
                    _ => self.find_tag_pretty(&tag),
                };

                opt_prop.unwrap_or_else(|| "".to_string())
            })
            .to_string();

        match format {
            Format::Markdown => format::clear_citation(&format::clear_markdown(&formated)),
            Format::LaTeX => {
                format::clear_citation(&format::clear_non_markdown(&format::clear_latex(&formated)))
            }
            Format::Plain => format::clear_citation(&format::clear_non_markdown(&formated)),
        }
    }

    pub fn serialize(&self) -> String {
        let citation_key = self.citation_key();
        let type_ = self.entry_type().to_string();
        let mut entry = format!("@{type_}{{{citation_key},\n");

        let lines = self
            .tags()
            .iter()
            .map(|(key, value)| format!("    {key} = {{{value}}},"))
            .collect::<Vec<String>>();
        entry += &lines.join("\n");

        entry += "\n}";

        entry
    }
}

pub trait EntryExt {
    fn find_tag(&self, tag: &str) -> Option<String>;
    fn insert_tag(&self, tag: &str, value: &str) -> Option<String>;
    fn set_prop(&self, tag: &str, value: &str);
    fn citation_key(&self) -> String;
    fn entry_type(&self) -> crate::EntryType;
}

impl<O: glib::IsA<Entry>> EntryExt for O {
    fn citation_key(&self) -> String {
        self.as_ref().imp().citation_key.borrow().clone().unwrap()
    }
    fn entry_type(&self) -> crate::EntryType {
        self.as_ref().imp().entry_type.get()
    }
    fn find_tag(&self, tag: &str) -> Option<String> {
        self.as_ref().imp().tags.borrow().get(tag).cloned()
    }

    fn insert_tag(&self, tag: &str, value: &str) -> Option<String> {
        let mut hashmap = self.as_ref().imp().tags.borrow_mut();

        hashmap.insert(tag.to_string(), value.into())
    }

    fn set_prop(&self, tag: &str, value: &str) {
        let old_value = self.insert_tag(tag, value);
        if old_value != Some(value.into()) {
            self.as_ref().notify(tag);
        }
    }
}

pub trait EntryImpl: ObjectImpl {}

unsafe impl<T: EntryImpl> IsSubclassable<T> for Entry {
    fn class_init(class: &mut glib::Class<Self>) {
        <glib::Object as IsSubclassable<T>>::class_init(class);
    }
    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <glib::Object as IsSubclassable<T>>::instance_init(instance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Bib;
    use gio::prelude::ListModelExt;

    const BIBFILE_DATA: &str = r#"
    @misc {my_citation_key,
        AUTHOR= "Some Author",
        title = "Some Title",
        year = "2020",
        volume = "5",
    }
"#;
    #[test]
    fn format() {
        let bib = Bib::default();
        bib.parse(BIBFILE_DATA).unwrap();

        let entry = bib.item(0).unwrap().downcast::<Entry>().unwrap();

        let format = r#"%AUTHOR%: "%TITLE%" (%YEAR%)"#;
        let expected = r#"Some Author: "Some Title" (2020)"#;
        assert_eq!(entry.format_citation(format, Format::Plain), expected);

        let format = r#"%AUTHOR%: "_%TITLE%_" (**%VOLUME%**) (%YEAR%)"#;
        let expected_plain = r#"Some Author: "Some Title" (5) (2020)"#;
        let expected_markdown = r#"Some Author: "_Some Title_" (**5**) (2020)"#;
        let expected_latex =
            r#"\bibitem{my_citation_key} Some Author: "\emph{Some Title}" (\textbf{5}) (2020)"#;
        assert_eq!(entry.format_citation(format, Format::Plain), expected_plain);
        assert_eq!(
            entry.format_citation(format, Format::Markdown),
            expected_markdown
        );
        assert_eq!(entry.format_citation(format, Format::LaTeX), expected_latex);

        let format = r#"%AUTHOR%, %TITLE%, %VOLUME%, %JOURNAL%, %YEAR%"#;
        let expected = r#"Some Author, Some Title, 5, 2020"#;
        assert_eq!(entry.format_citation(format, Format::Plain), expected);

        let format = r#"%AUTHOR%, %TITLE%, %VOLUME%, "%JOURNAL%", %YEAR%"#;
        let expected = r#"Some Author, Some Title, 5, 2020"#;
        assert_eq!(entry.format_citation(format, Format::Plain), expected);

        let format = r#"%AUTHOR%, %TITLE%, %VOLUME%, (%JOURNAL%), "%FAKE%", %YEAR%"#;
        let expected = r#"Some Author, Some Title, 5, 2020"#;
        assert_eq!(entry.format_citation(format, Format::Plain), expected);

        let format = r#"%AUTHOR%, "%TITLE%", %VOLUME% {%JOURNAL%}, '%FAKE%',%YEAR%"#;
        let expected = r#"Some Author, "Some Title", 5, 2020"#;
        assert_eq!(entry.format_citation(format, Format::Plain), expected);

        let format = r#"%AUTHOR%, "*%TITLE%*", %VOLUME% *%JOURNAL%*, '%FAKE%',%YEAR%"#;
        let expected = r#"\bibitem{my_citation_key} Some Author, "\emph{Some Title}", 5, 2020"#;
        assert_eq!(entry.format_citation(format, Format::LaTeX), expected);

        let format = r#"%AUTHOR%, "*%TITLE%*", %VOLUME% (*%JOURNAL%*), '%FAKE%',%YEAR%"#;
        let expected = r#"\bibitem{my_citation_key} Some Author, "\emph{Some Title}", 5, 2020"#;
        assert_eq!(entry.format_citation(format, Format::LaTeX), expected);
    }
}
