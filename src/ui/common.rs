use crate::animation::AnimationTicker;

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

pub struct Blinker<'a, D> {
    draw: D,
    marker: PhantomData<&'a ()>,
}

impl Blinker<'_, ()> {
    pub fn new<'a, M, D: Drawable<'a, M, State = ()>>(draw: D) -> Blinker<'a, D> {
        Blinker {
            draw,
            marker: Default::default(),
        }
    }
}

impl<'a, M, D: Drawable<'a, M, State = ()>> Drawable<'a, M> for Blinker<'a, D> {
    type State = &'a AnimationTicker;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        if state.render_blink {
            self.draw.draw((), frame, area)
        }
    }
}
