mod action;
mod adaptive_scroll;
mod annotations;
mod bar;
mod book_details;
#[cfg(test)]
mod diagnostics;
mod library_screen;
mod popover;
mod preferences;
mod reading_history;
mod surface;
mod toolbar;
#[cfg(test)]
mod transitions;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use app_core::OpenBookError;
use epub_core::{Book, BookError, BookMetadata, Chapter, parse_epub_progressive};
use iced::widget::{
    button, column, container, pick_list, row, scrollable, slider, stack, text, text_input,
};
use iced::{Alignment, Color, Element, Font, Length, Settings, Subscription, Task, Theme};
use serde::{Deserialize, Serialize};

use crate::brand;
use crate::fonts::FontCatalog;
use crate::library::{self, LibraryItem};

const ACCENT: Color = Color::from_rgb8(26, 79, 180);

fn icon_svg(name: &str, size: u16) -> Element<'static, Message> {
    let source: &'static [u8] = match name {
        "info" => include_bytes!("../../assets/icons/lucide/info.svg"),
        "palette" => include_bytes!("../../assets/icons/lucide/palette.svg"),
        "a-large-small" => include_bytes!("../../assets/icons/lucide/a-large-small.svg"),
        "move-horizontal" => include_bytes!("../../assets/icons/lucide/move-horizontal.svg"),
        "move-vertical" => include_bytes!("../../assets/icons/lucide/move-vertical.svg"),
        "highlighter" => include_bytes!("../../assets/icons/lucide/highlighter.svg"),
        "gauge" => include_bytes!("../../assets/icons/lucide/gauge.svg"),
        "chevron-down" => include_bytes!("../../assets/icons/lucide/chevron-down.svg"),
        "chevron-up" => include_bytes!("../../assets/icons/lucide/chevron-up.svg"),
        "menu" => include_bytes!("../../assets/icons/lucide/menu.svg"),
        "folder-plus" => include_bytes!("../../assets/icons/lucide/folder-plus.svg"),
        "book-open" => include_bytes!("../../assets/icons/lucide/book-open.svg"),
        "book-marked" => include_bytes!("../../assets/icons/lucide/book-marked.svg"),
        "bookmark" => include_bytes!("../../assets/icons/lucide/bookmark.svg"),
        "library" => include_bytes!("../../assets/icons/lucide/library.svg"),
        "house" => include_bytes!("../../assets/icons/lucide/house.svg"),
        "search" => include_bytes!("../../assets/icons/lucide/search.svg"),
        "grid-2x2" => include_bytes!("../../assets/icons/lucide/grid-2x2.svg"),
        "list" => include_bytes!("../../assets/icons/lucide/list.svg"),
        "rows-3" => include_bytes!("../../assets/icons/lucide/rows-3.svg"),
        "refresh-cw" => include_bytes!("../../assets/icons/lucide/refresh-cw.svg"),
        "align-left" => include_bytes!("../../assets/icons/lucide/align-left.svg"),
        "align-center" => include_bytes!("../../assets/icons/lucide/align-center.svg"),
        "align-right" => include_bytes!("../../assets/icons/lucide/align-right.svg"),
        "align-justify" => include_bytes!("../../assets/icons/lucide/align-justify.svg"),
        "pencil" => include_bytes!("../../assets/icons/lucide/pencil.svg"),
        "heart" => include_bytes!("../../assets/icons/lucide/heart.svg"),
        "heart-filled" => include_bytes!("../../assets/icons/lucide/heart-filled.svg"),
        "star" => include_bytes!("../../assets/icons/lucide/star.svg"),
        "image" => include_bytes!("../../assets/icons/lucide/image.svg"),
        "image-off" => include_bytes!("../../assets/icons/lucide/image-off.svg"),
        "play" => include_bytes!("../../assets/icons/lucide/play.svg"),
        "pause" => include_bytes!("../../assets/icons/lucide/pause.svg"),
        "chevron-left" => include_bytes!("../../assets/icons/lucide/chevron-left.svg"),
        "chevron-right" => include_bytes!("../../assets/icons/lucide/chevron-right.svg"),
        "sun" => include_bytes!("../../assets/icons/lucide/sun.svg"),
        "moon" => include_bytes!("../../assets/icons/lucide/moon.svg"),
        "type" => include_bytes!("../../assets/icons/lucide/type.svg"),
        "settings" => include_bytes!("../../assets/icons/lucide/settings.svg"),
        "x" => include_bytes!("../../assets/icons/lucide/x.svg"),
        "minus" => include_bytes!("../../assets/icons/lucide/minus.svg"),
        "plus" => include_bytes!("../../assets/icons/lucide/plus.svg"),
        _ => include_bytes!("../../assets/icons/lucide/check.svg"),
    };
    iced::widget::svg(iced::widget::svg::Handle::from_memory(source))
        .width(Length::Fixed(f32::from(size)))
        .height(Length::Fixed(f32::from(size)))
        .style(|theme: &Theme, _status| iced::widget::svg::Style {
            color: Some(theme.palette().text),
        })
        .into()
}

fn section_icon(section: LibrarySection) -> &'static str {
    match section {
        LibrarySection::Home => "house",
        LibrarySection::AllBooks => "library",
        LibrarySection::Reading => "book-open",
        LibrarySection::Rating => "star",
        LibrarySection::Favorites => "heart",
        _ => "book-marked",
    }
}

pub fn run(data_dir: PathBuf) -> iced::Result {
    iced::application(move || Reader::new(data_dir.clone()), update, view)
        .title(|state: &Reader| {
            brand::window_title(if state.page == Page::Reader {
                state.book.as_ref().map(|book| book.metadata.title.as_str())
            } else {
                None
            })
        })
        .settings(Settings {
            default_font: Font::with_name("Inter"),
            fonts: embedded_fonts(),
            ..Settings::default()
        })
        .theme(|state: &Reader| theme_for(state))
        .window(main_window_settings())
        .subscription(subscription)
        .run()
}

fn main_window_settings() -> iced::window::Settings {
    let settings = iced::window::Settings {
        size: iced::Size::new(1280.0, 820.0),
        min_size: Some(iced::Size::new(bar::MIN_WIDTH + 24.0, 360.0)),
        icon: Some(brand::window_icon()),
        exit_on_close_request: false,
        ..Default::default()
    };
    #[cfg(target_os = "linux")]
    let settings = iced::window::Settings {
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: brand::APP_ID.to_owned(),
            ..Default::default()
        },
        ..settings
    };
    settings
}

fn embedded_fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(&include_bytes!("../../assets/fonts/Inter/Inter[opsz,wght].ttf")[..]),
        Cow::Borrowed(&include_bytes!("../../assets/fonts/Literata/Literata[opsz,wght].ttf")[..]),
        Cow::Borrowed(
            &include_bytes!("../../assets/fonts/Atkinson/AtkinsonHyperlegibleNext[wght].ttf")[..],
        ),
        Cow::Borrowed(&include_bytes!("../../assets/fonts/Lexend/Lexend[wght].ttf")[..]),
    ]
}

fn theme_for(state: &Reader) -> Theme {
    let definition = theme_definition_for(state);
    let name = definition.map_or("Claro", |theme| theme.name.as_str());
    let background = definition.map_or(Color::WHITE, |theme| hex_color(&theme.background));
    let foreground = definition.map_or(Color::from_rgb8(34, 34, 34), |theme| {
        ensure_contrast(hex_color(&theme.foreground), background, 4.5)
    });
    let primary = definition.map_or(ACCENT, |theme| hex_color(&theme.accent));
    Theme::custom(
        name.to_owned(),
        iced::theme::Palette {
            background,
            text: foreground,
            primary,
            success: Color::from_rgb8(45, 125, 82),
            danger: Color::from_rgb8(179, 38, 30),
            warning: Color::from_rgb8(145, 92, 20),
        },
    )
}

fn theme_definition_for(state: &Reader) -> Option<&ThemeDefinition> {
    let id = current_theme_id(&state.settings);
    state.custom_themes.iter().find(|theme| theme.id == id)
}

fn selected_marker_color(state: &Reader) -> String {
    theme_definition_for(state)
        .and_then(|theme| {
            theme
                .marker_colors
                .iter()
                .find(|color| **color == state.settings.marker_color)
                .or_else(|| theme.marker_colors.first())
        })
        .cloned()
        .unwrap_or_else(default_selected_marker)
}

fn background_for(state: &Reader) -> Color {
    theme_definition_for(state).map_or(Color::WHITE, |theme| hex_color(&theme.background))
}

fn foreground_for(state: &Reader) -> Color {
    ensure_contrast(
        theme_definition_for(state).map_or(Color::from_rgb8(34, 34, 34), |theme| {
            hex_color(&theme.foreground)
        }),
        background_for(state),
        4.5,
    )
}

fn surface_for(state: &Reader) -> Color {
    let background = background_for(state);
    let foreground = foreground_for(state);
    let toward = if luminance(background) > 0.5 {
        Color::BLACK
    } else {
        foreground
    };
    mix(
        background,
        toward,
        if luminance(background) > 0.5 {
            0.035
        } else {
            0.075
        },
    )
}

fn border_for(state: &Reader) -> Color {
    mix(surface_for(state), foreground_for(state), 0.22)
}

fn muted_for(state: &Reader) -> Color {
    ensure_contrast(
        theme_definition_for(state).map_or(Color::from_rgb8(107, 107, 107), |theme| {
            hex_color(&theme.muted)
        }),
        surface_for(state),
        4.5,
    )
}

fn mix(a: Color, b: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color::from_rgba(
        a.r + (b.r - a.r) * amount,
        a.g + (b.g - a.g) * amount,
        a.b + (b.b - a.b) * amount,
        a.a + (b.a - a.a) * amount,
    )
}

