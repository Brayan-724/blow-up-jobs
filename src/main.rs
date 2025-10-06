#![feature(try_trait_v2)]
#![feature(associated_type_defaults)]
#![feature(stmt_expr_attributes)]

extern crate crossterm;
extern crate ratatui;
extern crate tokio;

mod animation;
mod app;
mod events;
mod job;
mod theme;
mod ui;
mod variadicts;
mod vterm;

use std::io;

use ratatui::DefaultTerminal;

use crate::app::App;
use crate::events::{CaptureMouse, TermEvents};
use crate::ui::Component;

// fn main() {
//     let (term_w, term_h) = crossterm::terminal::size().unwrap();
//     let term_w = term_w as i32 - 1;
//     let term_h = term_h as i32 - 1;
//
//     for level in (0..=(term_w / 2) as usize).rev() {
//         print!("\n");
//         print!("{level}\n");
//         for y in 0..term_h {
//             for x in 0..term_w {
//                 let Some(c) = render_char_at(level, x - term_w / 2, y - term_h / 2) else {
//                     print!(" ");
//                     continue;
//                 };
//
//                 print!("{c}");
//             }
//
//             print!("\n");
//         }
//         print!("\n");
//     }
// }

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app).await;

    ratatui::restore();
    result
}

async fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    let _ = CaptureMouse::scoped()?;

    'draw: loop {
        tokio::task::block_in_place(|| terminal.draw(|frame| App::draw(app, frame, frame.area())))?;

        loop {
            let job_tick = app.job_tick();
            let anim = app.anim.wait_tick();

            let action = tokio::select! {
                Ok(ev) = TermEvents => App::handle_event(app, ev).await,
                true = job_tick => continue 'draw,
                true = anim => app.anim.update(100),

            };

            match action {
                ui::Action::Noop => {}
                ui::Action::Quit => break 'draw,
                ui::Action::Tick => continue 'draw,
            }
        }
    }

    app.kill_jobs().await;

    Ok(())
}
