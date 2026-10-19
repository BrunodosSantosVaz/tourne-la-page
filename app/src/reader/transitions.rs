//! Keep Iced's existing widget tree when opening a real EPUB and returning to the library.
use super::diagnostics::{reader, renderer};
use super::*;
use iced::advanced::renderer::Renderer as _;
use iced::advanced::{Layout, layout, widget};
use iced::{Rectangle, Size};

#[test]
fn opening_epub_from_the_library_preserves_a_valid_widget_tree() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/amostra-tournepage.epub");
    let book = epub_core::parse_epub(std::fs::File::open(&path).unwrap()).unwrap();
    let mut reader = reader(1);
    let mut renderer = renderer();
    reader.page = Page::Library;
    reader.sidebar_open = true;
    let mut tree = widget::Tree::new(view(&reader));
    for width in [1280.0, 626.0, 246.0, 1920.0] {
        reader.window_width = width;
        frame(&reader, &mut tree, &mut renderer, width);
        let _ = update(
            &mut reader,
            Message::BookStreamStarted(path.clone(), book.metadata.clone(), book.chapters.len()),
        );
        frame(&reader, &mut tree, &mut renderer, width);
        for (index, chapter) in book.chapters.iter().cloned().enumerate() {
            let _ = update(
                &mut reader,
                Message::BookChapterLoaded(path.clone(), index, chapter),
            );
            frame(&reader, &mut tree, &mut renderer, width);
        }
        let _ = update(&mut reader, Message::BookStreamFinished(path.clone()));
        frame(&reader, &mut tree, &mut renderer, width);
        assert_eq!(reader.page, Page::Reader);
        assert_eq!(
            reader.book.as_ref().unwrap().chapters.len(),
            book.chapters.len()
        );
        let _ = update(&mut reader, Message::ReturnToLibrary);
        frame(&reader, &mut tree, &mut renderer, width);
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

fn frame(reader: &Reader, tree: &mut widget::Tree, renderer: &mut iced::Renderer, width: f32) {
    let size = Size::new(width, 900.0);
    let mut page = view(reader);
    tree.diff(&page);
    let node = page
        .as_widget_mut()
        .layout(tree, renderer, &layout::Limits::new(size, size));
    renderer.reset(Rectangle::with_size(size));
    page.as_widget().draw(
        tree,
        renderer,
        &theme_for(reader),
        &iced::advanced::renderer::Style {
            text_color: foreground_for(reader),
        },
        Layout::new(&node),
        iced::mouse::Cursor::Unavailable,
        &Rectangle::with_size(size),
    );
}
