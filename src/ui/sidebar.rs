use crate::animation::AnimationTicker;
use crate::app::App;
use crate::theme::AppTheme;
use crate::ui::prelude::*;

pub fn render(state: &mut App, area: Rect, frame: &mut Frame) {
    let area = Layout::vertical([Constraint::Percentage(100), Constraint::Length(5)]).split(area);

    frame.draw(
        common::Blinker::new(render_help),
        area[1].offset_x(1).reduce((2, 0)),
        (&state.anim, state.theme.as_ref()),
    );

    frame.draw(
        render_sidebar,
        area[0].offset_x(30 - state.anim.range(60..90) as i32),
        state,
    );
}

fn render_sidebar(state: &App, frame: &mut Frame, area: Rect) {
    let items = [
        "Process ABCDEFGHIJKLMNOPQ",
        "Process B",
        "Process C",
        "Process C",
    ];

    let area = area.offset_y(2).set_height(items.len() as u16 * 2 + 2);

    {
        let area = area.reduce((3, 0)).offset(Offset {
            x: 1,
            y: 1 - state.anim.map(95..100, 0..2i32),
        });

        for (idx, item) in items.iter().enumerate() {
            let area = area.offset_y(idx as i32 * 2).set_height(1);

            frame.draw(
                common::Blinker::new(
                    Line::from(vec![
                        "200ms ".to_span(),
                        "●".to_span().fg(if idx % 2 == 0 {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                    ])
                    .right_aligned(),
                ),
                area.offset_y(1).reduce((1, 0)).set_height(1),
                &state.anim,
            );

            let style = if idx == 1 {
                state.theme.job_selected
            } else {
                state.theme.job_normal
            };

            let bg = style.bg.unwrap_or(Color::Reset);

            let area = common::pill(bg, area, frame.buffer_mut());
            frame.draw_stateless(item.to_text().style(style), area);
        }
    }

    render_sidebar_borders(&state, area, frame.buffer_mut());
}

fn render_sidebar_borders(app: &App, area: Rect, buf: &mut Buffer) {
    let x = area.right() - 1;

    let is_island = area.width >= 30;

    let border_block = Block::new()
        .border_set(border::ROUNDED)
        .border_style(app.theme.border);

    if is_island {
        for y in area.top()..area.bottom() {
            buf[(x, y)]
                .set_symbol(line::VERTICAL)
                .set_style(app.theme.border);
        }

        let area = area
            .reduce((area.width.saturating_sub(30), 0))
            .offset(Offset::y(-app.anim.map(95..100, 0..2i32)));

        border_block.borders(Borders::all()).render(area, buf);
    } else {
        border_block.borders(!Borders::RIGHT).render(area, buf);

        if area.width > 1 {
            buf[(x, area.top())]
                .set_symbol(line::ROUNDED_TOP_LEFT)
                .set_style(app.theme.border);

            buf[(x, area.top())]
                .set_symbol(line::ROUNDED_BOTTOM_RIGHT)
                .set_style(app.theme.border);

            buf[(x, area.bottom() - 1)]
                .set_symbol(line::ROUNDED_TOP_RIGHT)
                .set_style(app.theme.border);
        } else {
            buf[(x, area.top())]
                .set_symbol(line::VERTICAL)
                .set_style(app.theme.border);

            buf[(x, area.bottom() - 1)]
                .set_symbol(line::VERTICAL)
                .set_style(app.theme.border);
        }
    }

    buf[(x, area.top().saturating_sub(2))]
        .set_symbol(line::ROUNDED_TOP_LEFT)
        .set_style(app.theme.border);

    buf[(x, area.top().saturating_sub(1))]
        .set_symbol(line::VERTICAL)
        .set_style(app.theme.border);

    let y = buf.area.bottom().saturating_sub(1);
    buf[(x, y)]
        .set_symbol(line::ROUNDED_BOTTOM_LEFT)
        .set_style(app.theme.border);

    for y in area.bottom()..y {
        buf[(x, y)]
            .set_symbol(line::VERTICAL)
            .set_style(app.theme.border);
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
