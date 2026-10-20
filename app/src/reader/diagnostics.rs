//! Exercise actual Iced widget events and the CPU renderer without touching a user's library.
use super::*;
use iced::advanced::Layout;
use iced::advanced::renderer::{Headless as _, Renderer as _};
use iced::advanced::{layout, mouse, widget};
use iced::{Event, Point, Rectangle, Size, Vector};
use std::time::Instant;

pub(super) fn reader(paragraphs: usize) -> Reader {
    static FONTS: std::sync::Once = std::sync::Once::new();
    FONTS.call_once(|| {
        let mut fonts = iced::advanced::graphics::text::font_system()
            .write()
            .unwrap();
        for font in embedded_fonts() {
            fonts.load_font(font);
        }
    });
    let dir = std::env::temp_dir().join(format!(
        "tournepage-native-test-{}-{}",
        std::process::id(),
        new_local_id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let (mut reader, _) = Reader::new(dir);
    let text=(0..paragraphs).map(|index|format!("Trecho {index}: uma leitura natural atravessa frases e parágrafos. A seleção mantém palavras repetidas, acentos e espaços. O texto continua sem interrupções e permite ler com tranquilidade.")).collect::<Vec<_>>().join("\n\n");
    let book = Book {
        metadata: BookMetadata {
            title: "Leitura contínua — ensaio local".into(),
            authors: vec!["Tournepage".into()],
            ..Default::default()
        },
        chapters: vec![Chapter {
            title: "Capítulo de teste".into(),
            text,
        }],
        ..Default::default()
    };
    reader.chapter_paragraph_ranges = book.chapters.iter().map(chapter_paragraph_ranges).collect();
    reader.book = Some(Arc::new(book));
    reader.book_path = Some(PathBuf::from("/tmp/tournepage-diagnostic.epub"));
    reader.page = Page::Reader;
    reader.sidebar_open = false;
    reader.settings.width = 100.0;
    reader.selected_font = "Literata".into();
    reader
}

pub(super) fn renderer() -> iced::Renderer {
    iced::futures::executor::block_on(iced::Renderer::new(
        Font::with_name("Inter"),
        14.0.into(),
        Some("tiny-skia"),
    ))
    .unwrap()
}

pub(super) fn send(
    element: &mut Element<'_, Message>,
    tree: &mut widget::Tree,
    node: &layout::Node,
    renderer: &iced::Renderer,
    event: Event,
    point: Point,
) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut shell = iced::advanced::Shell::new(&mut messages);
    element.as_widget_mut().update(
        tree,
        &event,
        Layout::new(node),
        mouse::Cursor::Available(point),
        renderer,
        &mut iced::advanced::clipboard::Null,
        &mut shell,
        &Rectangle::with_size(node.size()),
    );
    messages
}

pub(super) fn key(named: iced::keyboard::key::Named) -> Event {
    use iced::keyboard::{self, key};
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(named),
        modified_key: keyboard::Key::Named(named),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: None,
        repeat: false,
    })
}

#[test]
fn annotations_are_created_once_and_comments_are_optional() {
    let mut reader = reader(10);
    let selection = SelectedExcerpt {
        chapter_index: 0,
        text: "uma leitura natural".into(),
        start_offset: 10,
        end_offset: 28,
    };
    let _ = update(&mut reader, Message::TextSelected(vec![selection]));
    assert_eq!(reader.annotations.len(), 1);
    assert!(reader.annotations[0].text.is_empty());
    assert!(reader.show_annotations);
    let _ = update(
        &mut reader,
        Message::AnnotationDraftChanged("meu comentário".into()),
    );
    let _ = update(&mut reader, Message::SubmitAnnotationComment);
    assert_eq!(reader.annotations.len(), 1);
    assert_eq!(reader.annotations[0].text, "meu comentário");
    let _ = update(
        &mut reader,
        Message::TextSelected(vec![SelectedExcerpt {
            chapter_index: 0,
            text: "outro trecho".into(),
            start_offset: 100,
            end_offset: 112,
        }]),
    );
    assert_eq!(reader.annotations.len(), 2);
    assert_eq!(reader.annotations[0].text, "meu comentário");
    assert!(reader.annotations[1].text.is_empty());
    std::fs::remove_dir_all(reader.data_dir).unwrap();
}

#[test]
fn resizing_and_collapsing_cannot_leave_an_invisible_menu_open() {
    let mut reader = reader(1);
    for message in [
        Message::WindowResized(iced::window::Id::unique(), Size::new(390.0, 800.0)),
        Message::ToggleToolbarOverflow,
    ] {
        reader.theme_menu_open = true;
        reader.compact_menu = Some(toolbar::Item::Font);
        reader.annotation_palette_open = Some("test".into());
        let _ = update(&mut reader, message);
        assert!(!reader.theme_menu_open && reader.compact_menu.is_none());
        assert!(reader.annotation_palette_open.is_none());
    }
    std::fs::remove_dir_all(reader.data_dir).unwrap();
}

