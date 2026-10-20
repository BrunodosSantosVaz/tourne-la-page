//! Controls share their inline, compact, and tray presentations.
#[cfg(test)]
mod tests;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Item {
    Theme,
    Font,
    Size,
    Alignment,
    Width,
    LineHeight,
    Images,
    Marker,
    Speed,
}

impl Item {
    fn label(self) -> &'static str {
        match self {
            Self::Theme => "Tema",
            Self::Font => "Fonte",
            Self::Size => "Tamanho",
            Self::Alignment => "Alinhamento",
            Self::Width => "Largura",
            Self::LineHeight => "Entrelinha",
            Self::Images => "Exibir imagem",
            Self::Marker => "Marca-texto",
            Self::Speed => "Velocidade",
        }
    }
    fn icon(self, state: &Reader) -> &'static str {
        match self {
            Self::Theme => "palette",
            Self::Font => "type",
            Self::Size => "a-large-small",
            Self::Alignment => match state.settings.alignment {
                AlignmentChoice::Left => "align-left",
                AlignmentChoice::Center => "align-center",
                AlignmentChoice::Right => "align-right",
                AlignmentChoice::Justify => "align-justify",
            },
            Self::Width => "move-horizontal",
            Self::LineHeight => "move-vertical",
            Self::Images => {
                if state.settings.show_images {
                    "image"
                } else {
                    "image-off"
                }
            }
            Self::Marker => "highlighter",
            Self::Speed => "gauge",
        }
    }
    pub(super) fn trigger_id(self) -> &'static str {
        match self {
            Self::Theme => "theme-trigger",
            Self::Font => "font-trigger",
            Self::Marker => "marker-trigger",
            Self::Size => "compact-size",
            Self::Alignment => "compact-alignment",
            Self::Width => "compact-width",
            Self::LineHeight => "compact-line-height",
            Self::Images => "reading-images",
            Self::Speed => "compact-speed",
        }
    }
    pub(super) fn first_focus(self) -> &'static str {
        match self {
            Self::Theme => "theme-option-0",
            Self::Font => "font-option-0",
            Self::Marker => "marker-option-0",
            Self::Size => "font-smaller",
            Self::Alignment => "reading-align-left",
            Self::Width => "reading-width",
            Self::LineHeight => "reading-line-height",
            Self::Images => "reading-images",
            Self::Speed => "speed-smaller",
        }
    }
}

pub(super) fn view(state: &Reader) -> Element<'_, Message> {
    let controls = [
        Item::Speed,
        Item::Font,
        Item::Size,
        Item::Alignment,
        Item::Width,
        Item::LineHeight,
        Item::Images,
        Item::Marker,
    ]
    .into_iter()
    .map(|item| bar::Control {
        full: full_control(state, item),
        clean: clean_control(state, item),
        tray: if item == Item::Speed {
            scroll_speed_control_labeled(state)
        } else {
            full_control(state, item)
        },
    })
    .collect();
    bar::Bar::view(
        [
            toolbar_button(state, "menu", "Índice", Message::ToggleSidebar),
            toolbar_button(state, "house", "Início", Message::ReturnToLibrary),
        ],
        controls,
        [
            toolbar_button(
                state,
                if state.settings.auto_scroll {
                    "pause"
                } else {
                    "play"
                },
                "Rolagem automática",
                Message::ToggleAutoScroll,
            ),
            clean_control(state, Item::Theme),
            toolbar_button(state, "settings", "Configurações", Message::OpenSettings),
            toolbar_button(
                state,
                "chevron-down",
                "Comentários",
                Message::ToggleAnnotations,
            ),
            toolbar_button(
                state,
                if state.toolbar_overflow_open {
                    "chevron-up"
                } else {
                    "rows-3"
                },
                if state.toolbar_overflow_open {
                    "Recolher barra"
                } else {
                    "Expandir barra"
                },
                Message::ToggleToolbarOverflow,
            ),
        ],
        state.toolbar_overflow_open,
    )
}

fn full_control(state: &Reader, item: Item) -> Element<'_, Message> {
    match item {
        Item::Size => font_size_control(state),
        Item::Speed => scroll_speed_control(state),
        _ => clean_control(state, item),
    }
}

fn clean_control(state: &Reader, item: Item) -> Element<'_, Message> {
    if item == Item::Speed {
        return scroll_speed_control(state);
    }
    let message = if item == Item::Images {
        Message::ImagesToggled(!state.settings.show_images)
    } else {
        Message::ToggleCompactMenu(item)
    };
    let trigger = clean_trigger(state, item, message.clone());
    if item == Item::Images {
        return trigger;
    }
    let (menu, width) = match item {
        Item::Theme => (theme_menu(state), 260.0),
        Item::Font => (font_menu(state), 280.0),
        Item::Marker => (color_menu(state, None), 72.0),
        Item::Width => (overflow_panel(state, width_control(state)), 100.0),
        Item::LineHeight => (overflow_panel(state, line_height_control(state)), 110.0),
        Item::Alignment => (
            overflow_panel(state, alignment_controls(state.settings.alignment)),
            200.0,
        ),
        Item::Size => (overflow_panel(state, font_size_control(state)), 260.0),
        Item::Images | Item::Speed => unreachable!(),
    };
    let open = state.compact_menu == Some(item);
    popover::anchored(trigger, menu, open, message, width)
}

fn clean_trigger(state: &Reader, item: Item, message: Message) -> Element<'_, Message> {
    let symbol = if item == Item::Marker {
        color_swatch(&selected_marker_color(state), 16.0)
    } else {
        icon_svg(item.icon(state), 16)
    };
    let trigger = action::focusable(
        button(symbol)
            .width(32)
            .height(32)
            .padding(8)
            .style(move |theme, status| {
                let mut style = picker_style(state, status);
                if item == Item::Images && state.settings.show_images {
                    style.background = Some(iced::Background::Color(mix(
                        surface_for(state),
                        foreground_for(state),
                        0.26,
                    )));
                    style.border.color = theme.palette().primary;
                    style.border.width = 2.0;
                }
                style
            })
            .on_press(message.clone())
            .into(),
        item.trigger_id(),
        message,
        None,
    );
    iced::widget::tooltip(
        trigger,
        text(item.label()).size(12),
        iced::widget::tooltip::Position::Bottom,
    )
    .into()
}