fn luminance(color: Color) -> f32 {
    let linear = |channel: f32| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

fn contrast_ratio(a: Color, b: Color) -> f32 {
    let high = luminance(a).max(luminance(b));
    let low = luminance(a).min(luminance(b));
    (high + 0.05) / (low + 0.05)
}

fn ensure_contrast(color: Color, background: Color, minimum: f32) -> Color {
    if contrast_ratio(color, background) >= minimum {
        return color;
    }
    let target = if luminance(background) > 0.5 {
        Color::BLACK
    } else {
        Color::WHITE
    };
    let mut low = 0.0;
    let mut high = 1.0;
    for _ in 0..24 {
        let middle = (low + high) / 2.0;
        if contrast_ratio(mix(color, target, middle), background) >= minimum {
            high = middle;
        } else {
            low = middle;
        }
    }
    mix(color, target, high)
}

struct Reader {
    data_dir: PathBuf,
    page: Page,
    folders: Vec<PathBuf>,
    library: Vec<LibraryItem>,
    book_reviews: HashMap<PathBuf, book_details::Review>,
    reading_history: HashMap<PathBuf, reading_history::Progress>,
    reading_dirty: bool,
    book_details_path: Option<PathBuf>,
    book_review_editor: iced::widget::text_editor::Content,
    search: String,
    section: LibrarySection,
    sort_by: SortBy,
    list_view: bool,
    book: Option<Arc<Book>>,
    book_path: Option<PathBuf>,
    last_opened_path: Option<PathBuf>,
    book_cover: Option<iced::widget::image::Handle>,
    chapter_index: usize,
    chapter_paragraph_ranges: Vec<Vec<(usize, usize)>>,
    font_catalog: FontCatalog,
    selected_font: String,
    settings: ReadingSettings,
    custom_themes: Vec<ThemeDefinition>,
    theme_draft: Option<ThemeDraft>,
    window_width: f32,
    settings_open: bool,
    about_open: bool,
    toolbar_overflow_open: bool,
    compact_menu: Option<toolbar::Item>,
    theme_menu_open: bool,
    annotation_palette_open: Option<String>,
    annotations: Vec<BookAnnotation>,
    annotations_revision: u64,
    preference_writer: Arc<Mutex<()>>,
    preference_write_version: Arc<AtomicU64>,
    preference_versions: preferences::Pending,
    closing: bool,
    navigation_revision: u64,
    navigation_target: (usize, usize),
    selected_excerpt: Option<SelectedExcerpt>,
    active_mark_id: Option<String>,
    expected_chapters: usize,
    book_loading: bool,
    resume_chapter: Option<usize>,
    annotation_draft: String,
    annotation_editor: iced::widget::text_editor::Content,
    show_annotations: bool,
    status: Option<String>,
    catalog_loaded: bool,
    sidebar_open: bool,
    scroll_id: iced::widget::Id,
    scroll_y: f32,
    scroll_max: f32,
    scroll_viewport_height: f32,
    progress_percent: f32,
}

impl Reader {
    fn new(data_dir: PathBuf) -> (Self, Task<Message>) {
        let font_catalog = FontCatalog::with_families(std::iter::empty());
        let folders = library::saved_folders(&data_dir).unwrap_or_default();
        let settings = load_settings(&data_dir);
        let custom_themes = load_custom_themes(&data_dir);
        let last_opened_path = library::preference(&data_dir, "last-opened")
            .ok()
            .flatten()
            .map(PathBuf::from);
        let scan = scan_task(folders.clone());
        let book_reviews = book_details::load(&data_dir);
        let reading_history = reading_history::load(&data_dir, last_opened_path.as_deref());
        let state = Self {
            data_dir,
            page: Page::Library,
            folders,
            library: Vec::new(),
            book_reviews,
            reading_history,
            reading_dirty: false,
            book_details_path: None,
            book_review_editor: iced::widget::text_editor::Content::new(),
            search: String::new(),
            section: LibrarySection::AllBooks,
            sort_by: SortBy::Title,
            list_view: false,
            book: None,
            book_path: None,
            last_opened_path,
            book_cover: None,
            chapter_index: 0,
            chapter_paragraph_ranges: Vec::new(),
            selected_font: font_catalog.selected_default().to_owned(),
            font_catalog,
            settings,
            custom_themes,
            theme_draft: None,
            window_width: 1280.0,
            settings_open: false,
            about_open: false,
            toolbar_overflow_open: false,
            compact_menu: None,
            theme_menu_open: false,
            annotation_palette_open: None,
            annotations: Vec::new(),
            annotations_revision: 0,
            preference_writer: Arc::new(Mutex::new(())),
            preference_write_version: Arc::new(AtomicU64::new(0)),
            preference_versions: Arc::new(Mutex::new(HashMap::new())),
            closing: false,
            navigation_revision: 0,
            navigation_target: (0, 0),
            selected_excerpt: None,
            active_mark_id: None,
            expected_chapters: 0,
            book_loading: false,
            resume_chapter: None,
            annotation_draft: String::new(),
            annotation_editor: iced::widget::text_editor::Content::new(),
            show_annotations: false,
            status: None,
            catalog_loaded: false,
            sidebar_open: true,
            scroll_id: iced::widget::Id::unique(),
            scroll_y: 0.0,
            scroll_max: 0.0,
            scroll_viewport_height: 820.0,
            progress_percent: 0.0,
        };
        (
            state,
            Task::batch([
                scan,
                Task::perform(
                    async {
                        tokio::task::spawn_blocking(FontCatalog::system)
                            .await
                            .unwrap_or_else(|_| FontCatalog::with_families(std::iter::empty()))
                    },
                    Message::FontCatalogLoaded,
                ),
            ]),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Library,
    Reader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ThemeChoice {
    Automatic,
    Light,
    Paper,
    Sepia,
    Dark,
    Graphite,
    Night,
    Dracula,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    Custom,
}

impl Display for ThemeChoice {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Automatic => "Automático",
            Self::Light => "Claro",
            Self::Paper => "Papel",
            Self::Sepia => "Sépia",
            Self::Dark => "Escuro",
            Self::Graphite => "Grafite",
            Self::Night => "Noite",
            Self::Dracula => "Dracula",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::Custom => "Personalizado",
        };
        formatter.write_str(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ThemeDefinition {
    id: String,
    name: String,
    background: String,
    foreground: String,
    muted: String,
    accent: String,
    #[serde(default = "default_marker_palette")]
    marker_colors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BookAnnotation {
    id: String,
    chapter_index: usize,
    kind: AnnotationKind,
    text: String,
    #[serde(default)]
    excerpt: String,
    #[serde(default)]
    start_offset: usize,
    #[serde(default)]
    end_offset: usize,
    #[serde(default)]
    marker_color: String,
}

#[derive(Debug, Clone)]
struct SelectedExcerpt {
    chapter_index: usize,
    text: String,
    start_offset: usize,
    end_offset: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum AnnotationKind {
    Bookmark,
    Highlight,
    Comment,
}

#[derive(Debug, Clone, Default)]
struct ThemeDraft {
    id: Option<String>,
    name: String,
    background: String,
    foreground: String,
    muted: String,
    accent: String,
    marker_colors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThemeOption {
    id: String,
    label: String,
}

impl Display for ThemeOption {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.label)
    }
}

impl ThemeDraft {
    fn new() -> Self {
        Self {
            name: "Meu tema".to_owned(),
            background: "#FCFBF7".to_owned(),
            foreground: "#2B2722".to_owned(),
            muted: "#6D665D".to_owned(),
            accent: "#1A4FB4".to_owned(),
            marker_colors: marker_palette_for("#FCFBF7"),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum AlignmentChoice {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibrarySection {
    Rating,
    Favorites,
    Home,
    AllBooks,
    Reading,
    Authors,
    Series,
    Genres,
    Language,
    Publisher,
    Collections,
}

impl Display for LibrarySection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Rating => "Nota",
            Self::Favorites => "Favoritos",
            Self::Home => "Início",
            Self::AllBooks => "Todos os livros",
            Self::Reading => "Lendo",
            Self::Authors => "Autores",
            Self::Series => "Séries",
            Self::Genres => "Gêneros",
            Self::Language => "Idiomas",
            Self::Publisher => "Editoras",
            Self::Collections => "Coleções",
        };
        formatter.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortBy {
    Rating,
    Title,
    Author,
    Series,
    Publisher,
}

impl Display for SortBy {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Rating => "Nota",
            Self::Title => "Título",
            Self::Author => "Autor",
            Self::Series => "Série",
            Self::Publisher => "Editora",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct ReadingSettings {
    theme: ThemeChoice,
    custom_theme_id: Option<String>,
    #[serde(alias = "font_size")]
    font_scale: u16,
    width: f32,
    line_height: f32,
    alignment: AlignmentChoice,
    show_images: bool,
    #[serde(skip)]
    auto_scroll: bool,
    scroll_speed_percent: u16,
    #[serde(default = "default_selected_marker")]
    marker_color: String,
}

impl Default for ReadingSettings {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Light,
            custom_theme_id: None,
            font_scale: 100,
            width: 60.0,
            line_height: 1.65,
            alignment: AlignmentChoice::Left,
            show_images: false,
            auto_scroll: false,
            scroll_speed_percent: 100,
            marker_color: default_selected_marker(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    AddFolder,
    FolderSelected(Option<PathBuf>),
    RefreshLibrary,
    LibraryLoaded(Vec<LibraryItem>, usize),
    SelectSection(LibrarySection),
    RateBook(PathBuf, u8),
    ToggleBookFavorite(PathBuf),
    OpenBookDetails(PathBuf),
    CloseBookDetails,
    EditBookReview(iced::widget::text_editor::Action),
    DeleteBookComment,
    SearchChanged(String),
    SortSelected(SortBy),
    ToggleListView,
    ToggleSidebar,
    OpenSettings,
    CloseSettings,
    OpenAbout,
    CloseAbout,
    OpenProjectLink(brand::Link),
    ProjectLinkOpened(Result<(), String>),
    WindowResized(iced::window::Id, iced::Size),
    CloseRequested(iced::window::Id),
    CloseFlushed(iced::window::Id, Result<(), String>),
    SaveReadingProgress,
    ThemeSelected(ThemeOption),
    AddCustomTheme,
    EditCustomTheme(String),
    ThemeDraftChanged(ThemeField, String),
    SaveCustomTheme,
    DeleteCustomTheme(String),
    ImportThemes,
    ThemesImported(Result<Vec<ThemeDefinition>, String>),
    ExportThemes,
    ThemesExported(Result<(), String>),
    OpenLibraryBook(PathBuf),
    SelectChapter(usize),
    ToggleAnnotations,
    CloseAnnotations,
    OpenAnnotation(String),
    AnnotationColorSelected(String, String),
    ToggleAnnotationPalette(String),
    PersistAnnotations(u64),
    PreferencesSaved(Result<(), String>),
    AnnotationDraftChanged(String),
    AnnotationEdited(iced::widget::text_editor::Action),
    SubmitAnnotationComment,
    TextSelected(Vec<SelectedExcerpt>),
    SurfacePosition(usize, f32, f32, f32),
    DeleteAnnotation(String),
    ReturnToLibrary,
    ChooseEpub,
    EpubSelected(Option<PathBuf>),
    BookStreamStarted(PathBuf, BookMetadata, usize),
    BookChapterLoaded(PathBuf, usize, Chapter),
    BookCoverLoaded(PathBuf, Option<iced::widget::image::Handle>),
    BookStreamFailed(PathBuf, OpenBookError),
    BookStreamFinished(PathBuf),
    FontCatalogLoaded(FontCatalog),
    FontSelected(String),
    FontSizeChanged(i16),
    WidthChanged(f32),
    LineHeightChanged(f32),
    AlignmentSelected(AlignmentChoice),
    ImagesToggled(bool),
    ToggleAutoScroll,
    AdjustScrollSpeed(i16),
    ToggleToolbarOverflow,
    ToggleCompactMenu(toolbar::Item),
    ToggleThemeMenu,
    FocusTraverse(bool),
    MenuFocus(String, usize),
    MarkerColorSelected(String),
    AddMarkerColor,
    RemoveMarkerColor(usize),
    ProgressChanged(f32),
}

#[derive(Debug, Clone, Copy)]
enum ThemeField {
    Name,
    Background,
    Foreground,
    Muted,
    Accent,
    MarkerColor(usize),
}

fn update(state: &mut Reader, message: Message) -> Task<Message> {
    if state.closing
        && !matches!(
            message,
            Message::PreferencesSaved(_) | Message::CloseFlushed(_, _)
        )
    {
        return Task::none();
    }
    match message {
        Message::AddFolder => choose_folder(),
        Message::FolderSelected(Some(path)) => add_library_folder(state, path),
        Message::FolderSelected(None) => Task::none(),
        Message::RefreshLibrary => {
            state.status = Some("Atualizando a biblioteca em segundo plano…".to_owned());
            scan_task(state.folders.clone())
        }
        Message::LibraryLoaded(books, skipped) => {
            state.library = books;
            reading_history::restore_legacy(state);
            state.status = if skipped > 0 {
                Some(format!(
                    "{} arquivo(s) EPUB não puderam ser lidos.",
                    skipped
                ))
            } else {
                None
            };
            Task::none()
        }
        Message::RateBook(path, stars) => book_details::rate(state, path, stars),
        Message::ToggleBookFavorite(path) => book_details::toggle_favorite(state, path),
        Message::OpenBookDetails(path) => book_details::open(state, path),
        Message::CloseBookDetails => {
            state.book_details_path = None;
            Task::none()
        }
        Message::EditBookReview(action) => book_details::edit(state, action),
        Message::DeleteBookComment => book_details::clear_comment(state),
        Message::SelectSection(section) => {
            state.section = section;
            Task::none()
        }
        Message::SearchChanged(value) => {
            state.search = value;
            Task::none()
        }
        Message::SortSelected(sort) => {
            state.sort_by = sort;
            Task::none()
        }
        Message::ToggleListView => {
            state.list_view = !state.list_view;
            Task::none()
        }
        Message::ToggleSidebar => {
            state.sidebar_open = !state.sidebar_open;
            Task::none()
        }
        Message::OpenSettings => {
            state.book_details_path = None;
            state.about_open = false;
            state.settings_open = !state.settings_open;
            Task::none()
        }
        Message::CloseSettings => {
            state.settings_open = false;
            Task::none()
        }
        Message::OpenAbout => {
            state.book_details_path = None;
            state.about_open = state.page == Page::Library;
            close_reading_menus(state);
            iced_runtime::widget::operation::focus(iced::widget::Id::new("about-close"))
        }
        Message::CloseAbout => {
            state.about_open = false;
            Task::none()
        }
        Message::OpenProjectLink(link) => Task::perform(
            async move {
                tokio::task::spawn_blocking(move || brand::open_link(link))
                    .await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result)
            },
            Message::ProjectLinkOpened,
        ),
        Message::ProjectLinkOpened(result) => {
            if let Err(error) = result {
                state.status = Some(error);
            }
            Task::none()
        }
        Message::WindowResized(id, size) => {
            let minimum = bar::MIN_WIDTH + 24.0;
            state.window_width = size.width.max(minimum);
            close_reading_menus(state);
            if size.width < minimum {
                iced::window::resize(id, iced::Size::new(minimum, size.height))
            } else {
                Task::none()
            }
        }
        Message::AddCustomTheme => {
            state.theme_draft = Some(ThemeDraft::new());
            Task::none()
        }
        Message::EditCustomTheme(id) => {
            state.theme_draft =
                state
                    .custom_themes
                    .iter()
                    .find(|theme| theme.id == id)
                    .map(|theme| ThemeDraft {
                        id: Some(theme.id.clone()),
                        name: theme.name.clone(),
                        background: theme.background.clone(),
                        foreground: theme.foreground.clone(),
                        muted: theme.muted.clone(),
                        accent: theme.accent.clone(),
                        marker_colors: theme.marker_colors.clone(),
                    });
            Task::none()
        }
        Message::ThemeDraftChanged(field, value) => {
            if let Some(draft) = &mut state.theme_draft {
                set_theme_draft_field(draft, field, value);
            }
            Task::none()
        }
        Message::SaveCustomTheme => save_custom_theme(state),
        Message::DeleteCustomTheme(id) => delete_custom_theme(state, &id),
        Message::ImportThemes => import_themes(),
        Message::ThemesImported(result) => apply_imported_themes(state, result),
        Message::ExportThemes => export_themes(all_theme_definitions(&state.custom_themes)),
        Message::ThemesExported(result) => {
            state.status = Some(match result {
                Ok(()) => "Temas exportados.".to_owned(),
                Err(error) => error,
            });
            Task::none()
        }
        Message::OpenLibraryBook(path) => {
            let notes = persist_annotations(state);
            let saved = reading_history::flush(state);
            state.status = Some("Abrindo livro…".to_owned());
            Task::batch([notes, saved, open_library_book(state, &path)])
        }
        Message::SaveReadingProgress => reading_history::flush(state),
        Message::CloseRequested(id) => reading_history::close(state, id),
        Message::CloseFlushed(id, result) => match result {
            Ok(()) => iced::window::close(id),
            Err(error) => {
                state.closing = false;
                state.status = Some(format!("Não foi possível concluir o salvamento: {error}"));
                Task::none()
            }
        },
        Message::SelectChapter(index) => {
            if let Some(book) = &state.book {
                state.chapter_index = index.min(book.chapters.len().saturating_sub(1));
            }
            state.navigation_revision = state.navigation_revision.wrapping_add(1);
            state.navigation_target = (state.chapter_index, 0);
            save_chapter_position(state)
        }
        Message::ToggleAnnotations => {
            state.show_annotations = !state.show_annotations;
            state.annotation_palette_open = None;
            Task::none()
        }
        Message::CloseAnnotations => {
            state.show_annotations = false;
            state.annotation_palette_open = None;
            Task::none()
        }
        Message::OpenAnnotation(id) => open_annotation(state, &id),
        Message::AnnotationColorSelected(id, color) => {
            state.annotation_palette_open = None;
            if theme_definition_for(state).is_some_and(|theme| theme.marker_colors.contains(&color))
                && let Some(annotation) = state.annotations.iter_mut().find(|item| item.id == id)
            {
                annotation.marker_color = color.clone();
                if state.selected_excerpt.as_ref().is_some_and(|selection| {
                    selection.chapter_index == annotation.chapter_index
                        && selection.start_offset == annotation.start_offset
                        && selection.end_offset == annotation.end_offset
                }) {
                    state.settings.marker_color = color;
                }
                return Task::batch([persist_annotations(state), save_preferences(state)]);
            }
            Task::none()
        }
        Message::ToggleAnnotationPalette(id) => {
            state.annotation_palette_open = if state.annotation_palette_open.as_deref() == Some(&id)
            {
                None
            } else {
                Some(id)
            };
            Task::none()
        }
        Message::AnnotationEdited(action) => {
            if let iced::widget::text_editor::Action::Scroll { lines } = action {
                return iced_runtime::widget::operation::scroll_by(
                    iced::widget::Id::new("annotation-scroll"),
                    scrollable::AbsoluteOffset {
                        x: 0.0,
                        y: lines as f32 * 4.0,
                    },
                );
            }
            let changed = action.is_edit();
            state.annotation_editor.perform(action);
            if changed {
                let text = state.annotation_editor.text();
                update(state, Message::AnnotationDraftChanged(text))
            } else {
                Task::none()
            }
        }
        Message::AnnotationDraftChanged(value) => {
            if state.annotation_editor.text() != value {
                state.annotation_editor = iced::widget::text_editor::Content::with_text(&value);
            }
            state.annotation_draft = value;
            let mut changed = false;
            if let Some(selection) = &state.selected_excerpt
                && let Some(annotation) = state.annotations.iter_mut().find(|annotation| {
                    annotation.chapter_index == selection.chapter_index
                        && annotation.start_offset == selection.start_offset
                        && annotation.end_offset == selection.end_offset
                        && matches!(
                            annotation.kind,
                            AnnotationKind::Highlight | AnnotationKind::Comment
                        )
                })
            {
                annotation.text = state.annotation_draft.clone();
                changed = true;
            }
            if changed {
                schedule_annotation_persist(state)
            } else {
                Task::none()
            }
        }
        Message::SubmitAnnotationComment => {
            if let Some(selection) = state.selected_excerpt.as_ref()
                && let Some(annotation) = state.annotations.iter_mut().find(|annotation| {
                    annotation.chapter_index == selection.chapter_index
                        && annotation.start_offset == selection.start_offset
                        && annotation.end_offset == selection.end_offset
                        && matches!(
                            annotation.kind,
                            AnnotationKind::Highlight | AnnotationKind::Comment
                        )
                })
            {
                annotation.text = state.annotation_draft.trim().to_owned();
            }
            persist_annotations(state)
        }
        Message::PersistAnnotations(revision) => {
            if revision == state.annotations_revision {
                persist_annotations(state)
            } else {
                Task::none()
            }
        }
        Message::PreferencesSaved(result) => {
            if let Err(error) = result {
                state.status = Some(format!("Não foi possível salvar os dados locais: {error}"));
            }
            Task::none()
        }
        Message::TextSelected(selections) => {
            state.annotation_palette_open = None;
            let mut tasks = Vec::new();
            for selection in selections {
                state.active_mark_id = None;
                state.annotation_draft.clear();
                state.annotation_editor = iced::widget::text_editor::Content::new();
                state.selected_excerpt = Some(selection);
                tasks.push(persist_selected_excerpt(state));
            }
            Task::batch(tasks)
        }
        Message::DeleteAnnotation(id) => delete_annotation(state, &id),
        Message::ReturnToLibrary => {
            let notes = persist_annotations(state);
            let saved = reading_history::flush(state);
            state.page = Page::Library;
            close_reading_menus(state);
            Task::batch([notes, saved])
        }
        Message::ChooseEpub => choose_epub(),
        Message::EpubSelected(Some(path)) => open_book(state, path),
        Message::EpubSelected(None) => Task::none(),
        Message::BookStreamStarted(path, metadata, total) => {
            begin_progressive_book(state, path, metadata, total)
        }
        Message::BookChapterLoaded(path, index, chapter) => {
            append_progressive_chapter(state, path, index, chapter)
        }
        Message::BookCoverLoaded(path, cover) => {
            if state.book_path.as_deref() == Some(path.as_path()) {
                state.book_cover = cover;
            }
            Task::none()
        }
        Message::BookStreamFailed(path, error) => fail_progressive_book(state, path, error),
        Message::BookStreamFinished(path) => finish_progressive_book(state, path),
        Message::FontCatalogLoaded(catalog) => set_font_catalog(state, catalog),
        Message::FontSelected(family) => {
            state.compact_menu = None;
            let saved = set_font(state, family);
            Task::batch([
                saved,
                iced_runtime::widget::operation::focus(iced::widget::Id::new("font-trigger")),
            ])
        }
        Message::ThemeSelected(option) => {
            state.theme_menu_open = false;
            state.compact_menu = None;
            select_theme(state, &option.id);
            if !theme_definition_for(state)
                .is_some_and(|theme| theme.marker_colors.contains(&state.settings.marker_color))
            {
                state.settings.marker_color = theme_definition_for(state)
                    .and_then(|theme| theme.marker_colors.first())
                    .cloned()
                    .unwrap_or_else(default_selected_marker);
            }
            Task::batch([
                save_preferences(state),
                iced_runtime::widget::operation::focus(iced::widget::Id::new("theme-trigger")),
            ])
        }
        Message::FontSizeChanged(delta) => {
            state.settings.font_scale =
                adjust_percentage(state.settings.font_scale, delta, 50, 300);
            save_preferences(state)
        }
        Message::WidthChanged(value) => {
            state.settings.width = value.clamp(45.0, 100.0);
            save_preferences(state)
        }
        Message::LineHeightChanged(value) => {
            state.settings.line_height = value;
            save_preferences(state)
        }
        Message::AlignmentSelected(value) => {
            state.compact_menu = None;
            state.settings.alignment = value;
            save_preferences(state)
        }
        Message::ImagesToggled(value) => {
            state.settings.show_images = value;
            save_preferences(state)
        }
        Message::ToggleAutoScroll => {
            state.settings.auto_scroll = !state.settings.auto_scroll;
            Task::none()
        }
        Message::AdjustScrollSpeed(delta) => {
            state.settings.scroll_speed_percent =
                next_scroll_speed(state.settings.scroll_speed_percent, delta);
            save_preferences(state)
        }
        Message::ToggleCompactMenu(item) => {
            let was_open = state.compact_menu == Some(item);
            close_reading_menus(state);
            state.compact_menu = (!was_open).then_some(item);
            iced_runtime::widget::operation::focus(iced::widget::Id::new(if was_open {
                item.trigger_id()
            } else {
                item.first_focus()
            }))
        }
        Message::ToggleToolbarOverflow => {
            state.toolbar_overflow_open = !state.toolbar_overflow_open;
            close_reading_menus(state);
            Task::none()
        }
        Message::FocusTraverse(previous) => {
            if previous {
                iced_runtime::widget::operation::focus_previous()
            } else {
                iced_runtime::widget::operation::focus_next()
            }
        }
        Message::MenuFocus(prefix, index) => Task::batch([
            iced_runtime::widget::operation::focus(iced::widget::Id::from(format!(
                "{prefix}-{index}"
            ))),
            iced_runtime::widget::operation::scroll_to(
                iced::widget::Id::from(format!("{prefix}-scroll")),
                scrollable::AbsoluteOffset {
                    x: None,
                    y: Some((index as f32 * 34.0 - 100.0).max(0.0)),
                },
            ),
        ]),
        Message::ToggleThemeMenu => {
            state.theme_menu_open = !state.theme_menu_open;
            iced_runtime::widget::operation::focus(iced::widget::Id::from(
                if state.theme_menu_open {
                    "theme-option-0"
                } else {
                    "theme-trigger"
                }
                .to_owned(),
            ))
        }
        Message::MarkerColorSelected(color) => {
            state.compact_menu = None;
            if theme_definition_for(state).is_some_and(|theme| theme.marker_colors.contains(&color))
            {
                state.settings.marker_color = color;
                if let Some(selection) = state.selected_excerpt.as_ref()
                    && let Some(annotation) = state.annotations.iter_mut().find(|annotation| {
                        annotation.chapter_index == selection.chapter_index
                            && annotation.start_offset == selection.start_offset
                            && annotation.end_offset == selection.end_offset
                            && matches!(
                                annotation.kind,
                                AnnotationKind::Highlight | AnnotationKind::Comment
                            )
                    })
                {
                    annotation.marker_color = state.settings.marker_color.clone();
                }
                return Task::batch([
                    save_preferences(state),
                    persist_annotations(state),
                    iced_runtime::widget::operation::focus(iced::widget::Id::new("marker-trigger")),
                ]);
            }
            Task::none()
        }
        Message::AddMarkerColor => {
            if let Some(draft) = &mut state.theme_draft {
                draft.marker_colors.push("#F9E2AF".to_owned());
            }
            Task::none()
        }
        Message::RemoveMarkerColor(index) => {
            if let Some(draft) = &mut state.theme_draft
                && draft.marker_colors.len() > 1
                && index < draft.marker_colors.len()
            {
                draft.marker_colors.remove(index);
            }
            Task::none()
        }
        Message::SurfacePosition(chapter, value, maximum, viewport_height) => {
            reading_history::remember(state, chapter, value, maximum);
            state.scroll_y = value;
            state.scroll_max = maximum;
            state.scroll_viewport_height = viewport_height.max(1.0);
            state.progress_percent = if maximum <= 0.0 {
                0.0
            } else {
                (value / maximum * 100.0).clamp(0.0, 100.0)
            };
            let chapter_changed = state.chapter_index != chapter;
            state.chapter_index = chapter;
            if chapter_changed && !state.book_loading {
                save_chapter_position(state)
            } else {
                Task::none()
            }
        }
        Message::ProgressChanged(value) => {
            state.progress_percent = value;
            state.scroll_y = state.scroll_max * value / 100.0;
            scroll_to_position(state)
        }
    }
}

fn close_reading_menus(state: &mut Reader) {
    state.compact_menu = None;
    state.theme_menu_open = false;
    state.annotation_palette_open = None;
}

fn subscription(state: &Reader) -> Subscription<Message> {
    Subscription::batch([
        iced::window::close_requests().map(Message::CloseRequested),
        if state.reading_dirty {
            iced::time::every(Duration::from_secs(1)).map(|_| Message::SaveReadingProgress)
        } else {
            Subscription::none()
        },
        iced::window::resize_events().map(|(id, size)| Message::WindowResized(id, size)),
        iced::event::listen_with(|event, status, _| {
            if status == iced::event::Status::Ignored
                && let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
                    modifiers,
                    ..
                }) = event
            {
                return Some(Message::FocusTraverse(modifiers.shift()));
            }
            None
        }),
        if state.book_details_path.is_some() {
            iced::event::listen_with(|event, status, _| {
                (status == iced::event::Status::Ignored
                    && matches!(
                        event,
                        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                            ..
                        })
                    ))
                .then_some(Message::CloseBookDetails)
            })
        } else if state.about_open {
            iced::event::listen_with(|event, status, _| {
                (status == iced::event::Status::Ignored
                    && matches!(
                        event,
                        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                            ..
                        })
                    ))
                .then_some(Message::CloseAbout)
            })
        } else {
            Subscription::none()
        },
    ])
}

fn choose_folder() -> Task<Message> {
    let dialog = rfd::AsyncFileDialog::new().pick_folder();
    Task::perform(dialog, |folder| {
        Message::FolderSelected(folder.map(|item| item.path().to_owned()))
    })
}

fn scan_task(folders: Vec<PathBuf>) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || library::scan_library(&folders))
                .await
                .unwrap_or_else(|_| (Vec::new(), 1))
        },
        |(books, skipped)| Message::LibraryLoaded(books, skipped),
    )
}

