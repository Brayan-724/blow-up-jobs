use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::animation::AnimationTicker;
use crate::job::Job;
use crate::theme::AppTheme;
use crate::ui::{Action, Component};

#[derive(Default)]
pub struct App {
    current_job: Option<usize>,
    pub jobs: Vec<Job>,
    pub theme: Arc<AppTheme>,
    pub anim: AnimationTicker,
}

impl App {
    pub fn new() -> Self {
        let mut anim = AnimationTicker::default();
        anim.end_tick = 100;
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

    async fn propagate_event(state: &mut Self::State, event: crossterm::event::Event) -> Action {
        Job::handle_event(state, event).await?;

        Action::Noop
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
        use crate::ui::prelude::*;

        let area = Layout::horizontal([
            Constraint::Length(1 + state.anim.range(60..90) as u16),
            Constraint::Percentage(100),
        ])
        .split(area);

        sidebar::render(state, area[0], frame);
        Job::draw(state, frame, area[1]);

        intro_overlay::render(state, frame);
    }
}
