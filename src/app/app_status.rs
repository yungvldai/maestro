use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppStatus {
    // The app has been created, but no work has been done on it
    Init,

    // The app has running process attached to it
    Running,

    // Successfully invoked kill with the pid of the running process
    Stopping,

    // The process has stopped. Having a process to stop is mandatory.
    // There should be no transition Init -> Stopped
    Stopped,
}

impl fmt::Display for AppStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppStatus::Init => write!(f, "INIT"),
            AppStatus::Running => write!(f, "RUNNING"),
            AppStatus::Stopping => write!(f, "STOPPING"),
            AppStatus::Stopped => write!(f, "STOPPED"),
        }
    }
}
