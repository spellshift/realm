use std::{
    fs::{self},
    io::Write,
    path::Path,
};

use uuid::Uuid;

use crate::HostUniqueEngine;

pub struct File {}

impl File {
    /*
     * Returns a predefined path to the host id file based on the current platform.
     */
    fn get_host_id_path(&self) -> String {
        #[cfg(target_os = "windows")]
        return String::from("C:\\ProgramData\\system-id");

        #[cfg(target_os = "linux")]
        return String::from("/etc/system-id");

        #[cfg(target_os = "macos")]
        return String::from("/Users/Shared/system-id");

        #[cfg(target_os = "freebsd")]
        return String::from("/etc/systemd-id");
    }
}

impl HostUniqueEngine for File {
    fn get_name(&self) -> String {
        "file".to_string()
    }
    /*
     * Attempt to read a host-id from a predefined path on disk.
     * If the file exist, it's value will be returned as the identifier.
     * If the file does not exist, a new value will be generated and written to the file.
     * If there is any failure reading / writing the file, the generated id is still returned.
     */
    fn get_host_id(&self) -> Option<Uuid> {
        // Read Existing Host ID
        let file_path = self.get_host_id_path();
        let path = Path::new(file_path.as_str());
        if path.exists() {
            if let Ok(host_id) = fs::read_to_string(path) {
                match Uuid::parse_str(host_id.as_str()) {
                    Ok(res) => return Some(res),
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::debug!("Failed to deploy {:?}", _err);
                        return None;
                    }
                };
            }
        }

        // Generate New
        let host_id = Uuid::new_v4();

        // Save to file
        match std::fs::File::create(path) {
            Ok(mut f) => match f.write_all(host_id.as_bytes()) {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to write host id file: {_err}");
                }
            },
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to create host id file: {_err}");
            }
        };

        Some(host_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_file() {
        let engine = File {};
        let id_one = engine.get_host_id();
        let id_two = engine.get_host_id();

        assert_eq!(id_one, id_two);
    }
}
