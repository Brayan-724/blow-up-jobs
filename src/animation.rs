use std::ops::Range;
use std::time::Duration;

use tokio::time::Instant;

use crate::ui::{Action, Arithmetic, Cast};

#[derive(Default)]
pub struct AnimationTicker {
    pub end_tick: usize,
    ended: bool,
    is_debugging: bool,
    reverse: bool,
    pub tick: usize,
    tick_duration: Option<(Duration, Instant)>,
}

impl AnimationTicker {
    pub fn debug(&mut self) {
        self.is_debugging = true;
    }

    pub fn ended(&self) -> bool {
        self.ended
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
        self.next_tick(Duration::ZERO);
    }

    pub fn end(&mut self) {
        self.ended = true;
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

    pub fn range(&self, range: Range<usize>) -> usize {
        self.tick.min(range.end).max(range.start) - range.start
    }

    pub fn is_on_range(&self, range: Range<usize>) -> bool {
        range.contains(&self.tick)
    }

    pub fn map<T>(&self, range: Range<usize>, target: Range<T>) -> T
    where
        T: Copy + Cast<usize> + Arithmetic<T>,
        usize: Cast<T>,
    {
        let target_len: usize = Cast::cast(target.end - target.start);
        let range_len = range.end - range.start;
        let value = self.range(range) as f32 / range_len as f32;

        Cast::cast((value * (target_len as f32)) as usize) + target.start
    }
}
