use super::*;
#[test]
fn mouse_drag_publishes_one_selection_only_on_release_and_survives_reflow() {
    use super::super::diagnostics::{reader, renderer, send};
    let mut reader = reader(1000);
    let renderer = renderer();
    let book = std::sync::Arc::clone(reader.book.as_ref().unwrap());
    let mut element = view(&reader, &book);
    let mut tree = widget::Tree::new(&element);
    let viewport = Size::new(1000.0, 800.0);
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(viewport, viewport),
    );
    let state = tree.state.downcast_ref::<State>();
    let first = state
        .blocks
        .iter()
        .position(|block| matches!(block.kind, Kind::Text { .. }))
        .unwrap();
    let start = Point::new(
        state.text_x(Rectangle::with_size(viewport)) + 10.0,
        state.heights.prefix(first) - state.y + 12.0,
    );
    let end = Point::new(
        start.x + 130.0,
        state.heights.prefix(first + 1) - state.y + 12.0,
    );
    let _ = send(
        &mut element,
        &mut tree,
        &node,
        &renderer,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        start,
    );
    for step in 1..=20 {
        let point = Point::new(
            start.x + (end.x - start.x) * step as f32 / 20.0,
            start.y + (end.y - start.y) * step as f32 / 20.0,
        );
        let messages = send(
            &mut element,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::CursorMoved { position: point }),
            point,
        );
        assert!(
            !messages
                .iter()
                .any(|m| matches!(m, Message::TextSelected(_)))
        );
    }
    let messages = send(
        &mut element,
        &mut tree,
        &node,
        &renderer,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        end,
    );
    let selections = messages
        .into_iter()
        .filter_map(|m| {
            if let Message::TextSelected(s) = m {
                Some(s)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(selections.len(), 1);
    assert_eq!(selections[0].len(), 1);
    assert!(selections[0][0].text.contains("\n\n"));
    let original = selections[0][0].text.clone();
    drop(element);
    let state = tree.state.downcast_mut::<State>();
    let shaped = state.shaped;
    for _ in 0..100 {
        state.prepare(&book);
    }
    assert_eq!(
        shaped, state.shaped,
        "stationary frames must reuse text layouts"
    );
    assert!(state.paragraphs.len() < 128);
    reader.settings.font_scale = 140;
    state.configure(&reader, &book, Size::new(700.0, 800.0));
    assert_eq!(
        state.selected(&book)[0].text,
        original,
        "reflow must preserve source byte offsets"
    );
    std::fs::remove_dir_all(reader.data_dir).unwrap();
}
#[test]
fn measured_height_updates_preserve_fast_offsets_and_lookup() {
    let mut h = Heights::new(vec![10.0, 20.0, 30.0]);
    assert_eq!(h.find(29.0), 1);
    h.set(1, 40.0);
    assert_eq!(h.prefix(2), 50.0);
    assert_eq!(h.find(49.0), 1);
    assert_eq!(h.find(50.0), 2);
}
#[test]
fn word_selection_uses_utf8_offsets() {
    assert_eq!(word_range("uma seleção natural", 7), (4, 13));
}
#[test]
fn scrollbar_drag_matches_its_changing_thumb() {
    for progress in [0.0, 0.2, 0.5, 0.8, 1.0] {
        let (size, top) = super::super::adaptive_scroll_thumb(800.0, progress * 100.0);
        assert!((scrollbar_progress(800.0, top + size * 0.3, 0.3) - progress).abs() < 0.00001);
    }
}
#[test]
fn selection_spans_paragraphs_once_and_preserves_repeated_text() {
    let book = Book {
        chapters: vec![epub_core::Chapter {
            title: "Capítulo".into(),
            text: "eco eco\n\noutra frase".into(),
        }],
        ..Default::default()
    };
    let state = State {
        blocks: vec![
            Block {
                chapter: 0,
                kind: Kind::Text {
                    start: 0,
                    end: 7,
                    paragraph: 0,
                },
            },
            Block {
                chapter: 0,
                kind: Kind::Text {
                    start: 9,
                    end: 20,
                    paragraph: 1,
                },
            },
        ],
        selection: Some((
            Endpoint { block: 0, byte: 4 },
            Endpoint { block: 1, byte: 5 },
        )),
        ..State::default()
    };
    let selected = state.selected(&book);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].text, "eco\n\noutra");
    assert_eq!(selected[0].start_offset, 4);
}
#[test]
fn finishing_background_loading_reports_the_full_books_position_without_moving_it() {
    let mut reader = crate::reader::diagnostics::reader(10);
    let book = std::sync::Arc::clone(reader.book.as_ref().unwrap());
    let mut state = State::default();
    reader.book_loading = true;
    state.configure(&reader, &book, Size::new(800.0, 600.0));
    state.move_to(200.0);
    state.dirty = false;
    let position = state.y;
    reader.book_loading = false;
    state.configure(&reader, &book, Size::new(800.0, 600.0));
    assert!(state.dirty);
    assert!((state.y - position).abs() < 0.1);
    std::fs::remove_dir_all(reader.data_dir).unwrap();
}
