//! Keyboard focus for native Iced buttons, which do not expose it themselves.
use super::Message;
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::{Element, Event, Length, Rectangle, Size, Theme, Vector, keyboard};
use renderer::Renderer as _;

pub(super) fn focusable<'a>(
    content: Element<'a, Message>,
    id: impl Into<widget::Id>,
    activate: Message,
    neighbors: Option<(Message, Message)>,
) -> Element<'a, Message> {
    Element::new(Action {
        content,
        id: id.into(),
        activate,
        neighbors,
    })
}
struct Action<'a> {
    content: Element<'a, Message>,
    id: widget::Id,
    activate: Message,
    neighbors: Option<(Message, Message)>,
}
#[derive(Default)]
struct Focus(bool);
impl widget::operation::Focusable for Focus {
    fn is_focused(&self) -> bool {
        self.0
    }
    fn focus(&mut self) {
        self.0 = true;
    }
    fn unfocus(&mut self) {
        self.0 = false;
    }
}
impl Widget<Message, Theme, iced::Renderer> for Action<'_> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Focus>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Focus::default())
    }
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        if tree.state.downcast_ref::<Focus>().0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced::Border::default()
                        .color(theme.palette().primary)
                        .width(2.0)
                        .rounded(4.0),
                    ..Default::default()
                },
                iced::Color::TRANSPARENT,
            );
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
        let focus = tree.state.downcast_mut::<Focus>();
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) {
            focus.0 = cursor.is_over(layout.bounds());
        }
        if focus.0
            && let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event
        {
            use keyboard::key::Named;
            if matches!(key, keyboard::Key::Named(Named::Enter | Named::Space)) {
                shell.publish(self.activate.clone());
                shell.capture_event();
                return;
            }
            if let Some((previous, next)) = &self.neighbors {
                let target = match key {
                    keyboard::Key::Named(Named::ArrowUp | Named::ArrowLeft) => Some(previous),
                    keyboard::Key::Named(Named::ArrowDown | Named::ArrowRight) => Some(next),
                    _ => None,
                };
                if let Some(target) = target {
                    shell.publish(target.clone());
                    shell.capture_event();
                    return;
                }
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.focusable(
            Some(&self.id),
            layout.bounds(),
            tree.state.downcast_mut::<Focus>(),
        );
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

pub(super) fn menu_item<'a>(
    content: Element<'a, Message>,
    prefix: &str,
    index: usize,
    total: usize,
    message: Message,
) -> Element<'a, Message> {
    let id = widget::Id::from(format!("{prefix}-{index}"));
    let previous = Message::MenuFocus(prefix.to_owned(), (index + total - 1) % total);
    let next = Message::MenuFocus(prefix.to_owned(), (index + 1) % total);
    focusable(content, id, message, Some((previous, next)))
}

#[cfg(test)]
mod tests {
    use super::super::diagnostics::{key, renderer, send};
    use super::*;

    #[test]
    fn focused_choices_activate_and_arrow_keys_cycle_without_mouse() {
        let renderer = renderer();
        let mut choice = menu_item(
            iced::widget::button("Tema")
                .on_press(Message::ToggleThemeMenu)
                .into(),
            "theme-option",
            0,
            3,
            Message::ToggleThemeMenu,
        );
        let mut tree = widget::Tree::new(&choice);
        tree.state.downcast_mut::<Focus>().0 = true;
        let size = Size::new(260.0, 32.0);
        let node =
            choice
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        for (named, target) in [
            (keyboard::key::Named::ArrowDown, 1),
            (keyboard::key::Named::ArrowUp, 2),
        ] {
            let messages = send(
                &mut choice,
                &mut tree,
                &node,
                &renderer,
                key(named),
                iced::Point::ORIGIN,
            );
            assert!(
                matches!(messages.as_slice(), [Message::MenuFocus(prefix, index)] if prefix == "theme-option" && *index == target)
            );
        }
        let messages = send(
            &mut choice,
            &mut tree,
            &node,
            &renderer,
            key(keyboard::key::Named::Enter),
            iced::Point::ORIGIN,
        );
        assert!(matches!(messages.as_slice(), [Message::ToggleThemeMenu]));
    }
}
