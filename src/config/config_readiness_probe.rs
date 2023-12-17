use serde::Deserialize;

fn default_readiness_probe_period() -> u32 {
    1000
}

fn default_readiness_probe_http_method() -> String {
    "GET".to_string()
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ConfigReadinessProbe {
    None,
    ExitCode {
        exit_code: i32,
    },
    Delay {
        delay: u32,
    },
    Command {
        command: Vec<String>,

        #[serde(default = "default_readiness_probe_period")]
        period: u32,
    },
    Http {
        url: String,

        #[serde(default = "default_readiness_probe_http_method")]
        method: String,

        #[serde(default = "default_readiness_probe_period")]
        period: u32,
    },
}
