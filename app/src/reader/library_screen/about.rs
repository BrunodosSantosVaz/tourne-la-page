use super::*;
use iced::widget::{column, row};

pub(super) fn view(state: &Reader) -> Element<'_, Message> {
    let backdrop = button("")
        .on_press(Message::CloseAbout)
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.0, 0.0, 0.0, 0.65,
            ))),
            ..Default::default()
        });
    let card = container(column![
        container(row![iced::widget::space::horizontal(), close(state)])
            .padding(8)
            .width(Length::Fill),
        adaptive_scroll::new(container(content(state)).padding(iced::Padding {
            top: 0.0,
            right: 24.0,
            bottom: 24.0,
            left: 24.0,
        }))
        .id(iced::widget::Id::new("about-content"))
        .height(Length::Shrink),
    ])
    .id(iced::widget::Id::new("about-card"))
    .max_width(560)
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(surface_for(state))),
        text_color: Some(foreground_for(state)),
        border: iced::Border::default()
            .rounded(16)
            .width(1)
            .color(border_for(state)),
        ..Default::default()
    });
    stack![backdrop, container(card).padding(12).center(Length::Fill)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn close(state: &Reader) -> Element<'_, Message> {
    action::focusable(
        button(icon_svg("x", 16))
            .on_press(Message::CloseAbout)
            .width(32)
            .height(32)
            .padding(8)
            .style(move |_, status| picker_style(state, status))
            .into(),
        "about-close",
        Message::CloseAbout,
        None,
    )
}

fn content(state: &Reader) -> Element<'_, Message> {
    column![row![mark(64), iced::widget::space::horizontal(),
            action::focusable(button(icon_svg("x", 16)).on_press(Message::CloseAbout)
                .width(32).height(32).padding(8).style(move |_, status| picker_style(state, status)).into(), "about-close", Message::CloseAbout, None)]
            .align_y(Alignment::Center),
        text(brand::NAME).font(Font::with_name("Literata")).size(32),
        text("Menos ruído, mais leitura.").size(16).color(muted_for(state)),
        text(format!("Versão {} · edição local", brand::VERSION)).size(14),
        text("Uma biblioteca para chamar de sua.").font(Font::with_name("Literata")).size(22),
        text("Encontre a próxima leitura entre suas capas, organize seus livros e vire a página no seu ritmo. Seus arquivos EPUB permanecem no computador.").size(15),
        field("Desenvolvedor", "BrunodosSantosVaz"), field("Licença", "MIT · código aberto"),
        row![link("GitHub do projeto", brand::Link::Repository),
            link("Desenvolvedor", brand::Link::Developer)].spacing(8).wrap(),
        row![link("Relatar um problema", brand::Link::Issues),
            link("Licença", brand::Link::License)].spacing(8).wrap(),
    ].spacing(16).into()
}

fn field(label: &str, value: &str) -> Element<'static, Message> {
    column![
        text(label.to_owned()).size(12),
        text(value.to_owned()).size(15)
    ]
    .spacing(4)
    .into()
}

fn link(label: &str, target: brand::Link) -> Element<'static, Message> {
    let message = Message::OpenProjectLink(target);
    action::focusable(
        button(text(label.to_owned()).size(14))
            .on_press(message.clone())
            .into(),
        format!("about-{label}"),
        message,
        None,
    )
}
