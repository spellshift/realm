#[cfg(feature = "stdlib")]
use alloc::string::{String, ToString};
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use std::path::Path;

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone, Default)]
pub struct FileInfo {
    pub owner: String,
    pub group: String,
    pub permissions: String,
    pub size: u64,
    pub modified: String,
    pub absolute_path: String,
    pub file_name: String,
    pub file_type: String,
}

#[cfg(feature = "stdlib")]
pub fn get_file_info(path: &Path) -> AnyhowResult<FileInfo> {
    use std::fs;

    let metadata = fs::metadata(path)?;

    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let is_dir = metadata.is_dir();
    let type_str = if is_dir { "dir" } else { "file" };

    let size = metadata.len();

    let (owner, group) = get_ownership(&metadata);
    let permissions = get_permissions(&metadata);

    // Absolute Path
    let abs_path = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();

    // Times
    let modified = if let Ok(modified) = metadata.modified() {
        let dt: chrono::DateTime<chrono::Utc> = modified.into();
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        "".to_string()
    };

    Ok(FileInfo {
        owner,
        group,
        permissions,
        size,
        modified,
        absolute_path: abs_path,
        file_name: name,
        file_type: type_str.to_string(),
    })
}

#[cfg(all(feature = "stdlib", unix))]
fn get_ownership(metadata: &std::fs::Metadata) -> (String, String) {
    use nix::unistd::{Gid, Group, Uid, User};
    use std::os::unix::fs::MetadataExt;

    let uid = metadata.uid();
    let gid = metadata.gid();

    // Use `as _` to handle potential type differences (u32 vs uid_t) on different Unix/BSD platforms
    let user = User::from_uid(Uid::from_raw(uid as _)).ok().flatten();
    let group = Group::from_gid(Gid::from_raw(gid as _)).ok().flatten();

    let owner_name = user.map(|u| u.name).unwrap_or_else(|| uid.to_string());
    let group_name = group.map(|g| g.name).unwrap_or_else(|| gid.to_string());

    (owner_name, group_name)
}

#[cfg(all(feature = "stdlib", not(unix)))]
fn get_ownership(_metadata: &std::fs::Metadata) -> (String, String) {
    // Windows/non-Unix fallback
    ("".to_string(), "".to_string())
}

#[cfg(all(feature = "stdlib", unix))]
fn get_permissions(metadata: &std::fs::Metadata) -> String {
    use alloc::format;
    use std::os::unix::fs::PermissionsExt;
    format!("{:o}", metadata.permissions().mode())
}

#[cfg(all(feature = "stdlib", not(unix)))]
fn get_permissions(metadata: &std::fs::Metadata) -> String {
    use alloc::string::ToString;
    if metadata.permissions().readonly() {
        "r".to_string()
    } else {
        "rw".to_string()
    }
}