fn load_settings(data_dir: &Path) -> ReadingSettings {
    let mut settings: ReadingSettings = library::preference(data_dir, "reading-settings")
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_default();
    if settings.width > 100.0 {
        settings.width = (settings.width / 12.8).clamp(45.0, 100.0);
    } else {
        settings.width = settings.width.clamp(45.0, 100.0);
    }
    if !(50..=300).contains(&settings.font_scale) {
        settings.font_scale = 100;
    }
    settings.scroll_speed_percent = settings.scroll_speed_percent.max(10);
    settings
}

fn load_custom_themes(data_dir: &Path) -> Vec<ThemeDefinition> {
    let stored = library::preference(data_dir, "themes")
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_str::<Vec<ThemeDefinition>>(&value).ok());
    let mut themes = stored.unwrap_or_else(|| {
        let mut themes = builtin_themes();
        if let Some(legacy) = library::preference(data_dir, "custom-themes")
            .ok()
            .flatten()
            .and_then(|value| serde_json::from_str::<Vec<ThemeDefinition>>(&value).ok())
        {
            for theme in legacy {
                if let Some(existing) = themes.iter_mut().find(|existing| existing.id == theme.id) {
                    *existing = theme;
                } else {
                    themes.push(theme);
                }
            }
        }
        themes
    });
    let version = library::preference(data_dir, "themes-schema")
        .ok()
        .flatten()
        .unwrap_or_default();
    if version != "2" {
        for theme in builtin_themes()
            .into_iter()
            .filter(|theme| theme.id.starts_with("catppuccin-"))
        {
            if !themes.iter().any(|existing| existing.id == theme.id) {
                themes.push(theme);
            }
        }
        let value = serde_json::to_string_pretty(&themes).ok();
        if let Some(value) = value {
            let _ = library::save_preference(data_dir, "themes", &value);
        }
        let _ = library::save_preference(data_dir, "themes-schema", "2");
    }
    let defaults = builtin_themes();
    for theme in &mut themes {
        if theme.marker_colors == default_marker_palette() {
            theme.marker_colors = defaults
                .iter()
                .find(|builtin| builtin.id == theme.id)
                .map_or_else(
                    || marker_palette_for(&theme.background),
                    |builtin| builtin.marker_colors.clone(),
                );
        }
    }
    themes
}

