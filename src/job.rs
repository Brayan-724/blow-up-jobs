use std::io;
use std::sync::Arc;
use std::time::Duration;

use portable_pty::{MasterPty, PtyPair, native_pty_system};
use ratatui::layout::Size;
use rustix::process::Signal;
use rustix::termios::Pid;
use thiserror::Error;
use tokio::sync::RwLock;
use vt100::Parser;

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
    pub pty: Box<dyn MasterPty + Send + 'static>,
    pub vterm: Arc<RwLock<Parser>>,
    pub pid: u32,
    pub status: Arc<RwLock<Option<u32>>>,
}

pub struct Job {
    pub cmd: String,
    pub notify: Arc<tokio::sync::Notify>,
    pub running: Option<JobRunning>,
    pub size: Size,
}

impl Job {
    pub fn new(cmd: impl ToString) -> Self {
        Self {
            cmd: cmd.to_string(),
            notify: Default::default(),
            running: None,
            size: Size::new(80, 24),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
            .as_ref()
            .is_some_and(|r| r.status.blocking_read().is_none())
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

        let status = Arc::new(RwLock::new(None));

        tokio::task::spawn({
            let status = status.clone();

            async move {
                loop {
                    match child.try_wait() {
                        Ok(None) => {}
                        Ok(Some(s)) => {
                            *status.write_owned().await = Some(s.exit_code());
                            drop(slave);
                            break;
                        }
                        Err(err) => {
                            println!("Error waiting job child: {err}");
                            break;
                        }
                    }

                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
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
            status,
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
    }

    pub async fn restart(&mut self) -> Result<(), JobStartError> {
        self.kill().await;

        self.start().await
    }

    pub fn with_cmd(&mut self, cmd: String) {
        self.cmd = cmd;
    }
}
