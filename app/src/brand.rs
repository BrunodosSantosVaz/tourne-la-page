//! Public identity and the fixed destinations shown in the About dialog.
use std::process::Command;

pub const NAME: &str = "Tourne la page";
#[cfg(any(target_os = "linux", test))]
pub const APP_ID: &str = "io.github.BrunodosSantosVaz.TourneLaPage";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MARK: &[u8] = include_bytes!("../../assets/brand/mark.svg");

pub fn window_title(book: Option<&str>) -> String {
    book.map_or_else(|| NAME.to_owned(), |title| format!("{NAME} — {title}"))
}

pub fn window_icon() -> iced::window::Icon {
    let image = image::load_from_memory(include_bytes!("../../assets/brand/icons/app-256.png"))
        .expect("bundled application icon is validated by the asset test")
        .into_rgba8();
    let (width, height) = image.dimensions();
    iced::window::icon::from_rgba(image.into_raw(), width, height)
        .expect("decoded application icon has valid RGBA dimensions")
}

#[derive(Debug, Clone, Copy)]
pub enum Link {
    Repository,
    Developer,
    Issues,
    License,
}

impl Link {
    pub fn url(self) -> &'static str {
        match self {
            Self::Repository => "https://github.com/BrunodosSantosVaz/tourne-la-page",
            Self::Developer => "https://github.com/BrunodosSantosVaz",
            Self::Issues => "https://github.com/BrunodosSantosVaz/tourne-la-page/issues",
            Self::License => {
                "https://github.com/BrunodosSantosVaz/tourne-la-page/blob/main/LICENSE"
            }
        }
    }
}

pub fn open_link(link: Link) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let mut command = Command::new("xdg-open");
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("rundll32");
        command.arg("url.dll,FileProtocolHandler");
        command
    };
    command
        .arg(link.url())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Não foi possível abrir o navegador: {error}"))
}
