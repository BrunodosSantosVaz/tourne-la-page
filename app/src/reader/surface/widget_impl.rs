use super::*;

impl Widget<Message, Theme, iced::Renderer> for Surface<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }
    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        _: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);
        tree.state
            .downcast_mut::<State>()
            .configure(self.reader, self.book, size);
        layout::Node::new(size)
    }
    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        _: &iced::Renderer,
        op: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        op.focusable(Some(&self.reader.scroll_id), bounds, state);
        op.scrollable(
            Some(&self.reader.scroll_id),
            bounds,
            Rectangle::new(
                bounds.position(),
                Size::new(bounds.width, state.heights.total()),
            ),
            Vector::new(0.0, -state.y),
            state,
        );
        state.prepare(self.book);
    }
    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _: &Rectangle,
    ) {
        tree.state.downcast_mut::<State>().handle_event(
            self,
            input::Input {
                event,
                bounds: layout.bounds(),
                cursor,
                clipboard,
                shell,
            },
        );
    }
    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
    ) {
        self.paint(
            tree.state.downcast_ref::<State>(),
            renderer,
            layout.bounds(),
        );
    }
    fn mouse_interaction(
        &self,
        _: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &iced::Renderer,
    ) -> mouse::Interaction {
        if let Some(point) = cursor.position_in(layout.bounds()) {
            if point.x >= layout.bounds().width - 18.0 {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::None
        }
    }
}
