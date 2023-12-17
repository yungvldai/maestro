use libc::{SIGINT, SIGTERM};
use serde::Deserialize;
use serde_yaml::Value;

use crate::user::get_uid_by_name;

use super::config_readiness_probe::ConfigReadinessProbe;

fn default_app_user() -> u32 {
    unsafe { libc::geteuid() }
}

fn default_app_signal() -> i32 {
    SIGTERM
}

fn default_depends_on() -> Vec<String> {
    vec![]
}

fn default_ready() -> ConfigReadinessProbe {
    ConfigReadinessProbe::None
}

fn deserialize_and_get_uid<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;

    let uid = match value {
        Value::String(string_value) => {
            let first = match string_value.chars().next() {
                Some(value) => value,
                None => panic!("user value is empty"),
            };

            // linux user must start with an alphabetic character
            // so if the first char of passed value is number we consider it as number
            if first.is_ascii_digit() {
                match string_value.parse::<u32>() {
                    Ok(value) => value,
                    Err(_) => panic!("unable to parse string: {}", string_value),
                }
            } else {
                get_uid_by_name(string_value)
            }
        }
        Value::Number(number) => match number.as_u64() {
            Some(num_u64) => u32::try_from(num_u64).expect("provided uid is invalid (too large)"),
            None => panic!("unable to parse uid"),
        },
        _ => {
            panic!("unable to parse user value, expected string or number");
        }
    };

    Ok(uid)
}

fn deserialize_signal<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;

    let signal = match value {
        Value::String(string_value) => match string_value.to_lowercase().as_str() {
            "sigterm" | "term" => SIGTERM,
            "sigint" | "int" => SIGINT,
            _ => panic!("unknown signal name {}", string_value),
        },
        Value::Number(number) => match number.as_i64() {
            Some(num_i64) => {
                i32::try_from(num_i64).expect("provided signal is invalid (too large)")
            }
            None => panic!("unable to parse signal number"),
        },
        _ => {
            panic!("unable to parse signal, expected string or number");
        }
    };

    Ok(signal)
}

#[derive(Debug, Deserialize)]
pub struct ConfigApp {
    pub name: String,
    pub command: Vec<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,

    #[serde(default = "default_app_signal")]
    #[serde(deserialize_with = "deserialize_signal")]
    pub signal: i32,

    #[serde(default = "default_app_user")]
    #[serde(rename(deserialize = "user"))]
    #[serde(deserialize_with = "deserialize_and_get_uid")]
    pub uid: u32,

    #[serde(default = "default_depends_on")]
    pub depends_on: Vec<String>,

    #[serde(default = "default_ready")]
    pub ready: ConfigReadinessProbe,
}
