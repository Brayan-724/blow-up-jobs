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
use std::time::Duration;

use ratatui::DefaultTerminal;

use crate::app::App;
use crate::events::{CaptureMouse, TermEvents};
use crate::ui::Component;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let mut app = App::new();

    if std::env::var("BUJ_ANIMATION_DEBUG").is_ok() {
        app.anim.debug();
    }

    let result = run_app(&mut terminal, &mut app).await;

    ratatui::restore();
    result
}

async fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    let _ = CaptureMouse::scoped()?;

    let mut quitting = false;

    'draw: loop {
        tokio::task::block_in_place(|| terminal.draw(|frame| App::draw(app, frame, frame.area())))?;

        if quitting {
            app.anim.wait_tick().await;

            if app.anim.ended() {
                break 'draw;
            } else {
                app.anim.update();
                continue 'draw;
            }
        }

        loop {
            let job_tick = app.job_tick();
            let anim = app.anim.wait_tick();

            let action = tokio::select! {
                true = anim => app.anim.update(),
                true = job_tick => continue 'draw,
                Ok(ev) = TermEvents => App::handle_event(app, ev).await,
            };

            match action {
                ui::Action::Noop => {}
                ui::Action::Quit => {
                    app.anim.reverse();
                    app.anim.start();
                    app.anim.next_tick(Duration::from_millis(10));
                    quitting = true;
                    continue 'draw;
                }
                ui::Action::Tick => continue 'draw,
            }
        }
    }

    app.kill_jobs().await;

    Ok(())
}
