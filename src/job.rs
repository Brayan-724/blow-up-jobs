use std::io;
use std::sync::Arc;

use portable_pty::{MasterPty, PtyPair, native_pty_system};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::widgets::Widget;
use rustix::process::Signal;
use rustix::termios::Pid;
use thiserror::Error;
use tokio::sync::RwLock;
use vt100::Parser;

use crate::vterm;

#[derive(Debug, Error)]
pub enum JobStartError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Rustix(#[from] rustix::io::Errno),
    #[error("no command provided")]
    NoCommand,
    #[error("command not found: {0}")]
    NotFound(String),
    #[error(transparent)]
    Parse(#[from] shellish_parse::ParseError),
}

pub struct JobRunning {
    pty: Box<dyn MasterPty + Send + 'static>,
    vterm: Arc<RwLock<Parser>>,
    pid: u32,
    // child: Child,
}

pub struct Job {
    cmd: String,
    running: Option<JobRunning>,
    pub notify: Arc<tokio::sync::Notify>,
    size: Size,
}

impl Job {
    pub async fn new(cmd: impl ToString) -> Self {
        Self {
            cmd: cmd.to_string(),
            running: None,
            notify: Default::default(),
            size: Size::new(80, 24),
        }
    }

    pub async fn start(&mut self) -> Result<(), JobStartError> {
        let pty = native_pty_system();
        let PtyPair { slave, master } = pty
            .openpty(portable_pty::PtySize {
                rows: self.size.height,
                cols: self.size.width,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        let mut parsed = shellish_parse::parse(&self.cmd, true)?.into_iter();

        let cmd = parsed.next().ok_or(JobStartError::NoCommand)?;

        let mut cmd = portable_pty::CommandBuilder::new(cmd);
        cmd.args(parsed);

        let mut child = slave.spawn_command(cmd).unwrap();
        let pid = child.process_id().unwrap();

        tokio::task::spawn_blocking(move || {
            let _ = child.wait();
            drop(slave);
        });

        let vterm = Arc::new(RwLock::new(Parser::new(
            self.size.height,
            self.size.width,
            0,
        )));
        let vterm_ = vterm.clone();
        let notifier = self.notify.clone();
        let mut reader = master.try_clone_reader().unwrap();

        tokio::spawn(async move {
            let buf = &mut [0; 1024];

            loop {
                let Ok(size) = reader.read(buf) else {
                    return;
                };

                vterm_.write().await.process(&buf[0..size]);

                notifier.notify_one();
            }
        });

        self.running = Some(JobRunning {
            pty: master,
            vterm,
            pid,
        });

        Ok(())
    }

    pub async fn kill(&mut self) -> bool {
        let Some(job) = self.running.take() else {
            return false;
        };

        rustix::process::kill_process(
            unsafe { Pid::from_raw_unchecked(job.pid as i32) },
            Signal::KILL,
        )
        .is_ok()

        // job.pty;
    }

    pub async fn restart(&mut self) -> Result<(), JobStartError> {
        self.kill().await;

        self.start().await
    }

    pub fn with_cmd(&mut self, cmd: String) {
        self.cmd = cmd;
    }
}

impl Widget for &mut Job {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let size = area.as_size();
        self.size = size;
        if let Some(ref job) = self.running {
            _ = job.pty.resize(portable_pty::PtySize {
                rows: size.height,
                cols: size.width,
                pixel_width: 0,
                pixel_height: 0,
            });
            job.vterm.blocking_write().set_size(size.width, size.height);
            vterm::VTermWidget::new(job.vterm.blocking_read().screen()).render(area, buf);
        } else {
            ratatui::text::Text::from("No running job").render(area, buf);
        }
    }
}
