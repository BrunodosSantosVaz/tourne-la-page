use super::*;
use crate::reader::diagnostics::{geometry, reader, renderer, send};
use iced::advanced::{layout, widget};
use iced::futures::StreamExt;
use iced::{Event, Point, Size};

fn library_reader() -> Reader {
    let mut state = reader(1);
    state.page = Page::Library;
    state.library = ["Alpha", "Beta", "Gamma"]
        .into_iter()
        .map(|title| {
            let mut book = state.book.as_ref().unwrap().as_ref().clone();
            book.metadata.title = title.into();
            LibraryItem {
                path: PathBuf::from(format!("/tmp/{title}.epub")),
                book: Arc::new(book),
                cover: None,
            }
        })
        .collect();
    state
}

fn finish(task: Task<Message>) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        if let Some(mut stream) = iced_runtime::task::into_stream(task) {
            while let Some(action) = stream.next().await {
                if let iced_runtime::Action::Output(Message::PreferencesSaved(result)) = action {
                    result.unwrap();
                }
            }
        }
    });
}

#[test]
fn reviews_survive_reopening_and_deleting_a_comment_preserves_rating_and_favorite() {
    let mut state = library_reader();
    let path = state.library[0].path.clone();
    let original_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/amostra-tournepage.epub");
    let original = std::fs::read(&original_path).unwrap();
    state.library[0].path = original_path.clone();
    finish(rate(&mut state, original_path.clone(), 4));
    finish(toggle_favorite(&mut state, original_path.clone()));
    let _ = open(&mut state, original_path.clone());
    finish(edit(
        &mut state,
        text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(
            "Uma boa leitura.\nComentário com acentos e várias linhas.".into(),
        ))),
    ));
    let persisted = load(&state.data_dir);
    let saved = &persisted[&original_path];
    assert_eq!(saved.rating, 4);
    assert!(saved.favorite);
    assert!(saved.comment.contains("\nComentário com acentos"));
    assert!(!persisted.contains_key(&path));
    let _ = update(&mut state, Message::CloseBookDetails);
    let _ = open(&mut state, original_path.clone());
    assert_eq!(state.book_review_editor.text(), saved.comment);
    finish(clear_comment(&mut state));
    let persisted = load(&state.data_dir);
    let saved = &persisted[&original_path];
    assert!(saved.comment.is_empty());
    assert_eq!(saved.rating, 4);
    assert!(saved.favorite);
    assert_eq!(std::fs::read(original_path).unwrap(), original);
    assert_eq!(state.page, Page::Library);
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn latest_review_wins_even_if_older_saves_finish_last() {
    let mut state = library_reader();
    let path = state.library[0].path.clone();
    let earlier = rate(&mut state, path.clone(), 2);
    let latest = rate(&mut state, path.clone(), 5);
    finish(latest);
    finish(earlier);
    assert_eq!(load(&state.data_dir)[&path].rating, 5);
    finish(rate(&mut state, path.clone(), 6));
    finish(rate(&mut state, PathBuf::from("/tmp/unknown.epub"), 3));
    assert_eq!(state.book_reviews.len(), 1);
    assert_eq!(rating(&state, &path), 5);
    finish(rate(&mut state, path.clone(), 0));
    assert_eq!(load(&state.data_dir)[&path].rating, 0);
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn ratings_group_highest_first_and_favorites_respect_search_and_removal() {
    let mut state = library_reader();
    let alpha = state.library[0].path.clone();
    let beta = state.library[1].path.clone();
    let _ = rate(&mut state, alpha.clone(), 1);
    let _ = rate(&mut state, beta.clone(), 5);
    state.section = LibrarySection::Rating;
    let groups = grouped_books(&state, filtered_books(&state));
    assert_eq!(
        groups
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["5 estrelas", "1 estrela", "Sem nota"]
    );
    state.sort_by = SortBy::Rating;
    assert_eq!(filtered_books(&state)[0].path, beta);
    let _ = toggle_favorite(&mut state, alpha.clone());
    let _ = toggle_favorite(&mut state, beta.clone());
    state.section = LibrarySection::Favorites;
    assert_eq!(filtered_books(&state).len(), 2);
    assert_eq!(section_count(&state, LibrarySection::Favorites), 2);
    state.search = "alpha".into();
    assert_eq!(filtered_books(&state)[0].path, alpha);
    let _ = toggle_favorite(&mut state, alpha);
    assert!(filtered_books(&state).is_empty());
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn stars_heart_and_pencil_are_independent_of_opening_the_epub() {
    let state = library_reader();
    let renderer = renderer();
    let item = &state.library[0];
    for width in [140.0, 220.0, 400.0] {
        let mut page = footer(&state, item);
        let mut tree = widget::Tree::new(&page);
        let size = Size::new(width, 50.0);
        let node =
            page.as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let _ = send(
            &mut page,
            &mut tree,
            &node,
            &renderer,
            Event::Window(iced::window::Event::RedrawRequested(
                std::time::Instant::now(),
            )),
            Point::ORIGIN,
        );
        let measured = geometry(&mut page, &mut tree, &node, &renderer);
        let star =
            measured.bounds[&widget::Id::from(format!("book-rating-{}-4", item.path.display()))];
        let heart =
            measured.bounds[&widget::Id::from(format!("book-favorite-{}", item.path.display()))];
        let pencil =
            measured.bounds[&widget::Id::from(format!("book-details-{}", item.path.display()))];
        let progress =
            measured.bounds[&widget::Id::from(format!("book-progress-{}", item.path.display()))];
        assert!(progress.y + progress.height <= star.y);
        assert!(progress.x >= 0.0 && progress.x + progress.width <= width);
        assert!(star.x < heart.x && heart.x < pencil.x);
        assert!((pencil.x + pencil.width - width).abs() < 0.1);
        for (bounds, expected) in [(star, 0), (heart, 1), (pencil, 2)] {
            let point = Point::new(bounds.center_x(), bounds.center_y());
            for event in [
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left),
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left),
            ] {
                let messages = send(
                    &mut page,
                    &mut tree,
                    &node,
                    &renderer,
                    Event::Mouse(event),
                    point,
                );
                if matches!(event, iced::mouse::Event::ButtonReleased(_)) {
                    assert_eq!(messages.len(), 1);
                    match (&messages[0], expected) {
                        (Message::RateBook(path, 4), 0)
                        | (Message::ToggleBookFavorite(path), 1)
                        | (Message::OpenBookDetails(path), 2) => assert_eq!(path, &item.path),
                        _ => panic!("Unexpected footer action: {:?}", messages[0]),
                    }
                }
            }
        }
    }
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn details_keep_close_above_scroll_content_and_fit_narrow_windows() {
    let mut state = library_reader();
    let renderer = renderer();
    let path = state.library[0].path.clone();
    let _ = open(&mut state, path);
    for (width, height) in [(1280.0, 800.0), (390.0, 800.0), (284.0, 360.0)] {
        state.window_width = width;
        let mut page = view(&state);
        let mut tree = widget::Tree::new(&page);
        let size = Size::new(width, height);
        let node =
            page.as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let measured = geometry(&mut page, &mut tree, &node, &renderer);
        let card = measured.bounds[&widget::Id::new("book-details-modal")];
        let close = measured.bounds[&widget::Id::new("book-details-close")];
        let (scroll, _) = measured.scrolls[&widget::Id::new("book-details-scroll")];
        assert!(card.x >= 0.0 && card.x + card.width <= width);
        assert!(card.y >= 0.0 && card.y + card.height <= height + 0.1);
        assert!((close.x + close.width - (card.x + card.width - 8.0)).abs() < 0.1);
        assert!(close.y + close.height <= scroll.y);
        assert!(scroll.y + scroll.height <= card.y + card.height + 0.1);
        let point = Point::new(close.center_x(), close.center_y());
        let _ = send(
            &mut page,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
            point,
        );
        let messages = send(
            &mut page,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )),
            point,
        );
        assert!(matches!(messages.as_slice(), [Message::CloseBookDetails]));
    }
    std::fs::remove_dir_all(state.data_dir).unwrap();
}