fn theme_options(themes: &[ThemeDefinition]) -> Vec<ThemeOption> {
    themes
        .iter()
        .map(|theme| ThemeOption {
            id: if is_builtin_theme(&theme.id) {
                theme.id.clone()
            } else {
                format!("custom:{}", theme.id)
            },
            label: theme.name.clone(),
        })
        .collect()
}

fn is_builtin_theme(id: &str) -> bool {
    matches!(
        id,
        "auto"
            | "light"
            | "paper"
            | "sepia"
            | "dark"
            | "graphite"
            | "night"
            | "dracula"
            | "catppuccin-latte"
            | "catppuccin-frappe"
            | "catppuccin-macchiato"
            | "catppuccin-mocha"
    )
}

fn all_theme_definitions(themes: &[ThemeDefinition]) -> Vec<ThemeDefinition> {
    themes.to_vec()
}

fn builtin_themes() -> Vec<ThemeDefinition> {
    vec![
        theme_definition(
            "auto",
            "Automático",
            "#FFFFFF",
            "#222222",
            "#6B6B6B",
            "#1A4FB4",
        ),
        theme_definition("light", "Claro", "#FFFFFF", "#222222", "#6B6B6B", "#1A4FB4"),
        theme_definition("paper", "Papel", "#FCFBF7", "#2B2722", "#6D665D", "#1A4FB4"),
        theme_definition("sepia", "Sépia", "#FAF7F0", "#342D22", "#70624F", "#7A4B12"),
        theme_definition("dark", "Escuro", "#292A2D", "#D4D4D4", "#AAAAAA", "#8AB4F8"),
        theme_definition(
            "graphite", "Grafite", "#141516", "#DDDDDD", "#AAAAAA", "#8AB4F8",
        ),
        theme_definition("night", "Noite", "#000000", "#E6E6E6", "#AAAAAA", "#8AB4F8"),
        theme_definition(
            "dracula", "Dracula", "#282A36", "#F8F8F2", "#BFBFBF", "#BD93F9",
        ),
        theme_definition(
            "catppuccin-latte",
            "Catppuccin Latte",
            "#EFF1F5",
            "#4C4F69",
            "#6C6F85",
            "#8839EF",
        ),
        theme_definition(
            "catppuccin-frappe",
            "Catppuccin Frappé",
            "#303446",
            "#C6D0F5",
            "#A5ADCE",
            "#CA9EE6",
        ),
        theme_definition(
            "catppuccin-macchiato",
            "Catppuccin Macchiato",
            "#24273A",
            "#CAD3F5",
            "#A5ADCB",
            "#C6A0F6",
        ),
        theme_definition(
            "catppuccin-mocha",
            "Catppuccin Mocha",
            "#1E1E2E",
            "#CDD6F4",
            "#A6ADC8",
            "#CBA6F7",
        ),
    ]
}

fn theme_definition(
    id: &str,
    name: &str,
    background: &str,
    foreground: &str,
    muted: &str,
    accent: &str,
) -> ThemeDefinition {
    ThemeDefinition {
        id: id.to_owned(),
        name: name.to_owned(),
        background: background.to_owned(),
        foreground: foreground.to_owned(),
        muted: muted.to_owned(),
        accent: accent.to_owned(),
        marker_colors: marker_palette_for(background),
    }
}

fn default_marker_palette() -> Vec<String> {
    marker_palette_for("#282A36")
}

fn marker_palette_for(background: &str) -> Vec<String> {
    let colors = if luminance(hex_color(background)) > 0.5 {
        ["#C78900", "#2F7A3E", "#2864B4", "#A33C77", "#7145AE"]
    } else {
        ["#F9E2AF", "#A6E3A1", "#89B4FA", "#F5C2E7", "#CBA6F7"]
    };
    colors.into_iter().map(str::to_owned).collect()
}

fn default_selected_marker() -> String {
    "#F9E2AF".to_owned()
}

fn current_theme_id(settings: &ReadingSettings) -> String {
    match settings.theme {
        ThemeChoice::Automatic => "auto".to_owned(),
        ThemeChoice::Light => "light".to_owned(),
        ThemeChoice::Paper => "paper".to_owned(),
        ThemeChoice::Sepia => "sepia".to_owned(),
        ThemeChoice::Dark => "dark".to_owned(),
        ThemeChoice::Graphite => "graphite".to_owned(),
        ThemeChoice::Night => "night".to_owned(),
        ThemeChoice::Dracula => "dracula".to_owned(),
        ThemeChoice::CatppuccinLatte => "catppuccin-latte".to_owned(),
        ThemeChoice::CatppuccinFrappe => "catppuccin-frappe".to_owned(),
        ThemeChoice::CatppuccinMacchiato => "catppuccin-macchiato".to_owned(),
        ThemeChoice::CatppuccinMocha => "catppuccin-mocha".to_owned(),
        ThemeChoice::Custom => settings.custom_theme_id.clone().unwrap_or_default(),
    }
}

fn select_theme(state: &mut Reader, id: &str) {
    state.settings.custom_theme_id = None;
    state.settings.theme = match id {
        "auto" => ThemeChoice::Automatic,
        "light" => ThemeChoice::Light,
        "paper" => ThemeChoice::Paper,
        "sepia" => ThemeChoice::Sepia,
        "dark" => ThemeChoice::Dark,
        "graphite" => ThemeChoice::Graphite,
        "night" => ThemeChoice::Night,
        "dracula" => ThemeChoice::Dracula,
        "catppuccin-latte" => ThemeChoice::CatppuccinLatte,
        "catppuccin-frappe" => ThemeChoice::CatppuccinFrappe,
        "catppuccin-macchiato" => ThemeChoice::CatppuccinMacchiato,
        "catppuccin-mocha" => ThemeChoice::CatppuccinMocha,
        custom_id if custom_id.starts_with("custom:") => {
            let id = custom_id.trim_start_matches("custom:");
            if state.custom_themes.iter().any(|theme| theme.id == id) {
                state.settings.custom_theme_id = Some(id.to_owned());
                ThemeChoice::Custom
            } else {
                ThemeChoice::Light
            }
        }
        _ => ThemeChoice::Light,
    };
}

fn hex_color(value: &str) -> Color {
    let hex = value.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::WHITE;
    }
    let Ok(rgb) = u32::from_str_radix(hex, 16) else {
        return Color::WHITE;
    };
    Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

fn valid_hex(value: &str) -> bool {
    let hex = value.strip_prefix('#').unwrap_or(value);
    hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit())
}

fn set_theme_draft_field(draft: &mut ThemeDraft, field: ThemeField, value: String) {
    match field {
        ThemeField::Name => draft.name = value,
        ThemeField::Background => draft.background = value,
        ThemeField::Foreground => draft.foreground = value,
        ThemeField::Muted => draft.muted = value,
        ThemeField::Accent => draft.accent = value,
        ThemeField::MarkerColor(index) => {
            if let Some(color) = draft.marker_colors.get_mut(index) {
                *color = value;
            }
        }
    }
}

fn save_custom_themes(state: &mut Reader) -> Result<(), String> {
    let value =
        serde_json::to_string_pretty(&state.custom_themes).map_err(|error| error.to_string())?;
    library::save_preference(&state.data_dir, "themes", &value).map_err(|error| error.to_string())
}

