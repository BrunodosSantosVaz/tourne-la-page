use super::*;

impl State {
    pub(super) fn prepare(&mut self, book: &Book) {
        let started = Instant::now();
        let Some(format) = self.format else {
            return;
        };
        let anchor = self.heights.find(self.y);
        let mut within = self.y - self.heights.prefix(anchor);
        let top = (self.y - self.viewport.height * 0.5).max(0.0);
        let first = self.heights.find(top);
        let mut i = first;
        // Geometry is measured once, retained while visible, and reused for every wheel event.
        while i < self.blocks.len() && self.heights.prefix(i) < self.y + self.viewport.height * 2.0
        {
            let block = &self.blocks[i];
            if !matches!(block.kind, Kind::Cover) && !self.paragraphs.contains_key(&i) {
                let paragraph = Paragraph::with_text(text::Text {
                    content: block.text(book),
                    bounds: Size::new(format.width, f32::INFINITY),
                    size: (format.size * block.scale()).into(),
                    line_height: text::LineHeight::Relative(format.line),
                    font: format.font,
                    align_x: format.align,
                    align_y: iced::alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::WordOrGlyph,
                });
                let height = paragraph.min_height().max(format.size) + GAP;
                let delta = height - self.heights.values[i];
                self.heights.set(i, height);
                if i < anchor {
                    self.y += delta;
                }
                if i == anchor {
                    within = within.min((height - 1.0).max(0.0));
                }
                self.paragraphs.insert(i, paragraph);
                self.shaped += 1;
            }
            i += 1;
        }
        self.paragraphs
            .retain(|index, _| *index >= first && *index < i);
        self.y = (self.heights.prefix(anchor) + within).clamp(0.0, self.max());
        let elapsed = started.elapsed().as_micros();
        self.work_us += elapsed;
        self.worst_us = self.worst_us.max(elapsed);
        self.frames += 1;
        if self.profile && self.frames >= 120 {
            eprintln!(
                "reader_perf samples={} prepare_avg_us={} prepare_max_us={} shaped={} cached={} blocks={}",
                self.frames,
                self.work_us / self.frames as u128,
                self.worst_us,
                self.shaped,
                self.paragraphs.len(),
                self.blocks.len()
            );
            self.frames = 0;
            self.work_us = 0;
            self.worst_us = 0;
            self.shaped = 0;
        }
    }
    pub(super) fn report(&mut self, shell: &mut Shell<'_, Message>) {
        if self.dirty {
            let now = Instant::now();
            if now.duration_since(self.last_report) >= REPORT_INTERVAL {
                let block = self.heights.find(self.y);
                let chapter = self.blocks.get(block).map_or(0, |b| b.chapter);
                shell.publish(Message::SurfacePosition(
                    chapter,
                    self.y,
                    self.max(),
                    self.viewport.height,
                ));
                self.dirty = false;
                self.last_report = now;
            } else {
                shell.request_redraw_at(self.last_report + REPORT_INTERVAL);
            }
        }
    }
}
