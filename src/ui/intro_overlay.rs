use std::time::Duration;

use crate::app::App;
use crate::ui::prelude::*;

pub fn render(state: &mut App, frame: &mut Frame) {
    if state.anim.ended() {
        return;
    }

    let tick = state.anim.range(1..50);

    let term_w = frame.area().width as i32;
    let term_h = frame.area().height as i32;

    for y in 0..term_h {
        for x in 0..term_w {
            let Some(c) = render_char_at(50 - tick, x - term_w / 2, y - term_h / 2) else {
                continue;
            };

            let cell = frame.buffer_mut()[(x as u16, y as u16)].set_char(c);

            if c.is_whitespace() {
                cell.set_bg(Color::Reset);
            }
        }
    }

    if tick > 45 {
        state.anim.next_tick(Duration::from_millis(1000));
    } else {
        state.anim.next_tick(Duration::from_millis(16));
    }
}

pub fn render_char_at(level: usize, x: i32, y: i32) -> Option<char> {
    match level {
        0 => return None,
        1 => return Some('🮮'),
        _ => {}
    }

    // range: 1..
    let level = level - 1;

    // Every (2 * level) in y, repeat the pattern
    // range 2..
    let level_y_mod = level * 2;

    // Every (2 + 2 * level) in x, repeat the pattern
    // range 4..
    let level_x_mod = 2 + level_y_mod;

    let invert = x.is_negative() ^ y.is_negative();

    let c_al = if x.is_negative() { '🯝' } else { '🯟' };
    let c_ar = if x.is_negative() { '🯟' } else { '🯝' };
    let c_sl = if invert { '\\' } else { '/' };
    let c_sr = if invert { '/' } else { '\\' };

    // Fix wrong mirror effect
    let x = if x.is_negative() { x + 1 } else { x };

    let x = x.saturating_abs() as usize % level_x_mod;
    let y = y.saturating_abs() as usize % level_y_mod;

    #[rustfmt::skip]
    match (x, y) {
        //   Top
        //   🯟🯝
        (x, 0) if x == level     => Some(c_al),
        (x, 0) if x == level + 1 => Some(c_ar),
        (_, 0)                   => None,
        //   Mid
        //  🯝   🯟
        (0, y) if y == level                         => Some(c_ar),
        (x, y) if y == level && x == level_x_mod - 1 => Some(c_al),
        (_, y) if y == level                         => None,
        // Mid-Top
        //  /   \
        (x, y) if y < level && x == level - y                   => Some(c_sl),
        (x, y) if y < level && level_x_mod - x - 1 == level - y => Some(c_sr),
        (_, y) if y < level                                     => None,
        //  Bottom
        //  \   /
        (x, y) if x == y - level                   => Some(c_sr),
        (x, y) if level_x_mod - x - 1 == y - level => Some(c_sl),
        _ => None,
    }
}