fn save_custom_theme(state: &mut Reader) -> Task<Message> {
    let Some(draft) = state.theme_draft.take() else {
        return Task::none();
    };
    if draft.name.trim().is_empty()
        || [
            &draft.background,
            &draft.foreground,
            &draft.muted,
            &draft.accent,
        ]
        .iter()
        .any(|color| !valid_hex(color))
        || draft.marker_colors.is_empty()
        || draft.marker_colors.iter().any(|color| !valid_hex(color))
    {
        state.status = Some("Informe o nome e cores hexadecimais no formato #RRGGBB.".to_owned());
        state.theme_draft = Some(draft);
        return Task::none();
    }
    let id = draft.id.unwrap_or_else(|| {
        format!(
            "tema-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
    });
    let theme = ThemeDefinition {
        id: id.clone(),
        name: draft.name.trim().to_owned(),
        background: draft.background,
        foreground: draft.foreground,
        muted: draft.muted,
        accent: draft.accent,
        marker_colors: draft.marker_colors,
    };
    if let Some(existing) = state.custom_themes.iter_mut().find(|theme| theme.id == id) {
        *existing = theme;
    } else {
        state.custom_themes.push(theme);
    }
    state
        .custom_themes
        .sort_by_key(|theme| theme.name.to_lowercase());
    state.settings.theme = ThemeChoice::Custom;
    state.settings.custom_theme_id = Some(id);
    state.settings.marker_color = theme_definition_for(state)
        .and_then(|theme| theme.marker_colors.first())
        .cloned()
        .unwrap_or_else(default_selected_marker);
    state.status = save_custom_themes(state).err();
    save_preferences(state)
}

fn delete_custom_theme(state: &mut Reader, id: &str) -> Task<Message> {
    state.custom_themes.retain(|theme| theme.id != id);
    if current_theme_id(&state.settings) == id {
        let fallback = state
            .custom_themes
            .iter()
            .find(|theme| theme.id == "light")
            .or_else(|| state.custom_themes.first())
            .map(|theme| theme.id.clone())
            .unwrap_or_else(|| "light".to_owned());
        select_theme(state, &fallback);
    }
    state.status = save_custom_themes(state).err();
    save_preferences(state)
}

fn import_themes() -> Task<Message> {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter("Temas JSON", &["json"])
        .pick_file();
    Task::perform(
        async move {
            let Some(file) = dialog.await else {
                return Err("Importação cancelada.".to_owned());
            };
            let source = std::fs::read_to_string(file.path()).map_err(|error| error.to_string())?;
            let value: serde_json::Value =
                serde_json::from_str(&source).map_err(|error| error.to_string())?;
            let themes = value.get("themes").cloned().unwrap_or(value);
            serde_json::from_value::<Vec<ThemeDefinition>>(themes)
                .map_err(|error| error.to_string())
        },
        Message::ThemesImported,
    )
}

fn export_themes(themes: Vec<ThemeDefinition>) -> Task<Message> {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter("Temas JSON", &["json"])
        .set_file_name("tourne-la-page-temas.json")
        .save_file();
    Task::perform(
        async move {
            let Some(file) = dialog.await else {
                return Err("Exportação cancelada.".to_owned());
            };
            let source = serde_json::to_vec_pretty(&themes).map_err(|error| error.to_string())?;
            std::fs::write(file.path(), source).map_err(|error| error.to_string())
        },
        Message::ThemesExported,
    )
}

fn apply_imported_themes(
    state: &mut Reader,
    result: Result<Vec<ThemeDefinition>, String>,
) -> Task<Message> {
    match result {
        Ok(themes)
            if themes.iter().all(|theme| {
                !theme.name.trim().is_empty()
                    && [
                        &theme.background,
                        &theme.foreground,
                        &theme.muted,
                        &theme.accent,
                    ]
                    .iter()
                    .all(|color| valid_hex(color))
                    && !theme.marker_colors.is_empty()
                    && theme.marker_colors.iter().all(|color| valid_hex(color))
            }) =>
        {
            for theme in themes {
                if let Some(existing) = state
                    .custom_themes
                    .iter_mut()
                    .find(|existing| existing.id == theme.id)
                {
                    *existing = theme;
                } else {
                    state.custom_themes.push(theme);
                }
            }
            state
                .custom_themes
                .sort_by_key(|theme| theme.name.to_lowercase());
            state.status = save_custom_themes(state).err();
        }
        Ok(_) => state.status = Some("O JSON tem cores ou nomes inválidos.".to_owned()),
        Err(error) => state.status = Some(format!("Não foi possível importar temas: {error}")),
    }
    Task::none()
}

enum PreferenceValue {
    ReadingProgress(reading_history::Progress),
    BookReview(book_details::Review),
    Settings(ReadingSettings),
    Annotations(Vec<BookAnnotation>),
    Text(String),
}

fn queue_preference(state: &Reader, key: String, value: PreferenceValue) -> Task<Message> {
    preferences::queue(state, key, value)
}

fn save_preferences(state: &Reader) -> Task<Message> {
    queue_preference(
        state,
        "reading-settings".to_owned(),
        PreferenceValue::Settings(state.settings.clone()),
    )
}

fn set_last_opened(state: &mut Reader, path: &Path) -> Task<Message> {
    state.last_opened_path = Some(path.to_owned());
    let reading = reading_history::started(state, path);
    Task::batch([
        reading,
        queue_preference(
            state,
            "last-opened".to_owned(),
            PreferenceValue::Text(path.to_string_lossy().into_owned()),
        ),
    ])
}

fn load_chapter_position(state: &Reader, path: &Path) -> usize {
    let key = format!("chapter:{}", path.to_string_lossy());
    library::preference(&state.data_dir, &key)
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

fn save_chapter_position(state: &Reader) -> Task<Message> {
    let (Some(path), Some(book)) = (&state.book_path, &state.book) else {
        return Task::none();
    };
    let index = state
        .chapter_index
        .min(book.chapters.len().saturating_sub(1));
    queue_preference(
        state,
        format!("chapter:{}", path.to_string_lossy()),
        PreferenceValue::Text(index.to_string()),
    )
}

fn annotation_key(path: &Path) -> String {
    format!("annotations:{}", path.to_string_lossy())
}

fn load_annotations(data_dir: &Path, path: &Path) -> Vec<BookAnnotation> {
    library::preference(data_dir, &annotation_key(path))
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_default()
}

fn persist_annotations(state: &Reader) -> Task<Message> {
    let Some(path) = &state.book_path else {
        return Task::none();
    };
    queue_preference(
        state,
        annotation_key(path),
        PreferenceValue::Annotations(state.annotations.clone()),
    )
}

fn schedule_annotation_persist(state: &mut Reader) -> Task<Message> {
    state.annotations_revision = state.annotations_revision.wrapping_add(1);
    let revision = state.annotations_revision;
    Task::perform(
        async move {
            tokio::time::sleep(Duration::from_millis(250)).await;
            revision
        },
        Message::PersistAnnotations,
    )
}

fn persist_selected_excerpt(state: &mut Reader) -> Task<Message> {
    let Some(selection) = state.selected_excerpt.clone() else {
        return Task::none();
    };
    if let Some(active_id) = state.active_mark_id.clone()
        && let Some(existing) = state
            .annotations
            .iter_mut()
            .find(|annotation| annotation.id == active_id)
    {
        existing.chapter_index = selection.chapter_index;
        existing.excerpt = selection.text.clone();
        existing.start_offset = selection.start_offset;
        existing.end_offset = selection.end_offset;
    } else if let Some(existing) = state.annotations.iter_mut().find(|annotation| {
        annotation.chapter_index == selection.chapter_index
            && annotation.start_offset == selection.start_offset
            && annotation.end_offset == selection.end_offset
            && matches!(
                annotation.kind,
                AnnotationKind::Highlight | AnnotationKind::Comment
            )
    }) {
        state.annotation_draft = existing.text.clone();
        state.active_mark_id = Some(existing.id.clone());
    } else {
        let id = new_local_id();
        state.annotations.push(BookAnnotation {
            id: id.clone(),
            chapter_index: selection.chapter_index,
            kind: AnnotationKind::Highlight,
            text: state.annotation_draft.trim().to_owned(),
            excerpt: selection.text,
            start_offset: selection.start_offset,
            end_offset: selection.end_offset,
            marker_color: selected_marker_color(state),
        });
        state.active_mark_id = Some(id);
    }
    state.annotation_editor =
        iced::widget::text_editor::Content::with_text(&state.annotation_draft);
    state
        .annotation_editor
        .perform(iced::widget::text_editor::Action::Move(
            iced::widget::text_editor::Motion::DocumentEnd,
        ));
    state.show_annotations = true;
    let persist = persist_annotations(state);
    Task::batch([
        persist,
        iced_runtime::widget::operation::focus(iced::widget::Id::new("annotation-comment")),
    ])
}

fn chapter_paragraph_ranges(chapter: &Chapter) -> Vec<(usize, usize)> {
    let mut offset = 0usize;
    chapter
        .text
        .split("\n\n")
        .filter_map(|paragraph| {
            let start = offset;
            offset += paragraph.len() + 2;
            (!paragraph.trim().is_empty()).then_some((start, start + paragraph.len()))
        })
        .collect()
}

fn delete_annotation(state: &mut Reader, id: &str) -> Task<Message> {
    if state.annotation_palette_open.as_deref() == Some(id) {
        state.annotation_palette_open = None;
    }
    state.annotations.retain(|annotation| annotation.id != id);
    if state.active_mark_id.as_deref() == Some(id) {
        state.active_mark_id = None;
        state.selected_excerpt = None;
        state.annotation_draft.clear();
        state.annotation_editor = iced::widget::text_editor::Content::new();
    }
    persist_annotations(state)
}

fn open_annotation(state: &mut Reader, id: &str) -> Task<Message> {
    let Some(annotation) = state.annotations.iter().find(|item| item.id == id) else {
        return Task::none();
    };
    let chapter_index = annotation.chapter_index;
    let start_offset = annotation.start_offset;
    let end_offset = annotation.end_offset;
    let excerpt = annotation.excerpt.clone();
    let comment = annotation.text.clone();
    let marker_color = annotation.marker_color.clone();
    let is_bookmark = matches!(annotation.kind, AnnotationKind::Bookmark);
    let annotation_id = annotation.id.clone();
    let Some(book) = &state.book else {
        return Task::none();
    };
    let target_chapter = chapter_index.min(book.chapters.len().saturating_sub(1));
    state.chapter_index = target_chapter;
    state.selected_excerpt = (!is_bookmark).then_some(SelectedExcerpt {
        chapter_index: target_chapter,
        text: excerpt,
        start_offset,
        end_offset,
    });
    state.active_mark_id = (!is_bookmark).then_some(annotation_id);
    state.annotation_draft = comment;
    state.annotation_editor =
        iced::widget::text_editor::Content::with_text(&state.annotation_draft);
    state
        .annotation_editor
        .perform(iced::widget::text_editor::Action::Move(
            iced::widget::text_editor::Motion::DocumentEnd,
        ));
    state.settings.marker_color = marker_color;
    state.show_annotations = true;
    state.navigation_revision = state.navigation_revision.wrapping_add(1);
    state.navigation_target = (target_chapter, start_offset);
    state.annotation_palette_open = None;
    let scroll = iced_runtime::widget::operation::scroll_to(
        iced::widget::Id::new("annotation-scroll"),
        scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
    );
    if is_bookmark {
        scroll
    } else {
        Task::batch([
            scroll,
            iced_runtime::widget::operation::focus(iced::widget::Id::new("annotation-comment")),
        ])
    }
}

fn new_local_id() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn scroll_to_position(state: &Reader) -> Task<Message> {
    iced_runtime::task::widget(iced::advanced::widget::operation::scrollable::scroll_to(
        state.scroll_id.clone(),
        scrollable::AbsoluteOffset {
            x: None,
            y: Some(state.scroll_y),
        },
    ))
}

fn adaptive_scroll_thumb(viewport_height: f32, progress_percent: f32) -> (f32, f32) {
    let viewport = viewport_height.max(1.0);
    let max_height = (viewport * 0.24).max(1.0);
    let min_height = viewport.min(48.0);
    let progress = progress_percent.clamp(0.0, 100.0) / 100.0;
    let height =
        (viewport * (0.24 - 0.16 * progress)).clamp(min_height.min(max_height), max_height);
    let top = (viewport - height) * progress;
    (height, top)
}

fn add_library_folder(state: &mut Reader, path: PathBuf) -> Task<Message> {
    if let Err(error) = library::add_folder(&state.data_dir, &path) {
        state.status = Some(format!("Não foi possível salvar a pasta: {error}"));
        return Task::none();
    }
    if !state.folders.contains(&path) {
        state.folders.push(path);
    }
    state.status = Some("Indexando os EPUBs em segundo plano…".to_owned());
    scan_task(state.folders.clone())
}

fn open_library_book(state: &mut Reader, path: &Path) -> Task<Message> {
    let Some(item) = state.library.iter().find(|item| item.path == path) else {
        return Task::none();
    };
    let book = Arc::clone(&item.book);
    let cover = item.cover.clone();
    let book_path = item.path.clone();
    let chapter_count = book.chapters.len();
    state.book = Some(book.clone());
    state.selected_excerpt = None;
    state.book_loading = false;
    state.expected_chapters = chapter_count;
    state.chapter_paragraph_ranges = book.chapters.iter().map(chapter_paragraph_ranges).collect();
    state.resume_chapter = None;
    state.book_path = Some(book_path);
    state.book_cover = cover;
    state.annotations_revision = state.annotations_revision.wrapping_add(1);
    state.annotations = preferences::annotations(state, path);
    let opened = set_last_opened(state, path);
    state.chapter_index = load_chapter_position(state, path).min(chapter_count.saturating_sub(1));
    state.scroll_y = 0.0;
    state.page = Page::Reader;
    state.status = None;
    state.navigation_revision = state.navigation_revision.wrapping_add(1);
    state.navigation_target = (state.chapter_index, 0);
    opened
}

fn choose_epub() -> Task<Message> {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter("Livros EPUB", &["epub"])
        .pick_file();
    Task::perform(dialog, |file| {
        Message::EpubSelected(file.map(|item| item.path().to_owned()))
    })
}

#[derive(Debug)]
enum BookLoadEvent {
    Started(BookMetadata, usize),
    Chapter(usize, Chapter),
    Cover(Option<iced::widget::image::Handle>),
    Failed(OpenBookError),
    Finished,
}

fn open_book(state: &mut Reader, path: PathBuf) -> Task<Message> {
    state.status = Some("Preparando a primeira tela do livro…".to_owned());
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("epub"))
    {
        return Task::done(Message::BookStreamFailed(
            path,
            OpenBookError::UnsupportedFormat,
        ));
    }
    let stream_path = path.clone();
    let stream = iced::stream::channel(4, async move |mut output| {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(4);
        let worker = tokio::task::spawn_blocking(move || {
            let parsed = std::fs::File::open(&path)
                .map_err(|_| BookError::SourceUnavailable)
                .and_then(|file| {
                    parse_epub_progressive(
                        file,
                        |metadata, total| {
                            let _ = sender.blocking_send(BookLoadEvent::Started(metadata, total));
                        },
                        |index, chapter| {
                            let _ = sender.blocking_send(BookLoadEvent::Chapter(index, chapter));
                        },
                        |cover| {
                            let cover = cover.as_deref().and_then(library::decode_cover);
                            let _ = sender.blocking_send(BookLoadEvent::Cover(cover));
                        },
                    )
                });
            match parsed {
                Ok(()) => {
                    let _ = sender.blocking_send(BookLoadEvent::Finished);
                }
                Err(error) => {
                    let _ = sender.blocking_send(BookLoadEvent::Failed(OpenBookError::Book(error)));
                }
            }
        });
        while let Some(event) = receiver.recv().await {
            use iced::futures::sink::SinkExt;
            if output.send((stream_path.clone(), event)).await.is_err() {
                break;
            }
        }
        let _ = worker.await;
    });
    Task::run(stream, |(path, event)| match event {
        BookLoadEvent::Started(metadata, total) => {
            Message::BookStreamStarted(path, metadata, total)
        }
        BookLoadEvent::Chapter(index, chapter) => Message::BookChapterLoaded(path, index, chapter),
        BookLoadEvent::Cover(cover) => Message::BookCoverLoaded(path, cover),
        BookLoadEvent::Failed(error) => Message::BookStreamFailed(path, error),
        BookLoadEvent::Finished => Message::BookStreamFinished(path),
    })
}

fn begin_progressive_book(
    state: &mut Reader,
    path: PathBuf,
    metadata: BookMetadata,
    total: usize,
) -> Task<Message> {
    let notes = persist_annotations(state);
    let saved = reading_history::flush(state);
    let resume = load_chapter_position(state, &path);
    state.resume_chapter = (resume > 0).then_some(resume);
    state.navigation_revision = state.navigation_revision.wrapping_add(1);
    state.navigation_target = (0, 0);
    state.book_loading = true;
    state.expected_chapters = total;
    state.chapter_index = 0;
    state.scroll_y = 0.0;
    state.book_cover = None;
    state.book = Some(Arc::new(Book {
        metadata,
        chapters: Vec::with_capacity(total),
        cover: None,
    }));
    state.chapter_paragraph_ranges.clear();
    state.book_path = Some(path.clone());
    state.annotations_revision = state.annotations_revision.wrapping_add(1);
    state.annotations = preferences::annotations(state, &path);
    state.selected_excerpt = None;
    state.show_annotations = false;
    state.page = Page::Reader;
    let opened = set_last_opened(state, &path);
    state.status = Some(format!("Carregando o primeiro trecho · 0/{total}"));
    Task::batch([notes, saved, opened])
}

fn append_progressive_chapter(
    state: &mut Reader,
    path: PathBuf,
    index: usize,
    chapter: Chapter,
) -> Task<Message> {
    if state.book_path.as_deref() != Some(path.as_path()) {
        return Task::none();
    }
    let Some(book) = &state.book else {
        return Task::none();
    };
    if index != book.chapters.len() {
        return Task::none();
    }
    if state.resume_chapter == Some(index) {
        state.chapter_index = index;
        state.resume_chapter = None;
        state.navigation_revision = state.navigation_revision.wrapping_add(1);
        state.navigation_target = (index, 0);
    }
    Arc::make_mut(state.book.as_mut().expect("book exists"))
        .chapters
        .push(chapter);
    let ranges = state
        .book
        .as_ref()
        .and_then(|book| book.chapters.get(index))
        .map(chapter_paragraph_ranges)
        .unwrap_or_default();
    state.chapter_paragraph_ranges.push(ranges);
    let loaded = index + 1;
    state.status = Some(format!(
        "Carregando livro · {loaded}/{} capítulos",
        state.expected_chapters
    ));
    Task::none()
}

fn fail_progressive_book(state: &mut Reader, path: PathBuf, error: OpenBookError) -> Task<Message> {
    if state.book_path.as_deref() == Some(path.as_path()) {
        state.book_loading = false;
        state.status = Some(friendly_error(error));
        if state
            .book
            .as_ref()
            .is_none_or(|book| book.chapters.is_empty())
        {
            state.page = Page::Library;
            state.book = None;
        }
    }
    Task::none()
}

fn finish_progressive_book(state: &mut Reader, path: PathBuf) -> Task<Message> {
    if state.book_path.as_deref() != Some(path.as_path()) {
        return Task::none();
    }
    state.book_loading = false;
    if let (Some(resume), Some(book)) = (state.resume_chapter.take(), &state.book) {
        state.chapter_index = resume.min(book.chapters.len().saturating_sub(1));
        state.navigation_revision = state.navigation_revision.wrapping_add(1);
        state.navigation_target = (state.chapter_index, 0);
    }
    state.status = None;
    save_chapter_position(state)
}

fn set_font_catalog(state: &mut Reader, catalog: FontCatalog) -> Task<Message> {
    if !state.catalog_loaded {
        state.selected_font = library::preference(&state.data_dir, "reading-font")
            .ok()
            .flatten()
            .filter(|family| catalog.options().contains(family))
            .unwrap_or_else(|| catalog.selected_default().to_owned());
        state.catalog_loaded = true;
    }
    state.font_catalog = catalog;
    Task::none()
}

fn set_font(state: &mut Reader, family: String) -> Task<Message> {
    if state.font_catalog.options().contains(&family) {
        state.selected_font = family;
        return queue_preference(
            state,
            "reading-font".into(),
            PreferenceValue::Text(state.selected_font.clone()),
        );
    }
    Task::none()
}

fn view(state: &Reader) -> Element<'_, Message> {
    let page = match state.page {
        Page::Library => library_screen::page(state),
        Page::Reader => reader_page(state),
    };
    if state.book_details_path.is_some() && state.page == Page::Library {
        stack![page, book_details::view(state)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else if state.about_open && state.page == Page::Library {
        stack![page, library_screen::about(state)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else if state.settings_open {
        stack![page, settings_modal(state)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        page
    }
}

fn overflow_panel<'a>(state: &Reader, controls: Element<'a, Message>) -> Element<'a, Message> {
    let surface = surface_for(state);
    let border = border_for(state);
    container(controls)
        .padding(12)
        .width(Length::Fill)
        .max_width(520.0)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(surface)),
            border: iced::Border::default()
                .rounded(8.0)
                .color(border)
                .width(1.0),
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        })
        .into()
}

