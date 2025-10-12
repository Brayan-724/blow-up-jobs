use crate::animation::{AnimationTick, AnimationTicker};

use super::prelude::*;

pub fn pill(bg: Color, mut area: Rect, buf: &mut Buffer) -> Rect {
    if bg != Color::Reset && area.width > 0 {
        buf[(area.left(), area.y)].set_symbol("◖").set_fg(bg);
        if area.width > 1 {
            buf[(area.right() - 1, area.y)].set_symbol("◗").set_fg(bg);
        }
    }

    area.height = area.height.min(1);

    area.inner(Margin::horizontal(1))
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

const LAYOUT_SIDEBAR_WIDTH: u16 = 30;

pub struct AnimatedIsland<'a, D: Drawable<'a, M>, M, const STATEFUL: bool = { D::STATEFUL }> {
    border_style: Style,
    draw: D,
    marker: PhantomData<&'a M>,
    side: Side,
}

impl AnimatedIsland<'_, (), (), false> {
    pub fn new<'a, M, D: Drawable<'a, M>>(draw: D) -> AnimatedIsland<'a, D, M, { D::STATEFUL }> {
        AnimatedIsland {
            border_style: Style::new(),
            draw,
            marker: PhantomData,
            side: Side::Top,
        }
    }
}

impl<'a, M, D: Drawable<'a, M>, const STATEFUL: bool> AnimatedIsland<'a, D, M, STATEFUL> {
    pub fn border_style(mut self, style: impl Into<Style>) -> Self {
        self.border_style = style.into();
        self
    }

    pub fn direction(mut self, dir: Side) -> Self {
        self.side = dir;
        self
    }

    fn make_stateful(self) -> AnimatedIsland<'a, D, M, true> {
        AnimatedIsland {
            border_style: self.border_style,
            draw: self.draw,
            marker: self.marker,
            side: self.side,
        }
    }
}

impl<'a, M, D: Drawable<'a, M, State = ()>> Drawable<'a, M> for AnimatedIsland<'a, D, M, false> {
    type State = AnimationTick;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self.make_stateful().draw((state, ()), frame, area);
    }
}

impl<'a, M, D: Drawable<'a, M>> Drawable<'a, M> for AnimatedIsland<'a, D, M, true> {
    type State = (AnimationTick, D::State);

    fn draw(self, state: Self::State, frame: &mut Frame, mut area: Rect) {
        let is_inverted = area.left() < LAYOUT_SIDEBAR_WIDTH;

        if !state.0.ended() {
            match self.side {
                Side::Top => {
                    let bottom = state.0.map(0..area.bottom());
                    let y = bottom.saturating_sub(area.height);

                    area.y = y.saturating_add(1);
                    area.height = area.height.min(bottom);

                    // Is collapsed
                    if bottom == 0 {
                        return;
                    }
                }
                Side::Bottom => todo!(),
                Side::Left if is_inverted => {
                    let width = state.0.map(0..(LAYOUT_SIDEBAR_WIDTH - area.left()));
                    let right = LAYOUT_SIDEBAR_WIDTH - width;

                    area.x = right;
                    area.width = area.width.min(width);

                    // Is collapsed
                    if right == LAYOUT_SIDEBAR_WIDTH {
                        return;
                    }
                }
                Side::Left => {
                    let right = state.0.map(LAYOUT_SIDEBAR_WIDTH..area.right());
                    let x = right.saturating_sub(area.width).max(LAYOUT_SIDEBAR_WIDTH);

                    area.x = x.saturating_add(1);
                    area.width = area.width.min(right.saturating_sub(LAYOUT_SIDEBAR_WIDTH));

                    // Is collapsed
                    if right == LAYOUT_SIDEBAR_WIDTH {
                        return;
                    }
                }
                Side::Right => todo!(),
            }
        };

        area = area.intersection(frame.area().inner(Margin::both(1)));

        let border_area = area.outline((1, 1));

        frame.render_widget(Clear, border_area);
        self.draw.draw(state.1, frame, area);

        let mut borders = Borders::ALL;

        let frame_bottom = frame.area().bottom();

        if area.x > 30 {
            if area.y <= 1 {
                borders &= !Borders::TOP;
            }

            if area.bottom() >= frame_bottom.saturating_sub(2) {
                borders &= !Borders::BOTTOM;
            }
        }

        if !is_inverted && area.x <= LAYOUT_SIDEBAR_WIDTH + 1 {
            borders &= !Borders::LEFT;
        }

        if is_inverted && area.right() == LAYOUT_SIDEBAR_WIDTH {
            borders &= !Borders::RIGHT;
        }

        Block::new()
            .borders(borders)
            .border_set(border::ROUNDED)
            .border_style(self.border_style)
            .render(border_area, frame.buffer_mut());

        let buf = frame.buffer_mut();

        if !borders.contains(Borders::TOP) {
            buf[(border_area.left(), 0)].set_symbol(line::ROUNDED_TOP_RIGHT);
            buf[(border_area.right().saturating_sub(1), 0)].set_symbol(line::ROUNDED_TOP_LEFT);
        }

        if !borders.contains(Borders::BOTTOM) {
            let y = frame_bottom.saturating_sub(1);

            buf[(border_area.left(), y)].set_symbol(line::ROUNDED_BOTTOM_RIGHT);
            buf[(border_area.right().saturating_sub(1), y)].set_symbol(line::ROUNDED_BOTTOM_LEFT);
        }

        if !borders.contains(Borders::LEFT) {
            buf[(LAYOUT_SIDEBAR_WIDTH, border_area.top())].set_symbol(line::ROUNDED_BOTTOM_LEFT);
            buf[(LAYOUT_SIDEBAR_WIDTH, border_area.bottom().saturating_sub(1))]
                .set_symbol(line::ROUNDED_TOP_LEFT);
        }

        if !borders.contains(Borders::RIGHT) {
            buf[(LAYOUT_SIDEBAR_WIDTH, border_area.top())].set_symbol(line::ROUNDED_BOTTOM_RIGHT);
            buf[(LAYOUT_SIDEBAR_WIDTH, border_area.bottom().saturating_sub(1))]
                .set_symbol(line::ROUNDED_TOP_RIGHT);
        }
    }
}
