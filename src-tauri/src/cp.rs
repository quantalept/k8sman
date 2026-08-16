use std::io::Cursor;
use std::path::{Path, PathBuf};

use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams};
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

const CHUNK_SIZE: usize = 64 * 1024;

fn split_remote_path(remote_path: &str) -> AppResult<(String, String)> {
    let path = Path::new(remote_path);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::Message(format!("invalid remote path: {remote_path}")))?
        .to_string();
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    Ok((dir, name))
}

async fn check_exit_status(
    mut process: kube::api::AttachedProcess,
    context: &str,
) -> AppResult<()> {
    if let Some(status_fut) = process.take_status() {
        if let Some(status) = status_fut.await {
            if status.status.as_deref() != Some("Success") {
                let message = status
                    .message
                    .unwrap_or_else(|| "remote command failed".into());
                return Err(AppError::Message(format!("{context}: {message}")));
            }
        }
    }
    process.join().await.map_err(|e| AppError::Message(e.to_string()))
}

/// Upload a local file or directory into a pod, using the same tar-over-exec mechanism as
/// `kubectl cp`. `remote_dir` is the destination directory; the item lands at
/// `remote_dir/<basename of local_path>`.
#[tauri::command]
pub async fn cp_to_pod(
    app: AppHandle,
    state: State<'_, AppState>,
    context_name: String,
    namespace: String,
    pod: String,
    container: Option<String>,
    local_path: String,
    remote_dir: String,
    transfer_id: String,
) -> AppResult<()> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Pod> = Api::namespaced(client, &namespace);

    let progress_event = format!("cp-progress:{transfer_id}");
    let local_path_buf = PathBuf::from(&local_path);

    let tar_bytes = tokio::task::spawn_blocking(move || -> AppResult<Vec<u8>> {
        let name = local_path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AppError::Message(format!("invalid local path: {local_path}")))?
            .to_string();
        let mut builder = tar::Builder::new(Vec::new());
        if local_path_buf.is_dir() {
            builder.append_dir_all(&name, &local_path_buf)?;
        } else {
            builder.append_path_with_name(&local_path_buf, &name)?;
        }
        Ok(builder.into_inner()?)
    })
    .await
    .map_err(|e| AppError::Message(e.to_string()))??;

    let total = tar_bytes.len();
    let mut ap = AttachParams::default().stdin(true).stdout(false).stderr(true);
    if let Some(container) = container {
        ap = ap.container(container);
    }

    let mut process = api
        .exec(&pod, vec!["tar", "xf", "-", "-C", &remote_dir], &ap)
        .await
        .map_err(AppError::Kube)?;
    let mut stdin = process
        .stdin()
        .ok_or_else(|| AppError::Message("exec session has no stdin".into()))?;

    let mut sent = 0usize;
    for chunk in tar_bytes.chunks(CHUNK_SIZE) {
        stdin.write_all(chunk).await.map_err(AppError::Io)?;
        sent += chunk.len();
        let _ = app.emit(&progress_event, serde_json::json!({ "sent": sent, "total": total }));
    }
    stdin.shutdown().await.map_err(AppError::Io)?;
    drop(stdin);

    check_exit_status(process, "upload failed").await
}

/// Download a file or directory from a pod, using the same tar-over-exec mechanism as
/// `kubectl cp`. `local_dir` is the destination directory; the item lands at
/// `local_dir/<basename of remote_path>`.
#[tauri::command]
pub async fn cp_from_pod(
    app: AppHandle,
    state: State<'_, AppState>,
    context_name: String,
    namespace: String,
    pod: String,
    container: Option<String>,
    remote_path: String,
    local_dir: String,
    transfer_id: String,
) -> AppResult<()> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Pod> = Api::namespaced(client, &namespace);
    let (remote_parent, remote_name) = split_remote_path(&remote_path)?;

    let mut ap = AttachParams::default().stdin(false).stdout(true).stderr(true);
    if let Some(container) = container {
        ap = ap.container(container);
    }

    let mut process = api
        .exec(&pod, vec!["tar", "cf", "-", "-C", &remote_parent, &remote_name], &ap)
        .await
        .map_err(AppError::Kube)?;
    let mut stdout = process
        .stdout()
        .ok_or_else(|| AppError::Message("exec session has no stdout".into()))?;

    let progress_event = format!("cp-progress:{transfer_id}");
    let mut buf = [0u8; CHUNK_SIZE];
    let mut received = Vec::new();
    loop {
        let n = stdout.read(&mut buf).await.map_err(AppError::Io)?;
        if n == 0 {
            break;
        }
        received.extend_from_slice(&buf[..n]);
        let _ = app.emit(
            &progress_event,
            serde_json::json!({ "sent": received.len(), "total": received.len() }),
        );
    }

    check_exit_status(process, "download failed").await?;

    let local_dir_buf = PathBuf::from(&local_dir);
    tokio::task::spawn_blocking(move || -> AppResult<()> {
        let mut archive = tar::Archive::new(Cursor::new(received));
        archive.unpack(&local_dir_buf)?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Message(e.to_string()))??;

    Ok(())
}
