use anyhow::{anyhow, Result};
use starlark::eval::Evaluator;
use std::fs::{canonicalize, DirEntry};
use crate::runtime::{messages::ReportErrorMessage, Environment};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{path::Path, time::UNIX_EPOCH};

fn check_path(
    path: &Path,
    name: Option<String>,
    file_type: Option<String>,
    permissions: Option<u64>,
    modified_time: Option<u64>,
    create_time: Option<u64>,
) -> Result<bool> {
    if let Some(name) = name {
        if !path
            .file_name()
            .ok_or(anyhow!("Failed to get item file name"))?
            .to_str()
            .ok_or(anyhow!("Failed to convert file name to str"))?
            .contains(&name)
        {
            return Ok(false);
        }
    }
    if let Some(file_type) = file_type {
        if !path.is_file() && file_type == "file" {
            return Ok(false);
        }
        if !path.is_dir() && file_type == "dir" {
            return Ok(false);
        }
    }
    if let Some(permissions) = permissions {
        let metadata = path.metadata()?.permissions();
        #[cfg(unix)]
        {
            if metadata.mode() != (permissions as u32) {
                return Ok(false);
            }
        }
        #[cfg(windows)]
        {
            if permissions == 0 && metadata.readonly() {
                return Ok(false);
            }
            if permissions == 1 && !metadata.readonly() {
                return Ok(false);
            }
        }
    }
    if let Some(modified_time) = modified_time {
        if path
            .metadata()?
            .modified()?
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            != modified_time
        {
            return Ok(false);
        }
    }
    if let Some(create_time) = create_time {
        if path
            .metadata()?
            .created()?
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            != create_time
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn search_dir<'v>(
    starlark_eval: &mut Evaluator<'v, '_>,
    path: &str,
    name: Option<String>,
    file_type: Option<String>,
    permissions: Option<u64>,
    modified_time: Option<u64>,
    create_time: Option<u64>,
) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    let res = Path::new(&path);
    if !res.is_dir() {
        return Err(anyhow!("Search path is not a directory"));
    }
    let env = Environment::from_extra(starlark_eval.extra)?;
    let entries = match res.read_dir() {
        Ok(res) => res,
        Err(err) => {
            env.send(ReportErrorMessage { id: env.id(), error: format!("Failed to read directory {}: {}\n", path, err) })?;
            return Ok(out);
        },
    };
    for entry in entries {
        let entry: DirEntry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.append(&mut search_dir(
                starlark_eval,
                path.to_str()
                    .ok_or(anyhow!("Failed to convert path to str"))?,
                name.clone(),
                file_type.clone(),
                permissions,
                modified_time,
                create_time,
            )?);
        }
        if check_path(
            &path,
            name.clone(),
            file_type.clone(),
            permissions,
            modified_time,
            create_time,
        )? {
            out.push(
                canonicalize(path)?
                    .to_str()
                    .ok_or(anyhow!("Failed to convert path to str"))?
                    .to_owned(),
            );
        }
    }
    Ok(out)
}

pub fn find<'v>(
    starlark_eval: &mut Evaluator<'v, '_>,
    path: String,
    name: Option<String>,
    file_type: Option<String>,
    permissions: Option<u64>,
    modified_time: Option<u64>,
    create_time: Option<u64>,
) -> Result<Vec<String>> {
    if let Some(perms) = permissions {
        if !cfg!(unix) && (perms != 0 || perms != 1) {
            return Err(anyhow::anyhow!(
                "Only readonly permissions are available on non-unix systems. Please use 0 or 1."
            ));
        }
    }
    search_dir(
        starlark_eval,
        &path,
        name,
        file_type,
        permissions,
        modified_time,
        create_time,
    )
}
