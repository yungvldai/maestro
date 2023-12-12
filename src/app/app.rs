use std::{process::{Command, Child, Stdio, ExitStatus}, os::unix::process::CommandExt, env, time::{SystemTime, UNIX_EPOCH, Duration}, io::{Error, ErrorKind}};

use crate::{config::ConfigReady, readiness};

use super::AppStatus;

#[derive(Debug)]
pub struct App {
    name: String,
    command: Vec<String>,
    uid: u32,
    ready: ConfigReady,
    signal: i32,

    process: Option<Child>,
    status: AppStatus,
    exit_code: Option<i32>,

    started_at: Option<Duration>,
    updated_at: Duration,
    ready_checked_at: Option<Duration>,

    stdout: Option<String>,
    stderr: Option<String>
}

fn get_now() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

impl App {
    #[warn(clippy::too_many_arguments)]
    pub fn new(
        name: String, 
        command: Vec<String>,
        uid: u32,
        ready: ConfigReady,
        signal: i32,

        stdout: Option<String>,
        stderr: Option<String>
    ) -> Self {
        let app = Self {
            name: name.to_owned(),
            command,
            uid,
            ready,
            signal,

            process: None,
            status: AppStatus::Init,
            exit_code: None,

            started_at: None,
            updated_at: get_now(),
            ready_checked_at: None,

            stdout,
            stderr
        };

        log::info!("app \"{}\" created", name);

        app
    }

    fn set_status(&mut self, status: AppStatus) {
        self.status = status;

        log::info!("app \"{}\" status changed to {}", self.name, status);
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn get_status(&self) -> AppStatus {
        self.status
    }

    pub fn get_exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn get_pid(&self) -> Option<u32> {
        if let Some(p) = &self.process {
            return Some(p.id());
        }

        None
    }

    pub fn run(&mut self) {
        if self.status != AppStatus::Init {
            return;
        }

        if self.command.is_empty() {
            log::error!("command is not preseted for app \"{}\"", self.name);
            self.set_status(AppStatus::Stopped);

            return;
        }

        let mut full_command: Vec<_> = self.command.iter().map(|i| i.trim()).collect();
        let executable = full_command.remove(0);
        let args = full_command;
        let envs = env::vars();

        let stdout_pipe = match self.stdout {
            None => Stdio::null(),
            _ => Stdio::null()
        };

        let stderr_pipe = match self.stderr {
            None => Stdio::null(),
            _ => Stdio::null()
        };

        let result = Command::new(executable)
            .envs(envs)
            .args(args)
            .uid(self.uid)
            .stdin(Stdio::null())
            .stdout(stdout_pipe)
            .stderr(stderr_pipe)
            .spawn();

        match result {
            Ok(child) => {
                self.process = Some(child);
                self.started_at = Some(get_now());
                self.set_status(AppStatus::Started);
            },
            Err(err) => {
                log::error!("unable to run the app \"{}\", {}", self.name, err.to_string());
                self.set_status(AppStatus::Stopped);
            }
        }        
    }

    fn check_ready(&mut self) -> bool {
        let now = get_now();

        match &self.ready {
            ConfigReady::Command { command, period } => {
                match self.ready_checked_at {
                    None => {
                        self.ready_checked_at = Some(now);

                        readiness::command(command.to_owned())
                    },
                    Some(last_ready_checked) => {
                        if now.as_millis() - last_ready_checked.as_millis() >= *period as u128 { 
                            self.ready_checked_at = Some(now);

                            readiness::command(command.to_owned())
                        } else {
                            false
                        }
                    }
                }
            },
            ConfigReady::Delay { delay } => {
                match self.started_at {
                    None => false,
                    Some(started) => {
                        now.as_millis() - started.as_millis() >= *delay as u128
                    }
                }
            },
            ConfigReady::Http { url, method, period } => {
                match self.ready_checked_at {
                    None => {
                        self.ready_checked_at = Some(now);

                        readiness::http(method.to_owned(), url.to_owned())
                    },
                    Some(last_ready_checked) => {
                        if now.as_millis() - last_ready_checked.as_millis() >= *period as u128 { 
                            self.ready_checked_at = Some(now);

                            readiness::http(method.to_owned(), url.to_owned())
                        } else {
                            false
                        }
                    }
                }
            },
            ConfigReady::None => {
                log::info!("no readiness probe is preseted for app \"{}\", considering as ready", self.name);

                true
            },
        }
    }

    pub fn update(&mut self) {
        if self.status == AppStatus::Started && self.check_ready() {
            self.set_status(AppStatus::Running);
        }

        if self.status != AppStatus::Stopped && self.exit_code.is_none() {
            if let Some(process) = &mut self.process {
                match process.try_wait() {
                    Err(err) => {
                        log::error!("unable to check the app \"{}\", {}", self.name, err.to_string());
                        self.set_status(AppStatus::Stopped);
                    },
                    Ok(exit_status) => {
                        if let Some(es) = exit_status {
                            self.exit_code = es.code();
        
                            if let Some(code) = self.exit_code {
                                log::info!("app \"{}\" exited with code {}", self.name, code);
                            }
        
                            self.set_status(AppStatus::Stopped);
                        }
                    }
                }
            }
        }

        self.updated_at = get_now();
    }

    fn kill(&mut self) {
        if let Some(ref mut proc) = self.process {
            log::info!("killing with SIGKILL...");

            proc.kill().ok();
        }
    }

    pub fn stop(&mut self) {
        if self.status != AppStatus::Started && self.status != AppStatus::Running {
            return;
        }

        let exec_kill = || -> Result<ExitStatus, Error> {
            let pid = self.get_pid()
                .ok_or(Error::new(ErrorKind::Other, "no pid"))?
                .to_string();
            let signal = self.signal.to_string();

            let exit_status = Command::new("kill")
                .args([
                    format!("-{}", signal.as_str()), 
                    pid
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?
                .wait()?;

            Ok(exit_status)
        };

        match exec_kill() {
            Ok(status) => {
                if status.code().unwrap_or(1) != 0 {
                    log::warn!("unable to kill the app \"{}\" gracefully", self.name);
                    self.kill();
                } else {
                    self.set_status(AppStatus::Stopping);
                }
            },
            Err(err) => {
                log::warn!("unable to kill the app \"{}\" gracefully, {}", self.name, err.to_string());
                self.kill();
            }
        }
    }
}