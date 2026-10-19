use super::super::diagnostics::{reader, renderer, send};
use super::*;
use iced::advanced::renderer::{Headless as _, Renderer as _};
use iced::advanced::{Layout, layout, mouse, widget};
use iced::{Event, Point, Rectangle, Size};

#[test]
#[ignore = "manual CPU screenshots of the library; optional TOURNE_LA_PAGE_REVIEW_BOOKS directory"]
fn library_visual_review() {
    let mut reader = reader(1);
    let mut renderer = renderer();
    reader.page = Page::Library;
    let folder = std::env::var_os("TOURNE_LA_PAGE_REVIEW_BOOKS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples"));
    reader.library = library::scan_library(&[folder]).0;
    for (index, item) in reader.library.iter().enumerate() {
        let progress = reader.reading_history.entry(item.path.clone()).or_default();
        progress.percent = (index % 5) as f32 * 25.0;
        progress.last_read = index as u64;
        reader.book_reviews.insert(item.path.clone(), book_details::Review {
            rating: (index % 6) as u8,
            favorite: index % 2 == 0,
            comment: if index == 0 { "Uma leitura marcante, com ideias para revisitar.\nEste comentário pertence ao livro inteiro.".into() } else { String::new() },
        });
    }
    for (width, sidebar, about) in [
        (1920, true, false),
        (1280, true, false),
        (1280, false, false),
        (800, false, false),
        (390, false, false),
        (246, false, false),
        (1280, false, true),
        (390, false, true),
    ] {
        reader.window_width = width as f32;
        reader.sidebar_open = sidebar;
        reader.about_open = about;
        capture(
            &reader,
            &mut renderer,
            width,
            &format!("{width}-{sidebar}-{about}"),
        );
    }
    reader.about_open = false;
    reader.sidebar_open = true;
    reader.window_width = 1280.0;
    for section in [
        LibrarySection::Rating,
        LibrarySection::Favorites,
        LibrarySection::Reading,
    ] {
        reader.section = section;
        capture(&reader, &mut renderer, 1280, &section.to_string());
    }
    reader.section = LibrarySection::AllBooks;
    reader.list_view = true;
    capture(&reader, &mut renderer, 1280, "list");
    if let Some(path) = reader.library.first().map(|item| item.path.clone()) {
        let _ = book_details::open(&mut reader, path);
        for width in [1280, 390, 284] {
            reader.window_width = width as f32;
            capture(&reader, &mut renderer, width, &format!("details-{width}"));
        }
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

fn capture(reader: &Reader, renderer: &mut iced::Renderer, width: u32, name: &str) {
    let size = Size::new(width as f32, 960.0);
    let mut page = super::super::view(reader);
    let mut tree = widget::Tree::new(&page);
    let node = page
        .as_widget_mut()
        .layout(&mut tree, renderer, &layout::Limits::new(size, size));
    let _ = send(
        &mut page,
        &mut tree,
        &node,
        renderer,
        Event::Window(iced::window::Event::RedrawRequested(
            std::time::Instant::now(),
        )),
        Point::ORIGIN,
    );
    let measured = super::super::diagnostics::geometry(&mut page, &mut tree, &node, renderer);
    for item in &reader.library {
        let Some(open) = measured
            .bounds
            .get(&widget::Id::from(format!("book-{}", item.path.display())))
        else {
            continue;
        };
        let Some(progress) = measured.bounds.get(&widget::Id::from(format!(
            "book-progress-{}",
            item.path.display()
        ))) else {
            continue;
        };
        if let Some(card) = measured.bounds.get(&widget::Id::from(format!(
            "book-card-{}",
            item.path.display()
        ))) {
            assert!(
                open.x + open.width <= card.x + card.width + 0.1,
                "{name}: open {open:?} exceeds card {card:?}"
            );
        }
        assert!(
            progress.x + progress.width <= open.x + open.width + 0.1,
            "{name}: progress {progress:?} exceeds book {open:?}"
        );
    }
    renderer.reset(Rectangle::with_size(size));
    page.as_widget().draw(
        &tree,
        renderer,
        &theme_for(reader),
        &iced::advanced::renderer::Style {
            text_color: foreground_for(reader),
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(size),
    );
    let pixels = renderer.screenshot(Size::new(width, 960), 1.0, background_for(reader));
    image::save_buffer(
        format!("/tmp/tourne-la-page-library-{name}.png"),
        &pixels,
        width,
        960,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
