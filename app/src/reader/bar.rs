//! Measure controls before deciding which ones fit. Hidden controls retain their state.
mod arrange;
#[cfg(test)]
mod tests;

use super::Message;
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::{Element, Event, Length, Point, Rectangle, Size, Theme, Vector};

pub(super) const MIN_WIDTH: f32 = 7.0 * 32.0 + 6.0 * arrange::GAP;

pub(super) struct Control<'a> {
    pub full: Element<'a, Message>,
    pub clean: Element<'a, Message>,
    pub tray: Element<'a, Message>,
}

pub(super) struct Bar<'a> {
    children: Vec<Element<'a, Message>>,
    controls: usize,
    expanded: bool,
}

impl<'a> Bar<'a> {
    pub(super) fn view(
        left: [Element<'a, Message>; 2],
        controls: Vec<Control<'a>>,
        right: [Element<'a, Message>; 5],
        expanded: bool,
    ) -> Element<'a, Message> {
        let count = controls.len();
        Element::new(Self {
            children: left
                .into_iter()
                .chain(
                    controls
                        .into_iter()
                        .flat_map(|control| [control.full, control.clean, control.tray]),
                )
                .chain(right)
                .collect(),
            controls: count,
            expanded,
        })
    }
}

impl Widget<Message, Theme, iced::Renderer> for Bar<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }
    fn children(&self) -> Vec<widget::Tree> {
        self.children.iter().map(widget::Tree::new).collect()
    }
    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&self.children);
    }
    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let mut nodes = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(child, state)| {
                child.as_widget_mut().layout(
                    state,
                    renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(width, 32.0)),
                )
            })
            .collect::<Vec<_>>();
        let sizes = nodes.iter().map(layout::Node::size).collect::<Vec<_>>();
        let arranged = arrange::place(width, &sizes, self.controls, self.expanded);
        for (node, point) in nodes.iter_mut().zip(arranged.points) {
            if let Some(point) = point {
                node.move_to_mut(point);
            } else {
                *node = layout::Node::new(Size::ZERO);
            }
        }
        layout::Node::with_children(Size::new(width, arranged.height), nodes)
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
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, l)| l.bounds().width > 0.0)
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
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
        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter(|(_, l)| l.bounds().width > 0.0)
        {
            child.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }
    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        op: &mut dyn widget::Operation,
    ) {
        op.container(None, layout.bounds());
        op.traverse(&mut |op| {
            for ((child, state), layout) in self
                .children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .filter(|(_, l)| l.bounds().width > 0.0)
            {
                child.as_widget_mut().operate(state, layout, renderer, op);
            }
        });
    }
    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, l)| l.bounds().width > 0.0)
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }
    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        let overlays = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter(|(_, l)| l.bounds().width > 0.0)
            .filter_map(|((child, state), layout)| {
                child
                    .as_widget_mut()
                    .overlay(state, layout, renderer, viewport, translation)
            })
            .collect::<Vec<_>>();
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}
