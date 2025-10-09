use crossterm::event::KeyCode;

use crate::animation::AnimationTicker;
use crate::app::App;
use crate::job::Job;
use crate::ui::prelude::*;
use crate::vterm;

impl Component for Job {
    type State = App;

    async fn handle_key_events(_: &mut Self::State, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => Action::Noop,
            // KeyCode::Char('r') => {}
            _ => Action::Noop,
        }
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new()
                .border_set(border::ROUNDED)
                .borders(!Borders::LEFT)
                .border_style(state.theme.border),
            area,
        );

        let area = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(1),
        ])
        .split(area.inner(Margin::both(1)));

        frame.draw(render_help, area[0], &state);
        frame.draw(render_job, area[1], state);
        frame.draw(render_footer, area[2], &state);
    }
}

#[must_use = "is rendering loading bar"]
fn render_loading_bar(state: &App, area: Rect, buf: &mut Buffer) -> bool {
    if !state.anim.ended() {
        let area = area.inner(Margin::horizontal(
            area.width - state.anim.map(60..100, 0..area.width),
        ));

        let bg = state
            .theme
            .border
            .fg
            .or(state.theme.accent.fg)
            .unwrap_or(Color::Black);

        let area = common::pill(bg, area, buf);
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, area.y)];
            cell.reset();
            cell.set_bg(bg);
        }

        true
    } else {
        false
    }
}

fn render_help(state: &App, area: Rect, buf: &mut Buffer) {
    if render_loading_bar(state, area, buf) {
        return;
    }

    let area = area.inner(Margin::horizontal(1));

    Line::from(vec![
        "r".to_span().style(state.theme.keybind_accent),
        "estart ".to_span().style(state.theme.normal),
        "k".to_span().style(state.theme.keybind_accent),
        "ill ".to_span().style(state.theme.normal),
        "e".to_span().style(state.theme.keybind_accent),
        "dit ".to_span().style(state.theme.normal),
    ])
    .render(area, buf);
}

fn render_job(state: &mut App, frame: &mut Frame, area: Rect) {
    let area = area.inner(Margin::both(1));

    if let Some(job) = state.current_job_mut() {
        render_vterm(job, frame, area);
    } else if state.anim.render_blink {
        render_welcome_screen(state, area, frame.buffer_mut());
    }
}

fn render_welcome_screen(state: &mut App, area: Rect, buf: &mut Buffer) {
    let area = Layout::vertical([
        Constraint::Max(3),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .split(area);

    Text::from_iter(
        [
            Line::from_iter(
                [
                    Span::from("Welcome").style(state.theme.accent),
                    Span::from(" to your").style(state.theme.normal),
                ]
                .into_iter(),
            ),
            Line::from_iter(
                [
                    Span::from("blowing up ").style(state.theme.normal),
                    Span::from("jobs").style(state.theme.accent),
                ]
                .into_iter(),
            ),
        ]
        .into_iter(),
    )
    .centered()
    .render(area[1], buf);

    let area = Layout::horizontal([Constraint::Length(22)])
        .flex(Flex::Center)
        .split(area[3])[0];

    Text::from_iter(
        [
            Line::from_iter(
                [
                    Span::from("<Enter>").style(state.theme.accent),
                    Span::from(" Recover jobs").style(state.theme.normal),
                ]
                .into_iter(),
            ),
            Line::from_iter(
                [
                    Span::from("<Tab>").style(state.theme.accent),
                    Span::from(" Go to first job").style(state.theme.normal),
                ]
                .into_iter(),
            ),
        ]
        .into_iter(),
    )
    .render(area, buf);
}

fn render_vterm(job: &mut Job, frame: &mut Frame, area: Rect) {
    let size = area.as_size();
    job.size = size;
    if let Some(ref job) = job.running {
        _ = job.pty.resize(portable_pty::PtySize {
            rows: size.height,
            cols: size.width,
            pixel_width: 0,
            pixel_height: 0,
        });
        job.vterm.blocking_write().set_size(size.height, size.width);
        frame.render_widget(
            vterm::VTermWidget::new(job.vterm.blocking_read().screen()),
            area,
        );
    } else {
        frame.render_widget(Text::from("No running job"), area);
    }
}

fn render_footer(state: &App, frame: &mut Frame, area: Rect) {
    if render_loading_bar(state, area, frame.buffer_mut()) {
        return;
    }

    frame.draw_stateless(
        Line::from("Apika Luca".to_span().style(state.theme.accent))
            .centered()
            .bold(),
        area,
    );

    let buf = frame.buffer_mut();

    let left = "Exit code: 2";
    let right = "200ms";

    let area = Layout::horizontal([
        Constraint::Length(left.len() as u16 + 2),
        Constraint::Length(right.len() as u16 + 2),
    ])
    .flex(Flex::SpaceBetween)
    .split(area);

    Line::raw(left)
        .bg(Color::Red)
        .bold()
        .render(common::pill(Color::Red, area[0], buf), buf);

    Line::raw(right)
        .right_aligned()
        .bold()
        .render(common::pill(Color::Reset, area[1], buf), buf);
}
