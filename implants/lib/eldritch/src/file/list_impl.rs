use anyhow::Result;
use starlark::{values::{dict::Dict, Heap, Value}, collections::SmallMap, const_frozen_string};
use sysinfo::{System, SystemExt, UserExt};
use std::{path::Path, os::{linux::fs::MetadataExt, unix::prelude::PermissionsExt}, fs::DirEntry};
use chrono::{Utc, DateTime, NaiveDateTime};
use super::{File, FileType};

fn create_file_from_dir_entry(dir_entry: DirEntry) -> Result<File> {
    let mut sys = System::new();
    sys.refresh_users_list();

    let file_name = match dir_entry.file_name().into_string() {
        Ok(local_file_name) => local_file_name,
        Err(_) => return Err(anyhow::anyhow!("file.list: Unable to convert file name to string.")),
    };

    let mut file_type = FileType::Unknown;
    let tmp_file_type = dir_entry.file_type().expect("file.list: Unable to determine type");
    if tmp_file_type.is_dir() {
        file_type = FileType::Directory;
    } else if tmp_file_type.is_file() {
        file_type = FileType::File;
    } else if tmp_file_type.is_symlink() {
        file_type = FileType::Link;
    }

    let dir_entry_metadata = dir_entry.metadata().expect("file.list: Unable to get dir_entry metadata.");

    let file_size = dir_entry_metadata.len();

    let tmp_owner_uid = dir_entry_metadata.st_uid();
    let tmp_sysinfo_owner_uid = sysinfo::Uid::try_from(tmp_owner_uid as usize).expect("file.list: Failed to convert u32 to UID");
    let owner_username = sys.get_user_by_id(&tmp_sysinfo_owner_uid).expect("file.list: Failed to resolve username from UID");

    let tmp_group_gid = dir_entry_metadata.st_gid();

    #[cfg(not(target_os = "windows"))]
    let permissions = format!("{:o}", dir_entry_metadata.permissions().mode());
    // Windows sucks.
    #[cfg(target_os = "windows")]
    let permissions = match dir_entry_metadata.permissions().readonly(){
        true => "Read Only".to_string(),
        false => "Not Read Only".to_string(),
    };

    let timestamp = dir_entry_metadata.st_mtime();
    let naive_datetime = NaiveDateTime::from_timestamp_opt(timestamp, 0).expect("file.list: Failed to convert epoch to timedate.");
    let time_modified: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    
    Ok(File {
        name: file_name,
        file_type: file_type,
        size: file_size,
        owner: owner_username.name().to_string(),
        group: tmp_group_gid.to_string(),
        permissions: permissions,
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

pub fn list(starlark_heap: &Heap, path: String) -> Result<Vec<Dict>> {
    let mut final_res: Vec<Dict> = Vec::new();
    for file in handle_list(path)? {
        let res: SmallMap<Value, Value> = SmallMap::new();
        let mut tmp_res = Dict::new(res);

        let tmp_value1 = starlark_heap.alloc_str(&file.name);
        tmp_res.insert_hashed(const_frozen_string!("file_name").to_value().get_hashed().unwrap(), tmp_value1.to_value());

        let file_size = file.size.try_into().expect(format!("`file.list`: Failed to convert file size {} from u32 to i32.", file.size).as_str());
        tmp_res.insert_hashed(const_frozen_string!("size").to_value().get_hashed().unwrap(), Value::new_int(file_size));

        let tmp_value2 = starlark_heap.alloc_str(&file.owner);
        tmp_res.insert_hashed(const_frozen_string!("owner").to_value().get_hashed().unwrap(), tmp_value2.to_value());

        let tmp_value3 = starlark_heap.alloc_str(&file.group);
        tmp_res.insert_hashed(const_frozen_string!("group").to_value().get_hashed().unwrap(), tmp_value3.to_value());

        let tmp_value4 = starlark_heap.alloc_str(&file.permissions);
        tmp_res.insert_hashed(const_frozen_string!("permissions").to_value().get_hashed().unwrap(), tmp_value4.to_value());

        let tmp_value5 = starlark_heap.alloc_str(&file.time_modified);
        tmp_res.insert_hashed(const_frozen_string!("modified").to_value().get_hashed().unwrap(), tmp_value5.to_value());

        let tmp_value6 = starlark_heap.alloc_str(&file.file_type.to_string());
        tmp_res.insert_hashed(const_frozen_string!("type").to_value().get_hashed().unwrap(), tmp_value6.to_value());

        final_res.push(tmp_res);

    }
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::{NamedTempFile, tempdir};

    #[test]
    fn test_file_list() -> anyhow::Result<()>{
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

        let list_res = handle_list(test_dir.path().to_str().unwrap().to_string())?;

        for path in list_res {
            println!("{:?}", path);
        }

        Ok(())
    }
}
