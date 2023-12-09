use std::{process::{Command, ExitStatus, Stdio}, env, io::Error, time::Instant};

pub fn exec_sync_check_ok(cmd: Vec<String>) -> bool {
    let mut full_command: Vec<_> = cmd.iter().map(|i| i.trim()).collect();
    let executable = full_command.remove(0);
    let args = full_command;
    let envs = env::vars();

    let get_status = || -> Result<ExitStatus, Error> {
        let exit_status = Command::new(executable)
            .envs(envs)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
            .wait()?;

        Ok(exit_status)
    };

    let now = Instant::now();
    let status = get_status();
    let took = now.elapsed().as_millis();

    match status {
        Ok(value) => {
            log::debug!("command \"{}\" OK, took {} ms", cmd.join(" "), took);

            value.success()
        },
        Err(err) => {
            log::debug!("command \"{}\" FAILED, took {} ms, {}", cmd.join(" "), took, err.to_string());

            false
        }
    }
}