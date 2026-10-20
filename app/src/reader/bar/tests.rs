use super::*;

const CONTROLS: usize = 8;
const RIGHT: usize = 2 + CONTROLS * 3;

fn sizes() -> Vec<Size> {
    let mut sizes = vec![Size::new(32.0, 32.0); 2];
    for i in 0..CONTROLS {
        let full = match i {
            0 => 122.0, // Speed is next to play; its label only appears in the tray.
            2 => 175.0, // Font size is the one remaining full/clean control.
            _ => 32.0,
        };
        sizes.extend([
            Size::new(full, 32.0),
            Size::new(if i == 2 { 32.0 } else { full }, 32.0),
            Size::new(if i == 0 { 220.0 } else { full }, 32.0),
        ]);
    }
    sizes.extend([Size::new(32.0, 32.0); 5]);
    sizes
}

#[test]
fn speed_follows_play_and_theme_is_fixed_next_to_settings_without_an_empty_tray() {
    let sizes = sizes();
    let arranged = arrange::place(1600.0, &sizes, CONTROLS, false);
    assert_eq!(arranged.height, 32.0);
    for i in 0..CONTROLS {
        assert!(arranged.points[2 + i * 3].is_some());
        assert!(arranged.points[3 + i * 3].is_none());
    }
    assert_eq!(
        arranged.points[2].unwrap().x,
        arranged.points[RIGHT].unwrap().x + 38.0
    );
    assert_eq!(
        arranged.points[RIGHT + 2].unwrap().x,
        arranged.points[RIGHT + 1].unwrap().x + 38.0
    );
    assert!(arranged.points[RIGHT + 4].is_none());
    assert!(arranged.points[RIGHT].unwrap().x > arranged.points[1].unwrap().x + 38.0);
    assert_eq!(
        arranged.points[RIGHT + 1].unwrap().x,
        arranged.points[2 + (CONTROLS - 1) * 3].unwrap().x + 38.0
    );
    assert_eq!(arrange::place(1600.0, &sizes, CONTROLS, true).height, 32.0);
}

#[test]
fn speed_goes_directly_to_tray_and_other_controls_collapse_gradually() {
    let sizes = sizes();
    let clean = arrange::place(640.0, &sizes, CONTROLS, false);
    assert!(clean.points[2].is_some());
    assert!(clean.points[2 + 2 * 3 + 1].is_some());
    assert!(clean.points[RIGHT + 4].is_none());
    let tray = arrange::place(560.0, &sizes, CONTROLS, false);
    assert!(tray.points[2..5].iter().all(Option::is_none));
    assert!(tray.points[RIGHT + 4].is_some());
    for i in 1..CONTROLS {
        assert!(tray.points[2 + i * 3].is_some() || tray.points[3 + i * 3].is_some());
    }
}

#[test]
fn expanded_tray_uses_its_own_controls_and_wraps_inside_minimum_width() {
    let sizes = sizes();
    let arranged = arrange::place(MIN_WIDTH, &sizes, CONTROLS, true);
    assert!(
        arranged.points[4].unwrap().y > 0.0,
        "labeled speed is in tray"
    );
    for (i, point) in arranged.points.iter().enumerate() {
        if let Some(point) = point {
            assert!(point.x >= 0.0 && point.x + sizes[i].width <= MIN_WIDTH);
        }
    }
    assert!(arranged.height > 32.0);
}

#[test]
fn minimum_width_fits_exactly_the_fixed_buttons_without_overlap() {
    let sizes = sizes();
    let arranged = arrange::place(MIN_WIDTH, &sizes, CONTROLS, false);
    assert!(arranged.points[2..RIGHT].iter().all(Option::is_none));
    let mut bounds = [0, 1, RIGHT, RIGHT + 1, RIGHT + 2, RIGHT + 3, RIGHT + 4]
        .map(|i| Rectangle::new(arranged.points[i].unwrap(), sizes[i]));
    bounds.sort_by(|a, b| a.x.total_cmp(&b.x));
    assert_eq!(bounds[0].x, 0.0);
    assert_eq!(bounds[6].x + bounds[6].width, MIN_WIDTH);
    assert!(
        bounds
            .windows(2)
            .all(|pair| pair[0].x + pair[0].width < pair[1].x)
    );
}

#[test]
fn every_width_keeps_visible_controls_inside_the_bar_without_overlap() {
    let sizes = sizes();
    for width in (MIN_WIDTH as usize..1600).step_by(7) {
        for expanded in [false, true] {
            let arranged = arrange::place(width as f32, &sizes, CONTROLS, expanded);
            let mut bounds = arranged
                .points
                .iter()
                .enumerate()
                .filter_map(|(i, point)| point.map(|point| Rectangle::new(point, sizes[i])))
                .collect::<Vec<_>>();
            bounds.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
            for bound in &bounds {
                assert!(bound.x >= -0.1 && bound.x + bound.width <= width as f32 + 0.1);
            }
            assert!(
                bounds
                    .windows(2)
                    .all(|pair| pair[0].y != pair[1].y || pair[0].x + pair[0].width <= pair[1].x)
            );
        }
    }
}
