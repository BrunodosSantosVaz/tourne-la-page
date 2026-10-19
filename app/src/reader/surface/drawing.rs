use super::*;

impl Surface<'_> {
    pub(super) fn paint(&self, state: &State, renderer: &mut iced::Renderer, bounds: Rectangle) {
        let foreground = foreground_for(self.reader);
        let x = state.text_x(bounds);
        let first = state.heights.find(state.y);
        let selection = state
            .selection
            .map(|(a, b)| if a <= b { (a, b) } else { (b, a) });
        renderer.with_layer(bounds, |renderer| {
            for i in first..state.blocks.len() {
                let y = bounds.y + state.heights.prefix(i) - state.y;
                if y > bounds.y + bounds.height {
                    break;
                }
                let block = &state.blocks[i];
                if matches!(block.kind, Kind::Cover) {
                    if let Some(handle) = &self.reader.book_cover
                        && let Some(dimensions) = renderer.measure_image(handle)
                    {
                        let ratio = dimensions.width as f32 / (dimensions.height.max(1) as f32);
                        let height = 360.0_f32.min(state.format.unwrap().width / ratio);
                        renderer.draw_image(
                            image::Image {
                                handle: handle.clone(),
                                border_radius: 0.0.into(),
                                filter_method: iced::widget::image::FilterMethod::Linear,
                                rotation: iced::Radians(0.0),
                                opacity: 1.0,
                                snap: true,
                            },
                            Rectangle::new(Point::new(x, y), Size::new(height * ratio, height)),
                            bounds,
                        );
                    }
                    continue;
                }
                let Some(paragraph) = state.paragraphs.get(&i) else {
                    continue;
                };
                let origin = state.paragraph_origin(i, bounds);
                if let Kind::Text { start, end, .. } = block.kind {
                    for mark in self.reader.annotations.iter().filter(|mark| {
                        mark.chapter_index == block.chapter
                            && matches!(
                                mark.kind,
                                AnnotationKind::Highlight | AnnotationKind::Comment
                            )
                            && mark.start_offset < end
                            && mark.end_offset > start
                    }) {
                        paint_range(
                            renderer,
                            paragraph,
                            mark.start_offset.saturating_sub(start),
                            mark.end_offset.min(end) - start,
                            origin,
                            super::mix(
                                hex_color(&mark.marker_color),
                                super::background_for(self.reader),
                                0.25,
                            ),
                        );
                    }
                    if state.selecting
                        && let Some((a, b)) =
                            selection.filter(|(a, b)| a.block <= i && i <= b.block)
                    {
                        let from = if a.block == i { a.byte } else { 0 };
                        let to = if b.block == i { b.byte } else { end - start };
                        paint_range(
                            renderer,
                            paragraph,
                            from,
                            to,
                            origin,
                            super::mix(
                                hex_color(&selected_marker_color(self.reader)),
                                super::background_for(self.reader),
                                0.25,
                            ),
                        );
                    }
                }
                renderer.fill_paragraph(paragraph, origin, foreground, bounds);
            }
            if state.max() > 0.0 {
                let (height, top) = super::adaptive_scroll_thumb(bounds.height, state.progress());
                let accent = super::theme_definition_for(self.reader)
                    .map(|t| hex_color(&t.accent))
                    .unwrap_or(super::ACCENT);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + bounds.width - 12.0, bounds.y),
                            Size::new(10.0, bounds.height),
                        ),
                        border: iced::Border::default().rounded(6.0),
                        ..Default::default()
                    },
                    Color { a: 0.12, ..accent },
                );
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + bounds.width - 12.0, bounds.y + top),
                            Size::new(10.0, height),
                        ),
                        border: iced::Border::default().rounded(6.0),
                        ..Default::default()
                    },
                    accent,
                );
            }
        });
    }
}

fn paint_range(
    renderer: &mut iced::Renderer,
    paragraph: &Paragraph,
    start: usize,
    end: usize,
    origin: Point,
    color: Color,
) {
    if start >= end {
        return;
    }
    for run in paragraph.buffer().layout_runs() {
        let mut left = f32::INFINITY;
        let mut right = f32::NEG_INFINITY;
        for glyph in run.glyphs.iter().filter(|g| g.start < end && g.end > start) {
            left = left.min(glyph.x);
            right = right.max(glyph.x + glyph.w);
        }
        if left < right {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(origin.x + left, origin.y + run.line_top),
                        Size::new(right - left, run.line_height),
                    ),
                    ..Default::default()
                },
                color,
            );
        }
    }
}
