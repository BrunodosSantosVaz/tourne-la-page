use crate::brand;

#[test]
fn customer_name_and_window_title_use_the_new_brand() {
    assert_eq!(brand::NAME, "Tourne la page");
    assert_eq!(brand::window_title(None), "Tourne la page");
    assert_eq!(
        brand::window_title(Some("Meu livro")),
        "Tourne la page — Meu livro"
    );
    assert_eq!(
        brand::Link::Repository.url(),
        "https://github.com/BrunodosSantosVaz/tourne-la-page"
    );
}

#[test]
fn bundled_icon_is_valid_and_the_same_id_is_used_for_linux_desktop_integration() {
    let (_, size) = brand::window_icon().into_raw();
    assert_eq!(size, iced::Size::new(256, 256));
    assert_eq!(brand::APP_ID, "io.github.BrunodosSantosVaz.TourneLaPage");
}
