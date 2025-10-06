use std::ops::Range;
use std::time::Duration;

use tokio::time::Instant;

use crate::ui::Action;

#[derive(Default)]
pub struct AnimationTicker {
    pub tick: usize,
    tick_duration: Option<(Duration, Instant)>,
    ended: bool,
}

impl AnimationTicker {
    pub fn ended(&self) -> bool {
        self.ended
    }

    pub fn update(&mut self, end_tick: usize) -> Action {
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

        self.tick += 1;

        if self.tick >= end_tick {
            self.end();
        }

        Action::Tick
    }

    pub fn next_tick(&mut self, duration: Duration) {
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

    pub fn end(&mut self) {
        self.ended = true;
    }

    pub fn reset(&mut self) {
        self.ended = false;
        self.tick = 0;
    }

    pub fn range(&self, range: Range<usize>) -> usize {
        self.tick.min(range.end).max(range.start) - range.start
    }

    pub fn is_on_range(&self, range: Range<usize>) -> bool {
        range.contains(&self.tick)
    }
}
