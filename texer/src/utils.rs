use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

static COMBINING_MAP: LazyLock<HashMap<&'static str, char>> = LazyLock::new(|| {
    HashMap::from([
        ("'", '\u{301}'),  // COMBINING ACUTE ACCENT
        ("`", '\u{300}'),  // COMBINING GRAVE
        ("~", '\u{303}'),  // COMBINING TILDE
        ("^", '\u{302}'),  // COMBINING CIRCUMFLEX ACCENT
        ("\"", '\u{308}'), // COMBINING DIAERESIS
        ("=", '\u{304}'),  // COMBINING MACRON
        ("c", '\u{327}'),  // COMBINING CEDILLA
        ("u", '\u{306}'),  // COMBINING BREVE
        ("v", '\u{30C}'),  // COMBINING CARON
        ("k", '\u{328}'),  // COMBINING OGONEK
    ])
});

pub(crate) fn combine_unicode<'a>(method: &str, data: &'a str) -> Cow<'a, str> {
    if let Some(combining_char) = COMBINING_MAP.get(method) {
        let mut res = String::with_capacity(data.len() + 1);
        res.push_str(data);
        res.push(*combining_char);
        Cow::Owned(res)
    } else {
        Cow::Borrowed(data)
    }
}
