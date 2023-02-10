use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    SerdeJson(serde_json::Error)
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
    uri: String,
    priority: u8,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceConfig {
    name: String,
    description: String,
    executable_path: String
}

#[derive(Serialize, Deserialize)]
pub struct CallbackConfig {
    interval: u64,
    jitter: u64,
    timeout: u64,
    c2_configs: Vec<C2Config>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    target_name: String,
    target_forward_connect_ip: String,
    callback_config: CallbackConfig,
    service_configs: Vec<ServiceConfig>,
}

pub mod windows;
pub mod linux;
pub mod common;
pub mod graphql;