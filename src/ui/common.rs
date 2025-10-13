use std::cmp::Ordering;

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

pub fn round_button(bg: Color, mut area: Rect, buf: &mut Buffer) -> Rect {
    area.height = area.height.min(1);

    let orig_width = area.width;
    let area = area.inner(Margin::horizontal(2));

    if area.height == 0 {
        return area;
    }

    if bg != Color::Reset && orig_width > 2 {
        buf[(area.left(), area.y)].set_char(' ');
        buf[(area.left() - 2, area.y)].set_symbol("🭤🭓").set_fg(bg);
        if orig_width > 3 {
            buf[(area.right(), area.y)].set_symbol("🭍🬾").set_fg(bg);

            for x in area.left()..area.right() {
                buf[(x, area.y)].set_bg(bg);
            }
        }
    }

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

#[derive(Default)]
pub struct InputState {
    content: String,
    /// Current cursor position
    cursor: usize,
    /// Horizontal scroll offset
    offset: usize,
    /// Maybe selection
    /// (most left, length)
    selection: Option<(usize, usize)>,
}

impl InputState {
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
        self.offset = 0;
        _ = self.selection.take();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);
        let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('a') if ctrl_pressed => {
                self.select_all();
                true
            }
            KeyCode::Char(c) if shift_pressed => {
                self.push(c.to_ascii_uppercase());
                true
            }
            KeyCode::Char(c) => {
                self.push(c);
                true
            }
            KeyCode::Backspace => {
                self.backspace();
                true
            }
            KeyCode::Left => {
                self.move_left(shift_pressed);
                true
            }
            KeyCode::Right => {
                self.move_right(shift_pressed);
                true
            }
            KeyCode::Home => {
                self.move_home(shift_pressed);
                true
            }
            KeyCode::End => {
                self.move_end(shift_pressed);
                true
            }
            _ => false,
        }
    }

    pub fn push(&mut self, c: char) {
        if let Some((start, len)) = self.selection.take() {
            self.content.replace_range(
                start..start + len,
                c.encode_utf8(&mut [0; char::MAX_LEN_UTF8]),
            );
            self.cursor = start + 1;
        } else {
            self.content.insert(self.cursor, c);
            self.cursor += 1;
        }
        _ = self.selection.take();
    }

    pub fn backspace(&mut self) {
        if let Some((start, len)) = self.selection.take() {
            self.content.replace_range(start..start + len, "");
            self.cursor = start;
        } else {
            self.cursor = self.cursor.saturating_sub(1);
            self.offset = self.offset.saturating_sub(1);

            if self.cursor < self.content.len() {
                _ = self.content.remove(self.cursor);
            }
        }
    }

    fn update_selection(&mut self, select: bool, orig_cursor: usize) {
        if !select {
            _ = self.selection.take();
            return;
        }

        // No cursor movement
        if orig_cursor == self.cursor {
            return;
        }

        let (start, len) = self.selection.take().unzip();
        let start = start.unwrap_or(orig_cursor);
        let end = start + len.unwrap_or(0);

        let was_on_right_side = orig_cursor > start;
        let is_moving_right = self.cursor > orig_cursor;

        let cursor_direction = match (was_on_right_side, self.cursor.cmp(&end), is_moving_right) {
            (false, Ordering::Less, true) => Ordering::Greater,
            (_, _, _) => Ordering::Less,
            // Expanded for debugging purposes:
            // (false, Ordering::Less, false) => Ordering::Less,
            // (false, Ordering::Less, true) => Ordering::Greater,
            //
            // (false, Ordering::Equal, false) => todo!(),
            // (false, Ordering::Equal, true) => Ordering::Less,
            //
            // (false, Ordering::Greater, false) => todo!(),
            // (false, Ordering::Greater, true) => Ordering::Less,
            //
            // (true, Ordering::Less, false) => Ordering::Less,
            // (true, Ordering::Less, true) => todo!(),
            //
            // (true, Ordering::Equal, false) => todo!(),
            // (true, Ordering::Equal, true) => todo!(),
            //
            // (true, Ordering::Greater, false) => todo!(),
            // (true, Ordering::Greater, true) => Ordering::Less,
        };

        let new_start = match cursor_direction {
            Ordering::Less => self.cursor.min(start),
            Ordering::Equal => return,
            Ordering::Greater => self.cursor.max(start),
        };

        let len = start.abs_diff(self.cursor).max(end.abs_diff(self.cursor));

        if len != 0 {
            self.selection = Some((new_start, len));
        }
    }

    pub fn move_to(&mut self, select: bool, cursor: usize) {
        let orig_cursor = self.cursor;
        self.cursor = cursor.min(self.content.len());
        self.update_selection(select, orig_cursor);
    }

    pub fn move_left(&mut self, select: bool) {
        self.move_to(select, self.cursor.saturating_sub(1));
    }

    pub fn move_right(&mut self, select: bool) {
        self.move_to(select, self.cursor.saturating_add(1));
    }

    pub fn move_home(&mut self, select: bool) {
        self.move_to(select, 0);
    }

    pub fn move_end(&mut self, select: bool) {
        self.move_to(select, self.content.len());
    }

    pub fn select_all(&mut self) {
        self.cursor = 0;
        self.selection = Some((0, self.content.len()))
    }

    pub fn sync_cursor(&mut self, frame: &mut Frame, area: Rect) {
        let cursor_offset = (self.cursor as isize) - self.offset as isize;
        let area = area.inner(Margin::both(1));
        let mut pos = area.as_position();
        pos.x = pos.x.saturating_add(cursor_offset.unsigned_abs() as u16);
        frame.set_cursor_position(pos);
    }
}

pub struct Input;

impl StatefulWidget for Input {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = area.inner(Margin::both(1));

        let cursor_offset = (state.cursor as isize) - state.offset as isize;

        if cursor_offset >= area.width as isize {
            state.offset += (cursor_offset as usize).saturating_sub(area.width as usize);
        } else if cursor_offset < 0 as isize {
            state.offset = state.offset.saturating_add_signed(cursor_offset);
        }

        let viewport_start = state.offset;
        let viewport_end = state.offset + (area.width as usize).min(state.content.len());

        if let Some(str) = state.content.get(viewport_start..viewport_end) {
            Text::raw(str).render(area, buf);

            if let Some((start, len)) = state.selection {
                let end = (start + len)
                    .saturating_sub(viewport_start)
                    .min(area.width as usize);
                let start = start
                    .saturating_sub(viewport_start)
                    .min(area.width as usize);
                let len = end - start;

                if len == 0 {
                    return;
                }

                let start = start as u16 + area.x;
                for x in 0..len as u16 {
                    buf[(start + x, area.y)].set_style(Style::new().fg(Color::Cyan).reversed());
                }
            }
        }
    }
}
