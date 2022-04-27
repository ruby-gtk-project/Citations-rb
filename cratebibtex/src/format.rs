use gettextrs::gettext;
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
    pub fn to_translatable_string(&self) -> String {
        match self {
            // NOTE Plain as in Plain Text
            Format::Plain => gettext("Plain"),
            Format::Markdown => "Markdown".to_string(),
            Format::LaTeX => "LaTeX".to_string(),
        }
    }
}
