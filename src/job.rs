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

// tty spawn error messages
const NOT_FOUND_MESSAGE: &str = "No viable candidates found in PATH";
const NOT_EXISTS_MESSAGE: &str = " it does not exist";
const NOT_EXEC_MESSAGE: &str = " it is not executable";
const IS_DIR_MESSAGE: &str = " it is a directory";

#[derive(Debug, Error)]
pub enum JobStartError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Rustix error: {0}")]
    Rustix(#[from] rustix::io::Errno),
    #[error("No command provided")]
    NoCommand,
    #[error("Command not found")]
    NotFound,
    #[error("File is not executable")]
    NotExecutable,
    #[error("Command is a directory")]
    IsDir,
    #[error("Cannot parse command")]
    Parse(#[from] shellish_parse::ParseError),
}

pub struct JobRunning {
    pub pty: Box<dyn MasterPty + Send + 'static>,
    pub vterm: Arc<RwLock<Parser>>,
    pub pid: u32,
    pub status: Arc<RwLock<Option<u32>>>,
}

pub struct Job {
    pub title: String,
    pub cmd: String,
    pub notify: Arc<tokio::sync::Notify>,
    pub running: Option<JobRunning>,
    pub size: Size,
}

impl Job {
    pub fn new(cmd: impl ToString) -> Self {
        Self {
            title: cmd.to_string(),
            cmd: cmd.to_string(),
            notify: Default::default(),
            running: None,
            size: Size::new(80, 24),
        }
    }

    // None means that is running or is not started
    pub fn status(&self) -> Option<u32> {
        self.running
            .as_ref()
            .and_then(|r| *r.status.blocking_read())
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
        if let Ok(cwd) = std::env::current_dir() {
            cmd.cwd(cwd);
        }

        let mut child = slave.spawn_command(cmd).map_err(|err| {
            let err = err.to_string();
            let Some(because_idx) = err.find("because") else {
                unreachable!("malformed error (open an issue): {err}");
            };

            let mut offset = because_idx + 7;

            if &err[offset..offset + 1] == ":" {
                offset += 2;
            }

            let err_msg = err.chars().skip(offset).collect::<String>();

            if err_msg.starts_with(NOT_FOUND_MESSAGE) || err_msg.starts_with(NOT_EXISTS_MESSAGE) {
                JobStartError::NotFound
            } else if err_msg.starts_with(NOT_EXEC_MESSAGE) {
                JobStartError::NotExecutable
            } else if err_msg.starts_with(IS_DIR_MESSAGE) {
                JobStartError::IsDir
            } else {
                todo!("unhandled error (open an issue): {err_msg}");
            }
        })?;
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
        let Some(ref job) = self.running else {
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
