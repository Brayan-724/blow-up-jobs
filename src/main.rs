#![feature(associated_type_defaults)]
#![feature(char_max_len)]
#![feature(if_let_guard)]
#![feature(impl_trait_in_assoc_type)]
#![feature(generic_const_exprs)]
#![feature(stmt_expr_attributes)]
#![feature(try_trait_v2)]
// generic_const_exprs
#![allow(incomplete_features)]

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
        app.sidebar_anim.debug();
        app.popup.anim.debug();
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
            app.update_sidebar();
            app.anim.wait_tick().await;

            if app.anim.ended() {
                break 'draw;
            } else {
                app.sidebar_anim.update();
                app.anim.update();
                continue 'draw;
            }
        }

        loop {
            app::PopupsState::update(app);

            app.update_sidebar();

            let mut action = app.anim.update();
            action |= app.sidebar_anim.update();
            action |= app.popup.anim.update();

            let job_tick = app.job_tick();
            let anim = app.anim.wait_tick();
            let sidebar_anim = app.sidebar_anim.wait_tick();
            let popup_anim = app.popup.anim.wait_tick();

            let mut action = tokio::select! {
                true = popup_anim => action,
                true = anim => action,
                true = sidebar_anim => action,
                true = job_tick => continue 'draw,
                Ok(ev) = TermEvents => App::handle_event(app, ev).await,
            };

            action |= app.anim.update();
            action |= app.sidebar_anim.update();
            action |= app.popup.anim.update();

            match action {
                ui::Action::Noop | ui::Action::Intercept => {}
                ui::Action::Quit => {
                    app.sidebar_anim.reverse();
                    app.anim.reverse();
                    app.anim.start();
                    app.anim.next_tick(Duration::from_millis(20));
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

#[cfg(doc)]
pub mod type_bundle {
    //! A type bundle is a trait that allows variadicts in Rust, but it also
    //! opens a new type of iterator over types.
}
