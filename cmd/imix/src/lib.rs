use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct C2Config {
    uri: String,
    timeout: u32,
    priority: u8,
    sticky: bool,
    failsafe: bool
}

#[derive(Serialize, Deserialize)]
pub struct ServiceConfig {
    name: String,
    description: String,
    executable_path: String
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    target_name: String,
    callback_interval: u32,
    callback_jitter: u32,
    c2_configs: Vec<C2Config>,
    service_configs: Vec<ServiceConfig>,
}

pub mod windows;
pub mod linux;