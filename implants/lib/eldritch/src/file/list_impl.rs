use super::super::insert_dict_kv;
use super::{File, FileType};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
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

use sysinfo::{System, SystemExt, UserExt};
const UNKNOWN: &str = "UNKNOWN";

// https://stackoverflow.com/questions/6161776/convert-windows-filetime-to-second-in-unix-linux
#[cfg(target_os = "windows")]
fn windows_tick_to_unix_tick(windows_tick: u64) -> i64 {
    const WINDOWS_TICK: u64 = 10000000;
    const SEC_TO_UNIX_EPOCH: u64 = 11644473600;
    return (windows_tick / WINDOWS_TICK - SEC_TO_UNIX_EPOCH) as i64;
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
        absolute_path,
        file_type,
        size: file_size,
        owner: owner_username,
        group: group_id.to_string(),
        permissions: permissions.to_string(),
        time_modified: time_modified.to_string(),
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

fn create_dict_from_file(starlark_heap: &Heap, file: File) -> Result<Dict> {
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

pub fn list(starlark_heap: &Heap, path: String) -> Result<Vec<Dict>> {
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
            }
            Err(e) => println!("{:?}", e),
        }
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
        assert_eq!(list_res.len(), (expected_dirs.len() + expected_files.len()));

        Ok(())
    }
    #[test]
    fn test_file_list_glob() -> anyhow::Result<()> {
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

        // /tmpdir/down the/*
        let binding = Heap::new();
        let list_res = list(
            &binding,
            test_dir
                .path()
                .join("*")
                .join("win")
                .to_str()
                .unwrap()
                .to_string(),
        )?;
        println!("{:?}", list_res);
        Ok(())
    }
}
