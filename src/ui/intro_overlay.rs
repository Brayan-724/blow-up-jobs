use std::time::Duration;

use crate::app::App;
use crate::ui::prelude::*;

pub fn render(state: &mut App, frame: &mut Frame) {
    if !state.anim.is_on_range(0..50) {
        return;
    }

    let tick = 49 - *state.anim.range(1..50);

    let term_w = frame.area().width.casted::<i32>();
    let term_h = frame.area().height.casted::<i32>();

    // Take the maximum difference of ticks in the borders of screen
    let max_tick = {
        let (vx, vy) = (term_w / 2, term_h / 2);
        (vx.casted::<f32>() + vy.casted::<f32>())
            .log2()
            .casted::<usize>()
    };

    for y in 0..term_h {
        for x in 0..term_w {
            let cell = &mut frame.buffer_mut()[(x.casted::<u16>(), y.casted::<u16>())];

            let (vx, vy) = (x - term_w / 2, y - term_h / 2);

            let offset_tick = (vx.casted::<f32>() + vy.casted::<f32>())
                .abs()
                .log2()
                .casted::<usize>();

            let Some(c) = render_char_at((tick + offset_tick).saturating_sub(max_tick), vx, vy)
            else {
                continue;
            };

            cell.reset();

            if !c.is_whitespace() {
                cell.set_char(c);
                cell.set_style(state.theme.accent.dim().not_underlined());
            }
        }
    }

    let tick_duration = match tick {
        49 => Duration::from_millis(0),
        10.. => Duration::ZERO,
        5.. => Duration::from_millis(25),
        2.. => Duration::from_millis(50),
        1 => Duration::from_millis(20),
        0 => unreachable!(),
    };

    state.anim.next_tick(tick_duration);
}

fn render_char_at(level: usize, x: i32, y: i32) -> Option<char> {
    let invert = x.is_negative() ^ y.is_negative();

    let c_al = if x.is_negative() { '🯝' } else { '🯟' };
    let c_ar = if x.is_negative() { '🯟' } else { '🯝' };
    let c_sl = if invert { '\\' } else { '/' };
    let c_sr = if invert { '/' } else { '\\' };

    // Fix wrong mirror effect
    let x = if x.is_negative() { x + 1 } else { x };

    match level {
        // No char
        0 => return None,
        // Minimum size
        1 => return Some('🮮'),
        // Spaceship char <>
        2 => {
            let y = y.saturating_abs() as usize % 2;

            let x = (i32::from(y == 0) + x.saturating_abs()).casted::<usize>() % 2;

            if x == 0 {
                return Some(c_al);
            }

            return Some(c_ar);
        }
        _ => {}
    }

    // Diamond shape

    // range: 1..
    let level = level - 2;

    // Every (2 * level) in y, repeat the pattern
    // range 2..
    let level_y_mod = level * 2;

    // Every (2 + 2 * level) in x, repeat the pattern
    // range 4..
    let level_x_mod = 2 + level_y_mod;

    let x = x.saturating_abs() as usize % level_x_mod;
    let y = y.saturating_abs() as usize % level_y_mod;

    #[rustfmt::skip]
    match (x, y) {
        //   Top
        //   🯟🯝
        (x, 0) if x == level     => Some(c_al),
        (x, 0) if x == level + 1 => Some(c_ar),
        (_, 0)                   => Some(' '),
        //   Mid
        //  🯝   🯟
        (0, y) if y == level                         => Some(c_ar),
        (x, y) if y == level && x == level_x_mod - 1 => Some(c_al),
        (_, y) if y == level                         => Some(' '),
        // Mid-Top
        //  /   \
        (x, y) if y < level && x == level - y                   => Some(c_sl),
        (x, y) if y < level && level_x_mod - x - 1 == level - y => Some(c_sr),
        (_, y) if y < level                                     => Some(' '),
        //  Bottom
        //  \   /
        (x, y) if x == y - level                   => Some(c_sr),
        (x, y) if level_x_mod - x - 1 == y - level => Some(c_sl),
        _ => Some(' '),
    }
}
