use crate::app::App;
use crate::ui::prelude::*;

#[derive(Default)]
pub struct NewJobPopup {
    input: common::InputState,
}

impl Component for NewJobPopup {
    type State = App;

    fn on_mount(state: &mut Self::State) {
        state.popup_new_job.input.clear();
    }

    async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
        match key {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => Action::Quit,
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Action::Quit,
            _ if state.popup_new_job.input.handle_key(key) => Action::Tick,
            _ => Action::Noop,
        }
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
        let area = area.inner(Margin::new(1, 0));
        let area = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Percentage(100),
            Constraint::Length(1),
        ])
        .split(area);

        frame.draw_stateless(Text::raw("New Job").style(state.theme.normal), area[0]);

        frame.draw_stateless(
            Block::new()
                .borders(Borders::BOTTOM)
                .border_style(state.theme.border.dim()),
            area[1],
        );

        {
            common::Input.render(area[1], frame.buffer_mut(), &mut state.popup_new_job.input);
            state.popup_new_job.input.sync_cursor(frame, area[1]);
        }

        {
            let area = Layout::horizontal([Constraint::Length(11), Constraint::Length(11)])
                .flex(Flex::End)
                .split(area[3]);

            Text::raw("ESC").centered().bold().render(
                common::round_button(Color::LightRed, area[0], frame.buffer_mut()),
                frame.buffer_mut(),
            );
            Text::raw("Enter").centered().bold().render(
                common::round_button(Color::Blue, area[1], frame.buffer_mut()),
                frame.buffer_mut(),
            );
        }
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
            .reserve(area.set_width(35).centered((35, 8)).offset(Offset::x(10)))
            .border_style(app.theme.border)
    }
}
