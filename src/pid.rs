use std::io::Write;
use std::process;

use crate::fs::open_file;
use crate::utils::normalize_path;

pub fn init_pid(config_pid_file: Option<String>) {
    let pid = process::id();

    log::info!("pid: {}", pid);

    if let Some(value) = config_pid_file {
        let path = normalize_path(value);

        match open_file(path) {
            Ok(mut output) => {
                write!(output, "{}", pid).unwrap();
            }
            Err(err) => {
                log::warn!("unable to write pid, {}", err.to_string());
            }
        }
    }
}
