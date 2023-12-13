use std::{collections::HashMap, env, fs::File, path::Path};

use serde::Deserialize;
use serde_yaml::Value;
use signal_hook::consts::{SIGINT, SIGTERM};

use crate::user::get_uid_by_name;

const CONFIG_FILENAME: &str = "maestro.yml";
const CONFIG_DIR: &str = "/etc/maestro";

fn default_user() -> u32 {
    unsafe { libc::geteuid() }
}

fn default_signal() -> i32 {
    SIGTERM
}

fn default_apps() -> Vec<ConfigApp> {
    vec![]
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_depends_on() -> Vec<String> {
    vec![]
}

fn default_ready_command_period() -> u32 {
    1000
}

fn default_ready_http_method() -> String {
    "GET".to_string()
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

        #[serde(default = "default_ready_command_period")]
        period: u32,
    },
    Http {
        url: String,

        #[serde(default = "default_ready_http_method")]
        method: String,

        #[serde(default = "default_ready_command_period")]
        period: u32,
    },
}

#[derive(Debug, Deserialize)]
pub struct ConfigApp {
    pub name: String,
    pub command: Vec<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,

    #[serde(default = "default_signal")]
    #[serde(deserialize_with = "deserialize_signal")]
    pub signal: i32,

    #[serde(default = "default_user")]
    #[serde(rename(deserialize = "user"))]
    #[serde(deserialize_with = "deserialize_and_get_uid")]
    pub uid: u32,

    #[serde(default = "default_depends_on")]
    pub depends_on: Vec<String>,

    #[serde(default = "default_ready")]
    pub ready: ConfigReadinessProbe,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub pid: Option<String>,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_apps")]
    pub apps: Vec<ConfigApp>,
}

impl Config {
    pub fn new() -> Self {
        let pwd = env::current_dir().expect("unable to get cwd");
        let cwd_config_path = Path::new(pwd.as_path()).join(CONFIG_FILENAME);

        if let Ok(file) = File::open(&cwd_config_path) {
            let config: Config = serde_yaml::from_reader(file).unwrap();

            return config;
        }

        let etc_config_path = Path::new(CONFIG_DIR).join(CONFIG_FILENAME);

        let file = File::open(&etc_config_path).expect(
            format!(
                "config file not found, checked: {} and {}",
                cwd_config_path.to_str().unwrap(),
                etc_config_path.to_str().unwrap()
            )
            .as_str(),
        );

        let config: Config = serde_yaml::from_reader(file).unwrap();

        config
    }

    pub fn validate(self) -> Self {
        let mut apps_map: HashMap<String, &ConfigApp> = HashMap::new();

        for app in self.apps.iter() {
            if app.command.is_empty() {
                panic!("command is not presented for app: \"{}\"", app.name);
            }

            if apps_map.contains_key(&app.name) {
                panic!("Application names must be unique");
            }

            apps_map.insert(app.name.to_owned(), app);
        }

        for app in self.apps.iter() {
            for dep in app.depends_on.iter() {
                // TODO check cycles

                if app.name == *dep {
                    panic!("dependence on oneself: \"{}\"", dep);
                }

                let dep_app = match apps_map.get(dep) {
                    Some(value) => value,
                    None => panic!("unknown dependency: \"{}\"", dep),
                };

                if let ConfigReadinessProbe::None = dep_app.ready {
                    panic!(
                        "app \"{}\" has dependents, but does not have a readiness probe",
                        dep_app.name
                    );
                }
            }
        }

        self
    }
}
