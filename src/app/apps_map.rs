use std::{collections::{HashMap, hash_map::Values}, cell::{RefCell, Ref}};

use crate::config::ConfigApp;

use super::{App, AppStatus};

pub struct AppsMap {
    map: HashMap<String, RefCell<App>>,
    run_after: HashMap<String, Vec<String>>,
    run_before: HashMap<String, Vec<String>>
}

impl AppsMap {
    pub fn new(apps: Vec<ConfigApp>) -> Self {
        let mut run_after: HashMap<String, Vec<String>> = HashMap::new();
        let mut run_before: HashMap<String, Vec<String>> = HashMap::new();
        let mut apps_map: HashMap<String, RefCell<App>> = HashMap::new();

        for config_app in apps.into_iter() {
            let app = App::new(
                config_app.name.to_owned(),
                config_app.command,
                config_app.uid,
                config_app.ready,
                config_app.signal,
                config_app.stdout,
                config_app.stderr
            );

            run_after.insert(config_app.name.to_owned(), config_app.depends_on.to_owned());

            for dep in config_app.depends_on {
                run_before
                    .entry(dep)
                    .or_insert(vec![])
                    .push(config_app.name.to_owned());
            }
    
            apps_map.insert(config_app.name.to_owned(), RefCell::new(app));
        }

        Self {
            map: apps_map,
            run_after,
            run_before
        }
    }

    pub fn every(&self, predicate: fn(&RefCell<App>) -> bool) -> bool {
        for app in self.map.values() {
            if !predicate(app) {
                return false;
            }
        }
    
        true
    }

    pub fn get(&self, app_name: &String) -> Option<&RefCell<App>> {
        self.map.get(app_name)
    }

    pub fn list(&self) -> Values<'_, String, RefCell<App>> {
        self.map.values()
    }

    pub fn get_dependencies_for(&self, app_name: &String) -> Vec<String> {
        match self.run_after.get(app_name) {
            None => Vec::new(),
            Some(value) => value.to_owned()
        }
    }

    pub fn get_dependents_for(&self, app_name: &String) -> Vec<String> {
        match self.run_before.get(app_name) {
            None => Vec::new(),
            Some(value) => value.to_owned()
        }
    }

    // pub fn check_deps_resolved_for(&self, app: &Ref<'_, App>) -> bool {
    //     let mut deps_resolved = true;

    //     for dep in app.get_deps() {
    //         match self.0.get(&dep) {
    //             Some(dep_app) => {
    //                 if dep_app.borrow().get_status() != AppStatus::Running {
    //                     deps_resolved = false;

    //                     break;
    //                 }
    //             },
    //             None => {
    //                 /* 
    //                     * If the app depends on a non-existent app, 
    //                     * then we cannot launch it
    //                     */
    //                 deps_resolved = false;

    //                 break;
    //             }
    //         };
    //     }

    //     deps_resolved
    // }
}