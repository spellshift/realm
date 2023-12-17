use serde::{Deserialize, Serialize};

pub mod exec;
pub mod init;
pub mod install;
pub mod tasks;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(error)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct C2Config {
    pub uri: String,
    pub priority: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    name: String,
    description: String,
    executable_name: String,
    executable_args: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CallbackConfig {
    pub interval: u64,
    pub jitter: u64,
    pub timeout: u64,
    pub c2_configs: Vec<C2Config>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub target_name: String,
    pub target_forward_connect_ip: String,
    pub callback_config: CallbackConfig,
    pub service_configs: Vec<ServiceConfig>,
}

pub type TaskID = i64;
