use std::{collections::{HashMap, hash_map::{Keys, Values}}, cell::{RefCell, RefMut}};

use crate::config::ConfigApp;

use super::{App, AppStatus};

pub struct AppsMap(HashMap<String, RefCell<App>>);

impl AppsMap {
    pub fn new(apps: Vec<ConfigApp>) -> Self {
        let mut apps_map: HashMap<String, RefCell<App>> = HashMap::new();

        for config_app in apps.into_iter() {
            let app = App::new(
                config_app.name.to_owned(),
                config_app.command,
                config_app.uid,
                config_app.ready,
                config_app.depends_on,
                config_app.signal,
                config_app.stdout,
                config_app.stderr
            );
    
            apps_map.insert(config_app.name.to_owned(), RefCell::new(app));
        }

        Self(apps_map)
    }

    pub fn every(&self, predicate: fn(&RefCell<App>) -> bool) -> bool {
        for app in self.0.values() {
            if !predicate(app) {
                return false;
            }
        }
    
        true
    }

    pub fn get(&self, app_name: String) -> Option<&RefCell<App>> {
        self.0.get(&app_name)
    }

    pub fn list(&self) -> Values<'_, String, RefCell<App>> {
        self.0.values()
    }

    pub fn check_deps_resolved_for(&self, app: &RefMut<'_, App>) -> bool {
        let mut deps_resolved = true;

        for dep in app.get_deps() {
            match self.0.get(&dep) {
                Some(dep_app) => {
                    if dep_app.borrow().get_status() != AppStatus::Running {
                        deps_resolved = false;

                        break;
                    }
                },
                None => {
                    /* 
                        * If the app depends on a non-existent app, 
                        * then we cannot launch it
                        */
                    deps_resolved = false;

                    break;
                }
            };
        }

        deps_resolved
    }
}