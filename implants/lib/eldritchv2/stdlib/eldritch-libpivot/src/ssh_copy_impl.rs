use crate::std::Session;
use crate::std::StdPivotLibrary;
use alloc::format;
use alloc::string::{String, ToString};
use anyhow::Result;

#[allow(clippy::too_many_arguments)]
async fn handle_ssh_copy(
    target: String,
    port: u16,
    src: String,
    dst: String,
    username: String,
    password: Option<String>,
    key: Option<String>,
    key_password: Option<&str>,
    timeout: Option<u32>,
) -> Result<()> {
    let mut ssh = tokio::time::timeout(
        std::time::Duration::from_secs(timeout.unwrap_or(3) as u64),
        Session::connect(
            username,
            password,
            key,
            key_password,
            format!("{target}:{port}"),
        ),
    )
    .await??;
    ssh.copy(&src, &dst).await?;
    ssh.close().await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    lib: &StdPivotLibrary,
    target: String,
    port: i64,
    src: String,
    dst: String,
    username: String,
    password: Option<String>,
    key: Option<String>,
    key_password: Option<String>,
    timeout: Option<i64>,
) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    let target_clone = target.clone();
    let port_u16 = port as u16;
    let src_clone = src.clone();
    let dst_clone = dst.clone();
    let username_clone = username.clone();
    let password_clone = password.clone();
    let key_clone = key.clone();
    let key_password_clone = key_password.clone();
    let timeout_u32 = timeout.map(|t| t as u32);

    let fut = async move {
        let key_pass_ref = key_password_clone.as_deref();

        let res = handle_ssh_copy(
            target_clone,
            port_u16,
            src_clone,
            dst_clone,
            username_clone,
            password_clone,
            key_clone,
            key_pass_ref,
            timeout_u32,
        ).await;

        let _ = tx.send(res);
    };

    lib.agent
        .spawn_subtask(lib.task_id, "ssh_copy".to_string(), alloc::boxed::Box::pin(fut))
        .map_err(|e| e.to_string())?;

    let response = rx.recv().map_err(|e| format!("Failed to receive result: {}", e))?;

    match response {
        Ok(_) => Ok("Success".to_string()),
        Err(e) => Err(format!("ssh_copy failed: {:?}", e)),
    }
}
