#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(unix)]
use nix::unistd::{Gid, Group, Uid, User};
#[cfg(feature = "stdlib")]
use std::fs;
#[cfg(feature = "stdlib")]
use std::path::Path;

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct FileMetadataInfo {
    pub permissions: String,
    pub owner: String,
    pub group: String,
    pub modified: Option<String>,
    pub size: u64,
    pub is_dir: bool,
    pub absolute_path: String,
}

#[cfg(feature = "stdlib")]
pub fn get_file_metadata(path: &Path) -> AnyhowResult<FileMetadataInfo> {
    use alloc::format;

    let metadata = fs::metadata(path)?;

    // Permissions (simplified)
    #[cfg(unix)]
    use ::std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    let perms = format!("{:o}", metadata.permissions().mode());
    #[cfg(not(unix))]
    let perms = if metadata.permissions().readonly() {
        "r"
    } else {
        "rw"
    }
    .to_string();

    // Owner and Group
    #[cfg(unix)]
    let (owner, group) = {
        use ::std::os::unix::fs::MetadataExt;
        let uid = metadata.uid();
        let gid = metadata.gid();

        let user = User::from_uid(Uid::from_raw(uid)).ok().flatten();
        let group = Group::from_gid(Gid::from_raw(gid)).ok().flatten();

        let owner_name = user.map(|u| u.name).unwrap_or_else(|| uid.to_string());
        let group_name = group.map(|g| g.name).unwrap_or_else(|| gid.to_string());
        (owner_name, group_name)
    };

    #[cfg(not(unix))]
    let (owner, group) = ("".to_string(), "".to_string());

    // Absolute Path
    let abs_path = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();

    // Times
    let modified = if let Ok(modified) = metadata.modified() {
        let dt: chrono::DateTime<chrono::Utc> = modified.into();
        Some(dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
    } else {
        None
    };

    Ok(FileMetadataInfo {
        permissions: perms,
        owner,
        group,
        modified,
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        absolute_path: abs_path,
    })
}
