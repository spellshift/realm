use std::{
    io::{Read, Write},
    path::Path,
    str,
};

use uuid::Uuid;

use crate::HostIDSelector;

const UUID_SIZE: usize = 36;

#[derive(Default)]
pub struct File {
    path_override: Option<String>,
}

impl File {
    /*
     * Returns a predefined path to the host id file based on the current platform.
     */
    fn get_host_id_path(&self) -> String {
        if let Some(override_path) = &self.path_override {
            return override_path.to_string();
        }

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

impl HostIDSelector for File {
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
            if let Ok(mut f) = std::fs::File::open(path) {
                let mut host_id: [u8; UUID_SIZE] = [0; UUID_SIZE];
                match f.read_exact(&mut host_id) {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::debug!("Failed to read host_id {:?}", _err);
                    }
                }
                if let Ok(uuid_str) = str::from_utf8(&host_id) {
                    match Uuid::parse_str(uuid_str) {
                        Ok(res) => return Some(res),
                        Err(_err) => {
                            #[cfg(debug_assertions)]
                            log::debug!("Failed to deploy {:?}", _err);
                        }
                    };
                }
            }
        }

        // Generate New
        let host_id = Uuid::new_v4();
        let uuid_str: String = format!("{}", host_id);

        // Save to file
        match std::fs::File::create(path) {
            Ok(mut f) => match f.write(uuid_str.as_bytes()) {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::debug!("failed to write host id file: {_err}");
                }
            },
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::debug!("failed to create host id file: {_err}");
            }
        };

        Some(host_id)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_id_file() {
        let tmp_file = NamedTempFile::new().unwrap();
        let path = String::from(tmp_file.path().to_str().unwrap());

        let selector = File {
            path_override: Some(path),
        };
        let id_one = selector.get_host_id();
        let id_two = selector.get_host_id();

        let id_one_val = id_one.unwrap();
        println!("failed to create host id file: {id_one_val}");

        assert!(id_one.is_some());
        assert!(id_two.is_some());
        assert_eq!(id_one, id_two);
        assert!(id_two.unwrap().to_string().contains("-4"));
    }
}
