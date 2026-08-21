use std::collections::HashMap;

use kube::api::{Api, DynamicObject, ListParams};
use kube::core::GroupVersionKind;
use kube::discovery::ApiResource;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

const STORE_FILE: &str = "settings.json";
const KEY_PROMETHEUS_URLS: &str = "prometheusUrls";

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

/// The Prometheus query endpoint configured for a context, if any. Not auto-discovered -
/// users point this at whatever Prometheus is already reachable from their machine (their own
/// `kubectl port-forward`, an ingress, etc).
#[tauri::command]
pub async fn get_prometheus_url(app: AppHandle, context_name: String) -> AppResult<Option<String>> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Message(e.to_string()))?;
    let urls: HashMap<String, String> = store
        .get(KEY_PROMETHEUS_URLS)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    Ok(urls.get(&context_name).cloned())
}

#[tauri::command]
pub async fn set_prometheus_url(app: AppHandle, context_name: String, url: String) -> AppResult<()> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Message(e.to_string()))?;
    let mut urls: HashMap<String, String> = store
        .get(KEY_PROMETHEUS_URLS)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    if url.trim().is_empty() {
        urls.remove(&context_name);
    } else {
        urls.insert(context_name, url);
    }
    store.set(KEY_PROMETHEUS_URLS, serde_json::json!(urls));
    store.save().map_err(|e| AppError::Message(e.to_string()))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricPoint {
    pub timestamp: i64,
    pub value: f64,
}

/// Runs a PromQL range query against a directly-reachable Prometheus endpoint, returning a
/// single time series (the first `data.result` entry - callers use aggregate queries that
/// return one series).
#[tauri::command]
pub async fn query_prometheus_range(
    prometheus_url: String,
    promql: String,
    start_unix: i64,
    end_unix: i64,
    step_seconds: u32,
) -> AppResult<Vec<MetricPoint>> {
    let url = format!("{}/api/v1/query_range", prometheus_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .query(&[
            ("query", promql.as_str()),
            ("start", &start_unix.to_string()),
            ("end", &end_unix.to_string()),
            ("step", &step_seconds.to_string()),
        ])
        .send()
        .await
        .map_err(|e| AppError::Message(format!("prometheus request failed: {e}")))?;

    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Message(format!("invalid prometheus response: {e}")))?;

    parse_range_response(&body)
}

/// Parses a `/api/v1/query_range` response body, pulled out of `query_prometheus_range` so it
/// can be unit-tested against Prometheus's documented response shape without a live server.
fn parse_range_response(body: &Value) -> AppResult<Vec<MetricPoint>> {
    if body.get("status").and_then(|s| s.as_str()) != Some("success") {
        let err = body
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("unknown error");
        return Err(AppError::Message(format!("prometheus query failed: {err}")));
    }

    let values = body
        .get("data")
        .and_then(|d| d.get("result"))
        .and_then(|r| r.get(0))
        .and_then(|r| r.get("values"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(values
        .into_iter()
        .filter_map(|pair| {
            let pair = pair.as_array()?;
            let timestamp = pair.first()?.as_f64()? as i64;
            let value: f64 = pair.get(1)?.as_str()?.parse().ok()?;
            Some(MetricPoint { timestamp, value })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shape taken verbatim from Prometheus's documented /api/v1/query_range response:
    // https://prometheus.io/docs/prometheus/latest/querying/api/#range-queries
    const SAMPLE_RESPONSE: &str = r#"{
        "status": "success",
        "data": {
            "resultType": "matrix",
            "result": [
                {
                    "metric": {},
                    "values": [
                        [1435781430, "1.5"],
                        [1435781445, "2.25"],
                        [1435781460, "3"]
                    ]
                }
            ]
        }
    }"#;

    #[test]
    fn parses_documented_range_response_shape() {
        let body: Value = serde_json::from_str(SAMPLE_RESPONSE).unwrap();
        let points = parse_range_response(&body).expect("should parse");
        assert_eq!(points.len(), 3);
        assert_eq!(points[0].timestamp, 1435781430);
        assert_eq!(points[0].value, 1.5);
        assert_eq!(points[2].value, 3.0);
    }

    #[test]
    fn surfaces_prometheus_error_status() {
        let body: Value = serde_json::json!({ "status": "error", "error": "bad query" });
        let err = parse_range_response(&body).unwrap_err();
        assert!(err.to_string().contains("bad query"));
    }
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
