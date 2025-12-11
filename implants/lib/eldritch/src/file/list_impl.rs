use super::super::insert_dict_kv;
use super::{File, FileType};
use anyhow::{Context, Result};
use chrono::DateTime;
use glob::glob;
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap, Value},
};
use std::fs::{self};
#[cfg(target_os = "freebsd")]
use std::os::freebsd::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

use std::path::{Path, PathBuf};
#[cfg(not(target_os = "windows"))]
use sysinfo::UserExt;

use sysinfo::{System, SystemExt};
const UNKNOWN: &str = "UNKNOWN";

// https://stackoverflow.com/questions/6161776/convert-windows-filetime-to-second-in-unix-linux
#[cfg(target_os = "windows")]
fn windows_tick_to_unix_tick(windows_tick: u64) -> i64 {
    const WINDOWS_TICK: u64 = 10000000;
    const SEC_TO_UNIX_EPOCH: u64 = 11644473600;
    (windows_tick / WINDOWS_TICK - SEC_TO_UNIX_EPOCH) as i64
}

fn create_file_from_pathbuf(path_entry: PathBuf) -> Result<File> {
    let mut sys: System = System::new();
    sys.refresh_users_list();
    let file_name = path_entry
        .file_name()
        .context("file.list: Failed to get filename")?
        .to_str()
        .context("file.list: Unable to convert file name to string")?
        .to_string();

    let absolute_path = fs::canonicalize(&path_entry)?
        .to_str()
        .context("file.list: Failed to canonicalize path")?
        .to_string();

    let file_type = match &path_entry {
        t if t.is_dir() => FileType::Directory,
        t if t.is_file() => FileType::File,
        t if t.is_symlink() => FileType::Link,
        _ => FileType::Unknown,
    };

    let file_metadata = &path_entry.metadata()?;
    let file_size = file_metadata.len();

    #[cfg(unix)]
    let owner_username = match sysinfo::Uid::try_from(file_metadata.st_uid() as usize) {
        Ok(local_uid) => match sys.get_user_by_id(&local_uid) {
            Some(user_name_string) => user_name_string.name().to_string(),
            None => UNKNOWN.to_string(),
        },
        Err(_) => UNKNOWN.to_string(),
    };

    #[cfg(not(unix))]
    let owner_username = { UNKNOWN.to_string() };

    #[cfg(unix)]
    let group_id = { file_metadata.st_gid().to_string() };
    #[cfg(not(unix))]
    let group_id = {
        UNKNOWN // This is bad but windows file ownership is very different.
    };

    #[cfg(unix)]
    let timestamp = { file_metadata.st_mtime() };
    #[cfg(not(unix))]
    let timestamp = {
        let win_timestamp = file_metadata.last_write_time();
        windows_tick_to_unix_tick(win_timestamp)
    };

    #[cfg(unix)]
    let permissions = { format!("{:o}", file_metadata.permissions().mode()) };
    #[cfg(target_os = "windows")]
    let permissions = {
        if file_metadata.permissions().readonly() {
            "Read only"
        } else {
            "Not read only"
        }
    };

    let naive_datetime = match DateTime::from_timestamp(timestamp, 0) {
        Some(local_naive_datetime) => local_naive_datetime,
        None => {
            return Err(anyhow::anyhow!(
                "Failed to get time from timestamp for file {}",
                file_name
            ))
        }
    };

    Ok(File {
        name: file_name,
        absolute_path,
        file_type,
        size: file_size,
        owner: owner_username,
        group: group_id.to_string(),
        permissions: permissions.to_string(),
        time_modified: naive_datetime.to_string(),
    })
}

fn handle_list(path: String) -> Result<Vec<File>> {
    let mut final_res = Vec::new();
    let input_path = Path::new(&path);

    if input_path.is_dir() {
        let paths = std::fs::read_dir(path)?;
        for dirent in paths {
            let pathbuf = dirent?.path();
            final_res.push(create_file_from_pathbuf(pathbuf)?);
        }
    } else {
        let res = create_file_from_pathbuf(PathBuf::from(path))?;
        final_res.push(res);
    }

    Ok(final_res)
}

