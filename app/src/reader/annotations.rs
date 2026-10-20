//! Colored notes share one editor, while the list reserves room for its scrollbar.
use super::*;
use iced::widget::{column, row};

pub(super) fn view<'a>(state: &'a Reader, book: &'a Book) -> Element<'a, Message> {
    let mut entries = column![editor(state)].spacing(12).width(Length::Fill);
    for annotation in &state.annotations {
        entries = entries.push(note(state, book, annotation));
    }
    container(column![
        container(
            row![
                text("Marcações e comentários")
                    .font(Font::with_name("Literata"))
                    .size(18)
                    .width(Length::Fill),
                toolbar_button(state, "x", "Fechar marcações", Message::CloseAnnotations),
            ]
            .align_y(Alignment::Center)
        )
        .padding(12),
        adaptive_scroll::new(container(entries).padding(12))
            .id(iced::widget::Id::new("annotation-scroll"))
            .height(Length::Fill)
            .width(Length::Fill),
    ])
    .width(Length::Fixed(
        (if state.window_width < 1100.0 {
            260.0_f32
        } else {
            310.0
        })
        .min((state.window_width - 24.0).max(1.0)),
    ))
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(surface_for(state))),
        text_color: Some(foreground_for(state)),
        border: iced::Border::default()
            .rounded(10)
            .color(border_for(state))
            .width(1),
        ..Default::default()
    })
    .into()
}

