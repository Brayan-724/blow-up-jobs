use std::sync::Arc;

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::animation::AnimationTicker;
use crate::job::Job;
use crate::theme::AppTheme;
use crate::ui::popup::{self, NewJobPopup, SharedPopupState};
use crate::ui::{Action, Component};

type Popups = (popup::NewJobPopup,);

#[derive(Default)]
pub struct App {
    pub current_job: Option<usize>,
    pub jobs: Vec<Job>,
    pub theme: Arc<AppTheme>,
    pub anim: AnimationTicker,
    pub popup: SharedPopupState<Popups>,
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

    async fn handle_event(state: &mut Self::State, event: Event) -> Action {
        SharedPopupState::<Popups>::handle_event(state, event.clone()).await?;

        match event {
            Event::Key(key_event) => Self::handle_key_events(state, key_event).await?,
            Event::Mouse(mouse_event) => Self::handle_mouse_events(state, mouse_event).await?,
            Event::Resize(_, _) => Action::Tick?,
            _ => {}
        };

        Self::propagate_event(state, event).await
    }

    async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('n') => {
                state.popup.open::<NewJobPopup>();
                Action::Tick
            }
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

        SharedPopupState::<Popups>::draw(state, frame, area[1]);

        intro_overlay::render(state, frame);
    }
}
