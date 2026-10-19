//! Personal reviews live in local preferences; the original EPUB remains untouched.
#[cfg(test)]
mod tests;
use super::*;
use iced::widget::{column, row, text_editor};
const PREFIX: &str = "book-review:";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Review {
    pub rating: u8,
    pub favorite: bool,
    pub comment: String,
}

pub(super) fn load(data_dir: &Path) -> HashMap<PathBuf, Review> {
    library::preferences_with_prefix(data_dir, PREFIX)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(key, json)| {
            let path = PathBuf::from(key.strip_prefix(PREFIX)?);
            let review: Review = serde_json::from_str(&json).ok()?;
            (review.rating <= 5).then_some((path, review))
        })
        .collect()
}

pub(super) fn rating(state: &Reader, path: &Path) -> u8 {
    state
        .book_reviews
        .get(path)
        .map_or(0, |review| review.rating)
}

fn persist(state: &Reader, path: &Path) -> Task<Message> {
    let Some(review) = state.book_reviews.get(path) else {
        return Task::none();
    };
    queue_preference(
        state,
        format!("{PREFIX}{}", path.display()),
        PreferenceValue::BookReview(review.clone()),
    )
}

pub(super) fn rate(state: &mut Reader, path: PathBuf, stars: u8) -> Task<Message> {
    if stars > 5 || !state.library.iter().any(|item| item.path == path) {
        return Task::none();
    }
    state.book_reviews.entry(path.clone()).or_default().rating = stars;
    persist(state, &path)
}

pub(super) fn favorite(state: &Reader, path: &Path) -> bool {
    state
        .book_reviews
        .get(path)
        .is_some_and(|review| review.favorite)
}

pub(super) fn toggle_favorite(state: &mut Reader, path: PathBuf) -> Task<Message> {
    if !state.library.iter().any(|item| item.path == path) {
        return Task::none();
    }
    let review = state.book_reviews.entry(path.clone()).or_default();
    review.favorite = !review.favorite;
    persist(state, &path)
}

pub(super) fn open(state: &mut Reader, path: PathBuf) -> Task<Message> {
    if !state.library.iter().any(|item| item.path == path) {
        return Task::none();
    }
    let comment = state
        .book_reviews
        .get(&path)
        .map_or("", |review| review.comment.as_str());
    state.book_review_editor = text_editor::Content::with_text(comment);
    state.book_details_path = Some(path);
    state.about_open = false;
    state.settings_open = false;
    iced_runtime::widget::operation::focus(iced::widget::Id::new("book-details-close"))
}

pub(super) fn edit(state: &mut Reader, action: text_editor::Action) -> Task<Message> {
    let Some(path) = state.book_details_path.clone() else {
        return Task::none();
    };
    if let text_editor::Action::Scroll { lines } = action {
        return iced_runtime::widget::operation::scroll_by(
            iced::widget::Id::new("book-details-scroll"),
            scrollable::AbsoluteOffset {
                x: 0.0,
                y: lines as f32 * 4.0,
            },
        );
    }
    let changed = action.is_edit();
    state.book_review_editor.perform(action);
    if !changed {
        return Task::none();
    }
    state.book_reviews.entry(path.clone()).or_default().comment = state.book_review_editor.text();
    persist(state, &path)
}

pub(super) fn clear_comment(state: &mut Reader) -> Task<Message> {
    let Some(path) = state.book_details_path.clone() else {
        return Task::none();
    };
    state.book_review_editor = text_editor::Content::new();
    state
        .book_reviews
        .entry(path.clone())
        .or_default()
        .comment
        .clear();
    persist(state, &path)
}