fn create_dict_from_file(starlark_heap: &'_ Heap, file: File) -> Result<Dict<'_>> {
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut dict_res = Dict::new(res);

    insert_dict_kv!(dict_res, starlark_heap, "file_name", &file.name, String);
    insert_dict_kv!(
        dict_res,
        starlark_heap,
        "absolute_path",
        &file.absolute_path,
        String
    );
    insert_dict_kv!(dict_res, starlark_heap, "size", file.size, u64);
    insert_dict_kv!(dict_res, starlark_heap, "owner", &file.owner, String);
    insert_dict_kv!(dict_res, starlark_heap, "group", &file.group, String);
    insert_dict_kv!(
        dict_res,
        starlark_heap,
        "permissions",
        &file.permissions,
        String
    );
    insert_dict_kv!(
        dict_res,
        starlark_heap,
        "modified",
        &file.time_modified,
        String
    );
    insert_dict_kv!(
        dict_res,
        starlark_heap,
        "type",
        &file.file_type.to_string(),
        String
    );

    Ok(dict_res)
}

pub fn list(starlark_heap: &'_ Heap, path: String) -> Result<Vec<Dict<'_>>> {
    let mut final_res: Vec<Dict> = Vec::new();
    for entry in glob(&path)? {
        match entry {
            Ok(entry_path) => {
                let file_list = match handle_list(
                    entry_path
                        .to_str()
                        .context("Failed to convert string")?
                        .to_string(),
                ) {
                    Ok(local_file_list) => local_file_list,
                    Err(local_err) => {
                        return Err(anyhow::anyhow!("Failed to get file list: {}", local_err))
                    }
                };
                for file in file_list {
                    let tmp_res = create_dict_from_file(starlark_heap, file)?;
                    final_res.push(tmp_res);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(final_res)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::runtime::{messages::AsyncMessage, Message};

    use super::*;
    use pb::eldritch::Tome;
    use tempfile::{tempdir, NamedTempFile};

    fn init_logging() {
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Info)
            .parse_env("IMIX_LOG")
            .try_init();
    }

    #[tokio::test]
    async fn test_file_list_file() -> anyhow::Result<()> {
        init_logging();
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        let expected_file_name =
            String::from(tmp_file.path().file_name().unwrap().to_str().unwrap());

        // Run Eldritch (until finished)
        let mut runtime = crate::start(
            123,
            Tome {
                eldritch: String::from(r#"print(file.list(input_params['path'])[0]['file_name'])"#),
                parameters: HashMap::from([(String::from("path"), path.clone())]),
                file_names: Vec::new(),
            },
            pb::config::Config::default_with_imix_verison("0.0.0"),
        )
        .await;
        runtime.finish().await;

        // Read Messages
        let mut found = false;
        for msg in runtime.messages() {
            if let Message::Async(AsyncMessage::ReportText(m)) = msg {
                assert_eq!(123, m.id);
                assert!(m.text.contains(&expected_file_name));
                log::debug!("text: {:?}", m.text);
                found = true;
            }
        }
        assert!(found);
        Ok(())
    }

    #[tokio::test]
    async fn test_file_list_dir() -> anyhow::Result<()> {
        init_logging();

        let test_dir = tempdir()?;
        let path = test_dir
            .path()
            .to_str()
            .context("Failed to convert string")?
            .to_string();
        let expected_dirs = ["never gonna", "give you up", ".never gonna"];
        let expected_files = ["let_you_down", ".or desert you"];

        for dir in expected_dirs {
            let test_dir_to_create = test_dir.path().join(dir);
            std::fs::create_dir(test_dir_to_create)?;
        }

        for dir in expected_files {
            let test_dir_to_create = test_dir.path().join(dir);
            std::fs::File::create(test_dir_to_create)?;
        }

        // Run Eldritch (until finished)
        let mut runtime = crate::start(
            123,
            Tome {
                eldritch: String::from(
                    r#"
for f in file.list(input_params['path']):
    print(f['file_name'])"#,
                ),
                parameters: HashMap::from([(String::from("path"), path.clone())]),
                file_names: Vec::new(),
            },
            pb::config::Config::default_with_imix_verison("0.0.0"),
        )
        .await;
        runtime.finish().await;

        let mut counter = 0;
        for msg in runtime.messages() {
            if let Message::Async(AsyncMessage::ReportText(m)) = msg {
                counter += 1;
                log::debug!("text: {:?}", m.text);
            }
        }
        assert_eq!(counter, (expected_dirs.len() + expected_files.len()));

        Ok(())
    }

    #[tokio::test]
    async fn test_file_list_glob() -> anyhow::Result<()> {
        let test_dir = tempdir()?;
        let expected_dir = "down the";
        let nested_dir = "rabbit hole";
        let file = "win";

        let test_dir_to_create = test_dir.path().join(expected_dir);
        std::fs::create_dir(test_dir_to_create)?;
        let test_nested_dir_to_create = test_dir.path().join(expected_dir).join(nested_dir);
        std::fs::create_dir(test_nested_dir_to_create)?;
        let test_file = test_dir
            .path()
            .join(expected_dir)
            .join(nested_dir)
            .join(file);
        std::fs::File::create(test_file)?;

        // Run Eldritch (until finished) /tmp/.tmpabc123/down the/rabbit hole/win
        let mut runtime = crate::start(
            123,
            Tome {
                eldritch: String::from(
                    r#"
for f in file.list(input_params['path']):
    print(f['file_name'])"#,
                ),
                parameters: HashMap::from([(
                    String::from("path"),
                    test_dir
                        .path()
                        .join(expected_dir)
                        .join("*")
                        .join("win")
                        .to_str()
                        .unwrap()
                        .to_string(),
                )]),
                file_names: Vec::new(),
            },
            pb::config::Config::default_with_imix_verison("0.0.0"),
        )
        .await;
        runtime.finish().await;

        let mut found = false;
        for msg in runtime.messages() {
            if let Message::Async(AsyncMessage::ReportText(m)) = msg {
                assert_eq!(123, m.id);
                assert!(m.text.contains(file));
                log::debug!("text: {:?}", m.text);
                found = true;
            }
        }
        assert!(found);

        Ok(())
    }
}