fn font_menu(state: &Reader) -> Element<'_, Message> {
    let mut items = column![].spacing(2);
    let options = state.font_catalog.options();
    let total = options.len();
    if state.compact_menu == Some(toolbar::Item::Font) {
        for (index, family) in options.into_iter().enumerate() {
            let message = Message::FontSelected(family.clone());
            items = items.push(action::menu_item(
                button(text(family).size(14))
                    .width(Length::Fill)
                    .height(32)
                    .on_press(message.clone())
                    .style(iced::widget::button::text)
                    .into(),
                "font-option",
                index,
                total,
                message,
            ));
        }
    }
    overflow_panel(
        state,
        adaptive_scroll::new(items)
            .id(iced::widget::Id::new("font-option-scroll"))
            .height(Length::Fixed((total as f32 * 34.0).min(320.0)))
            .into(),
    )
}

fn font_size_control(state: &Reader) -> Element<'_, Message> {
    row![
        action::focusable(
            button(text("A−").size(14))
                .width(32)
                .height(32)
                .padding(4)
                .on_press(Message::FontSizeChanged(-10))
                .style(move |_, status| picker_style(state, status))
                .into(),
            "font-smaller",
            Message::FontSizeChanged(-10),
            None
        ),
        container(
            text(format!("{}%", state.settings.font_scale))
                .size(14)
                .align_x(iced::alignment::Horizontal::Center)
                .width(50)
        )
        .id(iced::widget::Id::new("font-size-value")),
        action::focusable(
            button(text("A+").size(14))
                .width(32)
                .height(32)
                .padding(4)
                .on_press(Message::FontSizeChanged(10))
                .style(move |_, status| picker_style(state, status))
                .into(),
            "font-larger",
            Message::FontSizeChanged(10),
            None
        ),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn width_control(state: &Reader) -> Element<'_, Message> {
    column![
        text("Largura").size(13),
        action::focusable(
            iced::widget::vertical_slider(
                45.0..=100.0,
                state.settings.width,
                Message::WidthChanged
            )
            .step(5.0_f32)
            .height(140)
            .width(32)
            .into(),
            "reading-width",
            Message::WidthChanged(state.settings.width),
            Some((
                Message::WidthChanged((state.settings.width - 5.0).max(45.0)),
                Message::WidthChanged((state.settings.width + 5.0).min(100.0))
            ))
        ),
        text(format!("{:.0}%", state.settings.width)).size(14),
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn line_height_control(state: &Reader) -> Element<'_, Message> {
    column![
        text("Entrelinha").size(13),
        action::focusable(
            iced::widget::vertical_slider(
                1.2..=2.2,
                state.settings.line_height,
                Message::LineHeightChanged
            )
            .step(0.05_f32)
            .height(140)
            .width(32)
            .into(),
            "reading-line-height",
            Message::LineHeightChanged(state.settings.line_height),
            Some((
                Message::LineHeightChanged((state.settings.line_height - 0.05).max(1.2)),
                Message::LineHeightChanged((state.settings.line_height + 0.05).min(2.2))
            ))
        ),
        text(format!("{:.2}", state.settings.line_height)).size(14),
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn scroll_speed_control(state: &Reader) -> Element<'_, Message> {
    row![
        action::focusable(
            button(icon_svg("minus", 16))
                .width(32)
                .height(32)
                .padding(8)
                .on_press(Message::AdjustScrollSpeed(-1))
                .style(move |_, status| picker_style(state, status))
                .into(),
            "speed-smaller",
            Message::AdjustScrollSpeed(-1),
            None
        ),
        container(
            text(format!("{}%", state.settings.scroll_speed_percent))
                .size(14)
                .width(50)
                .align_x(iced::alignment::Horizontal::Center)
                .color(muted_for(state))
        )
        .id(iced::widget::Id::new("scroll-speed-value")),
        action::focusable(
            button(icon_svg("plus", 16))
                .width(32)
                .height(32)
                .padding(8)
                .on_press(Message::AdjustScrollSpeed(1))
                .style(move |_, status| picker_style(state, status))
                .into(),
            "speed-larger",
            Message::AdjustScrollSpeed(1),
            None
        ),
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .into()
}

fn scroll_speed_control_labeled(state: &Reader) -> Element<'_, Message> {
    row![
        text("Velocidade").size(14).color(muted_for(state)),
        scroll_speed_control(state)
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn settings_modal(state: &Reader) -> Element<'_, Message> {
    let backdrop: Element<'_, Message> = button("")
        .on_press(Message::CloseSettings)
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.04, 0.05, 0.08, 0.72,
            ))),
            text_color: Color::TRANSPARENT,
            ..Default::default()
        })
        .into();
    let modal = settings_panel(state);
    let border = border_for(state);
    let surface = surface_for(state);
    let panel = container(modal)
        .width(Length::Fill)
        .max_width(900.0)
        .height(Length::Fill)
        .max_height(720.0)
        .padding(24)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(surface)),
            border: iced::Border::default()
                .rounded(14.0)
                .color(border)
                .width(1.0),
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.28),
                offset: iced::Vector::new(0.0, 14.0),
                blur_radius: 28.0,
            },
            ..Default::default()
        });
    let centered: Element<'_, Message> = container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .center(Length::Fill)
        .into();
    stack![backdrop, centered]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn settings_panel(state: &Reader) -> Element<'_, Message> {
    let body = if let Some(draft) = &state.theme_draft {
        theme_editor(draft)
    } else {
        column![
            row![
                text("Temas").font(Font::with_name("Literata")).size(28),
                iced::widget::space::horizontal(),
                button("Novo tema").on_press(Message::AddCustomTheme),
                button("Importar JSON").on_press(Message::ImportThemes),
                button("Exportar JSON").on_press(Message::ExportThemes),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text("Edite qualquer tema. Suas alterações ficam salvas neste computador.")
                .color(muted_for(state)),
            custom_theme_list(state),
        ]
        .spacing(16)
        .into()
    };
    container(
        column![
            row![
                text("Configurações de tema")
                    .font(Font::with_name("Literata"))
                    .size(26),
                iced::widget::space::horizontal(),
                button(icon_svg("x", 18)).on_press(Message::CloseSettings),
            ]
            .align_y(Alignment::Center),
            adaptive_scroll::new(body).height(Length::Fill),
            state
                .status
                .as_ref()
                .map(|status| {
                    text(status).color(ensure_contrast(
                        Color::from_rgb8(179, 38, 30),
                        surface_for(state),
                        4.5,
                    ))
                })
                .unwrap_or_else(|| text("")),
        ]
        .spacing(18)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn custom_theme_list(state: &Reader) -> Element<'_, Message> {
    let border = border_for(state);
    let mut list = column![text("Temas disponíveis").size(15)].spacing(8);
    if state.custom_themes.is_empty() {
        list = list.push(
            text("Nenhum tema disponível. Importe um JSON ou crie um novo.")
                .size(13)
                .color(muted_for(state)),
        );
    }
    for theme in &state.custom_themes {
        let id = theme.id.clone();
        let delete_id = id.clone();
        let option_id = if is_builtin_theme(&id) {
            id.clone()
        } else {
            format!("custom:{id}")
        };
        let current = current_theme_id(&state.settings) == id;
        let option = ThemeOption {
            id: option_id,
            label: theme.name.clone(),
        };
        list = list.push(
            container(
                row![
                    container(text(" "))
                        .width(Length::Fixed(26.0))
                        .height(Length::Fixed(26.0))
                        .style(move |_| container::Style {
                            background: Some(iced::Background::Color(hex_color(&theme.background))),
                            ..Default::default()
                        }),
                    column![
                        text(&theme.name).size(15),
                        text(format!("{} · {}", theme.background, theme.accent))
                            .size(12)
                            .color(muted_for(state))
                    ],
                    iced::widget::space::horizontal(),
                    button(if current { "Em uso" } else { "Usar" })
                        .on_press_maybe((!current).then_some(Message::ThemeSelected(option))),
                    button("Editar").on_press(Message::EditCustomTheme(id)),
                    button("Excluir").on_press(Message::DeleteCustomTheme(delete_id)),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .padding(10),
            )
            .width(Length::Fill)
            .style(move |_| container::Style {
                border: iced::Border::default()
                    .rounded(8.0)
                    .color(border)
                    .width(1.0),
                ..Default::default()
            }),
        );
    }
    list.into()
}

fn theme_editor(draft: &ThemeDraft) -> Element<'_, Message> {
    let mut editor = column![
        text(if draft.id.is_some() {
            "Editar tema"
        } else {
            "Criar tema"
        })
        .font(Font::with_name("Literata"))
        .size(22),
        theme_field("Nome", &draft.name, ThemeField::Name),
        theme_field("Fundo (#RRGGBB)", &draft.background, ThemeField::Background),
        theme_field("Texto (#RRGGBB)", &draft.foreground, ThemeField::Foreground),
        theme_field(
            "Texto secundário (#RRGGBB)",
            &draft.muted,
            ThemeField::Muted
        ),
        theme_field("Destaque (#RRGGBB)", &draft.accent, ThemeField::Accent),
    ]
    .spacing(12);
    for (index, color) in draft.marker_colors.iter().enumerate() {
        editor = editor.push(
            row![
                theme_field(
                    "Cor de marcação (#RRGGBB)",
                    color,
                    ThemeField::MarkerColor(index)
                ),
                button(icon_svg("x", 16)).on_press(Message::RemoveMarkerColor(index)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );
    }
    editor = editor.push(button("Adicionar cor de marcação").on_press(Message::AddMarkerColor));
    editor = editor.push(
        row![
            button("Salvar tema").on_press(Message::SaveCustomTheme),
            button("Cancelar").on_press(Message::CloseSettings)
        ]
        .spacing(8),
    );
    editor.into()
}

fn theme_field<'a>(label: &'static str, value: &'a str, field: ThemeField) -> Element<'a, Message> {
    text_input(label, value)
        .on_input(move |value| Message::ThemeDraftChanged(field, value))
        .padding(8)
        .into()
}

fn library_sidebar(state: &Reader) -> Element<'_, Message> {
    let surface = surface_for(state);
    let border = border_for(state);
    let foreground = foreground_for(state);
    let sections = [
        LibrarySection::Home,
        LibrarySection::AllBooks,
        LibrarySection::Reading,
        LibrarySection::Rating,
        LibrarySection::Favorites,
        LibrarySection::Authors,
        LibrarySection::Series,
        LibrarySection::Genres,
        LibrarySection::Language,
        LibrarySection::Publisher,
        LibrarySection::Collections,
    ];
    let mut menu = column![text("BIBLIOTECA").size(11).color(muted_for(state))].spacing(8);
    for section in sections {
        let label = section.to_string();
        let selected = state.section == section;
        let count = section_count(state, section);
        menu = menu.push(
            button(
                row![
                    icon_svg(section_icon(section), 16),
                    text(label).size(14),
                    iced::widget::space::horizontal(),
                    text(count.to_string()).size(12).color(muted_for(state)),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .on_press(Message::SelectSection(section))
            .style(move |theme, status| {
                if selected {
                    let mut style = iced::widget::button::secondary(theme, status);
                    style.background =
                        Some(iced::Background::Color(mix(surface, foreground, 0.16)));
                    style.text_color = foreground;
                    style
                } else {
                    let mut style = iced::widget::button::text(theme, status);
                    style.text_color = foreground;
                    style
                }
            })
            .width(Length::Fill),
        );
    }
    menu = menu.push(iced::widget::space().height(Length::Fill));
    for folder in &state.folders {
        menu = menu.push(
            text(
                folder
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
            .size(12)
            .color(muted_for(state)),
        );
    }
    container(menu.width(Length::Fixed(190.0)).padding([12, 8]))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(surface)),
            border: iced::Border::default()
                .rounded(12.0)
                .color(border)
                .width(1.0),
            ..Default::default()
        })
        .into()
}

fn section_count(state: &Reader, section: LibrarySection) -> usize {
    let values = state.library.iter().flat_map(|item| {
        let metadata = &item.book.metadata;
        match section {
            LibrarySection::Authors => metadata.authors.clone(),
            LibrarySection::Series => metadata.series.clone().into_iter().collect(),
            LibrarySection::Genres => metadata.subjects.clone(),
            LibrarySection::Language => metadata.language.clone().into_iter().collect(),
            LibrarySection::Publisher => metadata.publisher.clone().into_iter().collect(),
            _ => Vec::new(),
        }
    });
    match section {
        LibrarySection::AllBooks => state.library.len(),
        LibrarySection::Favorites => state
            .library
            .iter()
            .filter(|item| book_details::favorite(state, &item.path))
            .count(),
        LibrarySection::Rating => state
            .library
            .iter()
            .filter(|item| book_details::rating(state, &item.path) > 0)
            .count(),
        LibrarySection::Reading => state
            .library
            .iter()
            .filter(|item| reading_history::in_progress(state, &item.path))
            .count(),
        LibrarySection::Authors
        | LibrarySection::Series
        | LibrarySection::Genres
        | LibrarySection::Language
        | LibrarySection::Publisher => values.collect::<std::collections::BTreeSet<_>>().len(),
        _ => 0,
    }
}

fn library_content(state: &Reader) -> Element<'_, Message> {
    let title = text(state.section.to_string())
        .font(Font::with_name("Literata"))
        .size(30);
    let books = filtered_books(state);
    let controls = library_screen::controls(state, books.len());
    let groups = grouped_books(state, books);
    let mut content = column![title, controls].spacing(18);
    if state.section == LibrarySection::Home
        && let Some(item) = state
            .last_opened_path
            .as_ref()
            .and_then(|path| state.library.iter().find(|item| &item.path == path))
    {
        content = content.push(continue_reading_card(state, item));
    }
    if groups.is_empty() {
        content = content.push(empty_library(state));
    }
    for (heading, entries) in groups {
        if !heading.is_empty() {
            content = content.push(text(heading).font(Font::with_name("Literata")).size(20));
        }
        content = content.push(if state.list_view {
            book_list(state, &entries)
        } else {
            library_screen::grid(state, &entries)
        });
    }
    if let Some(status) = &state.status {
        content = content.push(text(status).color(ensure_contrast(
            Color::from_rgb8(179, 38, 30),
            background_for(state),
            4.5,
        )));
    }
    adaptive_scroll::new(
        container(content)
            .padding(if state.window_width < 600.0 { 12 } else { 24 })
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

fn empty_library(state: &Reader) -> Element<'_, Message> {
    let message = if state.folders.is_empty() {
        "Adicione uma pasta para montar sua biblioteca."
    } else if state.section == LibrarySection::Favorites {
        "Marque o coração de um livro para encontrá-lo aqui."
    } else if state.section == LibrarySection::Reading {
        "Abra um livro para ele aparecer nesta lista."
    } else if matches!(state.section, LibrarySection::Collections) {
        "Esta área ainda não está incluída na versão local de prova."
    } else if state.search.is_empty() {
        "Ainda não encontramos arquivos EPUB nesta pasta."
    } else {
        "Nenhum livro corresponde aos filtros atuais."
    };
    let mut empty = column![
        text(if state.folders.is_empty() {
            "Sua biblioteca começa aqui."
        } else {
            "Nenhum livro nesta seção."
        })
        .font(Font::with_name("Literata"))
        .size(30),
        text(message).size(16).color(muted_for(state)),
    ]
    .align_x(Alignment::Center)
    .spacing(16);
    if state.folders.is_empty() {
        empty = empty.push(button("Adicionar pasta").on_press(Message::AddFolder));
    } else if !state.search.is_empty() {
        empty = empty.push(button("Limpar busca").on_press(Message::SearchChanged(String::new())));
    }
    container(empty).center(Length::Fill).into()
}

fn filtered_books(state: &Reader) -> Vec<&LibraryItem> {
    let query = state.search.to_lowercase();
    let mut books = state
        .library
        .iter()
        .filter(|item| {
            if state.section == LibrarySection::Favorites
                && !book_details::favorite(state, &item.path)
            {
                return false;
            }
            if state.section == LibrarySection::Reading
                && !reading_history::in_progress(state, &item.path)
            {
                return false;
            }
            let metadata = &item.book.metadata;
            [
                metadata.title.as_str(),
                metadata.description.as_deref().unwrap_or(""),
                metadata.publisher.as_deref().unwrap_or(""),
                metadata.series.as_deref().unwrap_or(""),
            ]
            .into_iter()
            .chain(metadata.authors.iter().map(String::as_str))
            .chain(metadata.subjects.iter().map(String::as_str))
            .chain(metadata.language.iter().map(String::as_str))
            .any(|value| value.to_lowercase().contains(&query))
        })
        .collect::<Vec<_>>();
    if state.section == LibrarySection::Reading {
        books.sort_by_key(|item| {
            std::cmp::Reverse(
                state
                    .reading_history
                    .get(&item.path)
                    .map_or(0, |p| p.last_read),
            )
        });
        return books;
    }
    books.sort_by_cached_key(|item| match state.sort_by {
        SortBy::Rating => format!(
            "{}|{}",
            5 - book_details::rating(state, &item.path),
            item.book.metadata.title.to_lowercase()
        ),
        SortBy::Title => item.book.metadata.title.to_lowercase(),
        SortBy::Author => item
            .book
            .metadata
            .authors
            .first()
            .cloned()
            .unwrap_or_default()
            .to_lowercase(),
        SortBy::Series => item
            .book
            .metadata
            .series
            .clone()
            .unwrap_or_default()
            .to_lowercase(),
        SortBy::Publisher => item
            .book
            .metadata
            .publisher
            .clone()
            .unwrap_or_default()
            .to_lowercase(),
    });
    books
}

fn continue_reading_card<'a>(state: &'a Reader, item: &'a LibraryItem) -> Element<'a, Message> {
    let chapter = state
        .reading_history
        .get(&item.path)
        .map_or(0, |p| p.chapter);
    let chapter_count = item.book.chapters.len();
    let surface = surface_for(state);
    let foreground = foreground_for(state);
    button(
        row![
            cover_art(state, item, 64.0, 96.0),
            column![
                text("CONTINUAR LENDO").size(11).color(muted_for(state)),
                text(&item.book.metadata.title)
                    .font(Font::with_name("Literata"))
                    .size(21),
                text(format!("Capítulo {} de {}", chapter + 1, chapter_count))
                    .size(13)
                    .color(muted_for(state)),
            ]
            .spacing(6),
            iced::widget::space::horizontal(),
            icon_svg("chevron-right", 20),
        ]
        .spacing(16)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .on_press(Message::OpenLibraryBook(item.path.clone()))
    .width(Length::Fill)
    .padding(16)
    .style(move |theme, status| {
        let mut style = iced::widget::button::secondary(theme, status);
        style.background = Some(iced::Background::Color(surface));
        style.text_color = foreground;
        style
    })
    .into()
}

fn grouped_books<'a>(
    state: &Reader,
    books: Vec<&'a LibraryItem>,
) -> Vec<(String, Vec<&'a LibraryItem>)> {
    if state.section == LibrarySection::Rating {
        let mut groups = std::collections::BTreeMap::<u8, Vec<&LibraryItem>>::new();
        for book in books {
            groups
                .entry(book_details::rating(state, &book.path))
                .or_default()
                .push(book);
        }
        return groups
            .into_iter()
            .rev()
            .map(|(stars, books)| {
                (
                    if stars == 0 {
                        "Sem nota".to_owned()
                    } else {
                        format!(
                            "{stars} {}",
                            if stars == 1 { "estrela" } else { "estrelas" }
                        )
                    },
                    books,
                )
            })
            .collect();
    }
    let mut groups = std::collections::BTreeMap::<String, Vec<&LibraryItem>>::new();
    for item in books {
        let metadata = &item.book.metadata;
        let names = match state.section {
            LibrarySection::Authors => metadata.authors.clone(),
            LibrarySection::Series => metadata.series.clone().into_iter().collect(),
            LibrarySection::Genres => metadata.subjects.clone(),
            LibrarySection::Language => metadata.language.clone().into_iter().collect(),
            LibrarySection::Publisher => metadata.publisher.clone().into_iter().collect(),
            LibrarySection::Collections => Vec::new(),
            _ => vec![String::new()],
        };
        for name in names {
            groups
                .entry(
                    if name.is_empty()
                        && !matches!(
                            state.section,
                            LibrarySection::Home
                                | LibrarySection::AllBooks
                                | LibrarySection::Reading
                                | LibrarySection::Favorites
                        )
                    {
                        "Sem classificação".to_owned()
                    } else {
                        name
                    },
                )
                .or_default()
                .push(item);
        }
    }
    groups.into_iter().collect()
}

fn book_list<'a>(state: &'a Reader, items: &[&'a LibraryItem]) -> Element<'a, Message> {
    let mut rows = column![].spacing(8);
    for item in items {
        let surface = surface_for(state);
        let foreground = foreground_for(state);
        let author = item
            .book
            .metadata
            .authors
            .first()
            .map(String::as_str)
            .unwrap_or("Autor desconhecido");
        let open = button(
            row![
                cover_art(state, item, 48.0, 72.0),
                column![
                    text(&item.book.metadata.title),
                    text(author).size(12).color(muted_for(state)),
                    text(item.book.metadata.series.as_deref().unwrap_or(""))
                        .size(12)
                        .color(muted_for(state)),
                    text(if item.book.metadata.subjects.is_empty() {
                        "Sem gênero".to_owned()
                    } else {
                        item.book.metadata.subjects.join(" · ")
                    })
                    .size(12)
                    .color(muted_for(state)),
                    text(
                        item.book
                            .metadata
                            .language
                            .as_deref()
                            .unwrap_or("Idioma não informado")
                    )
                    .size(12)
                    .color(muted_for(state)),
                    text(item.book.metadata.date.as_deref().unwrap_or(""))
                        .size(12)
                        .color(muted_for(state)),
                ]
                .spacing(4),
                iced::widget::space::horizontal(),
                text(item.book.metadata.publisher.as_deref().unwrap_or(""))
            ]
            .spacing(14)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .on_press(Message::OpenLibraryBook(item.path.clone()))
        .width(Length::Fill)
        .padding(8)
        .style(move |theme, status| {
            let mut style = iced::widget::button::secondary(theme, status);
            style.background = Some(iced::Background::Color(surface));
            style.text_color = foreground;
            style
        });
        rows = rows.push(
            container(column![
                open,
                container(book_details::footer(state, item)).padding(iced::Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 8.0,
                    left: 70.0
                })
            ])
            .style(move |_| container::Style {
                background: Some(surface.into()),
                border: iced::Border::default().rounded(8),
                ..Default::default()
            }),
        );
    }
    rows.width(Length::Fill).into()
}

fn cover_art<'a>(
    state: &Reader,
    item: &'a LibraryItem,
    width: f32,
    height: f32,
) -> Element<'a, Message> {
    if let Some(cover) = &item.cover {
        return iced::widget::image::Image::new(cover.clone())
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .into();
    }
    let author = item
        .book
        .metadata
        .authors
        .first()
        .map(String::as_str)
        .unwrap_or("Autor desconhecido");
    let surface = surface_for(state);
    container(
        column![
            text(&item.book.metadata.title)
                .font(Font::with_name("Literata"))
                .size(16)
                .color(foreground_for(state)),
            text(author).size(11).color(muted_for(state)),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .width(Length::Fixed(width))
    .height(Length::Fixed(height))
    .center(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(surface)),
        ..Default::default()
    })
    .into()
}

fn reader_page(state: &Reader) -> Element<'_, Message> {
    let Some(book) = &state.book else {
        return library_screen::page(state);
    };
    let compact = state.window_width < 1000.0;
    let sidebar: Element<'_, Message> = if state.sidebar_open && !compact {
        chapter_sidebar(state, book)
    } else {
        iced::widget::space().width(0).into()
    };
    let notes: Element<'_, Message> = if state.show_annotations && !compact {
        annotation_panel(state, book)
    } else {
        iced::widget::space().width(0).into()
    };
    // Fixed slots preserve the retained reading viewport when either sidebar opens.
    let base = row![sidebar, book_view(state, book), notes]
        .spacing(8)
        .height(Length::Fill);
    let mut body = stack![base].width(Length::Fill).height(Length::Fill);
    if compact && state.sidebar_open {
        body = body.push(
            container(chapter_sidebar(state, book))
                .height(Length::Fill)
                .padding(8)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(surface_for(state))),
                    ..Default::default()
                }),
        );
    }
    if compact && state.show_annotations {
        body = body.push(
            container(annotation_panel(state, book))
                .height(Length::Fill)
                .align_right(Length::Fill),
        );
    }
    container(
        column![toolbar::view(state), body, reader_progress(state)]
            .spacing(8)
            .padding(12)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn reader_progress(state: &Reader) -> Element<'_, Message> {
    row![
        text(format!("{:.0}%", state.progress_percent))
            .size(12)
            .color(muted_for(state)),
        slider(
            0.0..=100.0,
            state.progress_percent,
            Message::ProgressChanged
        )
        .step(0.1_f32)
        .width(Length::Fill),
        text(if state.book_loading {
            "Carregando o restante do livro…"
        } else {
            state.status.as_deref().unwrap_or("Rolagem contínua")
        })
        .size(12)
        .color(muted_for(state)),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn chapter_sidebar<'a>(state: &'a Reader, book: &'a Book) -> Element<'a, Message> {
    let mut menu = column![text("SUMÁRIO").size(12).color(muted_for(state))].spacing(8);
    for (index, chapter) in book.chapters.iter().enumerate() {
        let selected = index == state.chapter_index;
        menu = menu.push(
            button(text(chapter.title.clone()))
                .on_press(Message::SelectChapter(index))
                .style(move |theme, status| {
                    if selected {
                        iced::widget::button::secondary(theme, status)
                    } else {
                        iced::widget::button::text(theme, status)
                    }
                })
                .width(Length::Fill),
        );
    }
    container(adaptive_scroll::new(menu).height(Length::Fill))
        .width(Length::Fixed((state.window_width - 40.0).clamp(1.0, 240.0)))
        .into()
}

fn toolbar_button<'a>(
    state: &'a Reader,
    name: &str,
    label: &'a str,
    message: Message,
) -> Element<'a, Message> {
    iced::widget::tooltip(
        action::focusable(
            button(icon_svg(name, 16))
                .on_press(message.clone())
                .width(32)
                .height(32)
                .padding(8)
                .style(move |_, status| picker_style(state, status))
                .into(),
            format!("toolbar-{label}"),
            message,
            None,
        ),
        text(label).size(12),
        iced::widget::tooltip::Position::Bottom,
    )
    .into()
}

