use std::{
    cell::RefCell,
    collections::{hash_map::Values, HashMap},
};

use super::App;

pub struct AppsMap {
    map: HashMap<String, RefCell<App>>,
    run_after: HashMap<String, Vec<String>>,
    run_before: HashMap<String, Vec<String>>,
}

impl AppsMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            run_after: HashMap::new(),
            run_before: HashMap::new(),
        }
    }

    pub fn add(&mut self, app: App, deps: Vec<String>) {
        self.run_after.insert(app.get_name(), deps.to_owned());

        for dep in deps {
            self.run_before.entry(dep).or_default().push(app.get_name());
        }

        self.map.insert(app.get_name(), RefCell::new(app));
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
            Some(value) => value.to_owned(),
        }
    }

    pub fn get_dependents_for(&self, app_name: &String) -> Vec<String> {
        match self.run_before.get(app_name) {
            None => Vec::new(),
            Some(value) => value.to_owned(),
        }
    }
}
