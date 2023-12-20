use crate::config::ConfigReadinessProbe;

#[derive(Debug)]
pub enum AppReadinessProbe {
    None,
    ExitCode {
        exit_code: i32,
    },
    Delay {
        delay: u32,
    },
    Command {
        command: Vec<String>,
        period: u32,
    },
    Http {
        url: String,
        method: String,
        period: u32,
    },
}

impl From<ConfigReadinessProbe> for AppReadinessProbe {
    fn from(value: ConfigReadinessProbe) -> Self {
        match value {
            ConfigReadinessProbe::None => AppReadinessProbe::None,
            ConfigReadinessProbe::ExitCode { exit_code } => {
                AppReadinessProbe::ExitCode { exit_code }
            }
            ConfigReadinessProbe::Command { command, period } => {
                AppReadinessProbe::Command { command, period }
            }
            ConfigReadinessProbe::Delay { delay } => AppReadinessProbe::Delay { delay },
            ConfigReadinessProbe::Http {
                url,
                method,
                period,
            } => AppReadinessProbe::Http {
                url,
                method,
                period,
            },
        }
    }
}
