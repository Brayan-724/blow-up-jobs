use crossterm::event::KeyCode;

use crate::app::App;
use crate::job::Job;
use crate::ui::prelude::*;
use crate::vterm;

impl Component for Job {
    type State = App;

    async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
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
                .border_style(Color::LightMagenta),
            area,
        );

        let area = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(1),
        ])
        .split(area.inner(Margin::both(1)));

        frame.draw(render_help, area[0], ());
        frame.draw(render_job, area[1], state);
        frame.draw(render_footer, area[2], ());
    }
}

fn render_help(area: Rect, buf: &mut Buffer) {
    let area = area.inner(Margin::horizontal(1));

    Line::from(vec![
        "r".to_span().fg(Color::Blue).bold(),
        "estart ".to_span(),
        "k".to_span().fg(Color::Blue).bold(),
        "ill ".to_span(),
        "e".to_span().fg(Color::Blue).bold(),
        "dit ".to_span(),
    ])
    .render(area, buf);
}

fn render_job(state: &mut App, frame: &mut Frame, area: Rect) {
    let area = area.inner(Margin::both(1));

    if let Some(job) = state.current_job_mut() {
        render_vterm(job, frame, area);
    } else {
        render_welcum_screen(state, area, frame.buffer_mut());
    }
}

fn render_welcum_screen(state: &mut App, area: Rect, buf: &mut Buffer) {
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

fn render_footer(area: Rect, buf: &mut Buffer) {
    Line::from("Apika Luca".to_span().fg(Color::Magenta))
        .centered()
        .bold()
        .render(area, buf);

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
