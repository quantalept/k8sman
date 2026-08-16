use futures::channel::mpsc::Sender as TerminalSizeSender;
use futures::SinkExt;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams, AttachedProcess, TerminalSize};
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub struct ExecSession {
    process: AttachedProcess,
    stdin: Box<dyn AsyncWrite + Unpin + Send>,
    terminal_size: Option<TerminalSizeSender<TerminalSize>>,
    read_task: tokio::task::AbortHandle,
}

/// Start an interactive exec session (a shell, by convention) in a pod/container.
/// Output is streamed to the frontend as `exec-output:{session_id}` events; a
/// `exec-done:{session_id}` event fires once the remote process's output stream ends.
#[tauri::command]
pub async fn start_exec(
    app: AppHandle,
    state: State<'_, AppState>,
    context_name: String,
    namespace: String,
    pod: String,
    container: Option<String>,
    command: Vec<String>,
) -> AppResult<String> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Pod> = Api::namespaced(client, &namespace);

    let mut ap = AttachParams::interactive_tty();
    if let Some(container) = container {
        ap = ap.container(container);
    }

    let mut process = api.exec(&pod, command, &ap).await.map_err(AppError::Kube)?;
    let stdin = process
        .stdin()
        .ok_or_else(|| AppError::Message("exec session has no stdin".into()))?;
    let mut stdout = process
        .stdout()
        .ok_or_else(|| AppError::Message("exec session has no stdout".into()))?;
    let terminal_size = process.terminal_size();

    let session_id = Uuid::new_v4();
    let output_event = format!("exec-output:{session_id}");
    let done_event = format!("exec-done:{session_id}");
    let app_handle = app.clone();

    let read_task = tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match stdout.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app_handle.emit(&output_event, text);
                }
            }
        }
        let _ = app_handle.emit(&done_event, ());
    });

    let session = ExecSession {
        process,
        stdin: Box::new(stdin),
        terminal_size,
        read_task: read_task.abort_handle(),
    };

    state
        .exec_sessions
        .0
        .lock()
        .await
        .insert(session_id, session);

    Ok(session_id.to_string())
}

/// Write raw bytes to an exec session's stdin (e.g. keystrokes from the terminal UI).
#[tauri::command]
pub async fn exec_write(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> AppResult<()> {
    let id = Uuid::parse_str(&session_id).map_err(|e| AppError::Message(e.to_string()))?;
    let mut sessions = state.exec_sessions.0.lock().await;
    let session = sessions
        .get_mut(&id)
        .ok_or_else(|| AppError::Message("unknown exec session".into()))?;
    session
        .stdin
        .write_all(data.as_bytes())
        .await
        .map_err(AppError::Io)?;
    Ok(())
}

/// Resize the exec session's tty.
#[tauri::command]
pub async fn exec_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> AppResult<()> {
    let id = Uuid::parse_str(&session_id).map_err(|e| AppError::Message(e.to_string()))?;
    let mut sessions = state.exec_sessions.0.lock().await;
    let session = sessions
        .get_mut(&id)
        .ok_or_else(|| AppError::Message("unknown exec session".into()))?;
    if let Some(tx) = session.terminal_size.as_mut() {
        tx.send(TerminalSize {
            width: cols,
            height: rows,
        })
        .await
        .map_err(|e| AppError::Message(e.to_string()))?;
    }
    Ok(())
}

/// Stop an exec session, aborting both the output reader and the remote process attachment.
#[tauri::command]
pub async fn stop_exec(state: State<'_, AppState>, session_id: String) -> AppResult<()> {
    let id = Uuid::parse_str(&session_id).map_err(|e| AppError::Message(e.to_string()))?;
    if let Some(session) = state.exec_sessions.0.lock().await.remove(&id) {
        session.read_task.abort();
        session.process.abort();
    }
    Ok(())
}
