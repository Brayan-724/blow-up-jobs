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

pub struct Blinker<'a, D, const STATEFUL: bool> {
    draw: D,
    marker: PhantomData<&'a ()>,
}

impl Blinker<'_, (), false> {
    pub fn new<'a, M, D: Drawable<'a, M>>(draw: D) -> Blinker<'a, D, { D::STATEFUL }> {
        Blinker {
            draw,
            marker: Default::default(),
        }
    }
}

impl<'a, M, D: Drawable<'a, M, State = ()>> Drawable<'a, M> for Blinker<'a, D, false> {
    type State = &'a AnimationTicker;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        if state.render_blink {
            self.draw.draw((), frame, area)
        }
    }
}

impl<'a, M, D: Drawable<'a, M>> Drawable<'a, M> for Blinker<'a, D, true> {
    type State = (&'a AnimationTicker, D::State);

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        if state.0.render_blink {
            self.draw.draw(state.1, frame, area)
        }
    }
}
