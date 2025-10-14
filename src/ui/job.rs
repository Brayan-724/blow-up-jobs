use crossterm::event::KeyCode;

use crate::app::{App, PopupsState};
use crate::job::Job;
use crate::ui::prelude::*;
use crate::vterm;

impl Component for Job {
    type State = App;

    async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => Action::Noop,
            KeyCode::Tab => {
                state.current_job = state
                    .current_job
                    .take()
                    .map(|i| i + 1)
                    .unwrap_or(0)
                    .checked_rem(state.jobs.len());

                Action::Tick
            }
            KeyCode::BackTab => {
                state.current_job = state
                    .current_job
                    .take()
                    .unwrap_or_else(|| state.jobs.len().min(1))
                    .checked_sub(1)
                    .or_else(|| state.jobs.len().checked_sub(1));

                Action::Tick
            }
            KeyCode::Char('k') if let Some(job) = state.current_job_mut() => {
                job.kill().await;
                Action::Tick
            }
            KeyCode::Char('m') if state.current_job.is_some() => {
                PopupsState::open::<popup::RenamePopup>(state);
                Action::Tick
            }
            KeyCode::Char('e') if state.current_job.is_some() => {
                PopupsState::open::<popup::EditPopup>(state);
                Action::Tick
            }
            KeyCode::Char('r') => {
                let start = if let Some(job) = state.current_job_mut() {
                    job.restart().await
                } else {
                    Ok(())
                };

                _ = start;

                Action::Tick
            }
            _ => Action::Noop,
        }
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::new()
                .borders(Borders::all())
                .border_set(border::ROUNDED)
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
        frame.draw(render_loading_bar, area[0], &state);

        frame.draw(render_job, area[1], state);

        frame.draw(render_footer, area[2], &state);
        frame.draw(render_loading_bar, area[2], &state);
    }
}

fn render_loading_bar(state: &App, area: Rect, buf: &mut Buffer) {
    if !state.anim.ended() {
        let transparent_area = if state.anim.is_on_range(80..121) {
            area.inner(Margin::horizontal(
                area.width - state.anim.range(80..120).map(0..area.width),
            ))
        } else {
            // Do not render anything
            Rect::ZERO
        };
        let pill_area = area.inner(Margin::horizontal(
            area.width - state.anim.range(60..100).map(0..area.width),
        ));

        let bg = state
            .theme
            .border
            .fg
            .or(state.theme.accent.fg)
            .unwrap_or(Color::Black);

        let opaque_area = common::pill(bg, pill_area, buf);
        for x in area.left()..area.right() {
            let pos = Position { x, y: area.y };

            if transparent_area.contains(pos) {
                continue;
            }

            let cell = &mut buf[pos];

            if !pill_area.contains(pos) {
                cell.reset();
                continue;
            }

            if !opaque_area.contains(pos) {
                continue;
            }

            cell.reset();
            cell.set_bg(bg);
        }
    }
}

fn render_help(state: &App, area: Rect, buf: &mut Buffer) {
    let area = area.inner(Margin::horizontal(1));

    Line::from(vec![
        "r".to_span().style(state.theme.keybind_accent),
        "estart ".to_span().style(state.theme.normal),
        "k".to_span().style(state.theme.keybind_accent),
        "ill ".to_span().style(state.theme.normal),
        "e".to_span().style(state.theme.keybind_accent),
        "dit ".to_span().style(state.theme.normal),
        "rena".to_span().style(state.theme.normal),
        "m".to_span().style(state.theme.keybind_accent),
        "e ".to_span().style(state.theme.normal),
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
    let area = area.inner(Margin::horizontal(1));
    frame.draw_stateless(
        Line::from("Apika Luca".to_span().style(state.theme.accent))
            .centered()
            .bold(),
        area,
    );

    let Some(status) = state.current_job().and_then(|j| j.status()) else {
        return;
    };

    let buf = frame.buffer_mut();

    let left0 = "Exit code: ";
    let left1 = status.to_string();
    let right = "200ms";

    let area = Layout::horizontal([
        Constraint::Length(left0.len() as u16 + left1.len() as u16 + 2),
        Constraint::Length(right.len() as u16),
    ])
    .flex(Flex::SpaceBetween)
    .split(area);

    Line::raw(left0).bold().render(area[0], buf);
    Line::raw(right).right_aligned().bold().render(area[1], buf);

    let exit_color = if status == 0 {
        Color::Green
    } else {
        Color::Red
    };

    let exit_area = area[0]
        .offset(Offset::x(left0.len() as i32))
        .set_width(left1.len() as u16 + 2);

    common::pill(exit_color, exit_area, buf);

    Text::raw(left1)
        .bold()
        .bg(exit_color)
        .render(exit_area.inner(Margin::horizontal(1)), buf);
}
