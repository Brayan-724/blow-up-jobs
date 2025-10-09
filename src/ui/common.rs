use super::prelude::*;

pub fn pill(bg: Color, mut area: Rect, buf: &mut Buffer) -> Rect {
    if bg != Color::Reset && area.width > 0 {
        buf[(area.left(), area.y)].set_symbol("◖").set_fg(bg);
        if area.width > 1 {
            buf[(area.right() - 1, area.y)].set_symbol("◗").set_fg(bg);
        }
    }

    area.x += 1;
    area.width = area.width.saturating_sub(2);
    area.height = area.height.min(1);

    area
}
