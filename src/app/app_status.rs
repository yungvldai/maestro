use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppStatus {
    Init,
    Running,
    Stopping,
    Stopped
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