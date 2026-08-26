use k8s_openapi::api::core::v1::{Node, Pod, Taint};
use kube::api::{Api, EvictParams, ListParams, Patch, PatchParams};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::cluster::get_client;
use crate::error::AppResult;
use crate::state::AppState;

/// The annotation static/mirror pods carry - these can't be evicted (the kubelet, not the
/// API server, owns their lifecycle), so a drain must skip them rather than error out.
const MIRROR_POD_ANNOTATION: &str = "kubernetes.io/config.mirror";

/// Cordon (or uncordon) a node by setting `spec.unschedulable`.
#[tauri::command]
pub async fn cordon_node(
    state: State<'_, AppState>,
    context_name: String,
    name: String,
    unschedulable: bool,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Node> = Api::all(client);
    let patch = serde_json::json!({ "spec": { "unschedulable": unschedulable } });
    let updated = api
        .patch(&name, &PatchParams::default(), &Patch::Merge(patch))
        .await?;
    Ok(serde_json::to_value(updated).unwrap_or(Value::Null))
}

/// Add (or replace, if the same key+effect already exists) a taint on a node.
#[tauri::command]
pub async fn taint_node(
    state: State<'_, AppState>,
    context_name: String,
    name: String,
    key: String,
    value: Option<String>,
    effect: String,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Node> = Api::all(client);
    let node = api.get(&name).await?;
    let mut taints: Vec<Taint> = node
        .spec
        .as_ref()
        .and_then(|s| s.taints.clone())
        .unwrap_or_default();
    taints.retain(|t| !(t.key == key && t.effect == effect));
    taints.push(Taint {
        key,
        value,
        effect,
        time_added: None,
    });

    let patch = serde_json::json!({ "spec": { "taints": taints } });
    let updated = api
        .patch(&name, &PatchParams::default(), &Patch::Merge(patch))
        .await?;
    Ok(serde_json::to_value(updated).unwrap_or(Value::Null))
}

/// Remove a taint matching key+effect from a node.
#[tauri::command]
pub async fn untaint_node(
    state: State<'_, AppState>,
    context_name: String,
    name: String,
    key: String,
    effect: String,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Node> = Api::all(client);
    let node = api.get(&name).await?;
    let mut taints: Vec<Taint> = node
        .spec
        .as_ref()
        .and_then(|s| s.taints.clone())
        .unwrap_or_default();
    taints.retain(|t| !(t.key == key && t.effect == effect));

    let patch = serde_json::json!({ "spec": { "taints": taints } });
    let updated = api
        .patch(&name, &PatchParams::default(), &Patch::Merge(patch))
        .await?;
    Ok(serde_json::to_value(updated).unwrap_or(Value::Null))
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum NodeDrainEvent {
    SkippedDaemonset { pod: String },
    SkippedMirror { pod: String },
    Evicting { pod: String },
    Evicted { pod: String },
    Error { pod: String, message: String },
    Done,
}

/// Drain a node: evict every pod scheduled on it, skipping DaemonSet-owned and static/mirror
/// pods (which can't be usefully evicted). This is a single pass, not a `kubectl drain`-style
/// retry-until-empty loop - a pod blocked by a PodDisruptionBudget surfaces as an `Error`
/// event and the caller re-runs the drain once that's resolved.
#[tauri::command]
pub async fn start_node_drain(
    app: AppHandle,
    state: State<'_, AppState>,
    context_name: String,
    name: String,
) -> AppResult<String> {
    let client = get_client(&state, &context_name)?;

    let stream_id = Uuid::new_v4();
    let event_name = format!("node-drain-event:{stream_id}");
    let app_handle = app.clone();
    let node_name = name.clone();

    let join_handle = tokio::spawn(async move {
        let pods: Api<Pod> = Api::all(client.clone());
        let lp = ListParams::default().fields(&format!("spec.nodeName={node_name}"));
        let list = match pods.list(&lp).await {
            Ok(list) => list,
            Err(err) => {
                let _ = app_handle.emit(
                    &event_name,
                    NodeDrainEvent::Error {
                        pod: String::new(),
                        message: err.to_string(),
                    },
                );
                let _ = app_handle.emit(&event_name, NodeDrainEvent::Done);
                return;
            }
        };

        for pod in list.items {
            let pod_name = match &pod.metadata.name {
                Some(n) => n.clone(),
                None => continue,
            };
            let is_daemonset = pod
                .metadata
                .owner_references
                .as_ref()
                .is_some_and(|refs| refs.iter().any(|r| r.kind == "DaemonSet"));
            if is_daemonset {
                let _ = app_handle.emit(&event_name, NodeDrainEvent::SkippedDaemonset { pod: pod_name });
                continue;
            }
            let is_mirror = pod
                .metadata
                .annotations
                .as_ref()
                .is_some_and(|a| a.contains_key(MIRROR_POD_ANNOTATION));
            if is_mirror {
                let _ = app_handle.emit(&event_name, NodeDrainEvent::SkippedMirror { pod: pod_name });
                continue;
            }

            let _ = app_handle.emit(
                &event_name,
                NodeDrainEvent::Evicting {
                    pod: pod_name.clone(),
                },
            );
            let ns_pods: Api<Pod> = match &pod.metadata.namespace {
                Some(ns) => Api::namespaced(client.clone(), ns),
                None => pods.clone(),
            };
            match ns_pods.evict(&pod_name, &EvictParams::default()).await {
                Ok(_) => {
                    let _ = app_handle.emit(&event_name, NodeDrainEvent::Evicted { pod: pod_name });
                }
                Err(err) => {
                    let _ = app_handle.emit(
                        &event_name,
                        NodeDrainEvent::Error {
                            pod: pod_name,
                            message: err.to_string(),
                        },
                    );
                }
            }
        }

        let _ = app_handle.emit(&event_name, NodeDrainEvent::Done);
    });

    state
        .streams
        .0
        .lock()
        .unwrap()
        .insert(stream_id, join_handle.abort_handle());

    Ok(stream_id.to_string())
}