fn editor(state: &Reader) -> Element<'_, Message> {
    let Some(annotation) = state
        .active_mark_id
        .as_deref()
        .and_then(|id| state.annotations.iter().find(|note| note.id == id))
    else {
        return text(
            "Selecione um trecho no livro ou clique em uma marcação para comentar e trocar a cor.",
        )
        .size(14)
        .color(muted_for(state))
        .into();
    };
    let message = Message::ToggleAnnotationPalette(annotation.id.clone());
    let color = action::focusable(
        button(
            row![
                color_swatch(&annotation.marker_color, 16.0),
                text("Cor").size(14),
                icon_svg("chevron-down", 12)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(message.clone())
        .height(32)
        .style(move |_, status| picker_style(state, status))
        .into(),
        "annotation-editor-color",
        message.clone(),
        None,
    );
    column![
        text("Trecho selecionado").size(12).color(muted_for(state)),
        text(format!("“{}”", annotation.excerpt))
            .size(14)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        iced::widget::text_editor(&state.annotation_editor)
            .id(iced::widget::Id::new("annotation-comment"))
            .placeholder("Comentário opcional…")
            .size(14)
            .height(Length::Shrink)
            .min_height(42)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            .padding(10)
            .on_action(Message::AnnotationEdited)
            .key_binding(comment_key_binding),
        text("Enter salva · Shift+Enter quebra a linha")
            .size(11)
            .color(muted_for(state)),
        popover::anchored(
            color,
            color_menu(state, Some(annotation)),
            state.annotation_palette_open.as_deref() == Some(annotation.id.as_str()),
            message,
            72.0
        ),
    ]
    .spacing(8)
    .width(Length::Fill)
    .into()
}

fn comment_key_binding(
    press: iced::widget::text_editor::KeyPress,
) -> Option<iced::widget::text_editor::Binding<Message>> {
    use iced::widget::text_editor::{Binding, Status};
    if matches!(press.status, Status::Focused { .. })
        && press.key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
        && !press.modifiers.shift()
    {
        Some(Binding::Custom(Message::SubmitAnnotationComment))
    } else {
        Binding::from_key_press(press)
    }
}

fn note<'a>(
    state: &'a Reader,
    book: &'a Book,
    annotation: &'a BookAnnotation,
) -> Element<'a, Message> {
    let background = if valid_hex(&annotation.marker_color) {
        hex_color(&annotation.marker_color)
    } else {
        surface_for(state)
    };
    let foreground = ensure_contrast(Color::BLACK, background, 4.5);
    let selected = state.active_mark_id.as_deref() == Some(annotation.id.as_str());
    let chapter = book
        .chapters
        .get(annotation.chapter_index)
        .map_or("Capítulo", |chapter| chapter.title.as_str());
    let kind = if matches!(annotation.kind, AnnotationKind::Bookmark) {
        "Posição salva"
    } else {
        "Trecho marcado"
    };
    let content = column![
        row![
            text(kind).size(12).color(foreground).width(Length::Fill),
            iced::widget::space().width(24)
        ]
        .height(24)
        .align_y(Alignment::Center),
        text(chapter)
            .size(12)
            .color(foreground)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        text(annotation.excerpt.as_str())
            .size(14)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            .color(foreground),
        text(annotation.text.as_str())
            .size(14)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            .color(foreground),
    ]
    .spacing(6)
    .width(Length::Fill);
    let open = Message::OpenAnnotation(annotation.id.clone());
    let body = action::focusable(
        button(content)
            .padding(12)
            .width(Length::Fill)
            .on_press(open.clone())
            .style(move |_, status| iced::widget::button::Style {
                background: Some(iced::Background::Color(
                    if matches!(status, iced::widget::button::Status::Hovered) {
                        mix(background, foreground, 0.06)
                    } else {
                        background
                    },
                )),
                text_color: foreground,
                border: iced::Border::default()
                    .rounded(6)
                    .width(if selected { 2 } else { 1 })
                    .color(mix(
                        background,
                        foreground,
                        if selected { 0.6 } else { 0.2 },
                    )),
                ..Default::default()
            })
            .into(),
        format!("annotation-note-{}", annotation.id),
        open,
        None,
    );
    let delete = Message::DeleteAnnotation(annotation.id.clone());
    let close = action::focusable(
        button(text("×").size(20).color(foreground))
            .width(24)
            .height(24)
            .padding(0)
            .style(iced::widget::button::text)
            .on_press(delete.clone())
            .into(),
        format!("annotation-delete-{}", annotation.id),
        delete,
        None,
    );
    stack![body, container(close).padding(4).align_right(Length::Fill)]
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::diagnostics::{geometry, reader, renderer, send};
    use iced::advanced::{layout, widget};
    use iced::{Event, Point, Size, mouse};

    #[test]
    fn scroll_never_covers_notes_and_delete_stays_in_the_note_corner() {
        let mut reader = reader(100);
        let renderer = renderer();
        reader.annotations = (0..40)
            .map(|i| BookAnnotation {
                id: format!("note-{i}"),
                chapter_index: 0,
                kind: AnnotationKind::Highlight,
                text: format!("Comentário {i}"),
                excerpt: "Trecho marcado para testar o post-it".into(),
                start_offset: i * 10,
                end_offset: i * 10 + 8,
                marker_color: "#F9E2AF".into(),
            })
            .collect();
        let book = Arc::clone(reader.book.as_ref().unwrap());
        let mut panel = view(&reader, &book);
        let mut tree = widget::Tree::new(&panel);
        let size = Size::new(310.0, 480.0);
        let node =
            panel
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let measured = geometry(&mut panel, &mut tree, &node, &renderer);
        let (scroll, content) = measured.scrolls[&widget::Id::new("annotation-scroll")];
        assert!(content.height > scroll.height);
        let note = measured.bounds[&widget::Id::new("annotation-note-note-0")];
        let delete = measured.bounds[&widget::Id::new("annotation-delete-note-0")];
        assert!(note.x + note.width <= scroll.x + scroll.width - 12.0);
        assert!((delete.x + delete.width - (note.x + note.width - 4.0)).abs() < 0.1);
        assert!((delete.y - note.y - 4.0).abs() < 0.1);
        let point = Point::new(delete.x + 12.0, delete.y + 12.0);
        let mut messages = Vec::new();
        for event in [
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left),
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left),
        ] {
            messages.extend(send(
                &mut panel,
                &mut tree,
                &node,
                &renderer,
                Event::Mouse(event),
                point,
            ));
        }
        assert!(matches!(messages.as_slice(), [Message::DeleteAnnotation(id)] if id == "note-0"));
        drop(panel);
        let _ = update(&mut reader, Message::OpenAnnotation("note-1".into()));
        assert_eq!(reader.active_mark_id.as_deref(), Some("note-1"));
        assert_eq!(reader.annotation_draft, "Comentário 1");
        let mut panel = view(&reader, &book);
        let mut tree = widget::Tree::new(&panel);
        let node =
            panel
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let measured = geometry(&mut panel, &mut tree, &node, &renderer);
        assert!(
            measured
                .bounds
                .contains_key(&widget::Id::new("annotation-editor-color"))
        );
        assert!(
            !measured
                .bounds
                .contains_key(&widget::Id::new("annotation-color-note-0"))
        );
        drop(panel);
        let color = theme_definition_for(&reader).unwrap().marker_colors[1].clone();
        let _ = update(
            &mut reader,
            Message::AnnotationColorSelected("note-1".into(), color.clone()),
        );
        assert_eq!(reader.annotations[1].marker_color, color);
        assert_eq!(reader.annotations[1].text, "Comentário 1");
        let _ = update(
            &mut reader,
            Message::ToggleAnnotationPalette("note-1".into()),
        );
        assert!(reader.annotation_palette_open.is_some());
        let _ = update(&mut reader, Message::CloseAnnotations);
        assert!(reader.annotation_palette_open.is_none());
        assert!(!reader.show_annotations);
        std::fs::remove_dir_all(&reader.data_dir).unwrap();
    }
    #[test]
    fn long_comments_wrap_inside_notes_and_editor_grows_with_the_text() {
        let mut reader = reader(1);
        let renderer = renderer();
        let _ = update(
            &mut reader,
            Message::TextSelected(vec![SelectedExcerpt {
                chapter_index: 0,
                text: "Trecho".into(),
                start_offset: 0,
                end_offset: 6,
            }]),
        );
        let book = Arc::clone(reader.book.as_ref().unwrap());
        for width in [210.0, 286.0] {
            let mut heights = Vec::new();
            for comment in [
                "curto".to_owned(),
                format!(
                    "{}\n{}",
                    "Uma ideia muito longa, mas dentro do post-it. ".repeat(50),
                    "https://exemplo.com/".to_owned() + &"caminhosemespacos".repeat(50)
                ),
            ] {
                let _ = update(&mut reader, Message::AnnotationDraftChanged(comment));
                let mut control = editor(&reader);
                let mut tree = widget::Tree::new(&control);
                let node = control.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)),
                );
                let measured = geometry(&mut control, &mut tree, &node, &renderer);
                let input = measured.bounds[&widget::Id::new("annotation-comment")];
                assert!(input.width <= width);
                drop(control);
                let annotation = &reader.annotations[0];
                let mut control = note(&reader, &book, annotation);
                let mut tree = widget::Tree::new(&control);
                let node = control.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)),
                );
                assert!(node.size().width <= width);
                heights.push((input.height, node.size().height));
            }
            assert!(
                heights[1].0 > heights[0].0 + 200.0,
                "editor must grow: {heights:?}"
            );
            assert!(
                heights[1].1 > heights[0].1 + 200.0,
                "note must grow: {heights:?}"
            );
        }
        std::fs::remove_dir_all(&reader.data_dir).unwrap();
    }
    #[test]
    fn multiline_edits_keep_autosave_and_enter_submit_while_shift_enter_breaks_lines() {
        use iced::widget::text_editor::{Action, Edit};
        let mut reader = reader(1);
        let renderer = renderer();
        let _ = update(
            &mut reader,
            Message::TextSelected(vec![SelectedExcerpt {
                chapter_index: 0,
                text: "Trecho".into(),
                start_offset: 0,
                end_offset: 6,
            }]),
        );
        let comment = "Primeira linha\nSegunda linha com observações.";
        let _ = update(
            &mut reader,
            Message::AnnotationEdited(Action::Edit(Edit::Paste(Arc::new(comment.into())))),
        );
        assert_eq!(reader.annotation_draft, comment);
        assert_eq!(reader.annotation_editor.text(), comment);
        assert_eq!(reader.annotations[0].text, comment);
        assert!(reader.annotations_revision > 0);
        let mut control = editor(&reader);
        let mut tree = widget::Tree::new(&control);
        let node = control.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(250.0, f32::INFINITY)),
        );
        let measured = geometry(&mut control, &mut tree, &node, &renderer);
        let input = measured.bounds[&widget::Id::new("annotation-comment")];
        let point = Point::new(input.x + 15.0, input.y + 15.0);
        let _ = send(
            &mut control,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            point,
        );
        let enter = super::super::diagnostics::key(iced::keyboard::key::Named::Enter);
        let messages = send(
            &mut control,
            &mut tree,
            &node,
            &renderer,
            enter.clone(),
            point,
        );
        assert!(matches!(
            messages.as_slice(),
            [Message::SubmitAnnotationComment]
        ));
        let mut shifted = enter;
        if let Event::Keyboard(iced::keyboard::Event::KeyPressed { modifiers, .. }) = &mut shifted {
            *modifiers = iced::keyboard::Modifiers::SHIFT;
        }
        let messages = send(&mut control, &mut tree, &node, &renderer, shifted, point);
        assert!(matches!(
            messages.as_slice(),
            [Message::AnnotationEdited(Action::Edit(Edit::Enter))]
        ));
        drop(control);
        // A growing editor must scroll the sidebar rather than trap the wheel internally.
        use iced::futures::StreamExt as _;
        let task = update(
            &mut reader,
            Message::AnnotationEdited(Action::Scroll { lines: 12 }),
        );
        let actions = iced::futures::executor::block_on(
            iced_runtime::task::into_stream(task)
                .unwrap()
                .collect::<Vec<_>>(),
        );
        assert!(matches!(
            actions.as_slice(),
            [iced_runtime::Action::Widget(_)]
        ));
        assert_eq!(reader.annotation_editor.text(), comment);
        std::fs::remove_dir_all(&reader.data_dir).unwrap();
    }
}
