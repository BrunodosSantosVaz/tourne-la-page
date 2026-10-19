//! Anchor menus to their own widget; positioning uses layout coordinates, including translation.
use super::Message;
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::{Element, Event, Length, Point, Rectangle, Size, Theme, Vector, keyboard};

pub(super) fn anchored<'a>(
    trigger: Element<'a, Message>,
    menu: Element<'a, Message>,
    open: bool,
    close: Message,
    width: f32,
) -> Element<'a, Message> {
    Element::new(Popover {
        trigger,
        menu,
        open,
        close,
        width,
    })
}
struct Popover<'a> {
    trigger: Element<'a, Message>,
    menu: Element<'a, Message>,
    open: bool,
    close: Message,
    width: f32,
}
impl Widget<Message, Theme, iced::Renderer> for Popover<'_> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Popover<'static>>()
    }
    fn size(&self) -> Size<Length> {
        self.trigger.as_widget().size()
    }
    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.trigger),
            widget::Tree::new(&self.menu),
        ]
    }
    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.trigger.as_widget(), self.menu.as_widget()]);
    }
    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.trigger
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
        self.trigger.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
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
        self.trigger.as_widget_mut().update(
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
        self.trigger
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
        self.trigger.as_widget().mouse_interaction(
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
        if !self.open {
            return self.trigger.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }
        Some(overlay::Element::new(Box::new(Popup {
            menu: &mut self.menu,
            tree: &mut tree.children[1],
            anchor: layout.bounds() + translation,
            width: self.width,
            close: self.close.clone(),
        })))
    }
}
struct Popup<'a, 'b> {
    menu: &'a mut Element<'b, Message>,
    tree: &'a mut widget::Tree,
    anchor: Rectangle,
    width: f32,
    close: Message,
}
impl overlay::Overlay<Message, Theme, iced::Renderer> for Popup<'_, '_> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let below = (bounds.height - self.anchor.y - self.anchor.height - 8.0).max(0.0);
        let above = (self.anchor.y - 8.0).max(0.0);
        let width = self.width.min((bounds.width - 16.0).max(1.0));
        let limits = layout::Limits::new(
            Size::new(width, 0.0),
            Size::new(width, below.max(above).min(360.0)),
        );
        let node = self
            .menu
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let x = self
            .anchor
            .x
            .min(bounds.width - node.size().width - 8.0)
            .max(8.0);
        let y = if node.size().height <= below {
            self.anchor.y + self.anchor.height + 4.0
        } else {
            (self.anchor.y - node.size().height - 4.0).max(8.0)
        };
        node.move_to(Point::new(x, y))
    }
    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.menu.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let outside = matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) && !cursor.is_over(layout.bounds());
        let escape = matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })
        );
        if outside || escape {
            shell.publish(self.close.clone());
            shell.capture_event();
            return;
        }
        self.menu.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
        if matches!(event, Event::Mouse(_)) && cursor.is_over(layout.bounds()) {
            shell.capture_event();
        }
    }
    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.menu
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.menu.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::overlay::Overlay as _;

    #[test]
    fn closed_popover_preserves_its_trigger_tooltip() {
        use super::super::diagnostics::{renderer, send};
        let renderer = renderer();
        let trigger = iced::widget::tooltip(
            iced::widget::button("A").width(32).height(32),
            iced::widget::text("Fonte"),
            iced::widget::tooltip::Position::Bottom,
        )
        .into();
        let mut control = anchored(
            trigger,
            iced::widget::text("Menu").into(),
            false,
            Message::ToggleThemeMenu,
            200.0,
        );
        let mut tree = widget::Tree::new(&control);
        let size = Size::new(32.0, 32.0);
        let node =
            control
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let _ = send(
            &mut control,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(16.0, 16.0),
            }),
            Point::new(16.0, 16.0),
        );
        assert!(
            control
                .as_widget_mut()
                .overlay(
                    &mut tree,
                    Layout::new(&node),
                    &renderer,
                    &Rectangle::with_size(Size::new(800.0, 600.0)),
                    Vector::ZERO
                )
                .is_some()
        );
    }

    #[test]
    fn menu_follows_trigger_and_stays_inside_small_windows() {
        let renderer = super::super::diagnostics::renderer();
        let mut menu: Element<'_, Message> = iced::widget::container(iced::widget::text("Tema"))
            .width(Length::Fill)
            .height(120)
            .into();
        let mut tree = widget::Tree::new(&menu);
        let mut popup = Popup {
            menu: &mut menu,
            tree: &mut tree,
            anchor: Rectangle::new(Point::new(40.0, 32.0), Size::new(130.0, 32.0)),
            width: 260.0,
            close: Message::ToggleThemeMenu,
        };
        let bounds = popup.layout(&renderer, Size::new(1280.0, 800.0)).bounds();
        assert_eq!(bounds.x, 40.0);
        assert_eq!(bounds.y, 68.0);
        assert_eq!(bounds.width, 260.0);
        popup.anchor = Rectangle::new(Point::new(310.0, 650.0), Size::new(40.0, 32.0));
        let node = popup.layout(&renderer, Size::new(390.0, 700.0));
        let bounds = node.bounds();
        assert!(bounds.x >= 8.0 && bounds.x + bounds.width <= 382.0);
        assert!(bounds.y + bounds.height < popup.anchor.y);
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        popup.update(
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(Point::ORIGIN),
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut shell,
        );
        assert!(shell.is_event_captured());
        assert!(matches!(messages.as_slice(), [Message::ToggleThemeMenu]));
    }
}
