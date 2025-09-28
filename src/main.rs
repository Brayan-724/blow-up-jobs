extern crate crossterm;
extern crate ratatui;
extern crate tokio;

mod app;
mod events;
mod job;
mod vterm;

use crate::app::App;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::run(&mut terminal).await;
    ratatui::restore();
    result
}
