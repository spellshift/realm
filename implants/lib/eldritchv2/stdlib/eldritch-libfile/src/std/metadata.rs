#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use std::fs;
#[cfg(feature = "stdlib")]
use std::path::Path;

#[cfg(feature = "stdlib")]
pub struct FileMetadata {
    pub owner: String,
    pub group: String,
    pub permissions: String,
}

#[cfg(feature = "stdlib")]
pub fn get_metadata(path: &Path) -> AnyhowResult<FileMetadata> {
    let metadata = fs::metadata(path)?;

    // Permissions
    #[cfg(unix)]
    use ::std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    let perms = alloc::format!("{:o}", metadata.permissions().mode());
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
        use nix::unistd::{Gid, Group, Uid, User};

        let uid = metadata.uid();
        let gid = metadata.gid();

        let user = User::from_uid(Uid::from_raw(uid)).ok().flatten();
        let group_obj = Group::from_gid(Gid::from_raw(gid)).ok().flatten();

        let owner_name = user.map(|u| u.name).unwrap_or_else(|| uid.to_string());
        let group_name = group_obj.map(|g| g.name).unwrap_or_else(|| gid.to_string());
        (owner_name, group_name)
    };

    #[cfg(not(unix))]
    let (owner, group) = { ("".to_string(), "".to_string()) };

    Ok(FileMetadata {
        owner,
        group,
        permissions: perms,
    })
}
