use super::*;
use crate::reader::diagnostics::reader;
use iced::futures::StreamExt;

fn library_reader() -> Reader {
    let mut state = reader(3);
    state.library = ["Alpha", "Beta", "Gamma"]
        .into_iter()
        .map(|name| LibraryItem {
            path: PathBuf::from(format!("/tmp/{name}.epub")),
            book: Arc::clone(state.book.as_ref().unwrap()),
            cover: None,
        })
        .collect();
    state
}

fn finish(task: Task<Message>) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            if let Some(mut stream) = iced_runtime::task::into_stream(task) {
                while let Some(action) = stream.next().await {
                    if let iced_runtime::Action::Output(
                        Message::PreferencesSaved(result) | Message::CloseFlushed(_, result),
                    ) = action
                    {
                        result.unwrap();
                    }
                }
            }
        });
}

#[test]
fn reading_lists_every_started_unfinished_book_in_recent_order() {
    let mut state = library_reader();
    let alpha = state.library[0].path.clone();
    let beta = state.library[1].path.clone();
    let gamma = state.library[2].path.clone();
    let _ = update(&mut state, Message::OpenLibraryBook(alpha.clone()));
    let _ = update(
        &mut state,
        Message::SurfacePosition(0, 400.0, 1000.0, 600.0),
    );
    let _ = update(&mut state, Message::OpenLibraryBook(beta.clone()));
    state.section = LibrarySection::Reading;
    state.sort_by = SortBy::Title;
    assert_eq!(
        filtered_books(&state)
            .iter()
            .map(|b| &b.path)
            .collect::<Vec<_>>(),
        [&beta, &alpha]
    );
    assert_eq!(section_count(&state, LibrarySection::Reading), 2);
    assert_eq!(state.last_opened_path.as_ref(), Some(&beta));
    assert_eq!(percent(&state, &gamma), 0.0);
    let _ = update(
        &mut state,
        Message::SurfacePosition(0, 1000.0, 1000.0, 600.0),
    );
    assert_eq!(percent(&state, &beta), 100.0);
    assert_eq!(filtered_books(&state).len(), 1);
    let _ = update(
        &mut state,
        Message::SurfacePosition(0, 200.0, 1000.0, 600.0),
    );
    assert_eq!(
        percent(&state, &beta),
        100.0,
        "rereading must not undo completion"
    );
    state.search = "not found".into();
    assert!(filtered_books(&state).is_empty());
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn partial_loading_and_invalid_positions_never_mark_a_book_as_finished() {
    let mut state = library_reader();
    let path = state.library[0].path.clone();
    let _ = update(&mut state, Message::OpenLibraryBook(path.clone()));
    state.book_loading = true;
    remember(&mut state, 0, 100.0, 100.0);
    assert_eq!(percent(&state, &path), 0.0);
    state.book_loading = false;
    for (position, maximum) in [(f32::NAN, 100.0), (10.0, f32::INFINITY), (50.0, -1.0)] {
        remember(&mut state, 0, position, maximum);
    }
    assert_eq!(percent(&state, &path), 0.0);
    remember(&mut state, 0, 999.5, 1000.0);
    assert_eq!(display_percent(percent(&state, &path)), 99);
    assert!(in_progress(&state, &path));
    remember(&mut state, 0, 1000.0, 1000.0);
    assert!(!in_progress(&state, &path));
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn returning_to_library_flushes_latest_progress_without_overwriting_other_books() {
    let mut state = library_reader();
    let alpha = state.library[0].path.clone();
    let beta = state.library[1].path.clone();
    finish(update(&mut state, Message::OpenLibraryBook(alpha.clone())));
    let _ = update(
        &mut state,
        Message::SurfacePosition(0, 123.4, 1000.0, 600.0),
    );
    let old = flush(&mut state);
    for position in [200.0, 400.0, 670.0] {
        let task = update(
            &mut state,
            Message::SurfacePosition(0, position, 1000.0, 600.0),
        );
        assert!(
            iced_runtime::task::into_stream(task).is_none(),
            "scroll must not write SQLite"
        );
    }
    finish(update(&mut state, Message::ReturnToLibrary));
    finish(old);
    finish(update(&mut state, Message::OpenLibraryBook(beta.clone())));
    let _ = update(
        &mut state,
        Message::SurfacePosition(0, 1000.0, 1000.0, 600.0),
    );
    finish(update(&mut state, Message::SaveReadingProgress));
    let (reopened, _) = Reader::new(state.data_dir.clone());
    assert_eq!(percent(&reopened, &alpha), 67.0);
    assert_eq!(percent(&reopened, &beta), 100.0);
    assert!(reopened.reading_history[&beta].last_read > reopened.reading_history[&alpha].last_read);
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn legacy_chapter_positions_restore_all_previously_opened_books() {
    let mut state = library_reader();
    let alpha = state.library[0].path.clone();
    let beta = state.library[1].path.clone();
    let mut book = state.library[0].book.as_ref().clone();
    book.chapters = vec![
        Chapter {
            title: "One".into(),
            text: "a".repeat(100),
        },
        Chapter {
            title: "Two".into(),
            text: "b".repeat(300),
        },
    ];
    state.library[0].book = Arc::new(book);
    library::save_preference(
        &state.data_dir,
        &format!("chapter:{}", alpha.display()),
        "1",
    )
    .unwrap();
    library::save_preference(&state.data_dir, &format!("chapter:{}", beta.display()), "0").unwrap();
    state.reading_history = load(&state.data_dir, Some(&beta));
    restore_legacy(&mut state);
    assert_eq!(percent(&state, &alpha), 25.0);
    assert!(in_progress(&state, &alpha) && in_progress(&state, &beta));
    state.section = LibrarySection::Reading;
    assert_eq!(filtered_books(&state)[0].path, beta);
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn closing_waits_for_progress_to_be_saved_and_stays_open_if_the_save_fails() {
    assert!(!main_window_settings().exit_on_close_request);
    let mut state = library_reader();
    let path = state.library[0].path.clone();
    finish(update(&mut state, Message::OpenLibraryBook(path.clone())));
    remember(&mut state, 0, 420.0, 1000.0);
    let window = iced::window::Id::unique();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let actions = runtime.block_on(async {
        iced_runtime::task::into_stream(close(&mut state, window))
            .unwrap()
            .collect::<Vec<_>>()
            .await
    });
    assert!(
        matches!(actions.as_slice(), [iced_runtime::Action::Output(Message::CloseFlushed(id, Ok(())))] if *id == window)
    );
    assert_eq!(load(&state.data_dir, None)[&path].percent, 42.0);
    assert!(!state.reading_dirty);
    std::fs::remove_dir_all(&state.data_dir).unwrap();
    let actions = runtime.block_on(async {
        iced_runtime::task::into_stream(close(&mut state, window))
            .unwrap()
            .collect::<Vec<_>>()
            .await
    });
    assert!(matches!(
        actions.as_slice(),
        [iced_runtime::Action::Output(Message::CloseFlushed(
            _,
            Err(_)
        ))]
    ));
}

#[test]
fn switching_books_immediately_preserves_the_previous_books_last_comment() {
    let mut state = library_reader();
    let alpha = state.library[0].path.clone();
    let beta = state.library[1].path.clone();
    finish(update(&mut state, Message::OpenLibraryBook(alpha.clone())));
    let _ = update(
        &mut state,
        Message::TextSelected(vec![SelectedExcerpt {
            chapter_index: 0,
            text: "leitura".into(),
            start_offset: 10,
            end_offset: 17,
        }]),
    );
    let _ = update(
        &mut state,
        Message::AnnotationDraftChanged("Últimos caracteres sem esperar o debounce".into()),
    );
    finish(update(&mut state, Message::OpenLibraryBook(beta.clone())));
    assert_eq!(
        load_annotations(&state.data_dir, &alpha)[0].text,
        "Últimos caracteres sem esperar o debounce"
    );
    assert!(load_annotations(&state.data_dir, &beta).is_empty());
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn closing_from_library_flushes_ratings_favorites_and_book_comments_without_a_reader() {
    let mut state = library_reader();
    state.page = Page::Library;
    state.book_path = None;
    let path = state.library[0].path.clone();
    let _ = book_details::rate(&mut state, path.clone(), 5);
    let _ = book_details::toggle_favorite(&mut state, path.clone());
    let _ = book_details::open(&mut state, path.clone());
    let _ = book_details::edit(
        &mut state,
        iced::widget::text_editor::Action::Edit(iced::widget::text_editor::Edit::Paste(Arc::new(
            "Comentário final".into(),
        ))),
    );
    finish(close(&mut state, iced::window::Id::unique()));
    let reviews = book_details::load(&state.data_dir);
    assert_eq!(reviews[&path].rating, 5);
    assert!(reviews[&path].favorite);
    assert_eq!(reviews[&path].comment, "Comentário final");
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn returning_to_a_book_before_its_async_write_finishes_reads_the_pending_snapshot() {
    let mut state = library_reader();
    let alpha = state.library[0].path.clone();
    let beta = state.library[1].path.clone();
    let _ = update(&mut state, Message::OpenLibraryBook(alpha.clone()));
    let _ = update(
        &mut state,
        Message::TextSelected(vec![SelectedExcerpt {
            chapter_index: 0,
            text: "leitura".into(),
            start_offset: 10,
            end_offset: 17,
        }]),
    );
    let _ = update(
        &mut state,
        Message::AnnotationDraftChanged("Ainda na fila".into()),
    );
    let _ = update(&mut state, Message::OpenLibraryBook(beta));
    let _ = update(&mut state, Message::OpenLibraryBook(alpha.clone()));
    assert_eq!(state.annotations[0].text, "Ainda na fila");
    finish(close(&mut state, iced::window::Id::unique()));
    assert_eq!(
        load_annotations(&state.data_dir, &alpha)[0].text,
        "Ainda na fila"
    );
    std::fs::remove_dir_all(state.data_dir).unwrap();
}

#[test]
fn an_earlier_write_failure_cannot_unlock_the_closing_snapshot() {
    let mut state = library_reader();
    let window = iced::window::Id::unique();
    let pending_close = close(&mut state, window);
    let _ = update(
        &mut state,
        Message::PreferencesSaved(Err("old write".into())),
    );
    assert!(state.closing);
    let previous = state.search.clone();
    let _ = update(
        &mut state,
        Message::SearchChanged("ignored during close".into()),
    );
    assert_eq!(state.search, previous);
    finish(pending_close);
    let _ = update(
        &mut state,
        Message::CloseFlushed(window, Err("disk full".into())),
    );
    assert!(
        !state.closing,
        "failed final flush keeps the window editable for retry"
    );
    std::fs::remove_dir_all(state.data_dir).unwrap();
}
