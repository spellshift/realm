extern crate alloc;
use alloc::collections::BTreeMap;

#[cfg(feature = "stdlib")]
use std::{fs::{exists, metadata}, time::{SystemTime, UNIX_EPOCH}};

#[cfg(feature = "stdlib")]
fn time_to_epoch(time: SystemTime) -> Result<i64, String> {
    // get the distance so the time can be represented as an epoch
    // epoch is in seconds
    let epoch_secs = time.duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs().cast_signed();
    Ok(epoch_secs)
}

fn fetch_times(path: String) -> Result<BTreeMap<String, i64>, String> {
    // check if exists
    if !exists(&path).map_err(|e| e.to_string())? {
        return Err(alloc::format!("Could not locate {}", path));
    }

    // open the file
    let meta = metadata(path).map_err(|e| e.to_string())?;
    
    // create the output
    let mut dict = BTreeMap::new();
    dict.insert("mtime".to_string(), time_to_epoch(meta.modified().map_err(|e| e.to_string())?)?);
    dict.insert("atime".to_string(), time_to_epoch(meta.accessed().map_err(|e| e.to_string())?)?);
    dict.insert("crtime".to_string(), time_to_epoch(meta.created().map_err(|e| e.to_string())?)?);

    // change time is only available on posix systems
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    #[cfg(unix)]
    dict.insert("ctime".to_string(), meta.ctime());

    Ok(dict)
}

#[cfg(feature = "stdlib")]
pub fn get_times(path: String) -> Result<BTreeMap<String, i64>, alloc::string::String> {
    Ok(fetch_times(path)?)
}

#[cfg(not(feature = "stdlib"))]
pub fn get_times(path: String) -> Result<BTreeMap<String, i64>, alloc::string::String> {
    return Err("stdlib required");
}