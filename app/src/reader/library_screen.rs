//! Library presentation only. The reading viewport and its toolbar live elsewhere.
mod about;
#[cfg(test)]
mod review;
#[cfg(test)]
mod tests;
use super::*;
use crate::brand;
use iced::widget::{column, row};

pub(super) fn page(state: &Reader) -> Element<'_, Message> {
    let visible = state.sidebar_open && state.window_width >= 1000.0;
    let sidebar: Element<'_, Message> = if visible {
        container(library_sidebar(state))
            .padding(iced::Padding {
                top: 24.0,
                right: 0.0,
                bottom: 24.0,
                left: 24.0,
            })
            .into()
    } else {
        iced::widget::space().width(0).into()
    };
    column![
        container(header(state))
            .padding(if state.window_width < 600.0 { 12 } else { 24 })
            .width(Length::Fill),
        row![sidebar, library_content(state)].height(Length::Fill)
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn wordmark(state: &Reader) -> Element<'_, Message> {
    row![
        header_mark(state, 36),
        text(brand::NAME)
            .font(Font::with_name("Literata"))
            .color(foreground_for(state))
            .size(if state.window_width < 400.0 { 18 } else { 26 })
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

pub(super) fn mark(size: u32) -> Element<'static, Message> {
    iced::widget::svg(iced::widget::svg::Handle::from_memory(brand::MARK))
        .width(size)
        .height(size)
        .into()
}

fn header_mark(state: &Reader, size: u32) -> Element<'static, Message> {
    let color = foreground_for(state);
    iced::widget::svg(iced::widget::svg::Handle::from_memory(include_bytes!(
        "../../../assets/brand/mark-symbolic.svg"
    )))
    .width(size)
    .height(size)
    .style(move |_, _| iced::widget::svg::Style { color: Some(color) })
    .into()
}

fn actions(state: &Reader) -> Element<'_, Message> {
    row![
        popover::anchored(
            theme_button(state),
            theme_menu(state),
            state.theme_menu_open,
            Message::ToggleThemeMenu,
            260.0
        ),
        icon_action(state, "settings", "Configurações", Message::OpenSettings),
        icon_action(state, "info", "Saiba mais", Message::OpenAbout),
        icon_action(state, "folder-plus", "Adicionar pasta", Message::AddFolder),
        icon_action(state, "book-open", "Abrir EPUB", Message::ChooseEpub)
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn icon_action<'a>(
    state: &'a Reader,
    icon: &str,
    label: &'static str,
    message: Message,
) -> Element<'a, Message> {
    let trigger = action::focusable(
        button(icon_svg(icon, 16))
            .width(32)
            .height(32)
            .padding(8)
            .style(move |_, status| picker_style(state, status))
            .on_press(message.clone())
            .into(),
        format!("library-{label}"),
        message,
        None,
    );
    iced::widget::tooltip(
        trigger,
        text(label).size(12),
        iced::widget::tooltip::Position::Bottom,
    )
    .into()
}

fn theme_button(state: &Reader) -> Element<'_, Message> {
    let trigger = action::focusable(
        button(icon_svg("palette", 16))
            .width(32)
            .height(32)
            .padding(8)
            .on_press(Message::ToggleThemeMenu)
            .style(move |_, status| picker_style(state, status))
            .into(),
        "theme-trigger",
        Message::ToggleThemeMenu,
        None,
    );
    iced::widget::tooltip(
        trigger,
        text("Tema").size(12),
        iced::widget::tooltip::Position::Bottom,
    )
    .into()
}

pub(super) fn controls(state: &Reader, count: usize) -> Element<'_, Message> {
    let count = text(format!(
        "{count} {}",
        if count == 1 { "livro" } else { "livros" }
    ))
    .size(13)
    .color(muted_for(state));
    let ordering: Element<'_, Message> = if state.section == LibrarySection::Reading {
        text("Mais recentes")
            .size(13)
            .color(muted_for(state))
            .into()
    } else {
        pick_list(
            vec![
                SortBy::Title,
                SortBy::Author,
                SortBy::Series,
                SortBy::Publisher,
                SortBy::Rating,
            ],
            Some(state.sort_by),
            Message::SortSelected,
        )
        .into()
    };
    let controls = row![
        ordering,
        icon_action(
            state,
            if state.list_view { "grid-2x2" } else { "list" },
            if state.list_view {
                "Exibir capas"
            } else {
                "Exibir lista"
            },
            Message::ToggleListView
        ),
        icon_action(
            state,
            "refresh-cw",
            "Atualizar biblioteca",
            Message::RefreshLibrary
        )
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    if state.window_width < 600.0 {
        column![count, container(controls).align_right(Length::Fill)]
            .spacing(10)
            .into()
    } else {
        row![count, iced::widget::space::horizontal(), controls]
            .align_y(Alignment::Center)
            .into()
    }
}

