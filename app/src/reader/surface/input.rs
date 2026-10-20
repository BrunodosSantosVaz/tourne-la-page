use super::*;

pub(super) struct Input<'a, 'b> {
    pub(super) event: &'a Event,
    pub(super) bounds: Rectangle,
    pub(super) cursor: mouse::Cursor,
    pub(super) clipboard: &'a mut dyn Clipboard,
    pub(super) shell: &'a mut Shell<'b, Message>,
}
impl State {
    pub(super) fn handle_event(&mut self, surface: &Surface<'_>, input: Input<'_, '_>) {
        let state = self;
        let Input {
            event,
            bounds,
            cursor,
            clipboard,
            shell,
        } = input;
        let old_y = state.y;
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let amount = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => -*y * 48.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -*y,
                };
                state.move_to(state.y + amount);
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.focused = cursor.is_over(bounds);
                if let Some(point) = cursor.position_in(bounds) {
                    state.pointer = Point::new(point.x + bounds.x, point.y + bounds.y);
                    if point.x >= bounds.width - 18.0 {
                        let (height, top) =
                            super::adaptive_scroll_thumb(bounds.height, state.progress());
                        let fraction = if point.y >= top && point.y <= top + height {
                            (point.y - top) / height
                        } else {
                            0.5
                        };
                        state.scrollbar_grab = Some(fraction);
                        let p = scrollbar_progress(bounds.height, point.y, fraction);
                        state.move_to(state.max() * p);
                    } else if let Some(endpoint) = state.endpoint(state.pointer, bounds) {
                        let now = Instant::now();
                        let count = state
                            .click
                            .filter(|(time, pos, _)| {
                                now.duration_since(*time) < Duration::from_millis(400)
                                    && pos.distance(state.pointer) < 5.0
                            })
                            .map_or(1, |(_, _, n)| n % 3 + 1);
                        state.click = Some((now, state.pointer, count));
                        state.selection = Some((endpoint, endpoint));
                        state.selecting = true;
                        if count >= 2 {
                            let source = state.blocks[endpoint.block].text(surface.book);
                            let (start, end) = if count == 3 {
                                (0, source.len())
                            } else {
                                word_range(source, endpoint.byte)
                            };
                            state.selection = Some((
                                Endpoint {
                                    byte: start,
                                    ..endpoint
                                },
                                Endpoint {
                                    byte: end,
                                    ..endpoint
                                },
                            ));
                        }
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                state.pointer = *position;
                if let Some(fraction) = state.scrollbar_grab {
                    let p = scrollbar_progress(bounds.height, position.y - bounds.y, fraction);
                    state.move_to(state.max() * p);
                    shell.capture_event();
                } else if state.selecting {
                    if let Some(endpoint) = state.endpoint(*position, bounds)
                        && let Some((anchor, _)) = state.selection
                    {
                        state.selection = Some((anchor, endpoint));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.selecting {
                    state.selecting = false;
                    let selections = state.selected(surface.book);
                    if !selections.is_empty() {
                        shell.publish(Message::TextSelected(selections));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
                if state.scrollbar_grab.take().is_some() {
                    shell.capture_event();
                    state.dirty = true;
                    state.last_report = Instant::now() - REPORT_INTERVAL;
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if state.focused =>
            {
                use keyboard::key::Named;
                let change = match key.as_ref() {
                    keyboard::Key::Named(Named::PageDown) => Some(state.y + bounds.height * 0.9),
                    keyboard::Key::Named(Named::PageUp) => Some(state.y - bounds.height * 0.9),
                    keyboard::Key::Named(Named::ArrowDown) => Some(state.y + 40.0),
                    keyboard::Key::Named(Named::ArrowUp) => Some(state.y - 40.0),
                    keyboard::Key::Named(Named::Home) => Some(0.0),
                    keyboard::Key::Named(Named::End) => Some(state.max()),
                    _ => None,
                };
                if let Some(y) = change {
                    state.move_to(y);
                    shell.capture_event();
                }
                if modifiers.command()
                    && matches!(key.as_ref(), keyboard::Key::Character("c" | "C"))
                {
                    let text = state
                        .selected(surface.book)
                        .into_iter()
                        .map(|s| s.text)
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    if !text.is_empty() {
                        clipboard.write(iced::advanced::clipboard::Kind::Standard, text);
                        shell.capture_event();
                    }
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let dt = state.last_frame.map_or(0.0, |last| {
                    now.saturating_duration_since(last).as_secs_f32().min(0.05)
                });
                state.last_frame = Some(*now);
                if surface.reader.settings.auto_scroll
                    && !surface.reader.settings_open
                    && surface.reader.compact_menu.is_none()
                    && !surface.reader.theme_menu_open
                    && !state.selecting
                    && state.scrollbar_grab.is_none()
                {
                    state.move_to(
                        state.y
                            + dt * 45.0 * f32::from(surface.reader.settings.scroll_speed_percent)
                                / 100.0,
                    );
                    if state.y < state.max() {
                        shell.request_redraw_at(*now + Duration::from_millis(16));
                    }
                }
                if state.selecting {
                    let distance = if state.pointer.y < bounds.y {
                        state.pointer.y - bounds.y
                    } else if state.pointer.y > bounds.y + bounds.height {
                        state.pointer.y - bounds.y - bounds.height
                    } else {
                        0.0
                    };
                    if distance != 0.0 {
                        state.move_to(state.y + distance.clamp(-700.0, 700.0) * dt * 3.0);
                        shell.request_redraw_at(*now + Duration::from_millis(16));
                    }
                }
            }
            Event::Window(window::Event::Unfocused) => {
                state.selecting = false;
                state.scrollbar_grab = None;
                state.last_frame = None;
            }
            _ => {}
        }
        if (state.y - old_y).abs() > 0.01 {
            state.prepare(surface.book);
            shell.request_redraw();
            if state.selecting
                && let Some(endpoint) = state.endpoint(state.pointer, bounds)
                && let Some((anchor, _)) = state.selection
            {
                state.selection = Some((anchor, endpoint));
            }
        }
        state.report(shell);
    }
}

pub(super) fn scrollbar_progress(height: f32, y: f32, grab: f32) -> f32 {
    super::super::adaptive_scroll::progress_for_pointer(height, y, grab)
}
