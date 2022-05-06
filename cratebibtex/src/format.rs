use gettextrs::gettext;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "Format")]
#[repr(u32)]
pub enum Format {
    Plain = 0,
    Markdown = 1,
    LaTeX = 2,
}

impl Default for Format {
    fn default() -> Self {
        Self::Plain
    }
}

impl From<u32> for Format {
    fn from(u: u32) -> Self {
        match u {
            1 => Self::Markdown,
            2 => Self::LaTeX,
            _ => Self::Plain,
        }
    }
}

impl From<String> for Format {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Markdown" => Self::Markdown,
            "Latex" => Self::LaTeX,
            _ => Self::Plain,
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Format {
    pub fn to_translatable_string(self) -> String {
        match self {
            // NOTE Plain as in Plain Text
            Format::Plain => gettext("Plain"),
            Format::Markdown => "Markdown".to_string(),
            Format::LaTeX => "LaTeX".to_string(),
        }
    }
}

pub(crate) fn format_plain(citation: &str) -> String {
    const MATCHES: &[char] = &['*', '_'];

    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\*\S[^\*]+\S\*|\*{2}[^\*]+\*{2}|_\S[^_]+\S_").unwrap());

    RE.replace_all(citation, |captures: &regex::Captures| {
        captures[0].trim_matches(MATCHES).to_string()
    })
    .to_string()
}

pub(crate) fn format_latex(citation: &str, key: &str) -> String {
    const MATCHES: &[char] = &['*', '_'];
    static IT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\*[^\*]+\*|_\S[^_]+\S_").unwrap());
    static BF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\*{2}(?P<b>[^\*]+)\*{2}").unwrap());

    let formated = BF_RE.replace_all(citation, r"\textbf{$b}").to_string();
    let formated = IT_RE
        .replace_all(&formated, |captures: &regex::Captures| {
            format!(r"\emph{{{}}}", captures[0].trim_matches(MATCHES))
        })
        .to_string();

    format!(r"\bibitem{{{key}}} {formated}")
}

pub(crate) fn clear_citation(citation: &str) -> String {
    static CLEAR_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\s*([,\.;:])(\s*[,\.;:]\s*)+|(\s)[\n\s]+").unwrap());
    static CLEAR_END_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*[,\.;:]\s*$").unwrap());
    static CLEAR_START_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\s*[,\.]\s*)+").unwrap());

    static NO_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"([\.,])\s*([Nn]o|[Vv]ol|pp)\.\s*[,\.]\s*").unwrap());

    static EMPTY_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"("\s*"\s*|'\s*'\s*|\(\s*\)\s*|`\s*`\s*|\{\s*\}\s*)"#).unwrap());

    let mut formated = String::from(citation);

    // Remove "no.  ," and "no,   ."
    formated = NO_RE.replace_all(&formated, "$1 ").to_string();

    // Remove empty quotes, braces, etc
    while EMPTY_RE.is_match(&formated) {
        formated = EMPTY_RE.replace_all(&formated, "").to_string();
    }

    // Flatten consecutive punctuation
    formated = CLEAR_RE.replace_all(&formated, "$1 ").to_string();
    // Turn last punctuation into a period
    formated = CLEAR_END_RE.replace_all(&formated, ".").to_string();
    // Remove punctuation at the start, e.g. " , some title" into "some title"
    formated = CLEAR_START_RE.replace_all(&formated, "").to_string();

    formated
}

pub(crate) fn clear_markdown(citation: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\*{2}\s*\*{2}|\*\s+\*|_\s*_)"#).unwrap());

    RE.replace_all(citation, "").to_string()
}

pub(crate) fn clear_non_markdown(citation: &str) -> String {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"\*\s*\*\s*|\*{2}\s*\*{2}\s*|_\s*_\s*"#).unwrap());

    RE.replace_all(citation, "").to_string()
}

pub(crate) fn clear_latex(citation: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\emph\{\s*\}|\\textbf\{\s*\}").unwrap());

    RE.replace_all(citation, "").to_string()
}

pub(crate) fn format_authors(authors: &str) -> String {
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

pub(crate) fn replace_c(citation: &str) -> String {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"\\c[\s\{]?(?P<letter>[a-zA-z])\}?"#).unwrap());
    RE.replace_all(citation, |captures: &regex::Captures| {
        format!("{}{}", &captures["letter"], '̧')
    })
    .to_string()
}

pub(crate) fn replace_latex_symbols(citation: &str) -> String {
    // See https://en.m.wikibooks.org/wiki/LaTeX/Special_Characters
    static RE: Lazy<Regex> = Lazy::new(|| {
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
    RE.replace_all(citation, |captures: &regex::Captures| {
        let mut end = &captures["end"];
        if end == "}" {
            end = ""
        }
        format!("{}{}", replaces[&captures["symbol"]], end)
    })
    .to_string()
}

pub(crate) fn replace_latex_accents(citation: &str) -> String {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"\\(?P<symbol>['`\^"u~v])(?P<letter>[a-zA-z])"#).unwrap());

    let replaces = HashMap::from([
        ("'", '́'),
        ("`", '̀'),
        ("u", '̆'),
        ("^", '̂'),
        (r#"""#, '̈'),
        ("~", '̃'),
        ("v", '̌'),
    ]);
    RE.replace_all(citation, |captures: &regex::Captures| {
        format!("{}{}", &captures["letter"], replaces[&captures["symbol"]])
    })
    .to_string()
}

pub fn pretty_author(author: &str) -> String {
    replace_latex_accents(&replace_latex_symbols(&replace_c(author)))
}

pub fn pretty_title(title: &str) -> String {
    let s = replace_c(title)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(|c| matches!(c, '{' | '}' | '$'), "");
    replace_latex_accents(&replace_latex_symbols(&s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_citation() {
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
