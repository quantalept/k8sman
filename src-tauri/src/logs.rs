use futures::{AsyncBufReadExt, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, LogParams};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Start tailing a pod/container's logs, streamed to the frontend as `log:{stream_id}` events
/// (one event per line). Returns the stream id, used to stop the stream later.
#[tauri::command]
pub async fn start_log_stream(
    app: AppHandle,
    state: State<'_, AppState>,
    context_name: String,
    namespace: String,
    pod: String,
    container: Option<String>,
    follow: bool,
    tail_lines: Option<i64>,
    previous: bool,
) -> AppResult<String> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Pod> = Api::namespaced(client, &namespace);

    let log_params = LogParams {
        container,
        follow,
        tail_lines,
        previous,
        timestamps: false,
        ..Default::default()
    };

    let reader = api
        .log_stream(&pod, &log_params)
        .await
        .map_err(AppError::Kube)?;

    let stream_id = Uuid::new_v4();
    let event_name = format!("log:{stream_id}");
    let done_event = format!("log-done:{stream_id}");
    let app_handle = app.clone();

    let join_handle = tokio::spawn(async move {
        let mut lines = reader.lines();
        while let Some(line) = lines.next().await {
            match line {
                Ok(line) => {
                    let _ = app_handle.emit(&event_name, line);
                }
                Err(err) => {
                    let _ = app_handle.emit(&event_name, format!("[stream error: {err}]"));
                    break;
                }
            }
        }
        let _ = app_handle.emit(&done_event, ());
    });

    state
        .streams
        .0
        .lock()
        .unwrap()
        .insert(stream_id, join_handle.abort_handle());

    Ok(stream_id.to_string())
}

/// Stop a previously started log stream.
#[tauri::command]
pub async fn stop_log_stream(state: State<'_, AppState>, stream_id: String) -> AppResult<()> {
    let id = Uuid::parse_str(&stream_id).map_err(|e| AppError::Message(e.to_string()))?;
    if let Some(handle) = state.streams.0.lock().unwrap().remove(&id) {
        handle.abort();
    }
    Ok(())
}
