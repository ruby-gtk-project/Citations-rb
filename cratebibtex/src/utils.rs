use glib::StaticType;

pub(crate) fn new_props(names: &[&str]) -> Vec<glib::ParamSpec> {
    let mut generated_props = names
        .iter()
        .map(|name| {
            let flags = if name.starts_with("p-") {
                glib::ParamFlags::READABLE | glib::ParamFlags::EXPLICIT_NOTIFY
            } else {
                glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY
            };
            glib::ParamSpecString::new(name, name, name, "".into(), flags)
        })
        .collect::<Vec<glib::ParamSpec>>();
    let citation_key_prop = glib::ParamSpecString::new(
        "citation-key",
        "citation-key",
        "citation-key",
        "".into(),
        glib::ParamFlags::READABLE,
    );
    let entry_type_prop = glib::ParamSpecEnum::new(
        "entry-type",
        "entry-type",
        "entry-type",
        crate::EntryType::static_type(),
        crate::EntryType::default() as i32,
        glib::ParamFlags::READABLE,
    );
    generated_props.push(citation_key_prop);
    generated_props.push(entry_type_prop);
    generated_props
}

pub(crate) fn format_string(string: &str) -> String {
    string.split_whitespace().collect::<Vec<_>>().join(" ")
}