fn annotation_panel<'a>(state: &'a Reader, book: &'a Book) -> Element<'a, Message> {
    annotations::view(state, book)
}

fn picker_style(
    state: &Reader,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let background = if matches!(
        status,
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
    ) {
        mix(surface_for(state), foreground_for(state), 0.08)
    } else {
        surface_for(state)
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(background)),
        text_color: foreground_for(state),
        border: iced::Border::default()
            .color(border_for(state))
            .width(1.0)
            .rounded(4.0),
        ..Default::default()
    }
}

fn color_swatch(color: &str, size: f32) -> Element<'static, Message> {
    let color = hex_color(color);
    container(text(""))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border::default()
                .rounded(2.0)
                .color(Color::from_rgba(0.1, 0.1, 0.1, 0.45))
                .width(1.0),
            ..Default::default()
        })
        .into()
}

fn theme_menu(state: &Reader) -> Element<'_, Message> {
    let options = theme_options(&state.custom_themes);
    let mut items = column![].spacing(2).width(Length::Fill);
    let total = options.len();
    for (index, option) in options.into_iter().enumerate() {
        let chosen = option.id.strip_prefix("custom:").unwrap_or(&option.id)
            == current_theme_id(&state.settings);
        items = items.push(action::menu_item(
            button(text(option.label.clone()).size(14).width(Length::Fill))
                .on_press(Message::ThemeSelected(option.clone()))
                .height(32)
                .width(Length::Fill)
                .style(move |theme, status| {
                    if chosen {
                        iced::widget::button::secondary(theme, status)
                    } else {
                        iced::widget::button::text(theme, status)
                    }
                })
                .into(),
            "theme-option",
            index,
            total,
            Message::ThemeSelected(option),
        ));
    }
    overflow_panel(
        state,
        adaptive_scroll::new(items)
            .id(iced::widget::Id::new("theme-option-scroll"))
            .height(Length::Fixed(280.0))
            .width(Length::Fill)
            .into(),
    )
}