#[test]
fn scroll_speed_steps_cross_the_hundred_percent_boundary_in_both_directions() {
    let mut reader = reader(1);
    reader.settings.scroll_speed_percent = 80;
    for expected in [90, 100, 200, 300] {
        let _ = update(&mut reader, Message::AdjustScrollSpeed(1));
        assert_eq!(reader.settings.scroll_speed_percent, expected);
    }
    for expected in [200, 100, 90, 80] {
        let _ = update(&mut reader, Message::AdjustScrollSpeed(-1));
        assert_eq!(reader.settings.scroll_speed_percent, expected);
    }
    reader.settings.scroll_speed_percent = 10;
    let _ = update(&mut reader, Message::AdjustScrollSpeed(-1));
    assert_eq!(reader.settings.scroll_speed_percent, 10);
    std::fs::remove_dir_all(reader.data_dir).unwrap();
}

/// Save full CPU-rendered screens for visual inspection and report viewport input costs.
#[test]
#[ignore = "manual performance and visual review; writes screenshots under /tmp"]
fn cpu_reader_review() {
    let mut reader = reader(100_000);
    let mut renderer = renderer();
    let book = Arc::clone(reader.book.as_ref().unwrap());
    let mut element = surface::view(&reader, &book);
    let mut tree = widget::Tree::new(&element);
    let size = Size::new(1280.0, 800.0);
    let node =
        element
            .as_widget_mut()
            .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
    let point = Point::new(600.0, 400.0);
    let mut samples = Vec::new();
    let mut raster_samples = Vec::new();
    for frame in 0..720 {
        let start = Instant::now();
        let _ = send(
            &mut element,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -24.0 },
            }),
            point,
        );
        renderer.reset(Rectangle::with_size(size));
        element.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Light,
            &iced::advanced::renderer::Style {
                text_color: Color::BLACK,
            },
            Layout::new(&node),
            mouse::Cursor::Available(point),
            &Rectangle::with_size(size),
        );
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
        if frame % 36 == 0 {
            let raster = Instant::now();
            let _ = renderer.screenshot(Size::new(1280, 800), 1.0, Color::WHITE);
            raster_samples.push(raster.elapsed().as_secs_f64() * 1000.0);
        }
    }
    samples.sort_by(f64::total_cmp);
    eprintln!(
        "CPU viewport, 100000 paragraphs / 720 wheel events: p50={:.3}ms p95={:.3}ms max={:.3}ms (event + shaping + draw commands, raster separately)",
        samples[360], samples[684], samples[719]
    );
    raster_samples.sort_by(f64::total_cmp);
    eprintln!(
        "CPU raster + RGBA readback: p50={:.3}ms p95={:.3}ms",
        raster_samples[10], raster_samples[19]
    );
    drop(element);
    review_full_page(&mut reader, &mut renderer);
    reader.font_catalog = FontCatalog::system();
    for (width, expanded, menu) in [
        (1920, false, 0),
        (1280, false, 0),
        (800, false, 0),
        (626, false, 0),
        (390, true, 0),
        (284, false, 0),
        (284, true, 0),
        (1280, true, 0),
        (1280, true, 1),
        (1280, true, 3),
        (626, false, 4),
        (626, false, 5),
        (626, false, 6),
        (626, false, 7),
        (626, false, 8),
        (626, false, 9),
        (626, false, 10),
        (626, false, 11),
        (1280, false, 12),
        (390, false, 12),
        (1280, false, 13),
        (390, false, 13),
    ] {
        reader.show_annotations = menu >= 12;
        if menu >= 12 {
            select_theme(&mut reader, "dracula");
            let colors = default_marker_palette();
            reader.annotations = (0..40)
                .map(|i| BookAnnotation {
                    id: format!("review-note-{i}"),
                    chapter_index: 0,
                    kind: AnnotationKind::Highlight,
                    text: format!("Comentário {i}: uma ideia para guardar e retomar depois."),
                    excerpt: "Uma leitura natural atravessa frases e parágrafos.".into(),
                    start_offset: i * 100,
                    end_offset: i * 100 + 20,
                    marker_color: colors[i % colors.len()].clone(),
                })
                .collect();
            if menu == 13 {
                reader.annotations[0].text = format!(
                    "{}\n{}",
                    "Um comentário que deve caber no post-it e crescer ao editar. ".repeat(4),
                    "https://exemplo.com/umcaminhocommuitasletrassemespacoparatestar"
                );
            }
            reader.active_mark_id = Some("review-note-0".into());
            reader.annotation_draft = reader.annotations[0].text.clone();
            reader.annotation_editor =
                iced::widget::text_editor::Content::with_text(&reader.annotation_draft);
        }
        reader.window_width = width as f32;
        reader.toolbar_overflow_open = expanded;
        reader.theme_menu_open = false;
        reader.compact_menu = match menu {
            1 | 8 => Some(toolbar::Item::Theme),
            2 | 4 => Some(toolbar::Item::Marker),
            3 | 9 => Some(toolbar::Item::Font),
            5 => Some(toolbar::Item::Width),
            6 => Some(toolbar::Item::Images),
            7 => Some(toolbar::Item::Size),
            10 => Some(toolbar::Item::LineHeight),
            11 => Some(toolbar::Item::Alignment),
            _ => None,
        };
        let size = Size::new(width as f32, 900.0);
        let mut page = view(&reader);
        let mut tree = widget::Tree::new(&page);
        let node =
            page.as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let _ = send(
            &mut page,
            &mut tree,
            &node,
            &renderer,
            Event::Window(iced::window::Event::RedrawRequested(Instant::now())),
            Point::ORIGIN,
        );
        renderer.reset(Rectangle::with_size(size));
        page.as_widget().draw(
            &tree,
            &mut renderer,
            &theme_for(&reader),
            &iced::advanced::renderer::Style {
                text_color: foreground_for(&reader),
            },
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &Rectangle::with_size(size),
        );
        if let Some(mut popup) = page.as_widget_mut().overlay(
            &mut tree,
            Layout::new(&node),
            &renderer,
            &Rectangle::with_size(size),
            Vector::ZERO,
        ) {
            let overlay_node = popup.as_overlay_mut().layout(&renderer, size);
            let mut messages = Vec::new();
            let mut shell = iced::advanced::Shell::new(&mut messages);
            popup.as_overlay_mut().update(
                &Event::Window(iced::window::Event::RedrawRequested(Instant::now())),
                Layout::new(&overlay_node),
                mouse::Cursor::Unavailable,
                &renderer,
                &mut iced::advanced::clipboard::Null,
                &mut shell,
            );
            renderer.with_layer(Rectangle::with_size(size), |renderer| {
                popup.as_overlay().draw(
                    renderer,
                    &theme_for(&reader),
                    &iced::advanced::renderer::Style {
                        text_color: foreground_for(&reader),
                    },
                    Layout::new(&overlay_node),
                    mouse::Cursor::Unavailable,
                );
            });
        }
        let pixels = renderer.screenshot(Size::new(width, 900), 1.0, background_for(&reader));
        image::save_buffer(
            format!("/tmp/tournepage-review-{width}-{expanded}-{menu}.png"),
            &pixels,
            width,
            900,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }
    std::fs::remove_dir_all(reader.data_dir).unwrap();
}