pub(super) fn footer<'a>(state: &'a Reader, item: &'a LibraryItem) -> Element<'a, Message> {
    iced::widget::responsive(move |size| {
        let actions = row![
            stars(state, &item.path, (size.width - 60.0).clamp(1.0, 120.0)),
            iced::widget::space::horizontal(),
            favorite_button(state, &item.path),
            iced::widget::tooltip(
                action::focusable(
                    button(icon_svg("pencil", 16))
                        .width(24)
                        .height(28)
                        .padding([6, 4])
                        .style(move |_, status| picker_style(state, status))
                        .on_press(Message::OpenBookDetails(item.path.clone()))
                        .into(),
                    format!("book-details-{}", item.path.display()),
                    Message::OpenBookDetails(item.path.clone()),
                    None,
                ),
                text("Detalhes e comentário").size(12),
                iced::widget::tooltip::Position::Top,
            )
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .width(Length::Fill);
        column![reading_history::view(state, &item.path), actions]
            .spacing(4)
            .width(Length::Fill)
            .into()
    })
    .height(50)
    .into()
}

fn favorite_button<'a>(state: &'a Reader, path: &'a Path) -> Element<'a, Message> {
    let selected = favorite(state, path);
    let message = Message::ToggleBookFavorite(path.to_owned());
    let trigger = action::focusable(
        button(icon_svg(
            if selected { "heart-filled" } else { "heart" },
            16,
        ))
        .width(24)
        .height(28)
        .padding([6, 4])
        .style(move |_, status| picker_style(state, status))
        .on_press(message.clone())
        .into(),
        format!("book-favorite-{}", path.display()),
        message,
        None,
    );
    iced::widget::tooltip(
        trigger,
        text(if selected {
            "Remover dos favoritos"
        } else {
            "Favoritar"
        })
        .size(12),
        iced::widget::tooltip::Position::Top,
    )
    .into()
}

fn stars<'a>(state: &'a Reader, path: &'a Path, width: f32) -> Element<'a, Message> {
    let selected = rating(state, path);
    let mut stars = row![].width(width).height(28);
    for value in 1..=5 {
        let message = Message::RateBook(path.to_owned(), if value == selected { 0 } else { value });
        let color = if value <= selected {
            let accent =
                theme_definition_for(state).map_or(ACCENT, |theme| hex_color(&theme.accent));
            ensure_contrast(accent, surface_for(state), 3.0)
        } else {
            muted_for(state)
        };
        let icon = star(
            value <= selected,
            color,
            ((width / 5.0 - 4.0).clamp(8.0, 16.0)) as u16,
        );
        let trigger = action::focusable(
            button(container(icon).center_x(Length::Fill))
                .width(Length::Fill)
                .height(28)
                .padding(2)
                .on_press(message.clone())
                .style(iced::widget::button::text)
                .into(),
            format!("book-rating-{}-{value}", path.display()),
            message,
            None,
        );
        stars = stars.push(iced::widget::tooltip(
            trigger,
            text(format!(
                "{value} {}",
                if value == 1 { "estrela" } else { "estrelas" }
            ))
            .size(12),
            iced::widget::tooltip::Position::Top,
        ));
    }
    stars.into()
}

fn star(filled: bool, color: Color, size: u16) -> Element<'static, Message> {
    use iced::widget::svg;
    use std::sync::OnceLock;
    static OUTLINE: OnceLock<svg::Handle> = OnceLock::new();
    static FILLED: OnceLock<svg::Handle> = OnceLock::new();
    let handle = if filled {
        FILLED.get_or_init(|| {
            svg::Handle::from_memory(
                include_str!("../../../assets/icons/lucide/star.svg")
                    .replace("fill=\"none\"", "fill=\"#000\"")
                    .into_bytes(),
            )
        })
    } else {
        OUTLINE.get_or_init(|| {
            svg::Handle::from_memory(include_bytes!("../../../assets/icons/lucide/star.svg"))
        })
    };
    svg(handle.clone())
        .width(f32::from(size))
        .height(f32::from(size))
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}

