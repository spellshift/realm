use super::super::insert_dict_kv;
use super::{File, FileType};
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap, Value},
};
use std::fs::DirEntry;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use sysinfo::{System, SystemExt, UserExt};

const UNKNOWN: &str = "UNKNOWN";

// https://stackoverflow.com/questions/6161776/convert-windows-filetime-to-second-in-unix-linux
#[cfg(target_os = "windows")]
fn windows_tick_to_unix_tick(windows_tick: u64) -> i64 {
    const WINDOWS_TICK: u64 = 10000000;
    const SEC_TO_UNIX_EPOCH: u64 = 11644473600;
    return (windows_tick / WINDOWS_TICK - SEC_TO_UNIX_EPOCH) as i64;
}

fn create_file_from_dir_entry(dir_entry: DirEntry) -> Result<File> {
    let mut sys = System::new();
    sys.refresh_users_list();

    let file_name = match dir_entry.file_name().into_string() {
        Ok(local_file_name) => local_file_name,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "file.list: Unable to convert file name to string."
            ))
        }
    };

    let file_type = match dir_entry.file_type() {
        Ok(tmp_file_type) => {
            if tmp_file_type.is_dir() {
                FileType::Directory
            } else if tmp_file_type.is_file() {
                FileType::File
            } else if tmp_file_type.is_symlink() {
                FileType::Link
            } else {
                FileType::Unknown
            }
        }
        Err(_) => FileType::Unknown,
    };

    let dir_entry_metadata = dir_entry.metadata()?;

    let file_size = dir_entry_metadata.len();

    #[cfg(unix)]
    let owner_username = match sysinfo::Uid::try_from(dir_entry_metadata.st_uid() as usize) {
        Ok(local_uid) => match sys.get_user_by_id(&local_uid) {
            Some(user_name_string) => user_name_string.name().to_string(),
            None => UNKNOWN.to_string(),
        },
        Err(_) => UNKNOWN.to_string(),
    };

    #[cfg(not(unix))]
    let owner_username = { UNKNOWN.to_string() };

    #[cfg(unix)]
    let group_id = { dir_entry_metadata.st_gid().to_string() };
    #[cfg(not(unix))]
    let group_id = {
        UNKNOWN // This is bad but windows file ownership is very different.
    };

    #[cfg(unix)]
    let timestamp = { dir_entry_metadata.st_mtime() };
    #[cfg(not(unix))]
    let timestamp = {
        let win_timestamp = dir_entry_metadata.last_write_time();
        windows_tick_to_unix_tick(win_timestamp)
    };

    #[cfg(unix)]
    let permissions = { format!("{:o}", dir_entry_metadata.permissions().mode()) };
    #[cfg(target_os = "windows")]
    let permissions = {
        if dir_entry.metadata()?.permissions().readonly() {
            "Read only"
        } else {
            "Not read only"
        }
    };

    let naive_datetime = match NaiveDateTime::from_timestamp_opt(timestamp, 0) {
        Some(local_naive_datetime) => local_naive_datetime,
        None => {
            return Err(anyhow::anyhow!(
                "Failed to get time from timestamp for file {}",
                file_name
            ))
        }
    };
    let time_modified: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);

    Ok(File {
        name: file_name,
        file_type: file_type,
        size: file_size,
        owner: owner_username,
        group: group_id.to_string(),
        permissions: permissions.to_string(),
        time_modified: time_modified.to_string(),
    })
}

fn handle_list(path: String) -> Result<Vec<File>> {
    let paths = std::fs::read_dir(path)?;
    let mut final_res = Vec::new();
    for path in paths {
        final_res.push(create_file_from_dir_entry(path?)?);
    }
    Ok(final_res)
}

fn create_dict_from_file(starlark_heap: &Heap, file: File) -> Result<Dict> {
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut dict_res = Dict::new(res);

    insert_dict_kv!(dict_res, starlark_heap, "file_name", &file.name, String);
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

pub fn list(starlark_heap: &Heap, path: String) -> Result<Vec<Dict>> {
    let mut final_res: Vec<Dict> = Vec::new();
    let file_list = match handle_list(path) {
        Ok(local_file_list) => local_file_list,
        Err(local_err) => {
            return Err(anyhow::anyhow!(
                "Failed to get file list: {}",
                local_err.to_string()
            ))
        }
    };
    for file in file_list {
        let tmp_res = create_dict_from_file(starlark_heap, file)?;
        final_res.push(tmp_res);
    }
    Ok(final_res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_list() -> anyhow::Result<()> {
        let test_dir = tempdir()?;
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

        let binding = Heap::new();
        let list_res = list(&binding, test_dir.path().to_str().unwrap().to_string())?;
        println!("{:?}", list_res);
        assert_eq!(list_res.len(), (expected_dirs.len() + expected_files.len()));

        Ok(())
    }
}
