use k8s_openapi::api::core::v1::Pod;
use kube::api::Api;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardInfo {
    pub forward_id: String,
    pub namespace: String,
    pub pod: String,
    pub local_port: u16,
    pub remote_port: u16,
}

/// Start forwarding a local TCP port to a pod's port. Each incoming local connection opens
/// its own port-forward stream against the cluster (matching how `kubectl port-forward`
/// behaves), bridged with a bidirectional byte copy. Connection failures are reported on
/// `portforward-status:{forward_id}`.
#[tauri::command]
pub async fn start_port_forward(
    app: AppHandle,
    state: State<'_, AppState>,
    context_name: String,
    namespace: String,
    pod: String,
    remote_port: u16,
    local_port: Option<u16>,
) -> AppResult<PortForwardInfo> {
    let client = get_client(&state, &context_name)?;

    let bind_addr = format!("127.0.0.1:{}", local_port.unwrap_or(0));
    let listener = TcpListener::bind(&bind_addr).await.map_err(AppError::Io)?;
    let actual_local_port = listener.local_addr().map_err(AppError::Io)?.port();

    let forward_id = Uuid::new_v4();
    let status_event = format!("portforward-status:{forward_id}");
    let task_namespace = namespace.clone();
    let task_pod = pod.clone();
    let app_handle = app.clone();

    let accept_task = tokio::spawn(async move {
        loop {
            let (mut inbound, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(err) => {
                    let _ = app_handle.emit(&status_event, format!("listener error: {err}"));
                    break;
                }
            };

            let client = client.clone();
            let namespace = task_namespace.clone();
            let pod = task_pod.clone();
            let app_handle = app_handle.clone();
            let status_event = status_event.clone();

            tokio::spawn(async move {
                let api: Api<Pod> = Api::namespaced(client, &namespace);
                let mut pf = match api.portforward(&pod, &[remote_port]).await {
                    Ok(pf) => pf,
                    Err(err) => {
                        let _ = app_handle.emit(&status_event, format!("connection failed: {err}"));
                        return;
                    }
                };
                if let Some(mut upstream) = pf.take_stream(remote_port) {
                    let _ = tokio::io::copy_bidirectional(&mut inbound, &mut upstream).await;
                }
            });
        }
    });

    state
        .streams
        .0
        .lock()
        .unwrap()
        .insert(forward_id, accept_task.abort_handle());

    Ok(PortForwardInfo {
        forward_id: forward_id.to_string(),
        namespace,
        pod,
        local_port: actual_local_port,
        remote_port,
    })
}

/// Stop accepting new connections for a port forward. Already-bridged connections drain
/// naturally rather than being force-closed.
#[tauri::command]
pub async fn stop_port_forward(state: State<'_, AppState>, forward_id: String) -> AppResult<()> {
    let id = Uuid::parse_str(&forward_id).map_err(|e| AppError::Message(e.to_string()))?;
    if let Some(handle) = state.streams.0.lock().unwrap().remove(&id) {
        handle.abort();
    }
    Ok(())
}
