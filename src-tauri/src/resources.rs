use std::sync::Arc;

use futures::StreamExt;
use kube::api::{Api, DynamicObject, ListParams};
use kube::discovery::{ApiCapabilities, ApiResource, Discovery, Scope};
use kube::runtime::{watcher, WatchStreamExt};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// A resource kind available on the connected cluster, as discovered from its API server.
#[derive(Debug, Clone, Serialize)]
pub struct ResourceKindRef {
    pub group: String,
    pub version: String,
    pub kind: String,
    pub namespaced: bool,
}

async fn discovery_for(
    state: &State<'_, AppState>,
    context_name: &str,
) -> AppResult<Arc<Discovery>> {
    if let Some(d) = state.discovery.0.lock().unwrap().get(context_name) {
        return Ok(d.clone());
    }

    let client = get_client(state, context_name)?;
    let discovery = Discovery::new(client)
        .run()
        .await
        .map_err(AppError::Kube)?;
    let arc = Arc::new(discovery);
    state
        .discovery
        .0
        .lock()
        .unwrap()
        .insert(context_name.to_string(), arc.clone());
    Ok(arc)
}

fn resolve_kind(discovery: &Discovery, kind: &str) -> AppResult<(ApiResource, ApiCapabilities)> {
    // Prefer the core group (e.g. plain "Pod" over some CRD also named Pod).
    if let Some(core) = discovery.groups().find(|g| g.name().is_empty()) {
        if let Some(found) = core.recommended_kind(kind) {
            return Ok(found);
        }
    }
    for group in discovery.groups() {
        if let Some(found) = group.recommended_kind(kind) {
            return Ok(found);
        }
    }
    Err(AppError::Message(format!("resource kind not found: {kind}")))
}

fn dynamic_api(
    client: kube::Client,
    ar: &ApiResource,
    caps: &ApiCapabilities,
    namespace: Option<&str>,
) -> Api<DynamicObject> {
    match (&caps.scope, namespace) {
        (Scope::Namespaced, Some(ns)) => Api::namespaced_with(client, ns, ar),
        _ => Api::all_with(client, ar),
    }
}

/// List every resource kind discovered on the connected cluster (built-ins and CRDs alike).
#[tauri::command]
pub async fn list_resource_kinds(
    state: State<'_, AppState>,
    context_name: String,
) -> AppResult<Vec<ResourceKindRef>> {
    let discovery = discovery_for(&state, &context_name).await?;
    let mut out = Vec::new();
    for group in discovery.groups() {
        for (ar, caps) in group.recommended_resources() {
            out.push(ResourceKindRef {
                group: ar.group.clone(),
                version: ar.version.clone(),
                kind: ar.kind.clone(),
                namespaced: caps.scope == Scope::Namespaced,
            });
        }
    }
    Ok(out)
}

/// List all objects of a given kind, optionally scoped to a namespace, as raw JSON.
#[tauri::command]
pub async fn list_resources(
    state: State<'_, AppState>,
    context_name: String,
    kind: String,
    namespace: Option<String>,
) -> AppResult<Vec<Value>> {
    let client = get_client(&state, &context_name)?;
    let discovery = discovery_for(&state, &context_name).await?;
    let (ar, caps) = resolve_kind(&discovery, &kind)?;
    let api = dynamic_api(client, &ar, &caps, namespace.as_deref());

    let list = api.list(&ListParams::default()).await?;
    Ok(list
        .items
        .into_iter()
        .map(|obj| serde_json::to_value(obj).unwrap_or(Value::Null))
        .collect())
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "object", rename_all = "lowercase")]
pub enum ResourceEvent {
    Upsert(Value),
    Delete(Value),
}

/// Start watching a resource kind for live changes, streamed to the frontend as
/// `resource-event:{stream_id}` events. Returns the stream id, used to stop the watch later.
#[tauri::command]
pub async fn start_watch(
    app: AppHandle,
    state: State<'_, AppState>,
    context_name: String,
    kind: String,
    namespace: Option<String>,
) -> AppResult<String> {
    let client = get_client(&state, &context_name)?;
    let discovery = discovery_for(&state, &context_name).await?;
    let (ar, caps) = resolve_kind(&discovery, &kind)?;
    let api = dynamic_api(client, &ar, &caps, namespace.as_deref());

    let stream_id = Uuid::new_v4();
    let event_name = format!("resource-event:{stream_id}");
    let app_handle = app.clone();

    let join_handle = tokio::spawn(async move {
        let mut stream = Box::pin(watcher(api, watcher::Config::default()).default_backoff());
        while let Some(event) = stream.next().await {
            let payload = match event {
                Ok(watcher::Event::Apply(obj)) | Ok(watcher::Event::InitApply(obj)) => {
                    Some(ResourceEvent::Upsert(
                        serde_json::to_value(obj).unwrap_or(Value::Null),
                    ))
                }
                Ok(watcher::Event::Delete(obj)) => Some(ResourceEvent::Delete(
                    serde_json::to_value(obj).unwrap_or(Value::Null),
                )),
                Ok(_) => None,
                Err(err) => {
                    tracing::warn!("resource watch error: {err}");
                    None
                }
            };
            if let Some(payload) = payload {
                let _ = app_handle.emit(&event_name, payload);
            }
        }
    });

    state
        .streams
        .0
        .lock()
        .unwrap()
        .insert(stream_id, join_handle.abort_handle());

    Ok(stream_id.to_string())
}

/// Stop a previously started resource watch.
#[tauri::command]
pub async fn stop_watch(state: State<'_, AppState>, stream_id: String) -> AppResult<()> {
    let id = Uuid::parse_str(&stream_id).map_err(|e| AppError::Message(e.to_string()))?;
    if let Some(handle) = state.streams.0.lock().unwrap().remove(&id) {
        handle.abort();
    }
    Ok(())
}
