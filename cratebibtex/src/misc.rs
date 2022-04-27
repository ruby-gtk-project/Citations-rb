use glib::subclass::prelude::*;

use crate::entry::EntryExt;

mod imp {
    use super::*;

    use glib::prelude::*;
    use once_cell::sync::Lazy;

    use crate::utils;

    #[derive(Debug, Default)]
    pub struct Misc;

    #[glib::object_subclass]
    impl ObjectSubclass for Misc {
        const NAME: &'static str = "Misc";
        type Type = super::Misc;
        type ParentType = crate::Entry;
    }

    impl ObjectImpl for Misc {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                utils::new_props(&[
                    "journal", "volume", "number", "pages", "doi",
                    "eprint",
                ])
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            obj.find_tag(psect.name())
                .unwrap_or_else(|| "".into())
                .to_value()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            let value = value.get::<String>().unwrap_or_else(|_| "".into());
            let prop_name = pspec.name();
            obj.set_prop(prop_name, &value);
        }
    }
    impl crate::EntryImpl for Misc {}
}

glib::wrapper! {
    pub struct Misc(ObjectSubclass<imp::Misc>) @extends crate::Entry;
}

impl Default for Misc {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

impl TryFrom<&nom_bibtex::Bibliography> for Misc {
    type Error = anyhow::Error;

    fn try_from(bib: &nom_bibtex::Bibliography) -> anyhow::Result<Self> {
        if bib.entry_type().to_lowercase() != "misc" {
            anyhow::bail!("Not an misc")
        }
        let tags = bib
            .tags()
            .iter()
            .map(|(x, y)| (x.to_lowercase(), y.to_string()))
            .collect();
        let misc = Self::default();
        misc.replace_tags(tags);

        Ok(misc)
    }
}