fn search(state: &Reader) -> Element<'_, Message> {
    text_input("Buscar título, autor, série, gênero…", &state.search)
        .on_input(Message::SearchChanged)
        .padding(10)
        .width(Length::Fill)
        .into()
}

fn header(state: &Reader) -> Element<'_, Message> {
    let menu = icon_action(state, "menu", "Biblioteca", Message::ToggleSidebar);
    if state.window_width < 1100.0 {
        let mut identity = row![menu].spacing(6).align_y(Alignment::Center);
        if state.window_width >= 300.0 {
            identity = identity.push(header_mark(state, 32));
        }
        return column![
            row![
                identity,
                container(actions(state)).align_right(Length::Fill)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            search(state)
        ]
        .spacing(12)
        .into();
    }
    row![menu, wordmark(state), search(state), actions(state)]
        .spacing(20)
        .align_y(Alignment::Center)
        .into()
}

pub(super) fn grid<'a>(state: &'a Reader, items: &[&'a LibraryItem]) -> Element<'a, Message> {
    iced::widget::Grid::with_children(items.iter().map(|item| card(state, item)))
        .fluid(232)
        .spacing(18)
        .height(Length::Shrink)
        .into()
}

fn card<'a>(state: &'a Reader, item: &'a LibraryItem) -> Element<'a, Message> {
    iced::widget::responsive(move |size| {
        let width = (size.width - 16.0).max(1.0);
        let author = item
            .book
            .metadata
            .authors
            .first()
            .map(String::as_str)
            .unwrap_or("Autor desconhecido");
        let body = column![
            cover_art(state, item, width, width * 1.5),
            container(
                text(&item.book.metadata.title)
                    .font(Font::with_name("Literata"))
                    .size(16)
            )
            .height(42)
            .clip(true),
            container(text(author).size(12).color(muted_for(state)))
                .height(20)
                .clip(true)
        ]
        .spacing(8)
        .width(Length::Fill);
        let open = action::focusable(
            button(body)
                .on_press(Message::OpenLibraryBook(item.path.clone()))
                .width(Length::Fill)
                .padding(8)
                .style(move |_, status| {
                    let mut style = card_style(state, status);
                    style.border.width = 0.0;
                    style
                })
                .into(),
            format!("book-{}", item.path.display()),
            Message::OpenLibraryBook(item.path.clone()),
            None,
        );
        container(column![
            open,
            container(book_details::footer(state, item)).padding(iced::Padding {
                top: 0.0,
                right: 8.0,
                bottom: 8.0,
                left: 8.0,
            })
        ])
        .id(iced::widget::Id::from(format!(
            "book-card-{}",
            item.path.display()
        )))
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(surface_for(state).into()),
            text_color: Some(foreground_for(state)),
            border: iced::Border::default()
                .rounded(10)
                .width(1)
                .color(border_for(state)),
            ..Default::default()
        })
        .into()
    })
    .height(Length::Shrink)
    .into()
}

fn card_style(state: &Reader, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let hovered = matches!(
        status,
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
    );
    iced::widget::button::Style {
        background: Some(iced::Background::Color(if hovered {
            mix(surface_for(state), foreground_for(state), 0.06)
        } else {
            surface_for(state)
        })),
        text_color: foreground_for(state),
        border: iced::Border::default()
            .rounded(10)
            .width(1)
            .color(if hovered {
                hex_color(theme_definition_for(state).map_or("#1A4FB4", |t| t.accent.as_str()))
            } else {
                border_for(state)
            }),
        ..Default::default()
    }
}

pub(super) fn about(state: &Reader) -> Element<'_, Message> {
    about::view(state)
}
