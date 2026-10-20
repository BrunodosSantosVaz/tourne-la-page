use super::*;

pub(super) const GAP: f32 = 6.0;
const ROW: f32 = 32.0;
const LEFT: usize = 2;
const VARIANTS: usize = 3;
const RIGHT: usize = 5;
const TRAY: usize = 4;

pub(super) struct Arrangement {
    pub points: Vec<Option<Point>>,
    pub height: f32,
}

struct Fit {
    clean: Vec<bool>,
    visible: Vec<bool>,
}

impl Fit {
    fn overflow(&self) -> bool {
        self.visible.iter().any(|visible| !visible)
    }
}

fn fit(width: f32, sizes: &[Size], controls: usize) -> Fit {
    let base = LEFT + controls * VARIANTS;
    let fixed_width = sizes[..LEFT].iter().map(|size| size.width).sum::<f32>()
        + (0..RIGHT)
            .filter(|i| *i != TRAY)
            .map(|i| sizes[base + i].width)
            .sum::<f32>();
    let mut used = fixed_width
        + (controls + LEFT + RIGHT - 2) as f32 * GAP
        + (0..controls)
            .map(|i| sizes[LEFT + i * VARIANTS].width)
            .sum::<f32>();
    let mut clean = vec![false; controls];
    for i in (0..controls).rev() {
        if used <= width {
            break;
        }
        let full = sizes[LEFT + i * VARIANTS].width;
        let compact = sizes[LEFT + i * VARIANTS + 1].width;
        if compact < full {
            used -= full - compact;
            clean[i] = true;
        }
    }
    let mut visible = vec![true; controls];
    if used > width {
        used += sizes[base + TRAY].width + GAP;
        // Speed follows play visually, but yields space before the other controls.
        for i in std::iter::once(0).chain((1..controls).rev()) {
            if used <= width {
                break;
            }
            visible[i] = false;
            used -= sizes[LEFT + i * VARIANTS + usize::from(clean[i])].width + GAP;
        }
    }
    Fit { clean, visible }
}

fn point(x: f32, y: f32, size: Size) -> Point {
    Point::new(x, y + (ROW - size.height).max(0.0) / 2.0)
}

pub(super) fn place(width: f32, sizes: &[Size], controls: usize, expanded: bool) -> Arrangement {
    let fit = fit(width, sizes, controls);
    let base = LEFT + controls * VARIANTS;
    let mut points = vec![None; sizes.len()];
    let mut x = 0.0;
    for i in 0..LEFT {
        points[i] = Some(point(x, 0.0, sizes[i]));
        x += sizes[i].width + GAP;
    }
    let mut right = width;
    for i in (1..RIGHT).rev().filter(|i| *i != TRAY || fit.overflow()) {
        let index = base + i;
        right -= sizes[index].width;
        points[index] = Some(point(right, 0.0, sizes[index]));
        right -= GAP;
    }
    for i in (0..controls).rev() {
        if fit.visible[i] {
            let index = LEFT + i * VARIANTS + usize::from(fit.clean[i]);
            right -= sizes[index].width;
            points[index] = Some(point(right, 0.0, sizes[index]));
            right -= GAP;
        }
    }
    right -= sizes[base].width;
    points[base] = Some(point(right, 0.0, sizes[base]));
    let mut height = ROW;
    if expanded && fit.overflow() {
        let mut x = 0.0;
        let mut y = ROW + GAP;
        for i in 0..controls {
            if fit.visible[i] {
                continue;
            }
            let index = LEFT + i * VARIANTS + 2;
            if x > 0.0 && x + sizes[index].width > width {
                x = 0.0;
                y += ROW + GAP;
            }
            points[index] = Some(point(x, y, sizes[index]));
            x += sizes[index].width + GAP;
            height = y + ROW;
        }
    }
    Arrangement { points, height }
}
