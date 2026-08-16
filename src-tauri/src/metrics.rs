use kube::api::{Api, DynamicObject, ListParams};
use kube::core::GroupVersionKind;
use kube::discovery::ApiResource;
use serde_json::Value;
use tauri::State;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

// The metrics.k8s.io API keeps the same plural resource names as core Pods/Nodes
// ("pods"/"nodes"), so auto-pluralizing the Kind ("PodMetrics" -> "podmetricses") would
// be wrong here - the plural has to be set explicitly.
fn pod_metrics_resource() -> ApiResource {
    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    ApiResource::from_gvk_with_plural(&gvk, "pods")
}

fn node_metrics_resource() -> ApiResource {
    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "NodeMetrics");
    ApiResource::from_gvk_with_plural(&gvk, "nodes")
}

/// Point-in-time pod CPU/memory usage from metrics-server, as raw `PodMetrics` JSON.
/// Requires metrics-server to be installed on the cluster.
#[tauri::command]
pub async fn get_pod_metrics(
    state: State<'_, AppState>,
    context_name: String,
    namespace: Option<String>,
) -> AppResult<Vec<Value>> {
    let client = get_client(&state, &context_name)?;
    let ar = pod_metrics_resource();
    let api: Api<DynamicObject> = match namespace {
        Some(ns) => Api::namespaced_with(client, &ns, &ar),
        None => Api::all_with(client, &ar),
    };
    let list = api.list(&ListParams::default()).await.map_err(AppError::Kube)?;
    Ok(list
        .items
        .into_iter()
        .map(|obj| serde_json::to_value(obj).unwrap_or(Value::Null))
        .collect())
}

/// Point-in-time node CPU/memory usage from metrics-server, as raw `NodeMetrics` JSON.
#[tauri::command]
pub async fn get_node_metrics(
    state: State<'_, AppState>,
    context_name: String,
) -> AppResult<Vec<Value>> {
    let client = get_client(&state, &context_name)?;
    let ar = node_metrics_resource();
    let api: Api<DynamicObject> = Api::all_with(client, &ar);
    let list = api.list(&ListParams::default()).await.map_err(AppError::Kube)?;
    Ok(list
        .items
        .into_iter()
        .map(|obj| serde_json::to_value(obj).unwrap_or(Value::Null))
        .collect())
}
