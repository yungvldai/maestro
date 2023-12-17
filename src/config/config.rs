use std::{collections::HashMap, env, fs::File, path::Path};

use serde::Deserialize;

use super::config_app::ConfigApp;

const CONFIG_FILENAME: &str = "maestro.yml";
const CONFIG_DIR: &str = "/etc/maestro";

fn default_apps() -> Vec<ConfigApp> {
    vec![]
}

fn default_log_level() -> String {
    "info".to_string()
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

        let file = File::open(&etc_config_path).unwrap_or_else(|_| {
            panic!(
                "config file not found, checked: {} and {}",
                cwd_config_path.to_str().unwrap(),
                etc_config_path.to_str().unwrap()
            )
        });

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
                panic!("App names must be unique");
            }

            apps_map.insert(app.name.to_owned(), app);
        }

        for app in self.apps.iter() {
            for dep in app.depends_on.iter() {
                // TODO check cycles

                if app.name == *dep {
                    panic!("dependence on oneself: \"{}\"", dep);
                }

                if apps_map.get(dep).is_none() {
                    panic!("unknown dependency: \"{}\"", dep);
                }
            }
        }

        self
    }
}
