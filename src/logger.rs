use std::env;

use env_logger::{Env, Builder};

pub fn init_logger(level: String) {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", level);
    }

    let logger_env = Env::default();

    Builder::from_env(logger_env)
        .format_level(true)
        .format_timestamp_secs()
        .format_module_path(false)
        .format_target(false)
        .init();
}