pub(super) fn view(state: &Reader) -> Element<'_, Message> {
    let Some(item) = state
        .book_details_path
        .as_ref()
        .and_then(|path| state.library.iter().find(|item| &item.path == path))
    else {
        return iced::widget::space().into();
    };
    let backdrop = button("")
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .on_press(Message::CloseBookDetails)
        .style(|_, _| iced::widget::button::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.65).into()),
            ..Default::default()
        });
    let close = action::focusable(
        button(icon_svg("x", 16))
            .width(32)
            .height(32)
            .padding(8)
            .style(move |_, status| picker_style(state, status))
            .on_press(Message::CloseBookDetails)
            .into(),
        "book-details-close",
        Message::CloseBookDetails,
        None,
    );
    let card = container(column![
        container(
            row![text("Sobre o livro").size(18).width(Length::Fill), close]
                .align_y(Alignment::Center)
        )
        .padding(8),
        adaptive_scroll::new(container(content(state, item)).padding(24))
            .id(iced::widget::Id::new("book-details-scroll"))
            .height(Length::Shrink),
    ])
    .id(iced::widget::Id::new("book-details-modal"))
    .width(Length::Fill)
    .max_width(760)
    .style(move |_| container::Style {
        background: Some(surface_for(state).into()),
        text_color: Some(foreground_for(state)),
        border: iced::Border::default()
            .rounded(14)
            .width(1)
            .color(border_for(state)),
        ..Default::default()
    });
    stack![backdrop, container(card).padding(12).center(Length::Fill)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn content<'a>(state: &'a Reader, item: &'a LibraryItem) -> Element<'a, Message> {
    let metadata = &item.book.metadata;
    let heading = column![
        text(&metadata.title)
            .font(Font::with_name("Literata"))
            .size(26)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        field("Autor", &metadata.authors.join(", ")),
        field(
            "Idioma",
            metadata.language.as_deref().unwrap_or("Não informado")
        ),
        field(
            "Editora",
            metadata.publisher.as_deref().unwrap_or("Não informada")
        ),
    ]
    .spacing(12)
    .width(Length::Fill);
    let header: Element<'_, Message> = if state.window_width < 600.0 {
        column![
            container(cover_art(state, item, 96.0, 144.0)).center_x(Length::Fill),
            heading
        ]
        .spacing(16)
        .into()
    } else {
        row![cover_art(state, item, 128.0, 192.0), heading]
            .spacing(24)
            .into()
    };
    let review = state.book_reviews.get(&item.path);
    let publication = field(
        "Publicação",
        metadata.date.as_deref().unwrap_or("Não informada"),
    );
    let series = field(
        "Série",
        metadata.series.as_deref().unwrap_or("Não informada"),
    );
    let chapters = field("Capítulos", &item.book.chapters.len().to_string());
    let details: Element<'_, Message> = if state.window_width < 600.0 {
        column![publication, series, chapters].spacing(16).into()
    } else {
        row![
            container(publication).width(Length::Fill),
            container(series).width(Length::Fill),
            container(chapters).width(Length::Fill),
        ]
        .spacing(20)
        .into()
    };
    let save_hint = text("Salvamento automático")
        .size(12)
        .color(muted_for(state));
    let delete = button(text("Excluir comentário").size(13))
        .on_press_maybe(
            review
                .filter(|r| !r.comment.is_empty())
                .map(|_| Message::DeleteBookComment),
        )
        .style(move |_, status| picker_style(state, status));
    let actions: Element<'_, Message> = if state.window_width < 600.0 {
        column![save_hint, delete].spacing(8).into()
    } else {
        row![save_hint, iced::widget::space::horizontal(), delete]
            .align_y(Alignment::Center)
            .spacing(8)
            .into()
    };
    let mut content = column![
        header,
        details,
        field("Gêneros", &metadata.subjects.join(" · ")),
        field(
            "Sinopse",
            metadata.description.as_deref().unwrap_or("Não informada")
        ),
        field("Arquivo", &item.path.display().to_string()),
        text("Sua nota").size(16),
        stars(state, &item.path, 140.0),
        text("Comentário sobre o livro").size(16),
        text_editor(&state.book_review_editor)
            .id(iced::widget::Id::new("book-review-comment"))
            .placeholder("O que você achou desta leitura?")
            .size(14)
            .padding(10)
            .min_height(80)
            .height(Length::Shrink)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            .on_action(Message::EditBookReview),
        actions,
    ]
    .spacing(16)
    .width(Length::Fill);
    if let Some(status) = &state.status {
        content = content.push(text(status).size(13).color(foreground_for(state)));
    }
    content.into()
}

fn field(label: &str, value: &str) -> Element<'static, Message> {
    column![
        text(label.to_owned()).size(12),
        text(if value.is_empty() {
            "Não informado".to_owned()
        } else {
            value.to_owned()
        })
        .size(14)
        .width(Length::Fill)
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
    ]
    .spacing(4)
    .into()
}