fn color_menu<'a>(
    state: &'a Reader,
    annotation: Option<&'a BookAnnotation>,
) -> Element<'a, Message> {
    let colors = theme_definition_for(state)
        .map(|theme| theme.marker_colors.as_slice())
        .unwrap_or(&[]);
    let mut items = column![].spacing(2).width(Length::Fill);
    for (index, color) in colors.iter().enumerate() {
        let message = annotation.map_or_else(
            || Message::MarkerColorSelected(color.clone()),
            |mark| Message::AnnotationColorSelected(mark.id.clone(), color.clone()),
        );
        items = items.push(action::menu_item(
            button(container(color_swatch(color, 20.0)).center_x(Length::Fill))
                .width(Length::Fill)
                .height(32)
                .padding(6)
                .on_press(message.clone())
                .style(iced::widget::button::text)
                .into(),
            "marker-option",
            index,
            colors.len(),
            message,
        ));
    }
    overflow_panel(
        state,
        adaptive_scroll::new(items)
            .id(iced::widget::Id::new("marker-option-scroll"))
            .height(Length::Fixed((colors.len() as f32 * 34.0).min(280.0)))
            .into(),
    )
}

fn alignment_controls(selected: AlignmentChoice) -> Element<'static, Message> {
    let choices = [
        (AlignmentChoice::Left, "align-left", "À esquerda"),
        (AlignmentChoice::Center, "align-center", "Centralizado"),
        (AlignmentChoice::Right, "align-right", "À direita"),
        (AlignmentChoice::Justify, "align-justify", "Justificado"),
    ];
    let mut controls = column![].spacing(4).width(Length::Fill);
    for (choice, icon_name, label) in choices {
        controls = controls.push(action::focusable(
            button(
                row![icon_svg(icon_name, 16), text(label).size(14)]
                    .spacing(8)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::AlignmentSelected(choice))
            .style(move |theme, status| {
                if selected == choice {
                    iced::widget::button::secondary(theme, status)
                } else {
                    iced::widget::button::text(theme, status)
                }
            })
            .padding(8)
            .width(Length::Fill)
            .height(32)
            .into(),
            format!("reading-{icon_name}"),
            Message::AlignmentSelected(choice),
            None,
        ));
    }
    controls.into()
}

fn book_view<'a>(state: &'a Reader, book: &'a Book) -> Element<'a, Message> {
    surface::view(state, book)
}

fn adjust_percentage(current: u16, delta: i16, minimum: u16, maximum: u16) -> u16 {
    (i32::from(current) + i32::from(delta)).clamp(i32::from(minimum), i32::from(maximum)) as u16
}

fn next_scroll_speed(current: u16, direction: i16) -> u16 {
    if direction > 0 {
        let step = if current < 100 { 10 } else { 100 };
        (current / step).saturating_add(1).saturating_mul(step)
    } else if current > 100 {
        ((current - 1) / 100 * 100).max(100)
    } else {
        current.saturating_sub(10).max(10)
    }
}

fn reading_font_px(state: &Reader, base: f32) -> f32 {
    base * f32::from(state.settings.font_scale) / 100.0
}

fn friendly_error(error: OpenBookError) -> String {
    match error {
        OpenBookError::UnsupportedFormat => "Escolha um arquivo com extensão .epub.".to_owned(),
        OpenBookError::Book(BookError::Protected) => {
            "Este livro tem DRM e não pode ser aberto.".to_owned()
        }
        OpenBookError::Book(BookError::UnsafePath | BookError::ArchiveLimit) => {
            "O EPUB excede os limites de segurança e não foi aberto.".to_owned()
        }
        OpenBookError::Book(
            BookError::InvalidArchive | BookError::InvalidPackage | BookError::MissingPackage,
        ) => "O arquivo não contém um EPUB válido.".to_owned(),
        OpenBookError::Book(BookError::SourceUnavailable) => {
            "Não foi possível ler o arquivo EPUB.".to_owned()
        }
    }
}

#[cfg(test)]
mod theme_tests {
    use super::{
        Chapter, Color, ThemeDefinition, ThemeOption, adaptive_scroll_thumb, adjust_percentage,
        builtin_themes, chapter_paragraph_ranges, contrast_ratio, ensure_contrast, theme_options,
        valid_hex,
    };

    #[test]
    fn lists_dracula_and_custom_themes_in_the_theme_picker() {
        let custom = ThemeDefinition {
            id: "ocean".to_owned(),
            name: "Oceano".to_owned(),
            background: "#102030".to_owned(),
            foreground: "#F0F0F0".to_owned(),
            muted: "#A0A0A0".to_owned(),
            accent: "#40A0FF".to_owned(),
            marker_colors: vec!["#FFAA00".to_owned()],
        };
        let mut themes = builtin_themes();
        themes.push(custom);
        let options = theme_options(&themes);

        assert!(options.contains(&ThemeOption {
            id: "dracula".to_owned(),
            label: "Dracula".to_owned()
        }));
        assert!(options.contains(&ThemeOption {
            id: "custom:ocean".to_owned(),
            label: "Oceano".to_owned()
        }));
        assert!(themes.iter().any(|theme| theme.id == "light"));
        assert!(themes.iter().any(|theme| theme.id == "dracula"));
    }

    #[test]
    fn rejects_theme_colors_that_are_not_six_digit_hex_values() {
        assert!(valid_hex("#BD93F9"));
        assert!(valid_hex("282A36"));
        assert!(!valid_hex("#BD93F"));
        assert!(!valid_hex("red"));
    }

    #[test]
    fn includes_all_four_catppuccin_flavors_with_five_marker_colors() {
        let themes = builtin_themes();
        for id in [
            "catppuccin-latte",
            "catppuccin-frappe",
            "catppuccin-macchiato",
            "catppuccin-mocha",
        ] {
            let theme = themes.iter().find(|theme| theme.id == id).unwrap();
            assert_eq!(theme.marker_colors.len(), 5, "{id}");
            assert!(
                theme.marker_colors.iter().all(|color| valid_hex(color)),
                "{id}"
            );
        }
    }

    #[test]
    fn percentage_controls_change_in_ten_point_steps_and_respect_bounds() {
        assert_eq!(adjust_percentage(100, -10, 50, 300), 90);
        assert_eq!(adjust_percentage(100, 10, 10, 1000), 110);
        assert_eq!(adjust_percentage(50, -10, 50, 300), 50);
        assert_eq!(adjust_percentage(300, 10, 50, 300), 300);
    }

    #[test]
    fn paragraph_index_preserves_utf8_byte_ranges_for_selection_offsets() {
        let chapter = Chapter {
            title: "Teste".to_owned(),
            text: "área\n\n\n\núltimo trecho".to_owned(),
        };
        let ranges = chapter_paragraph_ranges(&chapter);
        let paragraphs = ranges
            .iter()
            .map(|(start, end)| &chapter.text[*start..*end])
            .collect::<Vec<_>>();
        assert_eq!(paragraphs, ["área", "último trecho"]);
    }

    #[test]
    fn enforces_readable_contrast_for_muted_theme_text_on_light_and_dark_surfaces() {
        let dark_surface = super::hex_color("#343746");
        let light_surface = super::hex_color("#F5F4F0");
        let dracula_muted = super::hex_color("#6272A4");
        let subdued_gray = super::hex_color("#BFBFBF");

        assert!(
            contrast_ratio(
                ensure_contrast(dracula_muted, dark_surface, 4.5),
                dark_surface
            ) >= 4.5
        );
        assert!(
            contrast_ratio(
                ensure_contrast(subdued_gray, light_surface, 4.5),
                light_surface
            ) >= 4.5
        );
        assert!(contrast_ratio(Color::WHITE, dark_surface) >= 4.5);
    }

    #[test]
    fn adaptive_scroll_thumb_gets_smaller_as_reading_progresses() {
        let (start_height, start_top) = adaptive_scroll_thumb(800.0, 0.0);
        let (end_height, end_top) = adaptive_scroll_thumb(800.0, 100.0);

        assert!(start_height > end_height);
        assert_eq!(start_top, 0.0);
        assert!((end_top + end_height - 800.0).abs() < f32::EPSILON);
    }
}
