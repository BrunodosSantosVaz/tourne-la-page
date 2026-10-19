//! Keep native scrolling, focus and clipping; share the reader's adaptive thumb everywhere.
use super::{Message, adaptive_scroll_thumb};
use iced::advanced::renderer::Renderer as _;
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::widget::scrollable;
use iced::{Color, Element, Event, Length, Point, Rectangle, Size, Theme, Vector};

pub(super) fn new<'a>(content: impl Into<Element<'a, Message>>) -> AdaptiveScroll<'a> {
    AdaptiveScroll {
        inner: iced::widget::scrollable(content).direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden().spacing(14),
        )),
    }
}

pub(super) struct AdaptiveScroll<'a> {
    inner: iced::widget::Scrollable<'a, Message>,
}

impl AdaptiveScroll<'_> {
    pub(super) fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.inner = self.inner.id(id);
        self
    }
    pub(super) fn height(mut self, height: impl Into<Length>) -> Self {
        self.inner = self.inner.height(height);
        self
    }
    pub(super) fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }
    fn sync(&mut self, tree: &mut widget::Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        let mut snapshot = Snapshot::default();
        self.inner
            .operate(&mut tree.children[0], layout, renderer, &mut snapshot);
        tree.state.downcast_mut::<State>().metrics = snapshot;
    }
    fn snap(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        progress: f32,
    ) {
        self.inner
            .operate(&mut tree.children[0], layout, renderer, &mut Snap(progress));
        self.sync(tree, layout, renderer);
    }
}

#[derive(Default)]
struct State {
    metrics: Snapshot,
    grab: Option<f32>,
}

