use crate::fonts::FontCatalog;

#[test]
fn places_the_four_reading_suggestions_before_installed_families() {
    let catalog =
        FontCatalog::with_families(["Arial".to_owned(), "Georgia".to_owned(), "Arial".to_owned()]);

    assert_eq!(
        &catalog.options()[..4],
        [
            "Georgia",
            "Literata",
            "Atkinson Hyperlegible Next",
            "Lexend"
        ]
    );
    assert_eq!(
        catalog
            .options()
            .iter()
            .filter(|name| *name == "Arial")
            .count(),
        1
    );
    assert_eq!(catalog.iced_name("Georgia"), "Georgia");
}

#[test]
fn uses_georgia_when_installed_and_literata_as_the_fallback() {
    let georgia = FontCatalog::with_families(["Georgia".to_owned()]);
    let fallback = FontCatalog::with_families(["Arial".to_owned()]);

    assert_eq!(georgia.selected_default(), "Georgia");
    assert_eq!(fallback.selected_default(), "Literata");
}
