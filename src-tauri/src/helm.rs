use std::collections::HashMap;
use std::io::Read;

use base64::Engine;
use flate2::read::GzDecoder;
use k8s_openapi::api::core::v1::Secret;
use kube::api::{Api, ListParams};
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Decodes a Helm v3 release Secret (`type: helm.sh/release.v1`). Helm's own encoding -
/// stable since v3, unchanged for years - is `base64(gzip(json))`; `Secret.data` has already
/// had the outer, unrelated K8s-API-level base64 stripped by k8s-openapi.
fn decode_release_secret(secret: &Secret) -> AppResult<Value> {
    let data = secret
        .data
        .as_ref()
        .and_then(|d| d.get("release"))
        .ok_or_else(|| AppError::Message("secret has no 'release' data key".into()))?;

    let helm_b64 = String::from_utf8(data.0.clone())
        .map_err(|e| AppError::Message(format!("invalid helm release encoding: {e}")))?;
    let gz_bytes = base64::engine::general_purpose::STANDARD
        .decode(helm_b64)
        .map_err(|e| AppError::Message(format!("invalid base64 in release secret: {e}")))?;

    let mut decoder = GzDecoder::new(&gz_bytes[..]);
    let mut json_bytes = Vec::new();
    decoder.read_to_end(&mut json_bytes).map_err(AppError::Io)?;

    serde_json::from_slice(&json_bytes)
        .map_err(|e| AppError::Message(format!("invalid release JSON: {e}")))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmReleaseSummary {
    pub name: String,
    pub namespace: String,
    pub revision: i64,
    pub status: String,
    pub chart: String,
    pub chart_version: String,
    pub app_version: String,
    pub updated: String,
    pub description: String,
}

fn summarize(release: &Value, namespace: &str) -> Option<HelmReleaseSummary> {
    let name = release.get("name")?.as_str()?.to_string();
    let revision = release.get("version").and_then(|v| v.as_i64()).unwrap_or(0);
    let info = release.get("info");
    let status = info
        .and_then(|i| i.get("status"))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();
    let description = info
        .and_then(|i| i.get("description"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let updated = info
        .and_then(|i| i.get("last_deployed"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let chart_meta = release.get("chart").and_then(|c| c.get("metadata"));
    let chart = chart_meta
        .and_then(|m| m.get("name"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let chart_version = chart_meta
        .and_then(|m| m.get("version"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let app_version = chart_meta
        .and_then(|m| m.get("appVersion"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    Some(HelmReleaseSummary {
        name,
        namespace: namespace.to_string(),
        revision,
        status,
        chart,
        chart_version,
        app_version,
        updated,
        description,
    })
}

fn is_deployed(release: &Value) -> bool {
    release
        .get("info")
        .and_then(|i| i.get("status"))
        .and_then(|s| s.as_str())
        == Some("deployed")
}

/// Lists Helm releases (one row per release name - the `deployed` revision, or the highest
/// revision number if none are `deployed`, e.g. a failed install), decoded directly from
/// their `helm.sh/release.v1` Secrets. No `helm` binary required.
#[tauri::command]
pub async fn list_helm_releases(
    state: State<'_, AppState>,
    context_name: String,
    namespace: Option<String>,
) -> AppResult<Vec<HelmReleaseSummary>> {
    let client = get_client(&state, &context_name)?;
    let secrets: Api<Secret> = match &namespace {
        Some(ns) => Api::namespaced(client, ns),
        None => Api::all(client),
    };
    let lp = ListParams::default().fields("type=helm.sh/release.v1");
    let list = secrets.list(&lp).await?;

    let mut by_release: HashMap<(String, String), Vec<(Value, String)>> = HashMap::new();
    for secret in &list.items {
        let ns = secret.metadata.namespace.clone().unwrap_or_default();
        let Ok(release) = decode_release_secret(secret) else {
            continue;
        };
        let name = release
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or_default()
            .to_string();
        by_release
            .entry((ns.clone(), name))
            .or_default()
            .push((release, ns));
    }

    let mut out = Vec::new();
    for releases in by_release.into_values() {
        let mut releases = releases;
        releases.sort_by_key(|(r, _)| r.get("version").and_then(|v| v.as_i64()).unwrap_or(0));
        let chosen = releases
            .iter()
            .find(|(r, _)| is_deployed(r))
            .or_else(|| releases.last());
        if let Some((release, ns)) = chosen {
            if let Some(summary) = summarize(release, ns) {
                out.push(summary);
            }
        }
    }

    out.sort_by(|a, b| (&a.namespace, &a.name).cmp(&(&b.namespace, &b.name)));
    Ok(out)
}

/// Fetches one release's full decoded data (chart metadata, values, rendered manifest, info) -
/// the `deployed` revision, or a specific one if `revision` is given.
#[tauri::command]
pub async fn get_helm_release(
    state: State<'_, AppState>,
    context_name: String,
    namespace: String,
    name: String,
    revision: Option<i64>,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let secrets: Api<Secret> = Api::namespaced(client, &namespace);
    let selector = match revision {
        Some(v) => format!("owner=helm,name={name},version={v}"),
        None => format!("owner=helm,name={name}"),
    };
    let lp = ListParams::default().labels(&selector);
    let list = secrets.list(&lp).await?;

    let mut releases: Vec<Value> = list
        .items
        .iter()
        .filter_map(|s| decode_release_secret(s).ok())
        .collect();

    if revision.is_none() {
        releases.sort_by_key(|r| r.get("version").and_then(|v| v.as_i64()).unwrap_or(0));
        if let Some(idx) = releases.iter().position(is_deployed) {
            return Ok(releases.remove(idx));
        }
    }

    releases
        .pop()
        .ok_or_else(|| AppError::Message(format!("release not found: {namespace}/{name}")))
}

/// Lists every revision of one release, oldest first.
#[tauri::command]
pub async fn list_helm_release_history(
    state: State<'_, AppState>,
    context_name: String,
    namespace: String,
    name: String,
) -> AppResult<Vec<HelmReleaseSummary>> {
    let client = get_client(&state, &context_name)?;
    let secrets: Api<Secret> = Api::namespaced(client, &namespace);
    let lp = ListParams::default().labels(&format!("owner=helm,name={name}"));
    let list = secrets.list(&lp).await?;

    let mut out: Vec<HelmReleaseSummary> = list
        .items
        .iter()
        .filter_map(|s| decode_release_secret(s).ok())
        .filter_map(|r| summarize(&r, &namespace))
        .collect();
    out.sort_by_key(|r| r.revision);
    Ok(out)
}
