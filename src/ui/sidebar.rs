use crate::app::App;
use crate::ui::prelude::*;

pub fn render(state: &mut App, area: Rect, frame: &mut Frame) {
    let area = Layout::vertical([Constraint::Percentage(100), Constraint::Length(5)]).split(area);

    frame.draw(
        common::Blinker::new(render_help),
        area[1].offset_x(1).reduce((2, 0)),
        &mut state.anim,
    );
    frame.draw(render_sidebar, area[0], state);
}

fn render_sidebar(state: &App, frame: &mut Frame, area: Rect) {
    let items = [
        "Process ABCDEFGHIJKLMNOPQ",
        "Process B",
        "Process C",
        "Process C",
    ];

    let area = area.offset_y(2).set_height(items.len() as u16 * 2 + 2);

    let borders_area = area;

    let area = area.offset(Offset { x: 1, y: 1 }).reduce((2, 0));

    for (idx, item) in items.iter().enumerate() {
        let area = area.offset_y(idx as i32 * 2);

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
            area.offset_y(1).reduce((1, 0)),
            &state.anim,
        );

        let bg = if idx == 1 {
            Color::Magenta
        } else {
            Color::Reset
        };

        frame.draw(
            common::Blinker::new(move |area: Rect, buf: &mut Buffer| {
                let area = common::pill(bg, area, buf);

                item.to_text()
                    .fg(Color::White)
                    .bold()
                    .bg(bg)
                    .render(area, buf);
            }),
            area,
            &state.anim,
        );
    }

    render_sidebar_borders(borders_area, frame.buffer_mut());
}

fn render_sidebar_borders(area: Rect, buf: &mut Buffer) {
    Block::new()
        .borders(!Borders::RIGHT)
        .border_set(border::ROUNDED)
        .border_style(Color::LightMagenta)
        .render(area, buf);

    let x = area.right() - 1;

    if area.width > 1 {
        buf[(x, area.top())]
            .set_symbol(line::ROUNDED_TOP_LEFT)
            .set_fg(Color::LightMagenta);

        buf[(x, area.top())]
            .set_symbol(line::ROUNDED_BOTTOM_RIGHT)
            .set_fg(Color::LightMagenta);

        buf[(x, area.bottom() - 1)]
            .set_symbol(line::ROUNDED_TOP_RIGHT)
            .set_fg(Color::LightMagenta);
    } else {
        buf[(x, area.top())]
            .set_symbol(line::VERTICAL)
            .set_fg(Color::LightMagenta);

        buf[(x, area.bottom() - 1)]
            .set_symbol(line::VERTICAL)
            .set_fg(Color::LightMagenta);
    }

    buf[(x, area.top().saturating_sub(2))]
        .set_symbol(line::ROUNDED_TOP_LEFT)
        .set_fg(Color::LightMagenta);

    buf[(x, area.top().saturating_sub(1))]
        .set_symbol(line::VERTICAL)
        .set_fg(Color::LightMagenta);

    let y = buf.area.bottom().saturating_sub(1);
    buf[(x, y)]
        .set_symbol(line::ROUNDED_BOTTOM_LEFT)
        .set_fg(Color::LightMagenta);

    for y in area.bottom()..y {
        buf[(x, y)]
            .set_symbol(line::VERTICAL)
            .set_fg(Color::LightMagenta);
    }
}

fn render_help(area: Rect, buf: &mut Buffer) {
    Text::from(vec![
        Line::from(vec!["c".to_span().fg(Color::LightBlue), "onfig".to_span()]),
        Line::from(vec![
            "stop ".to_span(),
            "a".to_span().fg(Color::LightBlue),
            "ll processes".to_span(),
        ]),
        Line::from(vec![
            "n".to_span().fg(Color::LightBlue),
            "ew process".to_span(),
        ]),
    ])
    .render(area, buf);
}
