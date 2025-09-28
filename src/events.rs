use std::io;
use std::task::Poll;
use std::time::Duration;

use crossterm::event;
use futures::Stream;

pub struct TermEvents;

const TICK_DURATION: Duration = Duration::from_millis(100);

impl Future for TermEvents {
    type Output = io::Result<event::Event>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if event::poll(TICK_DURATION)? {
            Poll::Ready(event::read())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl Stream for TermEvents {
    type Item = io::Result<event::Event>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.poll(cx).map(Some)
    }
}
