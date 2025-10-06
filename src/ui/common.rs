use super::prelude::*;

pub fn pill(bg: Color, mut area: Rect, buf: &mut Buffer) -> Rect {
    if bg != Color::Reset {
        buf[(area.left(), area.y)].set_symbol("◖").set_fg(bg);
        buf[(area.right() - 1, area.y)].set_symbol("◗").set_fg(bg);
    }

    area.x += 1;
    area.width -= 2;
    area.height = 1;

    area
}
