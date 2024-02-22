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
            if metadata.mode() & 0o777 != (permissions as u32) {
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

#[cfg(test)]
mod tests {

    use std::{collections::HashMap, fs::Permissions, os::unix::fs::PermissionsExt};

    use tempfile::TempDir;
    use pb::eldritch::Tome;
    use crate::runtime::Message;

    #[tokio::test]
    #[cfg(unix)]
    async fn test_find_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "test").unwrap();
        let runtime = crate::start(
            123,
            Tome {
                eldritch: r#"print(file.find(input_params['dir_path'], name="test.txt", file_type="file"))"#
                    .to_owned(),
                parameters: HashMap::from([("dir_path".to_string(), dir.path().to_str().unwrap().to_string())]),
                file_names: Vec::new(),
            },
        )
        .await;

        //println!("{:?}", runtime.collect().into_iter().collect::<Vec<Message>>());

        let messages: Vec<Message> = runtime.collect().into_iter()
            .filter(|x| matches!(x, Message::ReportAggOutput(_))).collect();
        println!("{:?}", messages);
        let message = messages.first().unwrap();

        if let Message::ReportAggOutput(output) = message {
            let expected = if cfg!(target_os = "macos") {
                format!("[\"/private{}\"]\n", file.to_str().unwrap())
            } else {
                format!("[\"{}\"]\n", file.to_str().unwrap())
            };
            assert_eq!(output.text, expected);
        } else {
            panic!("Expected ReportAggOutputMessage");
        }
    }

    #[tokio::test]
    async fn test_find_dir() {
        let dir = TempDir::new().unwrap();
        let inner_dir = dir.path().join("testdir");
        std::fs::create_dir(&inner_dir).unwrap();
        let runtime = crate::start(
            123,
            Tome {
                eldritch: r#"print(file.find(input_params['dir_path'], name="testdir", file_type="dir"))"#
                    .to_owned(),
                parameters: HashMap::from([("dir_path".to_string(), dir.path().to_str().unwrap().to_string())]),
                file_names: Vec::new(),
            },
        )
        .await;

        //println!("{:?}", runtime.collect().into_iter().collect::<Vec<Message>>());

        let messages: Vec<Message> = runtime.collect().into_iter()
            .filter(|x| matches!(x, Message::ReportAggOutput(_))).collect();
        let message = messages.first().unwrap();

        if let Message::ReportAggOutput(output) = message {
            let expected = if cfg!(target_os = "macos") {
                format!("[\"/private{}\"]\n", inner_dir.to_str().unwrap())
            } else {
                format!("[\"{}\"]\n", inner_dir.to_str().unwrap())
            };
            assert_eq!(output.text, expected);
        } else {
            panic!("Expected ReportAggOutputMessage");
        }
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_runtime_error() {
        let dir = TempDir::new().unwrap();
        let inner_dir = dir.path().join("testdir");
        let inner_dir2 = dir.path().join("testdir2");
        std::fs::create_dir(&inner_dir).unwrap();
        std::fs::set_permissions(&inner_dir, Permissions::from_mode(0o000)).unwrap();
        std::fs::create_dir(&inner_dir2).unwrap();
        std::fs::set_permissions(&inner_dir2, Permissions::from_mode(0o000)).unwrap();
        let runtime = crate::start(
            123,
            Tome {
                eldritch: r#"print(file.find(input_params['dir_path'], name="randomname"))"#
                    .to_owned(),
                parameters: HashMap::from([("dir_path".to_string(), dir.path().to_str().unwrap().to_string())]),
                file_names: Vec::new(),
            },
        )
        .await;
        let messages: Vec<Message> = runtime.collect().into_iter()
        .filter(|x| matches!(x, Message::ReportAggOutput(_))).collect();
        let message = messages.first().unwrap();

        if let Message::ReportAggOutput(output) = message {
            assert!(output.error.is_some());
        } else {
            panic!("Expected ReportAggOutputMessage");
        }
    }

    #[tokio::test]
    async fn test_provided_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "test").unwrap();
        let runtime = crate::start(
            123,
            Tome {
                eldritch: r#"print(file.find(input_params['dir_path'], name="test.txt", file_type="file")"#
                    .to_owned(),
                parameters: HashMap::from([("dir_path".to_string(), file.to_str().unwrap().to_string())]),
                file_names: Vec::new(),
            },
        )
        .await;
        let messages: Vec<Message> = runtime.collect().into_iter()
        .filter(|x| matches!(x, Message::ReportAggOutput(_))).collect();
        let message = messages.first().unwrap();

        if let Message::ReportAggOutput(output) = message {
            assert!(output.error.is_some());
        } else {
            panic!("Expected ReportAggOutputMessage");
        }
    }
}