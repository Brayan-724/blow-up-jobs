use crate::app::App;
use crate::ui::prelude::*;

pub fn render(state: &App, area: Rect, frame: &mut Frame) {
    let area = Layout::vertical([Constraint::Percentage(100), Constraint::Length(5)]).split(area);

    frame.draw_stateless(render_help, area[1].offset_x(1).reduce((2, 0)));
    frame.draw(render_sidebar, area[0], state);
}

fn render_sidebar(_: &App, area: Rect, buf: &mut Buffer) {
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
        let area = area.offset(Offset {
            x: 0,
            y: idx as i32 * 2,
        });

        {
            let area = area.offset(Offset { x: 0, y: 1 }).reduce((1, 0));

            Line::from(vec![
                "200ms ".to_span(),
                "●".to_span().fg(if idx % 2 == 0 {
                    Color::Green
                } else {
                    Color::Red
                }),
            ])
            .right_aligned()
            .render(area, buf);
        }

        let bg = if idx == 1 {
            Color::Magenta
        } else {
            Color::Reset
        };

        let area = common::pill(bg, area, buf);

        let fg = Color::White;

        let max_size = area.width.saturating_sub(2) as usize;
        let item = if item.len() > max_size {
            &item[..max_size]
        } else {
            item
        };

        item.to_text().fg(fg).bold().bg(bg).render(area, buf);
    }

    render_sidebar_borders(borders_area, buf);
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
