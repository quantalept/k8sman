use std::sync::Arc;

use futures::StreamExt;
use kube::api::{Api, DeleteParams, DynamicObject, ListParams, Patch, PatchParams};
use kube::discovery::{ApiCapabilities, ApiResource, Discovery, Scope};
use kube::runtime::{watcher, WatchStreamExt};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

const FIELD_MANAGER: &str = "k8sman";

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

/// Resolves an exact group/version/kind (as parsed from a YAML document's `apiVersion`/`kind`)
/// rather than searching for a kind name across all groups. Needed because `apply_resource`
/// must target the version the document actually declares, not just "some" version of the kind.
fn resolve_by_gvk(
    discovery: &Discovery,
    group: &str,
    version: &str,
    kind: &str,
) -> AppResult<(ApiResource, ApiCapabilities)> {
    let api_group = discovery
        .groups()
        .find(|g| g.name() == group)
        .ok_or_else(|| AppError::Message(format!("unknown API group: {group:?}")))?;
    api_group
        .versioned_resources(version)
        .into_iter()
        .find(|(ar, _)| ar.kind == kind)
        .ok_or_else(|| {
            AppError::Message(format!("resource kind not found: {group}/{version} {kind}"))
        })
}

/// Splits a Kubernetes `apiVersion` string ("v1", "apps/v1") into (group, version).
fn split_api_version(api_version: &str) -> (&str, &str) {
    match api_version.split_once('/') {
        Some((group, version)) => (group, version),
        None => ("", api_version),
    }
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

/// List all objects of a given kind, optionally scoped to a namespace and/or filtered by a
/// field selector (e.g. `involvedObject.name=foo` for Events), as raw JSON.
#[tauri::command]
pub async fn list_resources(
    state: State<'_, AppState>,
    context_name: String,
    kind: String,
    namespace: Option<String>,
    field_selector: Option<String>,
) -> AppResult<Vec<Value>> {
    let client = get_client(&state, &context_name)?;
    let discovery = discovery_for(&state, &context_name).await?;
    let (ar, caps) = resolve_kind(&discovery, &kind)?;
    let api = dynamic_api(client, &ar, &caps, namespace.as_deref());

    let mut lp = ListParams::default();
    if let Some(selector) = field_selector {
        lp = lp.fields(&selector);
    }
    let list = api.list(&lp).await?;
    Ok(list
        .items
        .into_iter()
        .map(|obj| serde_json::to_value(obj).unwrap_or(Value::Null))
        .collect())
}

/// Fetch a single object by kind/name, optionally scoped to a namespace, as raw JSON.
#[tauri::command]
pub async fn get_resource(
    state: State<'_, AppState>,
    context_name: String,
    kind: String,
    namespace: Option<String>,
    name: String,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let discovery = discovery_for(&state, &context_name).await?;
    let (ar, caps) = resolve_kind(&discovery, &kind)?;
    let api = dynamic_api(client, &ar, &caps, namespace.as_deref());
    let obj = api.get(&name).await?;
    Ok(serde_json::to_value(obj).unwrap_or(Value::Null))
}

/// Delete a single object by kind/name, optionally scoped to a namespace.
#[tauri::command]
pub async fn delete_resource(
    state: State<'_, AppState>,
    context_name: String,
    kind: String,
    namespace: Option<String>,
    name: String,
) -> AppResult<()> {
    let client = get_client(&state, &context_name)?;
    let discovery = discovery_for(&state, &context_name).await?;
    let (ar, caps) = resolve_kind(&discovery, &kind)?;
    let api = dynamic_api(client, &ar, &caps, namespace.as_deref());
    api.delete(&name, &DeleteParams::default()).await?;
    Ok(())
}

/// Apply a YAML document via server-side apply. The target kind/version/namespace/name are
/// read from the document itself (`apiVersion`, `kind`, `metadata`).
#[tauri::command]
pub async fn apply_resource(
    state: State<'_, AppState>,
    context_name: String,
    yaml: String,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let obj: DynamicObject =
        serde_yaml::from_str(&yaml).map_err(|e| AppError::Message(format!("invalid YAML: {e}")))?;
    let types = obj
        .types
        .clone()
        .ok_or_else(|| AppError::Message("document is missing apiVersion/kind".into()))?;
    let name = obj
        .metadata
        .name
        .clone()
        .ok_or_else(|| AppError::Message("document is missing metadata.name".into()))?;

    let discovery = discovery_for(&state, &context_name).await?;
    let (group, version) = split_api_version(&types.api_version);
    let (ar, caps) = resolve_by_gvk(&discovery, group, version, &types.kind)?;
    let api = dynamic_api(client, &ar, &caps, obj.metadata.namespace.as_deref());

    let pp = PatchParams::apply(FIELD_MANAGER).force();
    let applied = api.patch(&name, &pp, &Patch::Apply(&obj)).await?;
    Ok(serde_json::to_value(applied).unwrap_or(Value::Null))
}

/// Set `spec.replicas` on a Deployment/ReplicaSet/StatefulSet via a merge patch.
#[tauri::command]
pub async fn scale_resource(
    state: State<'_, AppState>,
    context_name: String,
    kind: String,
    namespace: Option<String>,
    name: String,
    replicas: i32,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let discovery = discovery_for(&state, &context_name).await?;
    let (ar, caps) = resolve_kind(&discovery, &kind)?;
    let api = dynamic_api(client, &ar, &caps, namespace.as_deref());

    let patch = serde_json::json!({ "spec": { "replicas": replicas } });
    let updated = api
        .patch(&name, &PatchParams::default(), &Patch::Merge(patch))
        .await?;
    Ok(serde_json::to_value(updated).unwrap_or(Value::Null))
}

/// Trigger a rollout restart of a Deployment/StatefulSet/DaemonSet, the same way
/// `kubectl rollout restart` does: stamp the pod template with a restart annotation so the
/// controller rolls new pods.
#[tauri::command]
pub async fn restart_rollout(
    state: State<'_, AppState>,
    context_name: String,
    kind: String,
    namespace: Option<String>,
    name: String,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let discovery = discovery_for(&state, &context_name).await?;
    let (ar, caps) = resolve_kind(&discovery, &kind)?;
    let api = dynamic_api(client, &ar, &caps, namespace.as_deref());

    let now = chrono::Utc::now().to_rfc3339();
    let patch = serde_json::json!({
        "spec": {
            "template": {
                "metadata": {
                    "annotations": { "kubectl.kubernetes.io/restartedAt": now }
                }
            }
        }
    });
    let updated = api
        .patch(&name, &PatchParams::default(), &Patch::Merge(patch))
        .await?;
    Ok(serde_json::to_value(updated).unwrap_or(Value::Null))
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
    field_selector: Option<String>,
) -> AppResult<String> {
    let client = get_client(&state, &context_name)?;
    let discovery = discovery_for(&state, &context_name).await?;
    let (ar, caps) = resolve_kind(&discovery, &kind)?;
    let api = dynamic_api(client, &ar, &caps, namespace.as_deref());

    let stream_id = Uuid::new_v4();
    let event_name = format!("resource-event:{stream_id}");
    let app_handle = app.clone();

    let join_handle = tokio::spawn(async move {
        let mut watch_config = watcher::Config::default();
        if let Some(selector) = field_selector {
            watch_config = watch_config.fields(&selector);
        }
        let mut stream = Box::pin(watcher(api, watch_config).default_backoff());
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
