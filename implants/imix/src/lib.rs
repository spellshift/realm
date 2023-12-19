use std::sync::mpsc::Sender;

use eldritch::PrintHandler;
use serde::{Deserialize, Serialize};

pub mod exec;
pub mod init;
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
    executable_path: String,
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

struct ImixPrintHandler {
    pub sender: Sender<String>,
}

impl PrintHandler for ImixPrintHandler {
    fn println(&self, text: &str) -> anyhow::Result<()> {
        let res = match self.sender.send(text.to_string()) {
            Ok(local_res) => local_res,
            Err(local_err) => return Err(anyhow::anyhow!(local_err.to_string())),
        };
        Ok(res)
    }
}
