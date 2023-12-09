use std::{process, fs::File};
use std::io::Write;

use crate::utils::normalize_path;

pub fn init_pid(config_pid_file: Option<String>) {
    let pid = process::id();

    log::info!("pid: {}", pid);

    if let Some(value) = config_pid_file {
        let path = normalize_path(value);
        let mut output = File::create(path).unwrap();

        write!(output, "{}", pid).unwrap();
    }
}