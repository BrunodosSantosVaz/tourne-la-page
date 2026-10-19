use std::collections::{BTreeSet, HashMap};

use fontdb::Database;

const RECOMMENDED_FONTS: [&str; 4] = [
    "Georgia",
    "Literata",
    "Atkinson Hyperlegible Next",
    "Lexend",
];

#[derive(Debug, Clone)]
pub struct FontCatalog {
    options: Vec<String>,
    iced_names: HashMap<String, &'static str>,
    installed: BTreeSet<String>,
}

impl FontCatalog {
    pub fn system() -> Self {
        let mut database = Database::new();
        database.load_system_fonts();
        let families = database
            .faces()
            .flat_map(|face| face.families.iter().map(|(family, _)| family.clone()))
            .collect::<Vec<_>>();
        Self::with_families(families)
    }

    pub(crate) fn with_families(families: impl IntoIterator<Item = String>) -> Self {
        let installed = families.into_iter().collect::<BTreeSet<_>>();
        let options = ordered_families(&installed);
        let iced_names = options
            .iter()
            .map(|family| (family.clone(), intern(family)))
            .collect();
        Self {
            options,
            iced_names,
            installed,
        }
    }

    pub fn options(&self) -> Vec<String> {
        self.options.clone()
    }

    pub fn selected_default(&self) -> &'static str {
        if self
            .installed
            .iter()
            .any(|name| name.eq_ignore_ascii_case("Georgia"))
        {
            "Georgia"
        } else {
            "Literata"
        }
    }

    pub fn iced_name(&self, family: &str) -> &'static str {
        self.iced_names.get(family).copied().unwrap_or("Literata")
    }
}

fn ordered_families(installed: &BTreeSet<String>) -> Vec<String> {
    let mut options = RECOMMENDED_FONTS.map(str::to_owned).to_vec();
    options.extend(
        installed
            .iter()
            .filter(|family| {
                !RECOMMENDED_FONTS
                    .iter()
                    .any(|suggested| family.eq_ignore_ascii_case(suggested))
            })
            .cloned(),
    );
    options
}

fn intern(family: &str) -> &'static str {
    // shortcut: Iced requires static family names; keep one interned name per installed family until the process exits.
    Box::leak(family.to_owned().into_boxed_str())
}