#[derive(Default, Clone, Copy)]
struct Snapshot {
    bounds: Rectangle,
    content: Rectangle,
    offset: f32,
}
impl Snapshot {
    fn max(self) -> f32 {
        (self.content.height - self.bounds.height).max(0.0)
    }
    fn progress(self) -> f32 {
        if self.max() > 0.0 {
            (self.offset / self.max()).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
    fn rail(self) -> Rectangle {
        Rectangle::new(
            Point::new(self.bounds.x + self.bounds.width - 12.0, self.bounds.y),
            Size::new(10.0, self.bounds.height),
        )
    }
    fn thumb(self) -> Rectangle {
        let (height, top) = adaptive_scroll_thumb(self.bounds.height, self.progress() * 100.0);
        let mut thumb = self.rail();
        thumb.y += top;
        thumb.height = height;
        thumb
    }
}
impl widget::Operation for Snapshot {
    fn traverse(&mut self, _: &mut dyn FnMut(&mut dyn widget::Operation)) {}
    fn scrollable(
        &mut self,
        _: Option<&widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        _: &mut dyn widget::operation::Scrollable,
    ) {
        *self = Self {
            bounds,
            content,
            offset: translation.y,
        };
    }
}
struct Snap(f32);
impl widget::Operation for Snap {
    fn traverse(&mut self, _: &mut dyn FnMut(&mut dyn widget::Operation)) {}
    fn scrollable(
        &mut self,
        _: Option<&widget::Id>,
        _: Rectangle,
        _: Rectangle,
        _: Vector,
        state: &mut dyn widget::operation::Scrollable,
    ) {
        state.snap_to(scrollable::RelativeOffset {
            x: None,
            y: Some(self.0),
        });
    }
}

pub(super) fn progress_for_pointer(height: f32, y: f32, grab: f32) -> f32 {
    let mut low = 0.0;
    let mut high = 1.0;
    for _ in 0..20 {
        let middle = (low + high) / 2.0;
        let (size, top) = adaptive_scroll_thumb(height, middle * 100.0);
        if top + size * grab < y {
            low = middle;
        } else {
            high = middle;
        }
    }
    (low + high) / 2.0
}

impl<'a> From<AdaptiveScroll<'a>> for Element<'a, Message> {
    fn from(scroll: AdaptiveScroll<'a>) -> Self {
        Self::new(scroll)
    }
}

impl Widget<Message, Theme, iced::Renderer> for AdaptiveScroll<'_> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }
    fn size(&self) -> Size<Length> {
        self.inner.size()
    }
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(
            &self.inner as &dyn Widget<Message, Theme, iced::Renderer>,
        )]
    }
    fn diff(&self, tree: &mut widget::Tree) {
        tree.children[0].diff(&self.inner as &dyn Widget<Message, Theme, iced::Renderer>);
    }
    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = self.inner.layout(&mut tree.children[0], renderer, limits);
        self.sync(tree, Layout::new(&node), renderer);
        node
    }
    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.inner.draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        let mut metrics = tree.state.downcast_ref::<State>().metrics;
        metrics.bounds = layout.bounds();
        if metrics.max() <= 0.0 {
            return;
        }
        if let Some(clip) = layout.bounds().intersection(viewport) {
            renderer.with_layer(clip, |renderer| {
                let accent = theme.palette().primary;
                for (bounds, color) in [
                    (metrics.rail(), Color { a: 0.12, ..accent }),
                    (metrics.thumb(), accent),
                ] {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds,
                            border: iced::Border::default().rounded(6),
                            ..Default::default()
                        },
                        color,
                    );
                }
            });
        }
    }
    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.sync(tree, layout, renderer);
        let state = tree.state.downcast_mut::<State>();
        let metrics = state.metrics;
        let position = cursor.position().or(match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => Some(*position),
            _ => None,
        });
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if metrics.max() > 0.0
                    && position.is_some_and(|point| metrics.rail().contains(point)) =>
            {
                let point = position.unwrap();
                let thumb = metrics.thumb();
                let grab = if thumb.contains(point) {
                    (point.y - thumb.y) / thumb.height
                } else {
                    0.5
                };
                state.grab = Some(grab);
                let progress =
                    progress_for_pointer(metrics.bounds.height, point.y - metrics.bounds.y, grab);
                self.snap(tree, layout, renderer, progress);
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.grab.is_some() => {
                if let Some(point) = position {
                    let progress = progress_for_pointer(
                        metrics.bounds.height,
                        point.y - metrics.bounds.y,
                        state.grab.unwrap(),
                    );
                    self.snap(tree, layout, renderer, progress);
                }
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.grab.take().is_some() =>
            {
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            Event::Window(iced::window::Event::Unfocused) => {
                state.grab = None;
            }
            _ => {}
        }
        self.inner.update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        self.sync(tree, layout, renderer);
    }
    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.inner
            .operate(&mut tree.children[0], layout, renderer, operation);
        self.sync(tree, layout, renderer);
    }
    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let mut metrics = state.metrics;
        metrics.bounds = layout.bounds();
        if state.grab.is_some() {
            mouse::Interaction::Grabbing
        } else if metrics.max() > 0.0 && cursor.is_over(metrics.rail()) {
            mouse::Interaction::Grab
        } else {
            self.inner
                .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
        }
    }
    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        self.inner.overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::diagnostics::{renderer, send};
    use super::*;
    use iced::widget::{column, text};

    #[test]
    fn native_scrolling_and_dragging_share_the_readers_thumb_without_covering_content() {
        let renderer = renderer();
        let content =
            column((0..200).map(|i| text(format!("Opção {i}")).size(14).height(32).into()))
                .width(Length::Fill);
        let mut element: Element<'_, Message> = new(content)
            .id(widget::Id::new("scroll-test"))
            .width(240)
            .height(320)
            .into();
        let mut tree = widget::Tree::new(&element);
        let size = Size::new(240.0, 320.0);
        let node =
            element
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let first = tree.state.downcast_ref::<State>().metrics;
        assert!(first.max() > 0.0);
        assert!(first.content.x + first.content.width <= first.rail().x);
        let _ = send(
            &mut element,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -600.0 },
            }),
            Point::new(120.0, 100.0),
        );
        let after = tree.state.downcast_ref::<State>().metrics;
        assert!(after.progress() > 0.0);
        assert!(after.thumb().height < first.thumb().height);
        let mut snap = widget::operation::scrollable::snap_to(
            widget::Id::new("scroll-test"),
            scrollable::RelativeOffset {
                x: None,
                y: Some(0.5),
            },
        );
        element
            .as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut snap);
        let half = tree.state.downcast_ref::<State>().metrics;
        assert!((half.progress() - 0.5).abs() < 0.001);
        let press = Point::new(
            half.thumb().center_x(),
            half.thumb().y + half.thumb().height * 0.3,
        );
        let _ = send(
            &mut element,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            press,
        );
        assert!((tree.state.downcast_ref::<State>().metrics.progress() - 0.5).abs() < 0.001);
        let end = adaptive_scroll_thumb(size.height, 100.0);
        let point = Point::new(press.x, end.1 + end.0 * 0.3);
        let _ = send(
            &mut element,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::CursorMoved { position: point }),
            point,
        );
        let _ = send(
            &mut element,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            point,
        );
        let last = tree.state.downcast_ref::<State>();
        assert!(last.metrics.progress() > 0.999);
        assert!(last.grab.is_none());
        assert!(last.metrics.thumb().height < first.thumb().height);
    }

    #[test]
    fn short_content_does_not_add_a_bar_or_reserve_an_empty_gutter() {
        let renderer = renderer();
        let mut element: Element<'_, Message> =
            new(iced::widget::container(text("Curto")).width(Length::Fill))
                .width(240)
                .height(320)
                .into();
        let mut tree = widget::Tree::new(&element);
        let size = Size::new(240.0, 320.0);
        let node =
            element
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let metrics = tree.state.downcast_ref::<State>().metrics;
        assert_eq!(metrics.max(), 0.0);
        assert_eq!(metrics.content.width, size.width);
        let _ = send(
            &mut element,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Point::new(235.0, 100.0),
        );
        assert!(tree.state.downcast_ref::<State>().grab.is_none());
    }
}
