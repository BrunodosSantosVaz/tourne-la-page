use super::super::diagnostics::{reader, renderer};
use super::*;
use iced::advanced::{layout, widget};

#[test]
fn library_grid_fills_its_real_width_and_is_not_capped_at_five_columns() {
    let reader = reader(1);
    let renderer = renderer();
    let item = LibraryItem {
        path: PathBuf::from("/tmp/grid.epub"),
        book: Arc::clone(reader.book.as_ref().unwrap()),
        cover: None,
    };
    let items = vec![&item; 24];
    for width in [220.0, 850.0, 1800.0, 3300.0] {
        let mut grid = grid(&reader, &items);
        let mut tree = widget::Tree::new(&grid);
        let node = grid.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(iced::Size::ZERO, iced::Size::new(width, f32::INFINITY)),
        );
        assert_eq!(node.size().width, width);
        let first_row = node
            .children()
            .iter()
            .take_while(|child| child.bounds().y < 1.0)
            .collect::<Vec<_>>();
        let last = first_row.last().unwrap();
        assert!((last.bounds().x + last.size().width - width).abs() < 1.0);
        if width >= 1800.0 {
            assert!(first_row.len() > 5);
        }
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn about_dialog_is_separate_from_reader_and_can_be_closed() {
    let mut reader = reader(1);
    reader.page = Page::Library;
    let _ = update(&mut reader, Message::OpenAbout);
    assert!(reader.about_open);
    let _ = update(&mut reader, Message::CloseAbout);
    assert!(!reader.about_open);
    reader.page = Page::Reader;
    let _ = update(&mut reader, Message::OpenAbout);
    assert!(!reader.about_open);
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn sidebar_starts_below_the_header_and_does_not_reduce_its_width() {
    let mut reader = reader(1);
    let renderer = renderer();
    reader.page = Page::Library;
    reader.window_width = 1280.0;
    let size = iced::Size::new(1280.0, 900.0);
    let mut header_sizes = Vec::new();
    for open in [false, true] {
        reader.sidebar_open = open;
        let mut page = page(&reader);
        let mut tree = widget::Tree::new(&page);
        let node =
            page.as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let header = &node.children()[0];
        let body = &node.children()[1];
        header_sizes.push(header.size());
        assert_eq!(header.size().width, 1280.0);
        assert!(body.bounds().y >= header.bounds().y + header.size().height);
        let scroll = body.children().last().unwrap();
        assert!((scroll.bounds().x + scroll.size().width - 1280.0).abs() < 0.1);
    }
    assert_eq!(header_sizes[0], header_sizes[1]);
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn about_fits_without_scrolling_and_close_stays_in_its_corner_on_small_windows() {
    use super::super::diagnostics::{geometry, send};
    let mut reader = reader(1);
    let renderer = renderer();
    for (width, height) in [(1280.0, 800.0), (390.0, 800.0), (284.0, 360.0)] {
        reader.window_width = width;
        let mut page = about::view(&reader);
        let mut tree = widget::Tree::new(&page);
        let size = iced::Size::new(width, height);
        let node =
            page.as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let measured = geometry(&mut page, &mut tree, &node, &renderer);
        let card = measured.bounds[&widget::Id::new("about-card")];
        let close = measured.bounds[&widget::Id::new("about-close")];
        let (scroll, content) = measured.scrolls[&widget::Id::new("about-content")];
        assert!((close.x + close.width - (card.x + card.width - 8.0)).abs() < 0.1);
        assert!((close.y - card.y - 8.0).abs() < 0.1);
        assert!(close.y + close.height <= scroll.y);
        assert!(scroll.y + scroll.height <= card.y + card.height + 0.1);
        if width >= 390.0 {
            assert!(
                content.height <= scroll.height + 0.1,
                "{width}: content {content:?}, scroll {scroll:?}"
            );
        }
        let point = iced::Point::new(close.x + 16.0, close.y + 16.0);
        for event in [
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left),
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left),
        ] {
            let messages = send(
                &mut page,
                &mut tree,
                &node,
                &renderer,
                iced::Event::Mouse(event),
                point,
            );
            if matches!(event, iced::mouse::Event::ButtonReleased(_)) {
                assert!(matches!(messages.as_slice(), [Message::CloseAbout]));
            }
        }
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}
