use std::ops::{self, Range};
use std::time::Duration;

use tokio::time::Instant;

use crate::ui::{Action, Arithmetic, Cast, Casted};

#[derive(Default)]
pub enum AnimationState {
    #[default]
    Stop,
    Running {
        next_tick: Instant,
    },
}

#[derive(Default)]
pub struct AnimationTicker {
    is_debugging: bool,
    pub len: usize,
    reverse: bool,
    state: AnimationState,
    pub tick: usize,
    tick_duration: Duration,

    // Component things
    pub render_blink: bool,
}

impl AnimationTicker {
    pub fn debug(&mut self) {
        self.is_debugging = true;
    }

    pub fn running(&self) -> bool {
        matches!(self.state, AnimationState::Running { .. })
    }

    pub fn stopped(&self) -> bool {
        matches!(self.state, AnimationState::Stop)
    }

    pub fn tick(&self) -> AnimationTick {
        AnimationTick::new(self.tick, 0..self.len)
    }

    pub fn update(&mut self) -> Action {
        match self.state {
            AnimationState::Running { next_tick } if Instant::now() > next_tick => {
                self.state = AnimationState::Running {
                    next_tick: Instant::now() + self.tick_duration,
                };
            }
            _ => return Action::Noop,
        }

        if self.reverse {
            if self.tick == 0 {
                self.end();
                return Action::Noop;
            }

            self.tick -= 1;
        } else {
            self.tick += 1;

            if self.tick >= self.len {
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
    /// Set tick duration as [`Duration::ZERO`]
    pub fn start(&mut self) {
        self.render_blink = false;
        self.state = AnimationState::Running {
            next_tick: Instant::now() + self.tick_duration,
        }
    }

    pub fn end(&mut self) {
        self.state = AnimationState::Stop;
        self.render_blink = true;
    }

    /// Set tick duration.
    /// This affects to current tick and the upcoming
    pub fn next_tick(&mut self, duration: Duration) {
        let duration = if self.is_debugging {
            duration + Duration::from_millis(50)
        } else {
            duration
        };

        self.tick_duration = duration;
    }

    /// Waits for next tick.
    ///
    /// Returns whether is a pending tick
    pub async fn wait_tick(&self) -> bool {
        let AnimationState::Running { next_tick } = self.state else {
            return false;
        };

        tokio::time::sleep_until(next_tick).await;

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
        f32: Cast<T>,
    {
        let range_len = self.range();
        if range_len == 0 || target.end <= target.start {
            return target.start;
        }

        let target_len = (target.end - target.start).casted::<f32>();

        let value = self.tick.casted::<f32>() / range_len.casted::<f32>();

        (value * target_len).casted::<T>() + target.start
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
