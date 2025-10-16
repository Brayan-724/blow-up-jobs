use crate::app::App;
use crate::theme::AppTheme;
use crate::ui::prelude::*;

pub fn render(state: &mut App, area: Rect, frame: &mut Frame) {
    let area = Layout::vertical([Constraint::Percentage(100), Constraint::Length(5)]).split(area);

    frame.draw(
        common::Blinker::new(render_help),
        area[1].inner_x(1).reduce((2, 0)),
        (&state.anim, state.theme.as_ref()),
    );

    frame.draw(render_sidebar, area[0], state);
}

fn render_sidebar(state: &App, frame: &mut Frame, area: Rect) {
    let items = state.jobs.len() as u16;

    let area = area
        .inner_y(2)
        .inner_x(1)
        .reduce((2, 0))
        .set_height(items * 2)
        .offset(Offset::y(1 - state.sidebar_anim.range(35..40).map(0..2i32)));

    frame.draw(
        common::AnimatedIsland::new(render_sidebar_jobs)
            .direction(Side::Left)
            .border_style(state.theme.border),
        area,
        (state.sidebar_anim.range(0..29), state),
    );
}

fn render_sidebar_jobs(state: &App, frame: &mut Frame, area: Rect) {
    for (idx, item) in state.jobs.iter().enumerate() {
        let area = area.inner_y(idx as i32 * 2).set_height(1);

        {
            let content = if let Some(status) = item.status() {
                vec![
                    "200ms ".to_span(),
                    "●".to_span().fg(if status == 0 {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ]
            } else {
                vec!["●".to_span().fg(Color::Gray)]
            };

            frame.draw(
                common::Blinker::new(Line::from(content).right_aligned()),
                area.inner_y(1).reduce((1, 0)).set_height(1),
                &state.anim,
            );
        }

        let style = if state.current_job.is_some_and(|job| job == idx) {
            state.theme.job_selected
        } else {
            state.theme.job_normal
        };

        let bg = style.bg.unwrap_or(Color::Reset);

        let area = common::pill(bg, area, frame.buffer_mut());
        frame.draw_stateless(item.title.to_text().style(style), area);
    }
}

fn render_help(theme: &AppTheme, area: Rect, buf: &mut Buffer) {
    Text::from(vec![
        Line::from(vec![
            "c".to_span().style(theme.keybind_accent),
            "onfig".to_span().style(theme.normal),
        ]),
        Line::from(vec![
            "stop ".to_span().style(theme.normal),
            "a".to_span().style(theme.keybind_accent),
            "ll processes".to_span().style(theme.normal),
        ]),
        Line::from(vec![
            "n".to_span().style(theme.keybind_accent),
            "ew process".to_span().style(theme.normal),
        ]),
        Line::from(vec![
            "q".to_span().style(theme.keybind_accent),
            "uit".to_span().style(theme.normal),
        ]),
    ])
    .render(area, buf);
}
