//! A bounded, retained text viewport. Mouse motion never rebuilds application widgets.
mod cache;
#[path = "surface/layout.rs"]
mod document_layout;
mod drawing;
mod heights;
mod input;
mod selection;
#[cfg(test)]
mod tests;
mod widget_impl;
#[cfg(test)]
use input::scrollbar_progress;
use selection::word_range;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::advanced::image::Renderer as _;
use iced::advanced::text::{Paragraph as _, Renderer as _};
use iced::advanced::widget::operation::{self, Scrollable};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, image, layout, mouse, renderer, text, widget,
};
use iced::{
    Color, Element, Event, Font, Length, Point, Rectangle, Size, Theme, Vector, keyboard, window,
};
use renderer::Renderer as _;

use super::{
    ACCENT, AlignmentChoice, AnnotationKind, Message, Reader, SelectedExcerpt,
    adaptive_scroll_thumb, background_for, foreground_for, hex_color, mix, reading_font_px,
    selected_marker_color, theme_definition_for,
};
use epub_core::Book;

type Paragraph = <iced::Renderer as text::Renderer>::Paragraph;
const GAP: f32 = 14.0;
const REPORT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone)]
enum Kind {
    BookTitle,
    Caption(String),
    Cover,
    ChapterTitle,
    Text {
        start: usize,
        end: usize,
        paragraph: usize,
    },
}

#[derive(Clone)]
struct Block {
    chapter: usize,
    kind: Kind,
}

impl Block {
    fn text<'a>(&'a self, book: &'a Book) -> &'a str {
        match &self.kind {
            Kind::BookTitle => &book.metadata.title,
            Kind::Caption(text) => text,
            Kind::ChapterTitle => &book.chapters[self.chapter].title,
            Kind::Text { start, end, .. } => &book.chapters[self.chapter].text[*start..*end],
            Kind::Cover => "",
        }
    }
    fn scale(&self) -> f32 {
        match self.kind {
            Kind::BookTitle => 1.6,
            Kind::ChapterTitle => 1.3,
            Kind::Caption(_) => 0.8,
            _ => 1.0,
        }
    }
}

/// Prefix sums with logarithmic updates: measured paragraphs never require a book-wide rescan.
#[derive(Default)]
struct Heights {
    values: Vec<f32>,
    tree: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Endpoint {
    block: usize,
    byte: usize,
}

#[derive(Clone, Copy, PartialEq)]
struct Format {
    width: f32,
    size: f32,
    line: f32,
    font: Font,
    align: text::Alignment,
}

struct State {
    key: PathBuf,
    chapters: usize,
    loading: bool,
    cover: bool,
    blocks: Vec<Block>,
    heights: Heights,
    paragraphs: HashMap<usize, Paragraph>,
    format: Option<Format>,
    viewport: Size,
    y: f32,
    selection: Option<(Endpoint, Endpoint)>,
    selecting: bool,
    scrollbar_grab: Option<f32>,
    pointer: Point,
    focused: bool,
    click: Option<(Instant, Point, u8)>,
    navigation: u64,
    dirty: bool,
    last_report: Instant,
    last_frame: Option<Instant>,
    profile: bool,
    frames: usize,
    work_us: u128,
    worst_us: u128,
    shaped: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            key: PathBuf::new(),
            chapters: 0,
            loading: false,
            cover: false,
            blocks: Vec::new(),
            heights: Heights::default(),
            paragraphs: HashMap::new(),
            format: None,
            viewport: Size::ZERO,
            y: 0.0,
            selection: None,
            selecting: false,
            scrollbar_grab: None,
            pointer: Point::ORIGIN,
            focused: false,
            click: None,
            navigation: 0,
            dirty: true,
            last_report: Instant::now() - REPORT_INTERVAL,
            last_frame: None,
            profile: std::env::var_os("TOURNEPAGE_PROFILE").is_some(),
            frames: 0,
            work_us: 0,
            worst_us: 0,
            shaped: 0,
        }
    }
}

impl State {
    pub(super) fn max(&self) -> f32 {
        (self.heights.total() - self.viewport.height + GAP).max(0.0)
    }
    pub(super) fn progress(&self) -> f32 {
        if self.max() > 0.0 {
            self.y / self.max() * 100.0
        } else {
            0.0
        }
    }
    pub(super) fn move_to(&mut self, y: f32) {
        let next = y.clamp(0.0, self.max());
        if (next - self.y).abs() > 0.01 {
            self.y = next;
            self.dirty = true;
        }
    }
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl Scrollable for State {
    fn snap_to(&mut self, offset: operation::scrollable::RelativeOffset<Option<f32>>) {
        if let Some(y) = offset.y {
            self.move_to(self.max() * y);
        }
    }
    fn scroll_to(&mut self, offset: operation::scrollable::AbsoluteOffset<Option<f32>>) {
        if let Some(y) = offset.y {
            self.move_to(y);
        }
    }
    fn scroll_by(
        &mut self,
        offset: operation::scrollable::AbsoluteOffset,
        _: Rectangle,
        _: Rectangle,
    ) {
        self.move_to(self.y + offset.y);
    }
}

struct Surface<'a> {
    reader: &'a Reader,
    book: &'a Book,
}

pub(super) fn view<'a>(reader: &'a Reader, book: &'a Book) -> Element<'a, Message> {
    Element::new(Surface { reader, book })
}
