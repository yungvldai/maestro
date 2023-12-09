mod config;
mod app;
mod user;
mod logger;
mod exec;
mod pid;
mod utils;

use std::{thread, time, sync::mpsc};
use app::AppStatus;
use config::Config;
use signal_hook::{iterator::Signals, consts::{SIGTERM, SIGINT}};
use crate::{logger::init_logger, app::AppsMap, pid::init_pid};

const POLL_PERIOD: u64 = 100;

enum MainState {
    Running,
    Stopping
}

fn main() {
    let config = Config::new();

    init_logger(config.log_level.to_owned());
    init_pid(config.pid.to_owned());
    
    log::debug!("loaded config {:#?}", config);

    let mut signals = Signals::new([SIGTERM, SIGINT]).unwrap();
    let apps_map = AppsMap::new(config.apps);
    let (sender, receiver) = mpsc::channel::<i32>();
    let mut state = MainState::Running;

    thread::spawn(move || {
        for signal in signals.forever() {
            sender.send(signal).unwrap();
        }
    });

    loop {
        if let Ok(signal) = receiver.try_recv() {
            log::info!("received signal {:?}", signal);
            state = MainState::Stopping;
        }

        for app_rc in apps_map.list() {
            let mut app = app_rc.borrow_mut();

            app.update();

            match state {
                MainState::Running => {
                    if app.get_status() == AppStatus::Init && apps_map.check_deps_resolved_for(&app) {
                        app.run();
                    }
        
                    if app.get_status() == AppStatus::Stopped && app.get_exit_code().unwrap_or(1) != 0 {
                        /*
                         * The app failed, so system operation is not guaranteed
                         */
                        state = MainState::Stopping;
                    }
                },
                MainState::Stopping => {

                }
            }
        }

        if apps_map.every(|app| {
            [AppStatus::Stopped, AppStatus::Init].contains(&app.borrow().get_status())
        }) {
            /*
             * Finding all apps in the `Stopped` and `Init` statuses means 
             * that apps that were running have already been stopped, and apps that 
             * were not running will no longer start. 
             * 
             * In this case, we can do break
             */

            log::info!("all apps are stopped or have not been started, stopping...");
            
            break;
        }

        thread::sleep(time::Duration::from_millis(POLL_PERIOD));
    }
}
