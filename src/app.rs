use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, Frame};

use crate::events::TermEvents;
use crate::job::{Job, JobStartError};

pub struct App {
    should_exit: bool,
    job: Job,
}

impl App {
    pub async fn run(term: &mut DefaultTerminal) -> io::Result<()> {
        let mut app = App {
            should_exit: false,
            job: Job::new("nu -c 'print A; sleep 1sec; print B'").await,
        };

        crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture)?;

        while !app.should_exit {
            tokio::task::block_in_place(|| term.draw(|frame| app.draw(frame)))?;
            app.handle_events().await?;
        }

        crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture)?;

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        let job_notifier = self.job.notify.notified();

        tokio::select! {
            Ok(ev) = TermEvents => self.handle_term_events(ev).await?,
            _ = job_notifier => {},
        }

        Ok(())
    }

    async fn handle_term_events(&mut self, event: Event) -> io::Result<()> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                self.should_exit = true;

                self.job.kill().await;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                ..
            }) => {
                if let Err(e) = self.job.start().await {
                    match e {
                        JobStartError::Io(error) => return Err(error),
                        JobStartError::NoCommand => println!("NoCommand"),
                        JobStartError::NotFound(_) => println!("NotFound"),
                        JobStartError::Parse(_) => println!("ParseError"),
                        JobStartError::Rustix(_) => println!("RustixError"),
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.job.render(area, buf);
    }
}
