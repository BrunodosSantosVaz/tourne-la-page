use super::super::diagnostics::{key, reader, renderer, send};
use super::*;
use iced::advanced::{Layout, layout, widget};
use iced::{Event, Point, Rectangle, Size, Vector};

#[test]
fn minimum_window_width_matches_the_fixed_buttons_and_reading_margins() {
    assert_eq!(
        main_window_settings().min_size.unwrap().width,
        bar::MIN_WIDTH + 24.0
    );
}

#[test]
fn forced_resize_below_the_fixed_buttons_requests_the_minimum_width() {
    use iced::futures::StreamExt as _;
    let mut reader = reader(1);
    let id = iced::window::Id::unique();
    let task = update(
        &mut reader,
        Message::WindowResized(id, Size::new(160.0, 600.0)),
    );
    let actions = iced::futures::executor::block_on(
        iced_runtime::task::into_stream(task)
            .unwrap()
            .collect::<Vec<_>>(),
    );
    assert!(matches!(actions.as_slice(),
        [iced_runtime::Action::Window(iced_runtime::window::Action::Resize(window, size))]
        if *window == id && *size == Size::new(bar::MIN_WIDTH + 24.0, 600.0)));
    assert_eq!(reader.window_width, bar::MIN_WIDTH + 24.0);
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn clean_icons_open_their_own_controls_and_escape_closes_them() {
    let mut reader = reader(1);
    let renderer = renderer();
    for item in [
        Item::Theme,
        Item::Font,
        Item::Size,
        Item::Alignment,
        Item::Width,
        Item::LineHeight,
        Item::Marker,
    ] {
        let _ = update(&mut reader, Message::ToggleCompactMenu(item));
        assert_eq!(reader.compact_menu, Some(item));
        let mut control = clean_control(&reader, item);
        let mut tree = widget::Tree::new(&control);
        let size = Size::new(32.0, 32.0);
        let node =
            control
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let mut popup = control
            .as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &Rectangle::with_size(Size::new(246.0, 800.0)),
                Vector::ZERO,
            )
            .unwrap();
        let popup_node = popup
            .as_overlay_mut()
            .layout(&renderer, Size::new(246.0, 800.0));
        assert!(popup_node.bounds().width <= 230.0);
        let mut messages = Vec::new();
        let mut shell = iced::advanced::Shell::new(&mut messages);
        popup.as_overlay_mut().update(
            &key(iced::keyboard::key::Named::Escape),
            Layout::new(&popup_node),
            iced::mouse::Cursor::Unavailable,
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut shell,
        );
        assert!(shell.is_event_captured());
        drop(popup);
        drop(control);
        for message in messages {
            let _ = update(&mut reader, message);
        }
        assert!(reader.compact_menu.is_none());
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn pointer_release_on_a_clean_icon_publishes_its_toggle() {
    let reader = reader(1);
    let renderer = renderer();
    let mut control = clean_control(&reader, Item::Width);
    let mut tree = widget::Tree::new(&control);
    let size = Size::new(32.0, 32.0);
    let node =
        control
            .as_widget_mut()
            .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
    for event in [
        iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left),
        iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left),
    ] {
        let messages = send(
            &mut control,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(event),
            Point::new(16.0, 16.0),
        );
        if matches!(event, iced::mouse::Event::ButtonReleased(_)) {
            assert!(matches!(
                messages.as_slice(),
                [Message::ToggleCompactMenu(Item::Width)]
            ));
        }
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn images_toggle_directly_without_opening_a_menu() {
    let mut reader = reader(1);
    let renderer = renderer();
    let initial = reader.settings.show_images;
    for expected in [!initial, initial] {
        let mut control = clean_control(&reader, Item::Images);
        let mut tree = widget::Tree::new(&control);
        let size = Size::new(32.0, 32.0);
        let node =
            control
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let mut messages = Vec::new();
        for event in [
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left),
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left),
        ] {
            messages.extend(send(
                &mut control,
                &mut tree,
                &node,
                &renderer,
                Event::Mouse(event),
                Point::new(16.0, 16.0),
            ));
        }
        drop(control);
        assert!(
            matches!(messages.as_slice(), [Message::ImagesToggled(value)] if *value == expected)
        );
        for message in messages {
            let _ = update(&mut reader, message);
        }
        assert_eq!(reader.settings.show_images, expected);
        assert_eq!(
            Item::Images.icon(&reader),
            if expected { "image" } else { "image-off" }
        );
        assert!(reader.compact_menu.is_none());
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn width_and_line_height_sliders_are_vertical_and_respond_to_pointer() {
    use super::super::diagnostics::geometry;
    let reader = reader(1);
    let renderer = renderer();
    for item in [Item::Width, Item::LineHeight] {
        let mut control = match item {
            Item::Width => width_control(&reader),
            _ => line_height_control(&reader),
        };
        let mut tree = widget::Tree::new(&control);
        let node = control.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(110.0, 280.0)),
        );
        let measured = geometry(&mut control, &mut tree, &node, &renderer);
        let slider = measured.bounds[&widget::Id::new(item.first_focus())];
        assert!(slider.height > slider.width * 3.0);
        let mut values = Vec::new();
        for y in [slider.y + slider.height - 2.0, slider.y + 2.0] {
            for event in [
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left),
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left),
            ] {
                for message in send(
                    &mut control,
                    &mut tree,
                    &node,
                    &renderer,
                    Event::Mouse(event),
                    Point::new(slider.x + slider.width / 2.0, y),
                ) {
                    match message {
                        Message::WidthChanged(value) | Message::LineHeightChanged(value) => {
                            values.push(value)
                        }
                        _ => {}
                    }
                }
            }
        }
        assert!(values.first().unwrap() < values.last().unwrap());
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn alignment_choices_form_a_vertical_list_and_selection_closes_it() {
    use super::super::diagnostics::geometry;
    let mut reader = reader(1);
    let renderer = renderer();
    reader.compact_menu = Some(Item::Alignment);
    let mut control = alignment_controls(reader.settings.alignment);
    let mut tree = widget::Tree::new(&control);
    let node = control.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(180.0, 200.0)),
    );
    let measured = geometry(&mut control, &mut tree, &node, &renderer);
    let choices = [
        "reading-align-left",
        "reading-align-center",
        "reading-align-right",
        "reading-align-justify",
    ]
    .map(|id| measured.bounds[&widget::Id::new(id)]);
    assert!(
        choices
            .windows(2)
            .all(|pair| pair[0].x == pair[1].x && pair[0].y + pair[0].height < pair[1].y)
    );
    let choice = choices[2];
    let point = Point::new(choice.x + 16.0, choice.y + 16.0);
    let mut messages = Vec::new();
    for event in [
        iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left),
        iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left),
    ] {
        messages.extend(send(
            &mut control,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(event),
            point,
        ));
    }
    drop(control);
    for message in messages {
        let _ = update(&mut reader, message);
    }
    assert_eq!(reader.settings.alignment, AlignmentChoice::Right);
    assert!(reader.compact_menu.is_none());
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn actual_toolbar_keeps_fixed_controls_in_order_and_only_shows_a_nonempty_tray() {
    use super::super::diagnostics::geometry;
    let mut reader = reader(1);
    let renderer = renderer();
    for width in [284.0, 390.0, 626.0, 800.0, 1280.0, 1920.0] {
        reader.window_width = width;
        let mut control = view(&reader);
        let mut tree = widget::Tree::new(&control);
        let node = control.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width - 24.0, 400.0)),
        );
        let measured = geometry(&mut control, &mut tree, &node, &renderer);
        let fixed = [
            "toolbar-Índice",
            "toolbar-Início",
            "toolbar-Rolagem automática",
            "theme-trigger",
            "toolbar-Configurações",
            "toolbar-Comentários",
        ]
        .map(|id| measured.bounds[&widget::Id::new(id)]);
        assert!(
            fixed
                .windows(2)
                .all(|pair| pair[0].x + pair[0].width <= pair[1].x)
        );
        for bound in fixed {
            assert_eq!(bound.size(), Size::new(32.0, 32.0));
            assert_eq!(bound.y, 0.0);
        }
        assert_eq!(fixed[1].x, fixed[0].x + 38.0);
        assert_eq!(fixed[4].x, fixed[3].x + 38.0);
        if width >= 800.0 {
            assert!(fixed[2].x > fixed[1].x + 38.0);
        }
        let tray = measured
            .bounds
            .contains_key(&widget::Id::new("toolbar-Expandir barra"));
        if width >= 800.0 {
            assert!(!tray);
        }
        if width <= 390.0 {
            assert!(tray);
        }
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}

#[test]
fn percentages_are_centered_between_their_minus_and_plus_buttons() {
    use super::super::diagnostics::geometry;
    let mut reader = reader(1);
    let renderer = renderer();
    for value in [10, 100, 1000] {
        reader.settings.scroll_speed_percent = value;
        reader.settings.font_scale = value;
        for (item, smaller, number, larger) in [
            (
                Item::Speed,
                "speed-smaller",
                "scroll-speed-value",
                "speed-larger",
            ),
            (Item::Size, "font-smaller", "font-size-value", "font-larger"),
        ] {
            let mut control = full_control(&reader, item);
            let mut tree = widget::Tree::new(&control);
            let node = control.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(400.0, 32.0)),
            );
            let measured = geometry(&mut control, &mut tree, &node, &renderer);
            let minus = measured.bounds[&widget::Id::new(smaller)];
            let label = measured.bounds[&widget::Id::new(number)];
            let plus = measured.bounds[&widget::Id::new(larger)];
            assert!((label.center_x() - (minus.center_x() + plus.center_x()) / 2.0).abs() < 0.1);
        }
    }
    std::fs::remove_dir_all(&reader.data_dir).unwrap();
}
