use super::*;

impl State {
    pub(super) fn text_x(&self, bounds: Rectangle) -> f32 {
        bounds.x + (bounds.width - 24.0 - self.format.unwrap().width) / 2.0
    }
    pub(super) fn paragraph_origin(&self, index: usize, bounds: Rectangle) -> Point {
        let paragraph = &self.paragraphs[&index];
        Rectangle::new(
            Point::new(
                self.text_x(bounds),
                bounds.y + self.heights.prefix(index) - self.y,
            ),
            Size::new(self.format.unwrap().width, paragraph.min_height()),
        )
        .anchor(
            paragraph.min_bounds(),
            paragraph.align_x(),
            paragraph.align_y(),
        )
    }
    pub(super) fn endpoint(&self, point: Point, bounds: Rectangle) -> Option<Endpoint> {
        let y = (point.y - bounds.y + self.y).clamp(0.0, self.heights.total());
        let mut index = self.heights.find(y);
        if !matches!(self.blocks.get(index)?.kind, Kind::Text { .. }) {
            index = (index..self.blocks.len())
                .find(|i| matches!(self.blocks[*i].kind, Kind::Text { .. }))?;
        }
        let paragraph = self.paragraphs.get(&index)?;
        let local = Point::new(
            point.x - self.paragraph_origin(index, bounds).x,
            (y - self.heights.prefix(index)).max(0.0),
        );
        let byte = paragraph
            .hit_test(local)
            .map(|hit| hit.cursor())
            .unwrap_or_else(|| {
                if local.y > paragraph.min_height() {
                    self.block_len(index)
                } else {
                    0
                }
            });
        Some(Endpoint {
            block: index,
            byte: byte.min(self.block_len(index)),
        })
    }
    pub(super) fn block_len(&self, i: usize) -> usize {
        match self.blocks[i].kind {
            Kind::Text { start, end, .. } => end - start,
            _ => 0,
        }
    }
    pub(super) fn selected(&self, book: &Book) -> Vec<SelectedExcerpt> {
        let Some((a, b)) = self.selection else {
            return Vec::new();
        };
        if a == b {
            return Vec::new();
        }
        let (first, last) = if a <= b { (a, b) } else { (b, a) };
        let mut ranges: Vec<(usize, usize, usize, usize)> = Vec::new();
        for i in first.block..=last.block {
            if let Kind::Text {
                start,
                end,
                paragraph,
            } = self.blocks[i].kind
            {
                let from = if i == first.block {
                    start + first.byte
                } else {
                    start
                };
                let to = if i == last.block {
                    start + last.byte
                } else {
                    end
                };
                if from >= to {
                    continue;
                }
                if let Some(range) = ranges
                    .last_mut()
                    .filter(|range| range.0 == self.blocks[i].chapter)
                {
                    range.2 = to;
                } else {
                    ranges.push((self.blocks[i].chapter, from, to, paragraph));
                }
            }
        }
        ranges
            .into_iter()
            .filter_map(|(chapter, start, end, _paragraph)| {
                let text = book.chapters[chapter].text.get(start..end)?;
                (!text.trim().is_empty()).then(|| SelectedExcerpt {
                    chapter_index: chapter,
                    text: text.to_owned(),
                    start_offset: start,
                    end_offset: end,
                })
            })
            .collect()
    }
}

pub(super) fn word_range(text: &str, byte: usize) -> (usize, usize) {
    let at = byte.min(text.len());
    let start = text[..at]
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map_or(0, |(i, c)| i + c.len_utf8());
    let end = text[at..]
        .char_indices()
        .find(|(_, c)| c.is_whitespace())
        .map_or(text.len(), |(i, _)| at + i);
    (start, end)
}
