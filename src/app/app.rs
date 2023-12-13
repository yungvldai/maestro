use std::{
    env,
    fs::File,
    io::{Error, ErrorKind},
    os::unix::process::CommandExt,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{config::ConfigReadinessProbe, readiness_probe, utils::normalize_path};

use super::AppStatus;

#[derive(Debug)]
pub struct App {
    name: String,
    command: Vec<String>,
    uid: u32,
    ready: bool,
    readiness_probe: ConfigReadinessProbe,
    signal: i32,

    process: Option<Child>,
    status: AppStatus,
    exit_code: Option<i32>,

    started_at: Option<Duration>,
    updated_at: Duration,
    ready_checked_at: Option<Duration>,

    stdout: Option<String>,
    stderr: Option<String>,
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
        readiness_probe: ConfigReadinessProbe,
        signal: i32,

        stdout: Option<String>,
        stderr: Option<String>,
    ) -> Self {
        let app = Self {
            name: name.to_owned(),
            command,
            uid,
            readiness_probe,
            signal,
            stdout,
            stderr,

            process: None,
            status: AppStatus::Init,
            ready: false,
            exit_code: None,

            started_at: None,
            updated_at: get_now(),
            ready_checked_at: None,
        };

        log::info!("app \"{}\" created", name);

        app
    }

    fn set_status(&mut self, status: AppStatus) {
        self.status = status;

        log::info!("app \"{}\" status changed to {}", self.name, status);
    }

    fn set_readiness(&mut self, ready: bool) {
        self.ready = ready;

        if ready {
            log::info!("app \"{}\" is READY now", self.name);
        }
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

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    fn redirect_stdio(&self, to: Option<String>) -> Stdio {
        if to.to_owned().is_some_and(|x| x == "inherit") {
            return Stdio::inherit();
        }

        match to {
            None => Stdio::null(),
            Some(value) => match File::create(normalize_path(value.to_owned())) {
                Ok(file) => Stdio::from(file),
                Err(err) => {
                    log::warn!(
                        "unable to create log file for app \"{}\", {}",
                        self.name,
                        err.to_string()
                    );

                    Stdio::null()
                }
            },
        }
    }

    pub fn run(&mut self) {
        if self.status != AppStatus::Init {
            return;
        }

        let mut full_command: Vec<_> = self.command.iter().map(|i| i.trim()).collect();
        let executable = full_command.remove(0);
        let args = full_command;
        let envs = env::vars();

        let result = Command::new(executable)
            .envs(envs)
            .args(args)
            .uid(self.uid)
            .stdin(Stdio::null())
            .stdout(self.redirect_stdio(self.stdout.to_owned()))
            .stderr(self.redirect_stdio(self.stderr.to_owned()))
            .spawn();

        match result {
            Ok(child) => {
                let pid = child.id();

                self.process = Some(child);
                self.started_at = Some(get_now());

                log::info!("app \"{}\" is started, pid: {}", self.name, pid);
                self.set_status(AppStatus::Running);
            }
            Err(err) => {
                log::error!(
                    "unable to run the app \"{}\", {}",
                    self.name,
                    err.to_string()
                );
                self.set_status(AppStatus::Stopped);
            }
        }
    }

    fn update_readiness(&mut self) {
        if self.status == AppStatus::Init {
            /*
             * For an app to be considered ready, it must at least be RUNNING
             */
            return;
        }

        let now = get_now();

        match &self.readiness_probe {
            ConfigReadinessProbe::Command { command, period } => match self.ready_checked_at {
                None => {
                    self.ready_checked_at = Some(now);
                    self.set_readiness(readiness_probe::command(command.to_owned()));
                }
                Some(last_ready_checked) => {
                    if now.as_millis() - last_ready_checked.as_millis() >= *period as u128 {
                        self.ready_checked_at = Some(now);
                        self.set_readiness(readiness_probe::command(command.to_owned()));
                    }
                }
            },
            ConfigReadinessProbe::Delay { delay } => match self.started_at {
                None => (),
                Some(started) => {
                    self.set_readiness(now.as_millis() - started.as_millis() >= *delay as u128);
                }
            },
            ConfigReadinessProbe::Http {
                url,
                method,
                period,
            } => match self.ready_checked_at {
                None => {
                    self.ready_checked_at = Some(now);
                    self.set_readiness(readiness_probe::http(method.to_owned(), url.to_owned()));
                }
                Some(last_ready_checked) => {
                    if now.as_millis() - last_ready_checked.as_millis() >= *period as u128 {
                        self.ready_checked_at = Some(now);
                        self.set_readiness(readiness_probe::http(
                            method.to_owned(),
                            url.to_owned(),
                        ));
                    }
                }
            },
            ConfigReadinessProbe::ExitCode { exit_code } => self.set_readiness(
                self.status == AppStatus::Stopped
                    && self.exit_code.is_some_and(|x| x == *exit_code),
            ),
            ConfigReadinessProbe::None => {
                log::info!(
                    "no readiness probe is preseted for app \"{}\", considering as ready",
                    self.name
                );
                self.set_readiness(true);
            }
        }
    }

    pub fn update(&mut self) {
        self.update_readiness();

        if self.status != AppStatus::Stopped && self.exit_code.is_none() {
            if let Some(process) = &mut self.process {
                match process.try_wait() {
                    Err(err) => {
                        log::error!(
                            "unable to check the app \"{}\", {}",
                            self.name,
                            err.to_string()
                        );
                        self.set_status(AppStatus::Stopped);
                    }
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
        if self.status != AppStatus::Running {
            return;
        }

        let exec_kill = || -> Result<ExitStatus, Error> {
            let pid = self
                .get_pid()
                .ok_or(Error::new(ErrorKind::Other, "no pid"))?
                .to_string();
            let signal = self.signal.to_string();

            let exit_status = Command::new("kill")
                .args([format!("-{}", signal.as_str()), pid])
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
            }
            Err(err) => {
                log::warn!(
                    "unable to kill the app \"{}\" gracefully, {}",
                    self.name,
                    err.to_string()
                );
                self.kill();
            }
        }
    }
}
