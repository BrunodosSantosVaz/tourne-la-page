use super::*;

impl State {
    pub(super) fn estimate(block: &Block, book: &Book, format: Format) -> f32 {
        if matches!(block.kind, Kind::Cover) {
            return 376.0;
        }
        let size = format.size * block.scale();
        let lines =
            (block.text(book).len().max(1) as f32 * size * 0.5 / format.width.max(size)).ceil();
        lines.max(1.0) * size * format.line + GAP
    }
    pub(super) fn configure(&mut self, reader: &Reader, book: &Book, viewport: Size) {
        if self.loading != reader.book_loading {
            self.loading = reader.book_loading;
            self.dirty = true;
        }
        let key = reader
            .book_path
            .as_deref()
            .unwrap_or_else(|| std::path::Path::new(""));
        let cover = reader.settings.show_images && reader.book_cover.is_some();
        let format = Format {
            width: ((viewport.width - 24.0).max(1.0) * reader.settings.width / 100.0).max(1.0),
            size: super::reading_font_px(reader, 20.0),
            line: reader.settings.line_height,
            font: Font::with_name(reader.font_catalog.iced_name(&reader.selected_font)),
            align: match reader.settings.alignment {
                super::AlignmentChoice::Left => text::Alignment::Left,
                super::AlignmentChoice::Center => text::Alignment::Center,
                super::AlignmentChoice::Right => text::Alignment::Right,
                super::AlignmentChoice::Justify => text::Alignment::Justified,
            },
        };
        let anchor = self.heights.find(self.y);
        let within = self.y - self.heights.prefix(anchor);
        let anchor_source = self.blocks.get(anchor).cloned();
        let same_book = self.key == key;
        let reset = self.key != key || self.cover != cover || self.chapters > book.chapters.len();
        if reset || self.blocks.is_empty() {
            self.key = key.to_owned();
            self.cover = cover;
            self.chapters = 0;
            self.blocks = vec![
                Block {
                    chapter: 0,
                    kind: Kind::BookTitle,
                },
                Block {
                    chapter: 0,
                    kind: Kind::Caption(book.metadata.authors.join(", ")),
                },
            ];
            if cover {
                self.blocks.push(Block {
                    chapter: 0,
                    kind: Kind::Cover,
                });
            }
            self.paragraphs.clear();
            self.selection = None;
            self.y = 0.0;
        }
        let appended = self.chapters != book.chapters.len();
        for chapter in self.chapters..book.chapters.len() {
            self.blocks.push(Block {
                chapter,
                kind: Kind::ChapterTitle,
            });
            if let Some(ranges) = reader.chapter_paragraph_ranges.get(chapter) {
                for (paragraph, &(start, end)) in ranges.iter().enumerate() {
                    // Bound shaping even for malformed EPUBs with megabyte-long paragraphs.
                    let source = &book.chapters[chapter].text;
                    let mut from = start;
                    while from < end {
                        let mut to = (from + 4096).min(end);
                        while !source.is_char_boundary(to) {
                            to -= 1;
                        }
                        if to < end
                            && let Some(space) = source[from..to].rfind(char::is_whitespace)
                            && space > 2048
                        {
                            to = from
                                + space
                                + source[from + space..].chars().next().unwrap().len_utf8();
                        }
                        self.blocks.push(Block {
                            chapter,
                            kind: Kind::Text {
                                start: from,
                                end: to,
                                paragraph,
                            },
                        });
                        from = to;
                    }
                }
            }
        }
        self.chapters = book.chapters.len();
        if reset
            || appended
            || self.format != Some(format)
            || self.heights.values.len() != self.blocks.len()
        {
            let same_format = self.format == Some(format) && !reset;
            if same_format {
                for block in &self.blocks[self.heights.values.len()..] {
                    self.heights.push(Self::estimate(block, book, format));
                }
            } else {
                self.heights = Heights::new(
                    self.blocks
                        .iter()
                        .map(|b| Self::estimate(b, book, format))
                        .collect(),
                );
            }
            if !same_format {
                self.paragraphs.clear();
            }
            if !reset {
                self.y = self.heights.prefix(anchor) + within;
            } else if same_book && let Some(source) = anchor_source {
                let restored = self.blocks.iter().position(|block| {
                    block.chapter == source.chapter
                        && match (&block.kind, &source.kind) {
                            (Kind::Text { start: a, .. }, Kind::Text { start: b, .. }) => a == b,
                            (Kind::ChapterTitle, Kind::ChapterTitle) => true,
                            _ => false,
                        }
                });
                if let Some(index) = restored {
                    self.y = self.heights.prefix(index) + within;
                }
            }
            self.dirty = true;
        }
        self.format = Some(format);
        self.viewport = viewport;
        if self.navigation != reader.navigation_revision {
            self.navigation = reader.navigation_revision;
            let (chapter, byte) = reader.navigation_target;
            if let Some(index) = self.blocks.iter().position(|block| {
                block.chapter == chapter
                    && match block.kind {
                        Kind::ChapterTitle => byte == 0,
                        Kind::Text { end, .. } => byte < end,
                        _ => false,
                    }
            }) {
                self.y = self.heights.prefix(index);
                self.dirty = true;
            }
        }
        self.prepare(book);
    }
}
