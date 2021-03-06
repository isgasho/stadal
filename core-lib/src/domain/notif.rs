use std::path::PathBuf;
use core::fmt;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum CoreNotification {
    TracingConfig {
        enabled: bool,
    },

    SendHost {},
    SendMemory {},
    SendLanguages {},
    SendSizes {},
    SendDisks {},
    SendCpu {},
    SendProcesses {},
    ClientStarted {
        #[serde(default)]
        config_dir: Option<PathBuf>,
        /// Path to additional plugins, included by the client.
        #[serde(default)]
        client_extras_dir: Option<PathBuf>,
    },
}

impl fmt::Display for CoreNotification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}