use std::ops::{self, Range};
use std::time::Duration;

use tokio::time::Instant;

use crate::ui::{Action, Arithmetic, Cast};

#[derive(Default)]
pub struct AnimationTicker {
    pub end_tick: usize,
    ended: bool,
    is_debugging: bool,
    reverse: bool,
    tick: usize,
    tick_duration: Option<(Duration, Instant)>,

    // Component things
    pub render_blink: bool,
}

impl AnimationTicker {
    pub fn debug(&mut self) {
        self.is_debugging = true;
    }

    pub fn ended(&self) -> bool {
        self.ended
    }

    pub fn tick(&self) -> AnimationTick {
        AnimationTick::new(self.tick, 0..self.end_tick)
    }

    pub fn update(&mut self) -> Action {
        if self.ended {
            return Action::Noop;
        }

        if let Some((duration, tick_end)) = self.tick_duration {
            if Instant::now() < tick_end {
                return Action::Noop;
            } else {
                self.tick_duration = Some((duration, Instant::now() + duration))
            }
        }

        if self.reverse {
            if self.tick == 0 {
                self.end();
                return Action::Noop;
            }

            self.tick -= 1;
        } else {
            self.tick += 1;

            if self.tick >= self.end_tick {
                self.end();
            }
        }

        if self.is_on_range(60..100) {
            let tick = self.range(60..100);
            let haundreds = tick.pow(2) / 300;

            self.render_blink = haundreds % 2 == 1;
        }

        Action::Tick
    }

    /// Toggle reverse mode
    pub fn reverse(&mut self) {
        self.reverse ^= true;
    }

    /// Start animation.
    /// Set tick duration as [Duration::ZERO]
    pub fn start(&mut self) {
        self.ended = false;
        self.render_blink = false;
        self.next_tick(Duration::ZERO);
    }

    pub fn end(&mut self) {
        self.ended = true;

        self.render_blink = true;
        self.tick_duration = None;
    }

    /// Set tick duration.
    /// This affects to current tick and the upcoming
    pub fn next_tick(&mut self, duration: Duration) {
        let duration = if self.is_debugging {
            duration + Duration::from_millis(50)
        } else {
            duration
        };
        self.tick_duration = Some((duration, Instant::now() + duration))
    }

    /// Waits for next tick.
    ///
    /// Returns whether is a pending tick
    pub async fn wait_tick(&self) -> bool {
        let Some((_, tick_end)) = self.tick_duration else {
            return false;
        };

        tokio::time::sleep_until(tick_end).await;

        true
    }

    pub fn range(&self, range: Range<usize>) -> AnimationTick {
        AnimationTick::new(
            self.tick.min(range.end).max(range.start) - range.start,
            range,
        )
    }

    pub fn is_on_range(&self, range: Range<usize>) -> bool {
        range.contains(&self.tick)
    }
}

#[derive(Clone, Copy)]
pub struct AnimationTick {
    pub tick: usize,
    pub range: TickRange,
}

impl ops::Deref for AnimationTick {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.tick
    }
}

impl AnimationTick {
    pub fn new(tick: usize, range: impl Into<TickRange>) -> Self {
        Self {
            tick,
            range: range.into(),
        }
    }

    pub fn ended(&self) -> bool {
        self.tick + self.range.start == self.range.end
    }

    pub fn tick(&self) -> usize {
        self.tick + self.range.start
    }

    pub fn range(&self) -> usize {
        self.range.end - self.range.start
    }

    pub fn map<T>(self, target: Range<T>) -> T
    where
        T: Copy + Cast<f32> + Arithmetic<T>,
        usize: Cast<T>,
    {
        let range_len = self.range();
        if range_len == 0 || target.end <= target.start {
            return Cast::cast(0);
        }

        let target_len: f32 = Cast::cast(target.end - target.start);

        let value = self.tick as f32 / range_len as f32;

        Cast::cast((value * target_len) as usize) + target.start
    }
}

#[derive(Clone, Copy)]
pub struct TickRange {
    pub start: usize,
    pub end: usize,
}

impl From<Range<usize>> for TickRange {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}
