use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::animation::AnimationTicker;
use crate::job::Job;
use crate::theme::AppTheme;
use crate::ui::popup::{self, SharedPopupState};
use crate::ui::{Action, Component};

type Popups = (popup::EditPopup, popup::NewJobPopup, popup::RenamePopup);
pub type PopupsState = SharedPopupState<Popups>;

#[derive(Default)]
pub struct App {
    pub current_job: Option<usize>,
    pub jobs: Vec<Job>,
    pub theme: Arc<AppTheme>,
    pub anim: AnimationTicker,
    pub sidebar_anim: AnimationTicker,
    pub popup: PopupsState,

    pub popup_edit: popup::EditPopup,
    pub popup_new_job: popup::NewJobPopup,
    pub popup_rename: popup::RenamePopup,
}

impl App {
    pub fn new() -> Self {
        let mut anim = AnimationTicker::default();
        anim.len = 120;
        anim.start();

        let mut sidebar_anim = AnimationTicker::default();
        sidebar_anim.len = 40;
        sidebar_anim.next_tick(Duration::from_millis(20));

        Self {
            anim,
            sidebar_anim,
            ..Default::default()
        }
    }

    // Start sidebar animation on start with conditions
    pub fn update_sidebar(&mut self) {
        if self.sidebar_anim.running() {
            return;
        }

        let anim_tick = self.anim.tick;
        let is_on_bounds = anim_tick == 60 || anim_tick == 100;

        if is_on_bounds && !self.jobs.is_empty() {
            self.sidebar_anim.start();
        }
    }

    pub fn current_job(&self) -> Option<&Job> {
        self.jobs.get(self.current_job?)
    }

    pub fn current_job_mut(&mut self) -> Option<&mut Job> {
        self.jobs.get_mut(self.current_job?)
    }

    pub fn push_job(&mut self, job: Job) {
        if self.jobs.is_empty() {
            self.sidebar_anim.start();
        }

        let idx = self.jobs.len();
        self.jobs.push(job);
        self.current_job = Some(idx);
    }

    /// Returns whenever needs to waits
    pub async fn job_tick(&self) -> bool {
        if let Some(job) = self.current_job() {
            job.notify.notified().await;

            true
        } else {
            false
        }
    }

    pub fn kill_jobs(&mut self) {
        for job in &mut self.jobs {
            job.kill();
        }
    }
}

impl Component for App {
    type State = Self;

    async fn handle_event(state: &mut Self::State, event: Event) -> Action {
        PopupsState::handle_event(state, event.clone()).await?;

        match event {
            Event::Key(key_event) => Self::handle_key_events(state, key_event).await?,
            Event::Mouse(mouse_event) => Self::handle_mouse_events(state, mouse_event).await?,
            Event::Resize(_, _) => Action::Tick?,
            _ => {}
        }

        Self::propagate_event(state, event).await
    }

    async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('n') => {
                PopupsState::open::<popup::NewJobPopup>(state);
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

        PopupsState::draw(state, frame, area[1]);

        intro_overlay::render(state, frame);
    }
}
