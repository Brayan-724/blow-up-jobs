use crate::app::App;
use crate::job::{Job, JobStartError};
use crate::ui::prelude::*;

#[derive(Default)]
pub struct NewJobPopup {
    input: common::InputState,
    last_err: Option<JobStartError>,
}

impl Component for NewJobPopup {
    type State = App;

    fn on_mount(state: &mut Self::State) {
        state.popup_new_job.input.clear();
        state.popup_new_job.last_err = None;
    }

    async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
        match key {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => Action::Quit,
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                let content = state.popup_new_job.input.content.clone();

                let mut job = Job::new(content);

                if let Err(err) = job.start().await {
                    state.popup_new_job.last_err = Some(err);
                    Action::Tick
                } else {
                    let idx = state.jobs.len();
                    state.jobs.push(job);
                    state.current_job = Some(idx);

                    Action::Quit
                }
            }
            _ if state.popup_new_job.input.handle_key(key) => Action::Tick,
            _ => Action::Noop,
        }
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
        let area = area.inner(Margin::new(1, 0));
        let [title, input, error, _, buttons] = Layout::vertical([
            Constraint::Length(1), // Title
            Constraint::Length(3), // Input
            Constraint::Length(2), // Error
            Constraint::Length(1),
            Constraint::Length(1), // Buttons
        ])
        .split(area);

        frame.draw_stateless(Text::raw("New Job").style(state.theme.normal), title);

        frame.draw(
            common::Input::default().border_style(state.theme.border.dim()),
            input,
            &mut state.popup_new_job.input,
        );

        if let Some(ref err) = state.popup_new_job.last_err {
            let err_msg = err.to_text();

            Paragraph::new(err_msg)
                .wrap(Wrap { trim: true })
                .fg(Color::LightRed)
                .render(error, frame.buffer_mut());
        }

        popup::action_buttons(
            [("ESC", Color::LightRed), ("Enter", Color::Blue)],
            buttons,
            frame.buffer_mut(),
        );
    }
}

impl popup::Popup for NewJobPopup {
    const DURATION: usize = 7;
    const AUTO_CLOSE_EVENT: bool = false;

    fn build<'a: 'app, 'app>(
        island: popup::PopupBuilder<'a>,
        app: &'app mut App,
        area: Rect,
    ) -> popup::PopupBuilder<'a> {
        island
            .direction(Side::Left)
            .reserve(area.reduce_offset((10, 10)).set_width(35).centered((35, 8)))
            .border_style(app.theme.border)
    }
}
