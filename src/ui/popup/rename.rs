use crate::app::App;
use crate::ui::prelude::*;

#[derive(Default)]
pub struct RenamePopup {
    input: common::InputState,
}

impl Component for RenamePopup {
    type State = App;

    fn on_mount(state: &mut Self::State) {
        if let Some(job) = state.current_job() {
            state.popup_rename.input.change_all(job.title.clone());
        } else {
            state.popup_rename.input.clear();
        }
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
                let content = state.popup_rename.input.content.clone();

                if content.is_empty() {
                    return Action::Tick;
                }

                if let Some(job) = state.current_job_mut() {
                    job.title = content;
                }

                Action::Quit
            }
            _ if state.popup_rename.input.handle_key(key) => Action::Tick,
            _ => Action::Noop,
        }
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
        let area = area.inner(Margin::new(1, 0));
        let [input, _, buttons] = Layout::vertical([
            Constraint::Length(3), // Input
            Constraint::Percentage(100),
            Constraint::Length(1), // Buttons
        ])
        .split(area);

        frame.draw(
            common::Input::default().border_style(state.theme.border.dim()),
            input,
            &mut state.popup_rename.input,
        );

        popup::action_buttons(
            [("ESC", Color::LightRed), ("Enter", Color::Blue)],
            buttons,
            frame.buffer_mut(),
        );
    }
}

impl popup::Popup for RenamePopup {
    const DURATION: usize = 7;
    const AUTO_CLOSE_EVENT: bool = false;

    fn build<'a: 'app, 'app>(
        island: popup::PopupBuilder<'a>,
        app: &'app mut App,
        area: Rect,
    ) -> popup::PopupBuilder<'a> {
        island
            .direction(Side::Left)
            .reserve(
                area.reduce((0, 20))
                    .set_width(35)
                    .offset(Offset::x(10))
                    .centered((35, 8)),
            )
            .border_style(app.theme.border)
    }
}
