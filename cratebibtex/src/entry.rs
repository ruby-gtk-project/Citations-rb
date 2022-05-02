use glib::prelude::*;
use glib::subclass::prelude::*;
use once_cell::sync::OnceCell;
use regex::Regex;
use std::collections::HashMap;
use std::rc::Rc;

use crate::utils;
use crate::Format;

// TODO Move formating stuff into formating.rs
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
                    "p-author",
                    "p-title",
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
            if let Some(name) = pspec_name.strip_prefix("p-") {
                obj.find_tag_pretty(name)
                    .unwrap_or_else(|| "".into())
                    .to_value()
            } else {
                match pspec_name {
                    "citation-key" => obj.citation_key().to_value(),
                    "entry-type" => obj.entry_type().to_value(),
                    _ => {
                        let prop = obj.find_tag(pspec_name).unwrap_or_else(|| "".into());
                        utils::format_string(&prop).to_value()
                    }
                }
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            let value = value.get::<String>().unwrap_or_else(|_| "".into());
            obj.set_prop(pspec.name(), &value);
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
        let tags = bib
            .tags()
            .iter()
            .map(|(x, y)| (x.to_lowercase(), y.to_string()))
            .collect();
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

    fn find_tag_pretty(&self, tag: &str) -> Option<String> {
        let value = self.find_tag(tag)?;
        let s = replace_c(&value)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace(|c| matches!(c, '{' | '}' | '$'), "");
        Some(replace_latex_accents(&replace_latex_symbols(&s)))
    }

    pub fn fields(&self) -> Vec<String> {
        self.entry_type().fields()
    }

    pub fn format_citation(&self, citation: &str, format: Format) -> String {
        static REGEX: OnceCell<Regex> = OnceCell::new();
        static PLAIN_REGEX: OnceCell<Regex> = OnceCell::new();
        static LATEX_IT_REGEX: OnceCell<Regex> = OnceCell::new();
        static LATEX_BF_REGEX: OnceCell<Regex> = OnceCell::new();

        const MATCHES: &[char] = &['*', '_'];

        let re = REGEX.get_or_init(|| Regex::new(r"(%[A-Z]+%)").unwrap());

        let mut formated = String::from(citation);

        match format {
            Format::Plain => {
                let plain_re = PLAIN_REGEX.get_or_init(|| {
                    Regex::new(r"(\*\S[^\*]+\S\*|\*{2}[^\*]+\*{2}|_\S.+\S_)").unwrap()
                });

                for catch in plain_re.find_iter(&formated.clone()) {
                    let stripped = catch.as_str().trim_matches(MATCHES);
                    formated = formated.replace(catch.as_str(), stripped);
                }
            }
            Format::Markdown => (),
            Format::LaTeX => {
                let latex_bf_re =
                    LATEX_BF_REGEX.get_or_init(|| Regex::new(r"\*{2}(?P<b>[^\*]+)\*{2}").unwrap());
                let latex_it_re =
                    LATEX_IT_REGEX.get_or_init(|| Regex::new(r"(\*[^\*]+\*|_\S[^*]+\S_)").unwrap());

                formated = latex_bf_re
                    .replace_all(&formated, r"\textbf{$b}")
                    .to_string();

                for catch in latex_it_re.find_iter(&formated.clone()) {
                    let stripped = catch.as_str().trim_matches(MATCHES);
                    formated =
                        formated.replace(catch.as_str(), &format!(r"\emph{{{}}}", &stripped));
                }

                formated = format!(r"\bibitem{{{}}} {formated}", self.citation_key());
            }
        }

        let name_prop = match format {
            Format::LaTeX => self.find_tag("author"),
            _ => self.find_tag_pretty("author"),
        };
        formated = formated.replace(
            "%AUTHOR%",
            &format_authors(name_prop.unwrap_or_else(|| "".into()).as_str()),
        );

        let citation = formated.clone();
        for catch in re.find_iter(&citation) {
            let tag = catch.as_str().trim_matches('%').to_lowercase();
            let opt_prop = match format {
                Format::LaTeX => self.find_tag(&tag),
                _ => self.find_tag_pretty(&tag),
            };
            if let Some(prop) = opt_prop {
                formated = formated.replace(catch.as_str(), &prop);
            } else {
                formated = formated.replace(catch.as_str(), "");
            }
        }

        match format {
            Format::Markdown => clear_citation(&clear_markdown(&formated)),
            Format::LaTeX => clear_citation(&clear_non_markdown(&clear_latex(&formated))),
            Format::Plain => clear_citation(&clear_non_markdown(&formated)),
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

fn replace_latex_accents(citation: &str) -> String {
    static RE: OnceCell<Regex> = OnceCell::new();
    let re =
        RE.get_or_init(|| Regex::new(r#"\\(?P<symbol>['`\^"u~v])(?P<letter>[a-zA-z])"#).unwrap());

    let replaces = HashMap::from([
        ("'", '́'),
        ("`", '̀'),
        ("u", '̆'),
        ("^", '̂'),
        (r#"""#, '̈'),
        ("~", '̃'),
        ("v", '̌'),
    ]);
    re.replace_all(citation, |captures: &regex::Captures| {
        format!("{}{}", &captures["letter"], replaces[&captures["symbol"]])
    })
    .to_string()
}

fn replace_c(citation: &str) -> String {
    static RE: OnceCell<Regex> = OnceCell::new();
    let re = RE.get_or_init(|| Regex::new(r#"\\c[\s\{]?(?P<letter>[a-zA-z])\}?"#).unwrap());
    re.replace_all(citation, |captures: &regex::Captures| {
        format!("{}{}", &captures["letter"], '̧')
    })
    .to_string()
}

fn replace_latex_symbols(citation: &str) -> String {
    // See https://en.m.wikibooks.org/wiki/LaTeX/Special_Characters
    static RE: OnceCell<Regex> = OnceCell::new();
    let re = RE.get_or_init(|| {
        Regex::new(r#"[\s\{]?\\(?P<symbol>[oOlL]|aa|AA|ae|AE|ss)(?P<end>[\}\s\\\{])"#).unwrap()
    });
    let replaces = HashMap::from([
        ("o", "ø"),
        ("O", "Ø"),
        ("ae", "æ"),
        ("AE", "Æ"),
        ("aa", "å"),
        ("AA", "Å"),
        ("l", "ł"),
        ("L", "Ł"),
        ("ss", "ß"),
    ]);
    re.replace_all(citation, |captures: &regex::Captures| {
        let mut end = &captures["end"];
        if end == "}" {
            end = ""
        }
        format!("{}{}", replaces[&captures["symbol"]], end)
    })
    .to_string()
}

fn clear_citation(citation: &str) -> String {
    static CLEAR3_REGEX: OnceCell<Regex> = OnceCell::new();
    static CLEAR_END_REGEX: OnceCell<Regex> = OnceCell::new();
    static CLEAR_START_REGEX: OnceCell<Regex> = OnceCell::new();

    static NO_REGEX: OnceCell<Regex> = OnceCell::new();

    static EMPTY_REGEX: OnceCell<Regex> = OnceCell::new();

    let re3 = CLEAR3_REGEX
        .get_or_init(|| Regex::new(r"\s*([,\.;:])(\s*[,\.;:]\s*)+|(\s)[\n\s]+").unwrap());
    let no_re =
        NO_REGEX.get_or_init(|| Regex::new(r"([\.,])\s*([Nn]o|[Vv]ol|pp)\.\s*[,\.]\s*").unwrap());
    let end_re = CLEAR_END_REGEX.get_or_init(|| Regex::new(r"\s*[,\.;:]\s*$").unwrap());
    let start_re = CLEAR_START_REGEX.get_or_init(|| Regex::new(r"^(\s*[,\.]\s*)+").unwrap());

    let re_empty = EMPTY_REGEX.get_or_init(|| {
        Regex::new(r#"("\s*"\s*|'\s*'\s*|\(\s*\)\s*|`\s*`\s*|\{\s*\}\s*)"#).unwrap()
    });
    let mut formated = String::from(citation);

    // Remove "no.  ," and "no,   ."
    formated = no_re.replace_all(&formated, "$1 ").to_string();

    // Remove empty quotes, braces, etc
    while re_empty.is_match(&formated) {
        formated = re_empty.replace_all(&formated, "").to_string();
    }

    // Flatten consecutive punctuation
    formated = re3.replace_all(&formated, "$1 ").to_string();
    // Turn last punctuation into a period
    formated = end_re.replace_all(&formated, ".").to_string();
    // Remove punctuation at the start, e.g. " , some title" into "some title"
    formated = start_re.replace_all(&formated, "").to_string();

    formated
}

fn clear_markdown(citation: &str) -> String {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    let re = REGEX.get_or_init(|| Regex::new(r#"(\*{2}\s*\*{2}|\*\s+\*|_\s*_)"#).unwrap());

    re.replace_all(citation, "").to_string()
}

fn clear_non_markdown(citation: &str) -> String {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    let re = REGEX.get_or_init(|| Regex::new(r#"\*\s*\*\s*|\*{2}\s*\*{2}\s*|_\s*_\s*"#).unwrap());

    re.replace_all(citation, "").to_string()
}

fn clear_latex(citation: &str) -> String {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    let re = REGEX.get_or_init(|| Regex::new(r"\\emph\{\s*\}|\\textbf\{\s*\}").unwrap());

    re.replace_all(citation, "").to_string()
}

fn format_authors(authors: &str) -> String {
    let authors = authors.replace(" and ", " AND ").replace(" & ", " AND ");
    let mut vec_authors = authors
        .split(" AND ")
        .map(String::from)
        .collect::<Vec<String>>();
    if vec_authors.len() > 1 {
        if let Some(last_author) = vec_authors.last_mut() {
            let value = format!("and {}", last_author);
            *last_author = value;
        }
    }
    vec_authors.join(", ")
}

pub trait EntryExt {
    fn find_tag(&self, tag: &str) -> Option<String>;
    fn insert_tag(&self, tag: &str, value: &str) -> Option<String>;
    fn set_prop(&self, tag: &str, value: &str);
    fn replace_tags(&self, tags: HashMap<String, String>);
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
            let p_property = format!("p-{tag}");
            if self.as_ref().has_property(&p_property, None) {
                self.as_ref().notify(&p_property);
            }
        }
    }

    fn replace_tags(&self, tags: HashMap<String, String>) {
        let keys: Vec<String> = tags.keys().map(|x| x.to_string()).collect();
        self.as_ref().imp().tags.replace(tags);
        keys.iter().for_each(move |key| {
            let key = key.to_lowercase();
            self.notify(&key);
            let p_property = format!("p-{key}");
            if self.as_ref().has_property(&p_property, None) {
                self.as_ref().notify(&p_property);
            }
        });
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

    #[test]
    fn clear_garbage() {
        assert_eq!(clear_citation("some,  text  2"), "some, text 2");

        assert_eq!(clear_citation("some, , , text"), "some, text");
        assert_eq!(clear_citation("some, ,  , ,  , text"), "some, text");
        assert_eq!(clear_citation("some, .  . ,  . text"), "some, text");
        assert_eq!(clear_citation("some. .  . ,  . , text"), "some. text");
        assert_eq!(clear_citation("some..,., ,, . , text"), "some. text");

        assert_eq!(clear_citation("some, no. , text"), "some, text");
        assert_eq!(clear_citation("some. No. . text"), "some. text");
        assert_eq!(clear_citation("some. no. , text"), "some. text");
        assert_eq!(clear_citation("some. vol. , text"), "some. text");
        assert_eq!(clear_citation("some. Vol. , text"), "some. text");

        assert_eq!(clear_citation("some. {} , text"), "some. text");
        assert_eq!(clear_citation("some. () , text"), "some. text");

        assert_eq!(clear_citation("some., ,"), "some.");
        assert_eq!(clear_citation(" , .some., ,"), "some.");

        assert_eq!(clear_citation(r#""""#), "");
        assert_eq!(clear_citation(r#"some, , "" , ,  , text"#), "some, text");
        assert_eq!(
            clear_citation(r#""some", , "" , ,  , text"#),
            r#""some", text"#
        );
    }

    #[test]
    fn test_format_authors() {
        assert_eq!(format_authors("x and y"), "x, and y");
        assert_eq!(format_authors("x and y and z"), "x, y, and z");
        assert_eq!(format_authors("x & y"), "x, and y");
        assert_eq!(format_authors("x AND y"), "x, and y");
        assert_eq!(format_authors("x and y and z"), "x, y, and z");
    }

    #[test]
    fn test_clear_latex() {
        assert_eq!(clear_latex(r"\textbf{}"), "");
        assert_eq!(clear_latex(r"\emph{}"), "");
    }

    #[test]
    fn test_clear_markdown() {
        assert_eq!(
            clear_markdown(r"asdas asd _asd_ *asd* ** asdasd **asd** **** __"),
            "asdas asd _asd_ *asd* asdasd **asd** "
        );
    }

    #[test]
    fn test_zalgo() {
        assert_eq!(replace_latex_accents(r"\'u"), "u\u{301}");
        assert_eq!(replace_c(r"\c c"), "c\u{0327}");
        assert_eq!(replace_c(r"\c{c}"), "c\u{0327}");
        assert_eq!(replace_latex_symbols(r"\o a"), "ø a");
        assert_eq!(replace_latex_symbols(r"{\o}a"), "øa");
        assert_eq!(replace_latex_symbols(r"{\o}"), "ø");
        assert_eq!(replace_latex_symbols(r"{\ss}"), "ß");
        // FIXME
        // assert_eq!(replace_latex_symbols(r"\o"), "ø");
    }
}