fn review_full_page(reader: &mut Reader, renderer: &mut iced::Renderer) {
    let mut tree = widget::Tree::new(view(reader));
    let size = Size::new(1280.0, 800.0);
    let mut samples = Vec::new();
    for _ in 0..180 {
        let start = Instant::now();
        let mut page = view(reader);
        tree.diff(&page);
        let node =
            page.as_widget_mut()
                .layout(&mut tree, renderer, &layout::Limits::new(size, size));
        let messages = send(
            &mut page,
            &mut tree,
            &node,
            renderer,
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -24.0 },
            }),
            Point::new(600.0, 400.0),
        );
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
        drop(page);
        for message in messages {
            let _ = update(reader, message);
        }
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(f64::total_cmp);
    eprintln!(
        "Full reader UI, including view rebuild + layout + wheel + draw commands, 180 events: p50={:.3}ms p95={:.3}ms max={:.3}ms (raster excluded)",
        samples[90], samples[171], samples[179]
    );
    reader.progress_percent = 0.0;
}

#[derive(Default)]
pub(super) struct Geometry {
    pub bounds: HashMap<widget::Id, Rectangle>,
    pub scrolls: HashMap<widget::Id, (Rectangle, Rectangle)>,
}

impl widget::Operation for Geometry {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation)) {
        operate(self);
    }
    fn container(&mut self, id: Option<&widget::Id>, bounds: Rectangle) {
        if let Some(id) = id {
            self.bounds.insert(id.clone(), bounds);
        }
    }
    fn focusable(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        _: &mut dyn widget::operation::Focusable,
    ) {
        self.container(id, bounds);
    }
    fn scrollable(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        _: Vector,
        _: &mut dyn widget::operation::Scrollable,
    ) {
        if let Some(id) = id {
            self.scrolls.insert(id.clone(), (bounds, content));
        }
    }
}

pub(super) fn geometry(
    element: &mut Element<'_, Message>,
    tree: &mut widget::Tree,
    node: &layout::Node,
    renderer: &iced::Renderer,
) -> Geometry {
    let mut geometry = Geometry::default();
    element
        .as_widget_mut()
        .operate(tree, Layout::new(node), renderer, &mut geometry);
    geometry
}
