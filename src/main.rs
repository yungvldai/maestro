mod app;
mod config;
mod fs;
mod logger;
mod pid;
mod readiness_probe;
mod user;
mod utils;

use crate::{app::AppsMap, logger::init_logger, pid::init_pid};
use app::AppStatus;
use config::Config;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{sync::mpsc, thread, time};

const POLL_PERIOD: u64 = 100;

enum MainState {
    Running,
    Stopping,
}

fn main() {
    let config = Config::new().validate();

    init_logger(config.log_level.to_owned());
    init_pid(config.pid.to_owned());

    log::debug!("loaded config {:#?}", config);

    let mut signals = Signals::new([SIGTERM, SIGINT]).unwrap();
    let apps_map = AppsMap::new(config.apps);
    let (sender, receiver) = mpsc::channel::<i32>();
    let mut state = MainState::Running;
    let mut stop_flag = false;

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
                    if app.get_status() == AppStatus::Init {
                        let ready = apps_map
                            .get_dependencies_for(&app.get_name())
                            .iter()
                            .all(|app_name| apps_map.get(app_name).unwrap().borrow().is_ready());

                        if ready {
                            app.run();
                        }
                    }

                    if app.get_status() == AppStatus::Stopped
                        && app.get_exit_code().unwrap_or(1) != 0
                    {
                        /*
                         * The app failed, so system operation is not guaranteed
                         */

                        state = MainState::Stopping;
                    }
                }
                MainState::Stopping => {
                    if app.get_status() == AppStatus::Running {
                        let ready =
                            apps_map
                                .get_dependents_for(&app.get_name())
                                .iter()
                                .all(|app_name| {
                                    [AppStatus::Stopped, AppStatus::Init].contains(
                                        &apps_map.get(app_name).unwrap().borrow().get_status(),
                                    )
                                });

                        if ready {
                            app.stop();
                        }
                    }
                }
            }
        }

        /*
         * Finding all apps in the `Stopped` and `Init` statuses means
         * that apps that were running have already been stopped, and apps that
         * were not running will no longer start.
         *
         * In this case, we can do break
         */
        if apps_map
            .every(|app| [AppStatus::Stopped, AppStatus::Init].contains(&app.borrow().get_status()))
        {
            /*
             * If it happens that all apps are stopped or not running,
             * we stop maestro only in the second cycle.
             * An update needs to happen to make sure no app wants to run (after exit_code readiness probe, for example)
             */
            if !stop_flag {
                stop_flag = true
            } else {
                log::info!("all apps are stopped or have not been started, stopping...");
                break;
            }
        } else {
            stop_flag = false;
        }

        thread::sleep(time::Duration::from_millis(POLL_PERIOD));
    }
}
