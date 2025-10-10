use std::sync::Arc;

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::animation::AnimationTicker;
use crate::job::Job;
use crate::theme::AppTheme;
use crate::ui::{Action, Component};

#[derive(Default)]
pub struct App {
    pub current_job: Option<usize>,
    pub jobs: Vec<Job>,
    pub theme: Arc<AppTheme>,
    pub anim: AnimationTicker,
    // pub popup_anim: AnimationTicker,
}

impl App {
    pub fn new() -> Self {
        let mut anim = AnimationTicker::default();
        anim.end_tick = 120;
        anim.start();

        Self {
            anim,
            ..Default::default()
        }
    }

    pub fn current_job(&self) -> Option<&Job> {
        self.jobs.get(self.current_job?)
    }

    pub fn current_job_mut(&mut self) -> Option<&mut Job> {
        self.jobs.get_mut(self.current_job?)
    }

    /// Returns whether needs to waits
    pub async fn job_tick(&self) -> bool {
        if let Some(job) = self.current_job() {
            job.notify.notified().await;

            true
        } else {
            false
        }
    }

    pub async fn kill_jobs(&mut self) {
        tokio_scoped::scope(|scope| {
            for job in &mut self.jobs {
                scope.spawn(async {
                    job.kill().await;
                });
            }
        });
    }
}

impl Component for App {
    type State = Self;

    async fn handle_key_events(_: &mut Self::State, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            _ => Action::Noop,
        }
    }

    async fn propagate_event(state: &mut Self::State, event: Event) -> Action {
        Job::handle_event(state, event).await?;

        Action::Noop
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
        use crate::ui::prelude::*;

        let area =
            Layout::horizontal([Constraint::Length(30), Constraint::Percentage(100)]).split(area);

        Job::draw(state, frame, area[1]);
        sidebar::render(state, area[0], frame);

        // frame.draw_popup();

        if state.anim.is_on_range(50..120) || state.anim.ended() {
            let area = area[1].inner(Margin::both(20));

            frame.draw(
                common::AnimatedIsland::new(|area: Rect, buf: &mut Buffer| {
                    Block::new().borders(Borders::all()).render(area, buf);
                })
                .direction(Side::Left)
                .border_style(state.theme.border),
                area,
                state.anim.range(60..120),
            );
        }

        intro_overlay::render(state, frame);
    }
}
